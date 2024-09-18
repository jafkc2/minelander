use iced::subscription;
use reqwest::{self, Client};
use serde_json::Value;
use std::{
    env,
    fs::{self, File},
    hash::Hash,
    io::{BufReader, Read, Write},
    path::Path,
};
use zip::ZipArchive;

pub enum State {
    GettingDownloadList(String, VersionType),
    Downloading(DownloadList),
    PreparingJavaDownload(Java),
    DownloadingJava {
        downloaded: u64,
        total: u64,
        download: reqwest::Response,
        folder_to_store: String,
        file_to_write: File,
        java: Java,
    },
    ExtractingJava(String, Java),
    DownloadingMissingFiles(DownloadList),
    PreparingUpdate(String),
    DownloadingUpdate {
        downloaded: u64,
        total: u64,
        download: reqwest::Response,
        exec: File,
    },
    Idle,
}
#[derive(Debug, Clone, PartialEq)]
pub enum VersionType {
    Vanilla,
    Fabric,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Progress {
    GotDownloadList(i32),
    Downloaded(i32),
    Finished,

    StartedJavaDownload(u8),
    JavaDownloadProgressed(u8, u8),
    JavaDownloadFinished,
    JavaExtracted,

    MissingFilesDownloadProgressed(u16),
    MissingFilesDownloadFinished,

    UpdateStarted(u8),
    UpdateProgressed(u8, u8, u8),
    UpdateFinished,

    Errored(String),
}

pub fn start<I: 'static + Hash + Copy + Send + Sync>(
    id: I,
    version: String,
    version_type: VersionType,
) -> iced::Subscription<(I, Progress)> {
    subscription::unfold(
        id,
        State::GettingDownloadList(version, version_type),
        move |state| download(id, state),
    )
}
#[derive(Clone)]
pub enum Java {
    J8,
    J17,
    J21,
}
pub fn start_java<I: 'static + Hash + Copy + Send + Sync>(
    id: I,
    java: Java,
) -> iced::Subscription<(I, Progress)> {
    subscription::unfold(id, State::PreparingJavaDownload(java), move |state| {
        download(id, state)
    })
}

pub fn start_missing_files<I: 'static + Hash + Copy + Send + Sync>(
    id: I,
    files: DownloadList,
) -> iced::Subscription<(I, Progress)> {
    subscription::unfold(id, State::DownloadingMissingFiles(files), move |state| {
        download(id, state)
    })
}

