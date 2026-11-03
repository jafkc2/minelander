use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn get_minecraft_dir() -> String {
    match std::env::consts::OS {
        "linux" => format!("{}/.minecraft", std::env::var("HOME").unwrap()),
        "windows" => format!(
            "{}/AppData/Roaming/.minecraft",
            std::env::var("USERPROFILE").unwrap().replace('\\', "/")
        ),
        _ => panic!("System not supported."),
    }
}

pub fn get_minelander_dir() -> String {
    format!("{}/minelander", get_minecraft_dir())
}

pub fn get_instances_dir() -> PathBuf {
    PathBuf::from(get_minelander_dir()).join("instances")
}

pub fn get_instance_path(name: &str) -> Result<PathBuf> {
    let name = validate_instance_name(name)?;
    Ok(get_instances_dir().join(name))
}

pub fn get_instances() -> Result<Vec<String>> {
    let instances_dir = get_instances_dir();
    fs::create_dir_all(&instances_dir)?;

    let mut instances = fs::read_dir(instances_dir)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_dir() {
                let name = path.file_name()?.to_string_lossy().into_owned();
                (name != "Default").then_some(name)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    instances.sort();
    Ok(instances)
}

pub fn create_instance(name: &str) -> Result<()> {
    let name = validate_instance_name(name)?;
    if name == "Default" {
        anyhow::bail!("Default is reserved for the Minecraft directory");
    }
    let instances_dir = get_instances_dir();
    fs::create_dir_all(&instances_dir)?;
    let path = get_instance_path(name)?;
    if path.exists() {
        anyhow::bail!("Instance already exists: {name}");
    }
    fs::create_dir(path)?;
    Ok(())
}

pub fn delete_instance(name: &str) -> Result<()> {
    let name = validate_instance_name(name)?;
    if name == "Default" {
        anyhow::bail!("The Default instance cannot be deleted");
    }
    let path = get_instance_path(name)?;
    if !path.is_dir() {
        anyhow::bail!("Instance not found: {name}");
    }
    fs::remove_dir_all(path)?;
    Ok(())
}

fn validate_instance_name(name: &str) -> Result<&str> {
    let name = name.trim();
    if name.is_empty() || name == "." || name == ".." {
        anyhow::bail!("Instance name cannot be empty");
    }
    if name.contains('/') || name.contains('\\') {
        anyhow::bail!("Instance name cannot contain path separators");
    }
    Ok(name)
}

pub async fn get_installed_versions() -> Vec<String> {
    let versions_dir = format!("{}/versions", get_minecraft_dir());

    if !Path::new(&versions_dir).exists() {
        fs::create_dir_all(&versions_dir).unwrap();
    }
    let entries = fs::read_dir(versions_dir).unwrap();

    let mut versions = entries
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_dir() {
                Some(path.file_name().unwrap().to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    versions.sort_unstable_by(|a, b| get_version_order(b, a));
    versions
}

fn get_version_order(a: &str, b: &str) -> std::cmp::Ordering {
    let split_a: Vec<&str> = a.split(|c| c == '.' || c == '-').collect();
    let split_b: Vec<&str> = b.split(|c| c == '.' || c == '-').collect();

    if let (Some(major_a), Some(major_b)) = (
        split_a.first().and_then(|v| v.parse::<i32>().ok()),
        split_b.first().and_then(|v| v.parse::<i32>().ok()),
    ) {
        if major_a != major_b {
            return major_a.cmp(&major_b);
        }
    } else {
        return match (
            split_a.first().map(|v| v.parse::<i32>().is_ok()),
            split_b.first().map(|v| v.parse::<i32>().is_ok()),
        ) {
            (Some(true), Some(false)) => std::cmp::Ordering::Greater,
            (Some(false), Some(true)) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        };
    }

    if let (Some(minor_a), Some(minor_b)) = (
        split_a.get(1).and_then(|v| v.parse::<i32>().ok()),
        split_b.get(1).and_then(|v| v.parse::<i32>().ok()),
    ) {
        if minor_a != minor_b {
            return minor_a.cmp(&minor_b);
        }
    } else {
        return match (
            split_a.get(1).map(|v| v.parse::<i32>().is_ok()),
            split_b.get(1).map(|v| v.parse::<i32>().is_ok()),
        ) {
            (Some(true), Some(false)) => std::cmp::Ordering::Greater,
            (Some(false), Some(true)) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        };
    }

    if let (Some(release_a), Some(release_b)) = (
        split_a.get(2).and_then(|v| v.parse::<i32>().ok()),
        split_b.get(2).and_then(|v| v.parse::<i32>().ok()),
    ) {
        if release_a != release_b {
            return release_a.cmp(&release_b);
        }
    } else {
        return match (
            split_a.get(2).map(|v| v.parse::<i32>().is_ok()),
            split_b.get(2).map(|v| v.parse::<i32>().is_ok()),
        ) {
            (Some(true), Some(false)) => std::cmp::Ordering::Greater,
            (Some(false), Some(true)) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        };
    }

    split_a
        .get(3)
        .unwrap_or(&"")
        .cmp(split_b.get(3).unwrap_or(&""))
}
