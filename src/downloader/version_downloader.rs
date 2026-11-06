use super::{extract_zip, get_json, write_file};
use crate::minecraft::get_minecraft_dir;
use anyhow::{anyhow, Context, Result};
use iced::{
    futures::{stream::FuturesUnordered, SinkExt, StreamExt},
    Subscription,
};
use reqwest::Client;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

const VERSION_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";
const DOWNLOAD_CONCURRENCY: usize = 8;
const FABRIC_LOADER_VERSIONS_URL: &str = "https://meta.fabricmc.net/v2/versions/loader";
const NEOFORGE_VERSIONS_URL: &str = "https://meta.prismlauncher.org/v1/net.neoforged/index.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VersionType {
    Vanilla,
    Fabric,
    NeoForge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableVersions {
    pub minecraft: Vec<MinecraftVersion>,
    pub fabric_loader: String,
    pub neoforge_loaders: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftVersion {
    pub id: String,
    pub version_type: String,
}

fn platform_os() -> &'static str {
    std::env::consts::OS
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionRequest {
    pub version: String,
    pub version_type: VersionType,
    pub fabric_loader: Option<String>,
    pub neoforge_loader: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    Started { total: usize },
    Downloaded { completed: usize, total: usize },
    Finished,
    Errored(String),
}

#[derive(Debug, Clone, Hash)]
pub struct DownloadFile {
    pub path: PathBuf,
    pub url: String,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub extract_natives: bool,
}

#[derive(Debug, Clone)]
struct VersionDocument {
    json: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct VersionManifest {
    versions: Vec<ManifestVersion>,
}

#[derive(Debug, Deserialize)]
struct ManifestVersion {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct AssetIndex {
    objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
struct AssetObject {
    hash: String,
    size: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct LibraryDownload {
    path: Option<String>,
    url: Option<String>,
    sha1: Option<String>,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct LibraryArtifact {
    artifact: Option<LibraryDownload>,
    classifiers: Option<HashMap<String, LibraryDownload>>,
}

#[derive(Debug, Deserialize)]
struct Library {
    downloads: Option<LibraryArtifact>,
    rules: Option<Vec<Rule>>,
    name: String,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Rule {
    action: String,
    os: Option<RuleOs>,
}

#[derive(Debug, Deserialize)]
struct RuleOs {
    name: Option<String>,
    arch: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DownloadPlan {
    pub files: Vec<DownloadFile>,
}

pub async fn available_versions(client: Client) -> Result<AvailableVersions> {
    let manifest: VersionManifest = get_json(&client, VERSION_MANIFEST_URL).await?;
    let loaders: Vec<FabricLoader> = get_json(&client, FABRIC_LOADER_VERSIONS_URL).await?;
    let neoforge_index: NeoForgeIndex = get_json(&client, NEOFORGE_VERSIONS_URL).await?;
    let fabric_loader = loaders
        .into_iter()
        .find(|version| version.stable)
        .map(|version| version.version)
        .ok_or_else(|| anyhow!("Fabric loader versions are unavailable"))?;

    Ok(AvailableVersions {
        minecraft: manifest
            .versions
            .into_iter()
            .map(|version| MinecraftVersion {
                id: version.id,
                version_type: version.version_type,
            })
            .collect(),
        fabric_loader,
        neoforge_loaders: latest_neoforge_loaders(neoforge_index.versions),
    })
}

#[derive(Debug, Deserialize)]
struct FabricLoader {
    version: String,
    stable: bool,
}

#[derive(Debug, Deserialize)]
struct NeoForgeIndex {
    versions: Vec<NeoForgeVersion>,
}

#[derive(Debug, Deserialize)]
struct NeoForgeVersion {
    version: String,
    #[serde(rename = "type")]
    version_type: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
    requires: Vec<NeoForgeRequirement>,
}

#[derive(Debug, Deserialize)]
struct NeoForgeRequirement {
    uid: String,
    equals: String,
}

fn latest_neoforge_loaders(versions: Vec<NeoForgeVersion>) -> HashMap<String, String> {
    let mut latest = HashMap::<String, (String, String)>::new();
    for version in versions {
        if version.version_type != "release" {
            continue;
        }
        let Some(minecraft) = version
            .requires
            .iter()
            .find(|requirement| requirement.uid == "net.minecraft")
            .map(|requirement| requirement.equals.clone())
        else {
            continue;
        };
        let replace = latest
            .get(&minecraft)
            .map(|(_, release_time)| version.release_time > *release_time)
            .unwrap_or(true);
        if replace {
            latest.insert(minecraft, (version.version, version.release_time));
        }
    }
    latest
        .into_iter()
        .map(|(minecraft, (version, _))| (minecraft, version))
        .collect()
}

pub fn download(request: VersionRequest) -> Subscription<(VersionRequest, Progress)> {
    Subscription::run_with(request, build_subscription)
}

pub fn download_files(
    request: VersionRequest,
    files: Vec<DownloadFile>,
) -> Subscription<(VersionRequest, Progress)> {
    Subscription::run_with((request, files), build_files_subscription)
}

fn build_subscription(
    request: &VersionRequest,
) -> impl iced::futures::Stream<Item = (VersionRequest, Progress)> + 'static {
    let request = request.clone();
    iced::stream::channel(32, async move |mut output| {
        let result = run_download(request.clone(), &mut output).await;
        if let Err(error) = result {
            let _ = output
                .send((request.clone(), Progress::Errored(format!("{error:#}"))))
                .await;
        }
    })
}

fn build_files_subscription(
    input: &(VersionRequest, Vec<DownloadFile>),
) -> impl iced::futures::Stream<Item = (VersionRequest, Progress)> + 'static {
    let (request, files) = input.clone();
    iced::stream::channel(32, async move |mut output| {
        let client = Client::new();
        let plan = DownloadPlan { files };
        let result = async {
            output
                .send((
                    request.clone(),
                    Progress::Started {
                        total: plan.files.len(),
                    },
                ))
                .await
                .map_err(|_| anyhow!("download subscription was cancelled"))?;
            download_files_from_plan(&client, &plan, &request, &mut output).await?;
            output
                .send((request.clone(), Progress::Finished))
                .await
                .map_err(|_| anyhow!("download subscription was cancelled"))?;
            Ok::<(), anyhow::Error>(())
        }
        .await;

        if let Err(error) = result {
            let _ = output
                .send((request, Progress::Errored(format!("{error:#}"))))
                .await;
        }
    })
}

async fn run_download(
    request: VersionRequest,
    output: &mut iced::futures::channel::mpsc::Sender<(VersionRequest, Progress)>,
) -> Result<()> {
    let client = Client::new();
    let plan = create_plan(&client, &request).await?;
    let total = plan.files.len();
    output
        .send((request.clone(), Progress::Started { total }))
        .await
        .map_err(|_| anyhow!("download subscription was cancelled"))?;

    download_files_from_plan(&client, &plan, &request, output).await?;
    output
        .send((request, Progress::Finished))
        .await
        .map_err(|_| anyhow!("download subscription was cancelled"))?;
    Ok(())
}

pub async fn create_plan(client: &Client, request: &VersionRequest) -> Result<DownloadPlan> {
    let minecraft_dir = PathBuf::from(get_minecraft_dir());
    let version_name = match request.version_type {
        VersionType::Vanilla => request.version.clone(),
        VersionType::Fabric => format!("{}-fabric", request.version),
        VersionType::NeoForge => format!("{}-neoforge", request.version),
    };
    let version_dir = minecraft_dir.join("versions").join(&version_name);
    let vanilla_dir = minecraft_dir.join("versions").join(&request.version);
    fs::create_dir_all(&version_dir)?;
    fs::create_dir_all(&vanilla_dir)?;

    let vanilla = version_document(client, &request.version).await?;
    let version = match request.version_type {
        VersionType::Vanilla => vanilla.clone(),
        VersionType::Fabric => {
            let loader = request
                .fabric_loader
                .as_deref()
                .ok_or_else(|| anyhow!("Fabric loader version is required"))?;
            let url = format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
                request.version, loader
            );
            VersionDocument {
                json: get_json(client, &url).await?,
            }
        }
        VersionType::NeoForge => {
            let loader = request
                .neoforge_loader
                .as_deref()
                .ok_or_else(|| anyhow!("NeoForge loader version is required"))?;
            let url = format!("https://meta.prismlauncher.org/v1/net.neoforged/{loader}.json");
            VersionDocument {
                json: get_json(client, &url).await?,
            }
        }
    };
    let mut files = Vec::new();
    let client_download = vanilla.json["downloads"]["client"]
        .as_object()
        .ok_or_else(|| anyhow!("version has no client download"))?;
    let client_jar_path = match request.version_type {
        VersionType::NeoForge => version_dir.join(format!("{version_name}.jar")),
        VersionType::Vanilla | VersionType::Fabric => {
            vanilla_dir.join(format!("{}.jar", request.version))
        }
    };
    files.push(download_file(
        &client_jar_path,
        client_download,
        false,
    )?);
    write_json(
        &vanilla_dir.join(format!("{}.json", request.version)),
        &vanilla.json,
    )?;
    write_json(
        &version_dir.join(format!("{version_name}.json")),
        &version.json,
    )?;

    let asset_index = vanilla.json["assetIndex"]
        .as_object()
        .ok_or_else(|| anyhow!("version has no asset index"))?;
    let asset_index_url = asset_index["url"]
        .as_str()
        .ok_or_else(|| anyhow!("asset index has no URL"))?;
    let asset_index_path = minecraft_dir
        .join("assets")
        .join("indexes")
        .join(format!("{}.json", asset_index["id"].as_str().unwrap_or("")));
    let asset_index_sha1 = asset_index["sha1"].as_str().map(String::from);
    let asset_index_size = asset_index["size"].as_u64();

    files.push(DownloadFile {
        path: asset_index_path,
        url: asset_index_url.to_owned(),
        sha1: asset_index_sha1,
        size: asset_index_size,
        extract_natives: false,
    });

    let assets: AssetIndex = get_json(client, asset_index_url).await?;
    let assets_dir = minecraft_dir.join("assets").join("objects");
    for asset in assets.objects.values() {
        let prefix = &asset.hash[..2];
        files.push(DownloadFile {
            path: assets_dir.join(prefix).join(&asset.hash),
            url: format!(
                "https://resources.download.minecraft.net/{prefix}/{}",
                asset.hash
            ),
            sha1: Some(asset.hash.clone()),
            size: Some(asset.size),
            extract_natives: false,
        });
    }

    add_libraries(&mut files, &minecraft_dir, &vanilla.json)?;
    if matches!(
        request.version_type,
        VersionType::Fabric | VersionType::NeoForge
    ) {
        add_libraries(&mut files, &minecraft_dir, &version.json)?;
    }
    add_maven_files(&mut files, &minecraft_dir, &version.json)?;
    Ok(DownloadPlan { files })
}

async fn version_document(client: &Client, version: &str) -> Result<VersionDocument> {
    let manifest: VersionManifest = get_json(client, VERSION_MANIFEST_URL).await?;
    let entry = manifest
        .versions
        .into_iter()
        .find(|entry| entry.id == version)
        .ok_or_else(|| anyhow!("Minecraft version not found: {version}"))?;
    Ok(VersionDocument {
        json: get_json(client, &entry.url).await?,
    })
}

fn add_libraries(
    files: &mut Vec<DownloadFile>,
    minecraft_dir: &Path,
    json: &serde_json::Value,
) -> Result<()> {
    let libraries: Vec<Library> =
        serde_json::from_value(json["libraries"].clone()).context("invalid libraries section")?;
    for library in libraries {
        if !allowed_by_rules(library.rules.as_deref().unwrap_or(&[])) {
            continue;
        }
        let Some(downloads) = library.downloads else {
            if let Some(path) = crate::launcher::library_builder::maven_to_path(&library.name, None)
            {
                if let Some(base_url) = library.url.as_deref() {
                    files.push(DownloadFile {
                        path: minecraft_dir.join("libraries").join(&path),
                        url: format!("{}{}", base_url, path.to_string_lossy()),
                        sha1: None,
                        size: None,
                        extract_natives: false,
                    });
                }
            }
            continue;
        };
        if let Some(artifact) = downloads.artifact {
            if let Some(file) = library_file(minecraft_dir, artifact.clone(), false) {
                files.push(file);
            } else if let Some(path) =
                crate::launcher::library_builder::maven_to_path(&library.name, None)
            {
                let Some(url) = artifact.url else {
                    continue;
                };
                files.push(DownloadFile {
                    path: minecraft_dir.join("libraries").join(path),
                    url,
                    sha1: artifact.sha1,
                    size: artifact.size,
                    extract_natives: false,
                });
            }
        }
        if let Some(classifiers) = downloads.classifiers {
            if let Some(native) = native_classifier(&classifiers) {
                if let Some(file) = library_file(minecraft_dir, native, true) {
                    files.push(file);
                }
            }
        }
    }
    Ok(())
}

fn add_maven_files(
    files: &mut Vec<DownloadFile>,
    minecraft_dir: &Path,
    json: &serde_json::Value,
) -> Result<()> {
    let Some(maven_files) = json.get("mavenFiles") else {
        return Ok(());
    };
    let libraries: Vec<Library> =
        serde_json::from_value(maven_files.clone()).context("invalid mavenFiles section")?;
    for library in libraries {
        if !allowed_by_rules(library.rules.as_deref().unwrap_or(&[])) {
            continue;
        }
        let Some(downloads) = library.downloads else {
            continue;
        };
        let Some(artifact) = downloads.artifact else {
            continue;
        };
        if let Some(file) = library_file(minecraft_dir, artifact.clone(), false) {
            files.push(file);
        } else if let Some(path) =
            crate::launcher::library_builder::maven_to_path(&library.name, None)
        {
            let Some(url) = artifact.url else {
                continue;
            };
            files.push(DownloadFile {
                path: minecraft_dir.join("libraries").join(path),
                url,
                sha1: artifact.sha1,
                size: artifact.size,
                extract_natives: false,
            });
        }
    }
    Ok(())
}

fn library_file(minecraft_dir: &Path, file: LibraryDownload, native: bool) -> Option<DownloadFile> {
    Some(DownloadFile {
        path: minecraft_dir.join("libraries").join(file.path?),
        url: file.url?,
        sha1: file.sha1,
        size: file.size,
        extract_natives: native,
    })
}

fn native_classifier(classifiers: &HashMap<String, LibraryDownload>) -> Option<LibraryDownload> {
    let key = match std::env::consts::OS {
        "windows" => "natives-windows",
        "macos" => "natives-osx",
        "linux" => "natives-linux",
        _ => return None,
    };
    classifiers
        .iter()
        .find(|(name, _)| name.starts_with(key))
        .map(|(_, file)| file.clone())
}

fn allowed_by_rules(rules: &[Rule]) -> bool {
    let mut allowed = true;
    for rule in rules {
        let matches = rule.os.as_ref().map_or(true, |os| {
            os.name
                .as_deref()
                .map_or(true, |name| name == platform_os())
                && os
                    .arch
                    .as_deref()
                    .map_or(true, |arch| arch == std::env::consts::ARCH)
        });
        if matches {
            allowed = rule.action == "allow";
        }
    }
    allowed
}

fn download_file(
    path: &Path,
    value: &serde_json::Map<String, serde_json::Value>,
    extract_natives: bool,
) -> Result<DownloadFile> {
    Ok(DownloadFile {
        path: path.to_path_buf(),
        url: value["url"]
            .as_str()
            .ok_or_else(|| anyhow!("download has no URL"))?
            .to_owned(),
        sha1: value["sha1"].as_str().map(str::to_owned),
        size: value["size"].as_u64(),
        extract_natives,
    })
}

async fn download_files_from_plan(
    client: &Client,
    plan: &DownloadPlan,
    request: &VersionRequest,
    output: &mut iced::futures::channel::mpsc::Sender<(VersionRequest, Progress)>,
) -> Result<()> {
    let pending = plan
        .files
        .iter()
        .filter(|file| !is_valid(file))
        .cloned()
        .collect::<Vec<_>>();
    let total = plan.files.len();
    let mut completed = total - pending.len();
    let mut queue = pending.into_iter();
    let client = Arc::new(client.clone());
    let mut active = FuturesUnordered::new();
    let version_name = match request.version_type {
        VersionType::Vanilla => request.version.clone(),
        VersionType::Fabric => format!("{}-fabric", request.version),
        VersionType::NeoForge => format!("{}-neoforge", request.version),
    };
    let natives_dir = PathBuf::from(get_minecraft_dir())
        .join("versions")
        .join(version_name)
        .join("natives");

    for _ in 0..DOWNLOAD_CONCURRENCY {
        if let Some(file) = queue.next() {
            active.push(download_one(client.clone(), file, natives_dir.clone()));
        }
    }
    while let Some(result) = active.next().await {
        result?;
        completed += 1;
        output
            .send((request.clone(), Progress::Downloaded { completed, total }))
            .await
            .map_err(|_| anyhow!("download subscription was cancelled"))?;
        if let Some(file) = queue.next() {
            active.push(download_one(client.clone(), file, natives_dir.clone()));
        }
    }
    Ok(())
}

async fn download_one(client: Arc<Client>, file: DownloadFile, natives_dir: PathBuf) -> Result<()> {
    println!("Downloading {}...", file.path.display());
    let response = client.get(&file.url).send().await?.error_for_status()?;
    let bytes = response.bytes().await?;
    if let Some(size) = file.size {
        if bytes.len() as u64 != size {
            return Err(anyhow!("size mismatch for {}", file.path.display()));
        }
    }
    if let Some(expected) = &file.sha1 {
        if sha1_hex(&bytes) != *expected {
            return Err(anyhow!("SHA-1 mismatch for {}", file.path.display()));
        }
    }
    if let Some(parent) = file.path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&file.path, &bytes)?;
    if file.extract_natives {
        extract_zip(&file.path, &natives_dir)?;
        fs::remove_file(&file.path)?;
    }
    Ok(())
}

fn is_valid(file: &DownloadFile) -> bool {
    let Ok(bytes) = fs::read(&file.path) else {
        return false;
    };
    file.size.map_or(true, |size| bytes.len() as u64 == size)
        && file
            .sha1
            .as_deref()
            .map_or(true, |hash| sha1_hex(&bytes) == hash)
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<()> {
    write_file(path, &serde_json::to_vec_pretty(value)?)
}

fn sha1_hex(data: &[u8]) -> String {
    Sha1::digest(data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