pub fn start_update<I: 'static + Hash + Copy + Send + Sync>(
    id: I,
    url: String,
) -> iced::Subscription<(I, Progress)> {
    subscription::unfold(id, State::PreparingUpdate(url), move |state| {
        download(id, state)
    })
}
#[derive(Clone)]
pub struct DownloadList {
    pub download_list: Vec<Download>,
    pub client: Client,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Download {
    pub path: String,
    pub url: String,
}

async fn download<I: 'static + Hash + Copy + Send + Sync>(
    id: I,
    state: State,
) -> ((I, Progress), State) {
    match state {
        // Versions downloading
        State::GettingDownloadList(version, version_type) => {
            let mc_dir = match std::env::consts::OS {
                "linux" => format!("{}/.minecraft", std::env::var("HOME").unwrap()),
                "windows" => format!(
                    "{}/AppData/Roaming/.minecraft",
                    std::env::var("USERPROFILE").unwrap().replace('\\', "/")
                ),
                _ => panic!("System not supported."),
            };

            let version_name = match version_type {
                VersionType::Vanilla => version.clone(),
                VersionType::Fabric => format!("{}-fabric", &version),
            };

            let version_folder = format!("{}/versions/{}", &mc_dir, version_name);

            let client = Client::new();

            // the fabric json doesn't provide all required files url, so we are going to get the vanilla json for fabric.
            let vanilla_version_json = match version_type {
                VersionType::Vanilla => {
                    match downloadversionjson(&version_type, &version, &version_folder, &client)
                        .await
                    {
                        Ok(json) => json,
                        Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
                    }
                }
                VersionType::Fabric => {
                    match downloadversionjson(&version_type, &version, &version_folder, &client)
                        .await
                    {
                        Ok(json) => json,
                        Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
                    };
                    let mut file =
                        File::open(format!("{}/{}.json", version_folder, version)).unwrap();
                    let mut fcontent = String::new();
                    file.read_to_string(&mut fcontent).unwrap();
                    let content = serde_json::from_str(&fcontent);
                    content.unwrap()
                }
            };

            let version_json = super::getjson(format!("{}/{}.json", version_folder, version_name));

            // asset index, we need this file to get assets
            let asset_index_download = match client
                .get(vanilla_version_json["assetIndex"]["url"].as_str().unwrap())
                .send()
                .await
            {
                Ok(ok) => ok.bytes().await.unwrap(),
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            };

            let asset_index_path = format!(
                "{}/assets/indexes/{}.json",
                mc_dir,
                vanilla_version_json["assets"].as_str().unwrap()
            );
            match fs::create_dir_all(format!("{}/assets/indexes", mc_dir)) {
                Ok(ok) => ok,
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            }
            let mut asset_index_file = match File::create(&asset_index_path) {
                Ok(ok) => ok,
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            };

            match asset_index_file.write_all(&asset_index_download) {
                Ok(ok) => ok,
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            }

            let asset_index_json = super::getjson(asset_index_path);

            // variable to store download list
            let mut download_list = vec![];

            // push the version jar
            download_list.push(Download {
                path: format!("{}/{}.jar", version_folder, version_name),
                url: vanilla_version_json["downloads"]["client"]["url"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            });

            // push assets
            match get_assets(&mc_dir, asset_index_json) {
                Ok(ok) => download_list.extend_from_slice(&ok),
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            }
            // for older version (pre 1.6)

            // get library download list
            let libresult = &get_libraries(
                &mc_dir,
                vanilla_version_json["libraries"].as_array().unwrap(),
                &version_folder,
            );
            let libraries = match libresult {
                Ok(ok) => ok,
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            };

            download_list.extend_from_slice(libraries);
            if version_type == VersionType::Fabric {
                // fabric libraries
                let libresult = &get_libraries(
                    &mc_dir,
                    version_json["libraries"].as_array().unwrap(),
                    &version_folder,
                );
                let libraries = match libresult {
                    Ok(ok) => ok,
                    Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
                };
                download_list.extend_from_slice(libraries);
            }

            let mut filtered_download_list = Vec::new();
            for i in download_list {
                if !Path::new(&i.path).exists() {
                    filtered_download_list.push(i)
                }
            }

            (
                (
                    id,
                    Progress::GotDownloadList(filtered_download_list.len() as i32),
                ),
                State::Downloading(DownloadList {
                    download_list: filtered_download_list,
                    client,
                }),
            )
        }

        State::Downloading(download_list) => {
            if download_list.download_list.is_empty() {
                println!("finished");
                return ((id, Progress::Finished), State::Idle);
            }

            let mut list = download_list.download_list.clone();

            let current_file_to_download = list.remove(0);
            println!("Downloading {}", current_file_to_download.path);

            let current_download = match download_list
                .client
                .get(current_file_to_download.url)
                .send()
                .await
            {
                Ok(ok) => ok.bytes().await.unwrap(),
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            };

            let path = Path::new(&current_file_to_download.path);

            let mut path_vec  = vec![];
            for i in path.components(){
                path_vec.push(i.as_os_str().to_string_lossy())
            }

            if path_vec.len() > 1{
                path_vec.pop();
                let dir = path_vec.join("/");
                
                if !Path::new(&dir).exists(){
                    match fs::create_dir_all(dir){
                        Ok(ok) => ok,
                        Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
                    }
                }


            }

            let mut file = match File::create(&current_file_to_download.path) {
                Ok(file) => file,
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            };

            match file.write_all(&current_download) {
                Ok(ok) => ok,
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            }

            if current_file_to_download.path.contains("natives.jar") {
                let nativesfile = File::open(&current_file_to_download.path).unwrap();
                let reader = BufReader::new(nativesfile);
                let mut archive = ZipArchive::new(reader).unwrap();

                let folder_to_store_natives =
                    &current_file_to_download.path.replace("/natives.jar", "");

                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    let outpath = format!(
                        "{}/{}",
                        &folder_to_store_natives,
                        file.mangled_name().to_string_lossy()
                    );
                    if file.is_dir() {
                        println!("Creating directory: {:?}", outpath);
                        std::fs::create_dir_all(&outpath).unwrap();
                    } else {
                        println!("Extracting file: {:?}", outpath);
                        let mut outfile = File::create(&outpath).unwrap();
                        std::io::copy(&mut file, &mut outfile).unwrap();
                    }
                }
                fs::remove_file(&current_file_to_download.path).unwrap();
            };

            println!("starting next download.");

            (
                (id, Progress::Downloaded(list.len() as i32)),
                State::Downloading(DownloadList {
                    download_list: list,
                    client: download_list.client,
                }),
            )
        }
        // Idle
        State::Idle => iced::futures::future::pending().await,
        State::PreparingJavaDownload(java) => {
            let os = std::env::consts::OS;
            let java_url = match java{
                Java::J8 => {
                    match os{
                        "windows" => "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u412-b08/OpenJDK8U-jre_x64_windows_hotspot_8u412b08.zip",
                        "linux" => "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u412-b08/OpenJDK8U-jre_x64_linux_hotspot_8u412b08.tar.gz",
                        _ => panic!("Unsuported system.")
                    }
                },
                Java::J17 => {
                    match os{
                        "windows" => "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.9%2B9.1/OpenJDK17U-jre_x64_windows_hotspot_17.0.9_9.zip",
                        "linux" => "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.11%2B9/OpenJDK17U-jre_x64_linux_hotspot_17.0.11_9.tar.gz",
                        _ => panic!("Unsuported system.")
                    }
                },
                Java::J21 => {
                    match os{
                        "windows" => "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.3%2B9/OpenJDK21U-jre_x64_windows_hotspot_21.0.3_9.zip",
                        "linux" => "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.3%2B9/OpenJDK21U-jre_x64_linux_hotspot_21.0.3_9.tar.gz",
                        _ => panic!("Unsuported system.")
                    }
                },
            };

            let mc_dir = match std::env::consts::OS {
                "linux" => format!("{}/.minecraft", std::env::var("HOME").unwrap()),
                "windows" => format!(
                    "{}/AppData/Roaming/.minecraft",
                    std::env::var("USERPROFILE").unwrap().replace('\\', "/")
                ),
                _ => panic!("System not supported."),
            };

            let folder_to_store_download = format!("{}/minelander_java", mc_dir);

            match fs::create_dir_all(&folder_to_store_download) {
                Ok(ok) => ok,
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            }

            let download = reqwest::get(java_url).await;

            let file_name = match os {
                "linux" => "compressed.tar.gz",
                "windows" => "compressed.zip",
                _ => panic!("System not supported."),
            };

            let file_to_write =
                match File::create(format!("{}/{}", folder_to_store_download, file_name)) {
                    Ok(ok) => ok,
                    Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
                };

            match download {
                Ok(d) => {
                    let size = d.content_length().unwrap();
                    (
                        (id, Progress::StartedJavaDownload((size / 1048576) as u8)),
                        State::DownloadingJava {
                            downloaded: 0,
                            total: size,
                            download: d,
                            folder_to_store: folder_to_store_download,
                            file_to_write,
                            java,
                        },
                    )
                }

                Err(e) => ((id, Progress::Errored(e.to_string())), State::Idle),
            }
        }
        State::DownloadingJava {
            downloaded,
            total,
            mut download,
            folder_to_store,
            mut file_to_write,
            java,
        } => match download.chunk().await {
            Ok(Some(chunk)) => {
                let downloaded = downloaded + chunk.len() as u64;
                let percentage = ((downloaded as f32 / total as f32) * 100.0) as u8;
                let mb_downloaded = (downloaded / 1048576) as u8;

                match file_to_write.write_all(&chunk) {
                    Ok(ok) => ok,
                    Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
                }
                (
                    (
                        id,
                        Progress::JavaDownloadProgressed(mb_downloaded, percentage),
                    ),
                    State::DownloadingJava {
                        downloaded,
                        total,
                        download,
                        folder_to_store,
                        file_to_write,
                        java,
                    },
                )
            }
            Ok(None) => (
                (id, Progress::JavaDownloadFinished),
                State::ExtractingJava(folder_to_store, java),
            ),
            Err(e) => ((id, Progress::Errored(e.to_string())), State::Idle),
        },

        State::ExtractingJava(folder, java) => {
            let os = std::env::consts::OS;

            let file_name = match os {
                "linux" => "compressed.tar.gz",
                "windows" => "compressed.zip",
                _ => panic!("System not supported."),
            };

            let compressed_java = match File::open(format!("{}/{}", folder, file_name)) {
                Ok(ok) => ok,
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            };

            let java_folder_name = match java {
                Java::J8 => "java8",
                Java::J17 => "java17",
                Java::J21 => "java21",
            };

            let mut f_folder_name = String::new();

            match os {
                "windows" => {
                    let mut archive = ZipArchive::new(BufReader::new(compressed_java)).unwrap();
                    let mut got_first = false;

                    for i in 0..archive.len() {
                        let mut file = archive.by_index(i).unwrap();

                        if !got_first {
                            f_folder_name = file.name().to_string();
                            got_first = true;
                        }

                        let outpath =
                            format!("{}/{}", &folder, file.mangled_name().to_string_lossy());
                        if file.is_dir() {
                            std::fs::create_dir_all(&outpath).unwrap();
                        } else {
                            let mut outfile = File::create(&outpath).unwrap();
                            std::io::copy(&mut file, &mut outfile).unwrap();
                        }
                    }
                }
                "linux" => {
                    let gz_decoder = flate2::read::GzDecoder::new(BufReader::new(compressed_java));

                    let mut archive = tar::Archive::new(gz_decoder);

                    let archive_iterator = archive.entries().unwrap();

                    let mut got_first = false;

                    for i in archive_iterator {
                        let mut i = i.unwrap();

                        if !got_first {
                            f_folder_name = i
                                .header()
                                .path()
                                .unwrap()
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .into_owned();
                            got_first = true;
                        }
                        i.unpack_in(&folder).unwrap();
                    }
                }
                _ => panic!("System not supported."),
            }

            fs::rename(
                format!("{}/{}", folder, f_folder_name),
                format!("{}/{}", folder, java_folder_name),
            )
            .unwrap();
            fs::remove_file(format!("{}/{}", folder, file_name)).unwrap();

            ((id, Progress::JavaExtracted), State::Idle)
        }
        State::DownloadingMissingFiles(download_list) => {
            if download_list.download_list.is_empty() {
                println!("finished");
                return ((id, Progress::MissingFilesDownloadFinished), State::Idle);
            }

            let mut list = download_list.download_list.clone();

            let current_file_to_download = list.remove(0);
            println!("Downloading {}", current_file_to_download.path);

            let current_download = match download_list
                .client
                .get(current_file_to_download.url)
                .send()
                .await
            {
                Ok(ok) => ok.bytes().await.unwrap(),
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            };

            let mut file = match File::create(&current_file_to_download.path) {
                Ok(file) => file,
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            };

            match file.write_all(&current_download) {
                Ok(ok) => ok,
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            }

            if current_file_to_download.path.contains("natives.jar") {
                let nativesfile = File::open(&current_file_to_download.path).unwrap();
                let reader = BufReader::new(nativesfile);
                let mut archive = ZipArchive::new(reader).unwrap();

                let folder_to_store_natives =
                    &current_file_to_download.path.replace("/natives.jar", "");

                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    let outpath = format!(
                        "{}/{}",
                        &folder_to_store_natives,
                        file.mangled_name().to_string_lossy()
                    );
                    if file.is_dir() {
                        println!("Creating directory: {:?}", outpath);
                        std::fs::create_dir_all(&outpath).unwrap();
                    } else {
                        println!("Extracting file: {:?}", outpath);
                        let mut outfile = File::create(&outpath).unwrap();
                        std::io::copy(&mut file, &mut outfile).unwrap();
                    }
                }
                fs::remove_file(&current_file_to_download.path).unwrap();
            };

            println!("starting next download.");

            (
                (
                    id,
                    Progress::MissingFilesDownloadProgressed(list.len() as u16),
                ),
                State::DownloadingMissingFiles(DownloadList {
                    download_list: list,
                    client: download_list.client,
                }),
            )
        }

        State::PreparingUpdate(url) => {
            let exec_path = env::current_exe().unwrap();
            let exec_file = File::create(&exec_path.with_extension("new")).unwrap();

            #[cfg(target_os = "linux")]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut permission = fs::metadata(&exec_path.with_extension("new")).unwrap().permissions();
                permission.set_mode(0o755);
                fs::set_permissions(&exec_path.with_extension("new"), permission).unwrap();
            }

            let download = reqwest::get(url).await;

            match download {
                Ok(d) => {
                    let size = d.content_length().unwrap();
                    (
                        (id, Progress::UpdateStarted((size / 1048576) as u8)),
                        State::DownloadingUpdate {
                            downloaded: 0,
                            total: size,
                            download: d,
                            exec: exec_file,
                        },
                    )
                }
                Err(e) => ((id, Progress::Errored(e.to_string())), State::Idle),
            }
        }

        State::DownloadingUpdate {
            downloaded,
            total,
            mut download,
            mut exec,
        } => match download.chunk().await {
            Ok(Some(chunk)) => {
                let downloaded = downloaded + chunk.len() as u64;
                let percentage = ((downloaded as f32 / total as f32) * 100.) as u8;
                let mb_downloaded = (downloaded / 1048576) as u8;

                match exec.write_all(&chunk) {
                    Ok(ok) => ok,
                    Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
                }

                (
                    (
                        id,
                        Progress::UpdateProgressed(
                            mb_downloaded,
                            percentage,
                            (total / 1048576) as u8,
                        ),
                    ),
                    State::DownloadingUpdate {
                        downloaded,
                        total,
                        download,
                        exec,
                    },
                )
            }
            Ok(None) => ((id, Progress::UpdateFinished), State::Idle),
            Err(e) => ((id, Progress::Errored(e.to_string())), State::Idle),
        },
    }
}

