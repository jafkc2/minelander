use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::minecraft::{get_instance_path, get_minecraft_dir};

pub fn mods_directory(instance: &str) -> Result<PathBuf> {
    let base = if instance.is_empty() || instance == "Default" {
        PathBuf::from(get_minecraft_dir())
    } else {
        get_instance_path(instance)?
    };
    Ok(base.join("mods"))
}

pub fn list(instance: &str) -> Result<Vec<String>> {
    let directory = mods_directory(instance)?;
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut mods = fs::read_dir(directory)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() && is_mod_file(&path) {
                Some(path.file_name()?.to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    mods.sort();
    Ok(mods)
}

pub fn add(instance: &str, source: &Path) -> Result<()> {
    if !is_mod_file(source) {
        bail!("Selected file is not a supported mod file (.jar or .zip)");
    }
    if !source.is_file() {
        bail!("Selected mod file does not exist");
    }

    let directory = mods_directory(instance)?;
    fs::create_dir_all(&directory)?;
    let file_name = source
        .file_name()
        .context("Selected mod file has no file name")?;
    fs::copy(source, directory.join(file_name))?;
    Ok(())
}

pub fn remove(instance: &str, file_name: &str) -> Result<()> {
    let file_name = Path::new(file_name)
        .file_name()
        .filter(|name| *name == file_name)
        .context("Invalid mod file name")?;
    let path = mods_directory(instance)?.join(file_name);
    if !path.is_file() {
        bail!("Mod file not found");
    }
    fs::remove_file(path)?;
    Ok(())
}

fn is_mod_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("jar" | "zip")
    )
}
