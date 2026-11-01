use serde_json::Value;
use std::path::{Path, PathBuf};

fn is_library_allowed(library: &Value) -> bool {
    let current_os = match std::env::consts::OS {
        "linux" => "linux",
        "windows" => "windows",
        "macos" => "osx",
        _ => return false,
    };

    if let Some(rules) = library.get("rules").and_then(Value::as_array) {
        let mut allowed = false;
        for rule in rules {
            let action = rule
                .get("action")
                .and_then(Value::as_str)
                .unwrap_or("allow")
                == "allow";
            if let Some(os_name) = rule
                .get("os")
                .and_then(|os| os.get("name"))
                .and_then(Value::as_str)
            {
                if os_name == current_os {
                    allowed = action;
                }
            } else {
                allowed = action;
            }
        }
        allowed
    } else {
        true
    }
}

pub fn maven_to_path(name: &str, native_classifier: Option<&str>) -> Option<PathBuf> {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let classifier = if parts.len() >= 4 {
        Some(parts[3])
    } else {
        native_classifier
    };

    let filename = if let Some(clf) = classifier {
        format!("{}-{}-{}.jar", artifact, version, clf)
    } else {
        format!("{}-{}.jar", artifact, version)
    };

    let mut path = PathBuf::from(group);
    path.push(artifact);
    path.push(version);
    path.push(filename);

    Some(path)
}

pub fn get_library_paths(p: &Value, mc_dir: &Path) -> Vec<PathBuf> {
    let current_os = match std::env::consts::OS {
        "linux" => "linux",
        "windows" => "windows",
        "macos" => "osx",
        _ => return Vec::new(),
    };

    let libraries_dir = mc_dir.join("libraries");
    let mut result = Vec::new();

    let libs = match p.get("libraries").and_then(Value::as_array) {
        Some(l) => l,
        None => return result,
    };

    for lib in libs {
        if !is_library_allowed(lib) {
            continue;
        }

        let name = lib.get("name").and_then(Value::as_str).unwrap_or("");
        if name == "tv.twitch:twitch-platform:6.5" {
            continue;
        }

        if let Some(artifact) = lib
            .get("downloads")
            .and_then(|downloads| downloads.get("artifact"))
            .and_then(Value::as_object)
        {
            if let Some(artifact_path) = artifact.get("path").and_then(Value::as_str) {
                result.push(libraries_dir.join(artifact_path));
                continue;
            }
        }

        let native_classifier = lib
            .get("natives")
            .and_then(|natives| natives.get(current_os))
            .and_then(Value::as_str);

        if let Some(classifier) = native_classifier {
            if let Some(clf_path) = lib
                .get("downloads")
                .and_then(|downloads| downloads.get("classifiers"))
                .and_then(|classifiers| classifiers.get(classifier))
                .and_then(|file| file.get("path"))
                .and_then(Value::as_str)
            {
                result.push(libraries_dir.join(clf_path));
                continue;
            }
        }

        if let Some(rel_path) = maven_to_path(name, native_classifier) {
            result.push(libraries_dir.join(rel_path));
        }
    }

    result
}

pub fn build_classpath(libraries: &[PathBuf], client_jar: &Path) -> String {
    let separator = match std::env::consts::OS {
        "windows" => ";",
        _ => ":",
    };

    let mut paths: Vec<String> = libraries
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    paths.push(client_jar.to_string_lossy().to_string());
    paths.join(separator)
}