// Json file
pub async fn downloadversionjson(
    version_type: &VersionType,
    version: &String,
    foldertosave: &String,
    client: &Client,
) -> Result<Value, reqwest::Error> {
    match version_type {
        VersionType::Vanilla => {
            let versionlistjson = reqwest::Client::new()
                .get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
                .send()
                .await?
                .text()
                .await?;

            let content = serde_json::from_str(&versionlistjson);

            let p: Value = content.unwrap();

            let mut url = "";

            if let Some(versions) = p["versions"].as_array() {
                for i in versions {
                    if i["id"].as_str().unwrap() == version {
                        url = i["url"].as_str().unwrap();
                        break;
                    }
                }
            }
            println!("Downloading json...");
            let versionjson = reqwest::Client::new()
                .get(url)
                .send()
                .await?
                .bytes()
                .await?;

            let jfilelocation = format!("{}/{}.json", foldertosave, version);
            fs::create_dir_all(foldertosave).unwrap();
            let mut jfile = File::create(&jfilelocation).unwrap();

            jfile.write_all(&versionjson).unwrap();
            drop(jfile);

            let mut jfile = File::open(jfilelocation).unwrap();
            let mut fcontent = String::new();
            jfile.read_to_string(&mut fcontent).unwrap();
            let content = serde_json::from_str(&fcontent);
            let json: Value = content.unwrap();
            Ok(json)
        }
        VersionType::Fabric => {
            // fabric versions also need the vanilla json, so we are downloading it too.

            // vanilla json
            let versionlistjson = reqwest::Client::new()
                .get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
                .send()
                .await?
                .text()
                .await?;

            let content = serde_json::from_str(&versionlistjson);

            let p: Value = content.unwrap();

            let mut url = "";

            if let Some(versions) = p["versions"].as_array() {
                for i in versions {
                    if i["id"].as_str().unwrap() == version {
                        url = i["url"].as_str().unwrap();
                        break;
                    }
                }
            }

            println!("Downloading json...");
            let versionjson = reqwest::Client::new()
                .get(url)
                .send()
                .await?
                .bytes()
                .await?;

            let jfilelocation = format!("{}/{}.json", foldertosave, version);
            fs::create_dir_all(foldertosave).unwrap();
            let mut jfile = File::create(jfilelocation).unwrap();

            jfile.write_all(&versionjson).unwrap();

            // fabric json
            let fabricloaderlist = client
                .get("https://meta.fabricmc.net/v2/versions/loader")
                .send()
                .await?
                .text()
                .await?;

            let content: Value = serde_json::from_str(&fabricloaderlist).unwrap();

            let fabricloaderversion =
                if let Some(first_object) = content.as_array().and_then(|arr| arr.first()) {
                    first_object["version"].as_str().unwrap()
                } else {
                    panic!("Failed to get fabric loader name")
                };

            let verjson = client
                .get(format!(
                    "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
                    version, fabricloaderversion
                ))
                .send()
                .await?
                .bytes()
                .await?;

            let jfilelocation = format!("{}/{}-fabric.json", foldertosave, version);
            fs::create_dir_all(foldertosave).unwrap();
            let mut jfile = File::create(&jfilelocation).unwrap();

            jfile.write_all(&verjson).unwrap();
            let mut jfile = File::open(jfilelocation).unwrap();

            let mut fcontent = String::new();
            jfile.read_to_string(&mut fcontent).unwrap();
            let content = serde_json::from_str(&fcontent);
            let p: Value = content.unwrap();
            Ok(p)
        }
    }
}

