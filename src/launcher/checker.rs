use anyhow::{anyhow, Result};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{downloader::version_downloader::DownloadFile, launcher::library_builder, minecraft};

#[derive(Debug, Clone)]
pub struct VersionMetadata {
    pub json: Value,
    pub vanilla_json: Option<Value>,
    pub main_class: String,
    pub asset_index: String,
    pub inherits_from: Option<String>,
}

impl VersionMetadata {
    pub fn fabric_loader_version(&self) -> Option<String> {
        self.json["libraries"]
            .as_array()?
            .iter()
            .filter_map(|library| library["name"].as_str())
            .find_map(|name| {
                name.strip_prefix("net.fabricmc:fabric-loader:")
                    .map(str::to_owned)
            })
    }

    pub fn neoforge_loader_version(&self) -> Option<String> {
        self.json["version"]
            .as_str()
            .filter(|_| self.json["uid"].as_str() == Some("net.neoforged"))
            .map(str::to_owned)
    }
}

#[derive(Debug, Clone)]
pub enum CheckResult {
    Ready(VersionMetadata),
    NeedsVanillaJson {
        version: String,
        destination: PathBuf,
    },
    NeedsDownload {
        missing_files: Vec<DownloadFile>,
        metadata: VersionMetadata,
    },
}

pub fn load_version_metadata(mc_dir: &Path, game_version: &str) -> Result<VersionMetadata> {
    let version_dir = mc_dir.join("versions").join(game_version);
    let json_path = version_dir.join(format!("{}.json", game_version));

    if !json_path.exists() {
        return Err(anyhow!("Version {} not installed", game_version));
    }

    let content = fs::read_to_string(&json_path)?;
    let p: Value = serde_json::from_str(&content)?;

    let mut vanilla_json: Option<Value> = None;
    let inherits_from = p["inheritsFrom"].as_str().map(String::from).or_else(|| {
        p["requires"].as_array()?.iter().find_map(|requirement| {
            (requirement["uid"].as_str() == Some("net.minecraft"))
                .then(|| requirement["equals"].as_str().map(String::from))?
        })
    });

    if let Some(ref vanilla_ver) = inherits_from {
        let local_vanilla_json = version_dir.join(format!("{}.json", vanilla_ver));
        let central_vanilla_json = mc_dir
            .join("versions")
            .join(vanilla_ver)
            .join(format!("{}.json", vanilla_ver));

        if local_vanilla_json.exists() {
            let vcontent = fs::read_to_string(&local_vanilla_json)?;
            vanilla_json = Some(serde_json::from_str(&vcontent)?);
        } else if central_vanilla_json.exists() {
            let vbytes = fs::read(&central_vanilla_json)?;
            let _ = fs::write(&local_vanilla_json, vbytes);
            let vcontent = fs::read_to_string(&local_vanilla_json)?;
            vanilla_json = Some(serde_json::from_str(&vcontent)?);
        }
    }

    let main_class = p["mainClass"]
        .as_str()
        .or_else(|| {
            vanilla_json
                .as_ref()
                .and_then(|vanilla| vanilla["mainClass"].as_str())
        })
        .unwrap_or("")
        .to_string();
    let asset_index = if let Some(ref v) = vanilla_json {
        v["assets"].as_str().unwrap_or("").to_string()
    } else {
        p["assets"].as_str().unwrap_or("").to_string()
    };

    Ok(VersionMetadata {
        json: p,
        vanilla_json,
        main_class,
        asset_index,
        inherits_from,
    })
}

