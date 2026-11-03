use super::{download_file, extract_zip, get_json};
use crate::java::JavaType;
use anyhow::{anyhow, Result};
use iced::{
    futures::{SinkExt, Stream},
    Subscription,
};
use reqwest::Client;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

const ZULU_PACKAGES_URL: &str = "https://api.azul.com/metadata/v1/zulu/packages";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    Started,
    Downloaded { bytes: u64, total: Option<u64> },
    Finished,
    Errored(String),
}

#[derive(Debug, Deserialize)]
struct ZuluPackage {
    download_url: String,
    latest: bool,
}

pub fn download(request: JavaType) -> Subscription<(JavaType, Progress)> {
    Subscription::run_with(request, build_subscription)
}

fn build_subscription(request: &JavaType) -> impl Stream<Item = (JavaType, Progress)> + 'static {
    let request = request.clone();
    iced::stream::channel(8, async move |mut output| {
        let result = run_download(&request, &mut output).await;
        if let Err(error) = result {
            let _ = output
                .send((request, Progress::Errored(format!("{error:#}"))))
                .await;
        }
    })
}

async fn run_download(
    request: &JavaType,
    output: &mut iced::futures::channel::mpsc::Sender<(JavaType, Progress)>,
) -> Result<()> {
    ensure_supported_platform()?;
    let destination = request.installation_path()?;
    if destination.is_dir() {
        send_progress(output, request, Progress::Finished).await?;
        return Ok(());
    }

    send_progress(output, request, Progress::Started).await?;
    let client = Client::new();
    let package = find_package(&client, request).await?;
    let parent = destination
        .parent()
        .ok_or_else(|| anyhow!("Java installation has no parent directory"))?;
    fs::create_dir_all(parent)?;

    let temporary = parent.join(format!(
        ".{}-download-{}",
        request.directory_name()?,
        uuid::Uuid::new_v4()
    ));
    let archive_path = temporary.join("package.zip");
    let extracted_path = temporary.join("extracted");
    let result = async {
        fs::create_dir_all(&temporary)?;
        download_file(&client, &package.download_url, &archive_path).await?;
        send_progress(
            output,
            request,
            Progress::Downloaded {
                bytes: fs::metadata(&archive_path)?.len(),
                total: None,
            },
        )
        .await?;
        extract_zip(&archive_path, &extracted_path)?;
        let source = installation_source(&extracted_path)?;
        fs::rename(source, &destination)?;
        #[cfg(unix)]
        set_as_executable(&destination)?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    if result.is_ok() {
        fs::remove_dir_all(&temporary)?;
        send_progress(output, request, Progress::Finished).await?;
    }
    result
}

async fn find_package(client: &Client, version: &JavaType) -> Result<ZuluPackage> {
    let os = match std::env::consts::OS {
        "linux" | "windows" => std::env::consts::OS,
        os => return Err(anyhow!("unsupported operating system: {os}")),
    };
    let url = format!(
        "{ZULU_PACKAGES_URL}?java_version={}&os={os}&arch=x86_64&archive_type=zip&java_package_type=jre&page=1&page_size=1",
        version.version_number()?
    );
    let packages: Vec<ZuluPackage> = get_json(client, &url).await?;
    let selected = packages
        .iter()
        .position(|package| package.latest)
        .unwrap_or(0);
    let version_number = version.version_number()?;
    packages
        .into_iter()
        .nth(selected)
        .ok_or_else(|| anyhow!("no JRE found for Java {version_number}"))
}

fn ensure_supported_platform() -> Result<()> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux" | "windows", "x86_64") => Ok(()),
        (os, arch) => Err(anyhow!("unsupported platform: {os}/{arch}")),
    }
}

fn installation_source(extracted: &Path) -> Result<PathBuf> {
    let entries = fs::read_dir(extracted)?.collect::<std::result::Result<Vec<_>, _>>()?;
    if entries.len() == 1 && entries[0].path().is_dir() {
        Ok(entries[0].path())
    } else {
        Ok(extracted.to_path_buf())
    }
}

#[cfg(unix)]
fn set_as_executable(java_dir: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let bin_dir = java_dir.join("bin");
    if !bin_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&bin_dir)? {
        let path = entry?.path();
        if path.is_file() {
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(perms.mode() | 0o111);
            fs::set_permissions(&path, perms)?;
        }
    }
    Ok(())
}

async fn send_progress(
    output: &mut iced::futures::channel::mpsc::Sender<(JavaType, Progress)>,
    request: &JavaType,
    progress: Progress,
) -> Result<()> {
    output
        .send((request.clone(), progress))
        .await
        .map_err(|_| anyhow!("download subscription was cancelled"))
}