pub async fn get_downloadable_version_list(
    showallversions: bool,
) -> Result<Vec<Vec<String>>, String> {
    let client = reqwest::Client::new();
    // vanilla
    let vanillaversionlistjson = match client
        .get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .await
    {
        Ok(ok) => match ok.text().await {
            Ok(ok) => ok,
            Err(e) => return Err(format!("failed to get download list: {}", e)),
        },
        Err(e) => return Err(format!("failed to get download list: {}", e)),
    };

    let content = serde_json::from_str(&vanillaversionlistjson);

    let p: Value = match content {
        Ok(ok) => ok,
        Err(e) => return Err(format!("failed to read list as json: {}", e)),
    };

    let mut vanillaversionlist: Vec<String> = vec![];
    if let Some(versions) = p["versions"].as_array() {
        if showallversions {
            for i in versions {
                vanillaversionlist.push(i["id"].as_str().unwrap().to_owned())
            }
        } else {
            for i in versions {
                if i["type"] == "release" {
                    vanillaversionlist.push(i["id"].as_str().unwrap().to_owned())
                }
            }
        }
    }
    // fabric
    let fabricversionlistjson = match client
        .get("https://meta.fabricmc.net/v2/versions/game")
        .send()
        .await
    {
        Ok(ok) => match ok.text().await {
            Ok(ok) => ok,
            Err(e) => return Err(format!("failed to get fabric download list: {}", e)),
        },
        Err(e) => return Err(format!("failed to get fabric download list: {}", e)),
    };

    let content = serde_json::from_str(&fabricversionlistjson);

    let p: Value = match content {
        Ok(ok) => ok,
        Err(e) => return Err(format!("failed to read fabric list as json: {}", e)),
    };

    let mut fabricversionlist: Vec<String> = vec![];
    if let Some(versions) = p.as_array() {
        if showallversions {
            for i in versions {
                fabricversionlist.push(i["version"].as_str().unwrap().to_owned())
            }
        } else {
            for i in versions {
                if i["stable"] == true {
                    fabricversionlist.push(i["version"].as_str().unwrap().to_owned())
                }
            }
        }
    }
    Ok(vec![vanillaversionlist, fabricversionlist])
}

