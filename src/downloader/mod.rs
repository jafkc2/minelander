use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::{fs, io, path::Path};

pub mod java_downloader;
pub mod version_downloader;

pub(crate) async fn get_json<T: DeserializeOwned>(client: &Client, url: &str) -> Result<T> {
    Ok(client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

pub(crate) async fn download_file(client: &Client, url: &str, path: &Path) -> Result<()> {
    let bytes = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    write_file(path, &bytes)
}

pub(crate) fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}

pub(crate) fn extract_zip(archive_path: &Path, target: &Path) -> Result<()> {
    let file = fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    fs::create_dir_all(target)?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| anyhow!("ZIP entry has an unsafe path"))?
            .to_path_buf();
        let output = target.join(relative);

        if entry.is_dir() {
            fs::create_dir_all(&output)?;
            continue;
        }

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output_file = fs::File::create(output)?;
        io::copy(&mut entry, &mut output_file)?;
    }
    Ok(())
}
