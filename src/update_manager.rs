use std::env;

use reqwest::header::{HeaderValue, USER_AGENT};
use serde_json::Value;

pub async fn check_launcher_updates() -> Result<(String, String), String> {
    let last_release_request = match reqwest::Client::new()
        .get("https://api.github.com/repos/jafkc2/minelander/releases/latest")
        .header(USER_AGENT, HeaderValue::from_static("minelander"))
        .send()
        .await
    {
        Ok(ok) => ok,
        Err(e) => return Err(format!("Failed to check for updates: {}", e)),
    };

    let last_release_json = match last_release_request.text().await {
        Ok(ok) => ok,
        Err(e) => return Err(format!("Failed to read response: {}", e)),
    };

    let content = serde_json::from_str(&last_release_json);

    let j: Value = match content {
        Ok(ok) => ok,
        Err(e) => return Err(format!("Failed to read json: {}", e)),
    };

    let current_version = env!("CARGO_PKG_VERSION");
    let latest_version = j["tag_name"].as_str().unwrap();

    let numeric_c_version = current_version.replace(".", "").parse::<i32>().unwrap();
    let numeric_l_version = match latest_version.replace(".", "").parse::<i32>() {
        Ok(ok) => ok,
        Err(_) => return Err("Failed to parse latest version number.".to_string()),
    };

    if numeric_l_version > numeric_c_version {
        let mut url = String::new();
        if let Some(release_assets) = j["assets"].as_array() {
            for i in release_assets {
                match env::consts::OS {
                    "windows" => {
                        if i["name"].as_str().unwrap().contains("windows") {
                            url = i["browser_download_url"].as_str().unwrap().to_string();
                            break;
                        }
                    }
                    "linux" => {
                        if i["name"].as_str().unwrap().contains("linux") {
                            url = i["browser_download_url"].as_str().unwrap().to_string();
                            break;
                        }
                    }
                    _ => panic!("System not supported."),
                }
            }
        }
        Ok((url, latest_version.to_string()))
    } else {
        Err("Your Minelander is updated!".to_string())
    }
}
