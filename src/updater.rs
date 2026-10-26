use std::{env, fs::File, io::Write};

use anyhow::Result;
use reqwest::header::{HeaderValue, USER_AGENT};
use serde_json::Value;

async fn check_update() -> Result<(String, String)> {
    let last_release_request = reqwest::Client::new()
        .get("https://api.github.com/repos/jafkc2/minelander/releases/latest")
        .header(USER_AGENT, HeaderValue::from_static("minelander"))
        .send()
        .await?;

    let last_release_json: Value = last_release_request.json().await?;

    let current_version = env!("CARGO_PKG_VERSION");
    let latest_version = last_release_json["tag_name"].as_str().unwrap();

    let numeric_c_version = current_version.replace(".", "").parse::<i32>().unwrap();
    let numeric_l_version = latest_version
        .replace(".", "")
        .replace("V", "")
        .parse::<i32>()?;

    if numeric_l_version > numeric_c_version {
        let mut url = String::new();
        if let Some(release_assets) = last_release_json["assets"].as_array() {
            for i in release_assets {
                match env::consts::OS {
                    "windows" => {
                        if i["name"].as_str().unwrap().contains("Windows") {
                            url = i["browser_download_url"].as_str().unwrap().to_string();
                            break;
                        }
                    }
                    "linux" => {
                        if i["name"].as_str().unwrap().contains("Linux") {
                            url = i["browser_download_url"].as_str().unwrap().to_string();
                            break;
                        }
                    }
                    _ => panic!("System not supported."),
                }
            }
        }
        Ok((latest_version.to_owned(), url))
    } else {
        anyhow::bail!("Last version is already installed.");
    }
}

pub async fn install_update(url: String) -> Result<()> {
    let exec_path = env::current_exe()?;
    let mut exec_file = File::create(exec_path.with_extension("new"))?;

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permission = std::fs::metadata(exec_path.with_extension("new"))?.permissions();
        permission.set_mode(0o755);
        std::fs::set_permissions(exec_path.with_extension("new"), permission)?;
    }

    println!("{url}");
    let download = reqwest::get(url).await;

    match download {
        Ok(ok) => {
            exec_file.write_all(&ok.bytes().await.unwrap()).unwrap();
            Ok(())
        }
        Err(e) => {
            println!("Error downloading update: {e}");
            Err(anyhow::anyhow!("Error downloading update: {e}"))
        }
    }
}