pub fn get_libraries(
    mc_dir: &String,
    libraries: &Vec<Value>,
    foldertosave: &String,
) -> Result<Vec<Download>, Box<dyn std::error::Error>> {
    //libraries and natives
    let lib_dir = format!("{}/libraries/", mc_dir);
    let os = std::env::consts::OS;

    enum LibraryType {
        Natives,
        Normal,
        Old,
    }

    let mut library_download_list = vec![];

    for library in libraries {
        if library["rules"][0]["os"]["name"] == os || library["rules"][0]["os"]["name"].is_null() {
            let libraryname = library["name"].as_str().unwrap();
            let mut lpieces: Vec<&str> = libraryname.split(':').collect();
            let firstpiece = lpieces.remove(0).replace('.', "/");

            let libtype = if library["name"]
                .as_str()
                .unwrap()
                .contains(&format!("natives-{}", os))
            {
                LibraryType::Natives
            } else if library["natives"][os].is_null() {
                LibraryType::Normal
            } else {
                LibraryType::Old
            };

            match libtype {
                LibraryType::Natives => {
                    let last_piece = lpieces.pop().unwrap();
                    let lib = format!(
                        "{}/{}/{}-{}-{}.jar",
                        &firstpiece,
                        &lpieces.join("/"),
                        &lpieces[&lpieces.len() - 2],
                        &lpieces[&lpieces.len() - 1],
                        last_piece
                    );

                    // create folder for lib
                    match fs::create_dir_all(format!(
                        "{}/{}/{}",
                        lib_dir,
                        &firstpiece,
                        &lpieces.join("/")
                    )) {
                        Ok(ok) => ok,
                        Err(err) => panic!("{err}"),
                    };

                    let libpath = format!("{}{}", lib_dir, lib);

                    let unmodifiedurl = if !library["downloads"]["artifact"]["url"].is_null() {
                        library["downloads"]["artifact"]["url"].as_str().unwrap()
                    } else if !library["url"].is_null() {
                        library["url"].as_str().unwrap()
                    } else {
                        ""
                    };

                    let url = get_library_url(unmodifiedurl, lib);

                    library_download_list.push(Download { path: libpath, url })
                }

                LibraryType::Normal => {
                    let lib = format!(
                        "{}/{}/{}-{}.jar",
                        &firstpiece,
                        &lpieces.join("/"),
                        &lpieces[&lpieces.len() - 2],
                        &lpieces[&lpieces.len() - 1]
                    );

                    // create folder for lib
                    match fs::create_dir_all(format!(
                        "{}/{}/{}",
                        lib_dir,
                        &firstpiece,
                        &lpieces.join("/")
                    )) {
                        Ok(ok) => ok,
                        Err(err) => panic!("{err}"),
                    };

                    let libpath = format!("{}{}", lib_dir, lib);

                    let unmodifiedurl = if !library["downloads"]["artifact"]["url"].is_null() {
                        library["downloads"]["artifact"]["url"].as_str().unwrap()
                    } else if !library["url"].is_null() {
                        library["url"].as_str().unwrap()
                    } else {
                        ""
                    };

                    let url = get_library_url(unmodifiedurl, lib);

                    library_download_list.push(Download { path: libpath, url })
                }

                LibraryType::Old => {
                    let lib = format!(
                        "{}/{}/{}-{}-natives-{}.jar",
                        &firstpiece,
                        &lpieces.join("/"),
                        &lpieces[&lpieces.len() - 2],
                        &lpieces[&lpieces.len() - 1],
                        os
                    );

                    // create folder for lib
                    match fs::create_dir_all(format!(
                        "{}/{}/{}",
                        lib_dir,
                        &firstpiece,
                        &lpieces.join("/")
                    )) {
                        Ok(ok) => ok,
                        Err(err) => panic!("{err}"),
                    };

                    let libpath = format!("{}{}", lib_dir, lib);

                    let unmodifiedurl = if !library["downloads"]["artifact"]["url"].is_null() {
                        library["downloads"]["artifact"]["url"].as_str().unwrap()
                    } else if !library["url"].is_null() {
                        library["url"].as_str().unwrap()
                    } else if !library["downloads"]["classifiers"][format!("natives-{}", os)]["url"]
                        .is_null()
                        || library["downloads"]["classifiers"][format!("natives-{}-64", os)]["url"]
                            .is_string()
                    {
                        let url = if !library["downloads"]["classifiers"][format!("natives-{}", os)]
                            ["url"]
                            .is_null()
                        {
                            library["downloads"]["classifiers"][format!("natives-{}", os)]["url"]
                                .as_str()
                                .unwrap()
                        } else {
                            library["downloads"]["classifiers"][format!("natives-{}-64", os)]["url"]
                                .as_str()
                                .unwrap()
                        };

                        url
                    } else {
                        ""
                    };

                    let url = get_library_url(unmodifiedurl, lib);

                    library_download_list.push(Download { path: libpath, url })
                }
            }
        }

        if !library["downloads"]["classifiers"][format!("natives-{}", os)].is_null() {
            let url = library["downloads"]["classifiers"][format!("natives-{}", os)]["url"]
                .as_str()
                .unwrap()
                .to_string();

            fs::create_dir_all(format!("{}/natives", foldertosave)).unwrap();
            let path = format!("{}/natives/natives.jar", foldertosave);

            library_download_list.push(Download { path, url });
        }
    }
    Ok(library_download_list)
}