pub async fn check_version(game_version: &str) -> Result<CheckResult> {
    let p = minecraft::get_minecraft_dir();
    let mc_dir = Path::new(&p);

    let meta = load_version_metadata(mc_dir, game_version)?;
    let version_dir = mc_dir.join("versions").join(game_version);

    // checks for version json manifest
    if let Some(ref vanilla_ver) = meta.inherits_from {
        if meta.vanilla_json.is_none() {
            return Ok(CheckResult::NeedsVanillaJson {
                version: vanilla_ver.clone(),
                destination: version_dir.join(format!("{}.json", vanilla_ver)),
            });
        }
    }

    let mut missing_files: Vec<DownloadFile> = Vec::new();

    // Checks for client jar.
    if meta.inherits_from.is_none() {
        let client_jar_path = version_dir.join(format!("{}.jar", game_version));
        let jar_is_valid = client_jar_path.is_file()
            && fs::metadata(&client_jar_path)
                .map(|m| m.len() > 0)
                .unwrap_or(false);

        if !jar_is_valid {
            if let Some(client) = meta.json["downloads"]["client"].as_object() {
                if let Some(url) = client["url"].as_str() {
                    missing_files.push(DownloadFile {
                        path: client_jar_path,
                        url: url.to_string(),
                        sha1: client["sha1"].as_str().map(String::from),
                        size: client["size"].as_u64(),
                        extract_natives: false,
                    });
                }
            }
        }
    }
    // Checks for natives
    let natives_dir = version_dir.join("natives");
    let natives_empty = !natives_dir.is_dir()
        || fs::read_dir(&natives_dir)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true);

    // checks for libs
    let current_os = match std::env::consts::OS {
        "windows" => "natives-windows",
        "macos" => "natives-osx",
        "linux" => "natives-linux",
        _ => "",
    };

    let mut check_libraries = |json: &Value| {
        let libraries_dir = mc_dir.join("libraries");
        if let Some(libs) = json["libraries"].as_array() {
            for lib in libs {
                let name = lib["name"].as_str().unwrap_or("");
                if name == "tv.twitch:twitch-platform:6.5" {
                    continue;
                }

                if let Some(artifact) = lib
                    .get("downloads")
                    .and_then(|downloads| downloads.get("artifact"))
                    .and_then(Value::as_object)
                {
                    if let Some(rel_path) = artifact.get("path").and_then(Value::as_str) {
                        let full_path = libraries_dir.join(rel_path);
                        if !full_path.is_file() {
                            if let Some(url) = artifact.get("url").and_then(Value::as_str) {
                                missing_files.push(DownloadFile {
                                    path: full_path,
                                    url: url.to_string(),
                                    sha1: artifact
                                        .get("sha1")
                                        .and_then(Value::as_str)
                                        .map(String::from),
                                    size: artifact.get("size").and_then(Value::as_u64),
                                    extract_natives: false,
                                });
                            }

                            if lib
                                .get("downloads")
                                .and_then(|downloads| downloads.get("artifact"))
                                .is_none()
                            {
                                if let (Some(name), Some(base_url)) =
                                    (lib["name"].as_str(), lib["url"].as_str())
                                {
                                    if let Some(rel_path) =
                                        library_builder::maven_to_path(name, None)
                                    {
                                        let full_path = libraries_dir.join(&rel_path);
                                        if !full_path.is_file() {
                                            missing_files.push(DownloadFile {
                                                path: full_path,
                                                url: format!(
                                                    "{}{}",
                                                    base_url,
                                                    rel_path.to_string_lossy()
                                                ),
                                                sha1: None,
                                                size: None,
                                                extract_natives: false,
                                            });
                                        }
                                    }
                                } else if let (Some(name), Some(url)) = (
                                    lib["name"].as_str(),
                                    artifact.get("url").and_then(Value::as_str),
                                ) {
                                    if let Some(rel_path) =
                                        library_builder::maven_to_path(name, None)
                                    {
                                        let full_path = libraries_dir.join(rel_path);
                                        if !full_path.is_file() {
                                            missing_files.push(DownloadFile {
                                                path: full_path,
                                                url: url.to_string(),
                                                sha1: artifact
                                                    .get("sha1")
                                                    .and_then(Value::as_str)
                                                    .map(String::from),
                                                size: artifact.get("size").and_then(Value::as_u64),
                                                extract_natives: false,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if natives_empty && !current_os.is_empty() {
                    if let Some(classifiers) = lib
                        .get("downloads")
                        .and_then(|downloads| downloads.get("classifiers"))
                        .and_then(Value::as_object)
                    {
                        for (classifier_key, file_val) in classifiers {
                            if classifier_key.starts_with(current_os) {
                                if let Some(clf_obj) = file_val.as_object() {
                                    if let Some(rel_path) =
                                        clf_obj.get("path").and_then(Value::as_str)
                                    {
                                        let full_path = libraries_dir.join(rel_path);
                                        if let Some(url) =
                                            clf_obj.get("url").and_then(Value::as_str)
                                        {
                                            missing_files.push(DownloadFile {
                                                path: full_path,
                                                url: url.to_string(),
                                                sha1: clf_obj
                                                    .get("sha1")
                                                    .and_then(Value::as_str)
                                                    .map(String::from),
                                                size: clf_obj.get("size").and_then(Value::as_u64),
                                                extract_natives: true,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    check_libraries(&meta.json);
    if let Some(ref vjson) = meta.vanilla_json {
        check_libraries(vjson);
    }

    // checks asset index
    let asset_index_url = meta.json["assetIndex"]["url"].as_str().or_else(|| {
        meta.vanilla_json
            .as_ref()
            .and_then(|v| v["assetIndex"]["url"].as_str())
    });

    if !meta.asset_index.is_empty() {
        let index_path = mc_dir
            .join("assets")
            .join("indexes")
            .join(format!("{}.json", meta.asset_index));

        if !index_path.is_file() {
            if let Some(url) = asset_index_url {
                missing_files.push(DownloadFile {
                    path: index_path,
                    url: url.to_string(),
                    sha1: None,
                    size: None,
                    extract_natives: false,
                });
            }
        }
    }
    if missing_files.is_empty() {
        Ok(CheckResult::Ready(meta))
    } else {
        println!("Missing files detected: {:?}", missing_files);
        Ok(CheckResult::NeedsDownload {
            missing_files,
            metadata: meta,
        })
    }
}