fn get_library_url(unmodifiedurl: &str, lib: String) -> String {
    if unmodifiedurl.ends_with('/') {
        format!("{}{}", unmodifiedurl, lib)
    } else if unmodifiedurl.is_empty() {
        format!("https://libraries.minecraft.net/{}", lib)
    } else {
        unmodifiedurl.to_string()
    }
}

pub fn get_assets(mc_dir: &String, asset_index_json: Value) -> Result<Vec<Download>, String> {
    let save_to_resources = !asset_index_json["map_to_resources"].is_null();
    let mut download_list = Vec::new();

    if let Some(assets) = asset_index_json["objects"].as_object() {
        let assets_directory = format!("{}/assets/objects/", &mc_dir);
        let old_assets_directory = format!("{}/resources", &mc_dir);

        for (key, value) in assets.iter() {
            if let Some(hash) = value["hash"].as_str() {
                match save_to_resources {
                    true => {
                        match fs::create_dir_all(&old_assets_directory) {
                            Ok(ok) => ok,
                            Err(e) => return Err(e.to_string()),
                        };

                        let asset_path = format!("{}/{}", old_assets_directory, key);
                        let asset_url = format!(
                            "https://resources.download.minecraft.net/{}/{}",
                            &hash[0..2],
                            hash
                        );

                        download_list.push(Download {
                            path: asset_path,
                            url: asset_url,
                        });
                    }

                    false => {
                        match fs::create_dir_all(format!("{}/{}", assets_directory, &hash[0..2])) {
                            Ok(ok) => ok,
                            Err(e) => return Err(e.to_string()),
                        };
                        let asset_path = format!("{}/{}/{}", &assets_directory, &hash[0..2], &hash);

                        let asset_url = format!(
                            "https://resources.download.minecraft.net/{}/{}",
                            &hash[0..2],
                            hash
                        );

                        download_list.push(Download {
                            path: asset_path,
                            url: asset_url,
                        });
                    }
                }
            }
        }
    }

    Ok(download_list)
}
