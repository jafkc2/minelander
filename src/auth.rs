use iced::subscription;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::hash::Hash;

const AZURE_CLIENT_ID: &str = "7f8e9d75-ca8f-4603-b2ab-ae7fc0f871d9";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MinecraftAccount {
    pub username: String,
    pub token: String,
    pub uuid: String,
}


#[derive(Clone, Debug, Default)]
pub struct AuthCode {
    pub code: String,
    pub link: String,
    pub device_code: String,
}

#[derive(Debug, Clone, Default)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Clone, Debug, Default)]
pub struct XboxLiveData {
    user_hash: String,
    xsts_token: String,
}
// Login process

pub async fn request_code() -> AuthCode {
    let client = Client::new();
    let response = match client
        .get("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .query(&[
            ("client_id", AZURE_CLIENT_ID),
            ("scope", &"XboxLive.signin offline_access".to_string()),
        ])
        .send()
        .await
    {
        Ok(ok) => ok.text().await.unwrap(),
        Err(e) => panic!("{e}"),
    };

    let response_json: Value = serde_json::from_str(&response).unwrap();

    let code = response_json["user_code"].as_str().unwrap().to_owned();
    let link = response_json["verification_uri"]
        .as_str()
        .unwrap()
        .to_owned();

    let device_code = response_json["device_code"].as_str().unwrap().to_owned();

    AuthCode {
        code,
        link,
        device_code,
    }
}

#[derive(Debug, Clone)]
pub enum WaitProgress {
    GotAuthToken(AuthToken),
    Waiting,
    Error(String),
    Finished,
}

pub enum WaitState {
    Waiting(Client, String),
    Finished,
}

pub fn start_wait_for_login<I: 'static + Hash + Copy + Send + Sync>(
    id: I,
    device_code: String,
) -> iced::Subscription<(I, WaitProgress)> {
    subscription::unfold(
        id,
        WaitState::Waiting(Client::new(), device_code),
        move |state| wait_for_login(id, state),
    )
}

pub async fn wait_for_login<Id: Copy>(id: Id, state: WaitState) -> ((Id, WaitProgress), WaitState) {
    match state {
        WaitState::Waiting(client, device_code) => {
            let response = match client
                .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
                .form(&[
                    ("client_id", AZURE_CLIENT_ID),
                    ("scope", &"XboxLive.signin offline_access".to_string()),
                    (
                        "grant_type",
                        &"urn:ietf:params:oauth:grant-type:device_code".to_string(),
                    ),
                    ("device_code", &device_code),
                ])
                .send()
                .await
            {
                Ok(ok) => ok,
                Err(e) => panic!("{e}"),
            };

            match response.status() {
                StatusCode::OK => {
                    let response_json: Value =
                        serde_json::from_str(&response.text().await.unwrap()).unwrap();

                    let access_token = response_json["access_token"].as_str().unwrap().to_string();
                    let refresh_token =
                        response_json["refresh_token"].as_str().unwrap().to_string();

                    let token = AuthToken {
                        access_token,
                        refresh_token,
                    };

                    ((id, WaitProgress::GotAuthToken(token)), WaitState::Finished)
                }

                _ => (
                    (id, WaitProgress::Waiting),
                    WaitState::Waiting(client, device_code),
                ),
            }
        }
        WaitState::Finished => ((id, WaitProgress::Finished), WaitState::Finished),
    }
}

pub async fn login_to_xbox(access_token: String) -> XboxLiveData {
    let client = Client::new();

    // Xbox live
    let xbox_live_response_request_data = json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": &format!("d={}", access_token)
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let xbox_live_response = match client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&xbox_live_response_request_data)
        .send()
        .await
    {
        Ok(ok) => ok.text().await.unwrap(),
        Err(e) => panic!("{e}"),
    };

    let xbox_live_response_json: Value = serde_json::from_str(&xbox_live_response).unwrap();

    let xbox_live_token = xbox_live_response_json["Token"]
        .as_str()
        .unwrap()
        .to_owned();

    let user_hash = xbox_live_response_json["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .unwrap()
        .to_owned();

    // Xsts

    let xbox_xsts_response_request_data = json!(
        {
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [
                    xbox_live_token
                ]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
         }
    );

    let xbox_xsts_response = match client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&xbox_xsts_response_request_data)
        .send()
        .await
    {
        Ok(ok) => ok.text().await.unwrap(),
        Err(e) => panic!("{e}"),
    };

    let xbox_xsts_response_json: Value = serde_json::from_str(&xbox_xsts_response).unwrap();

    let xsts_token = xbox_xsts_response_json["Token"]
        .as_str()
        .unwrap()
        .to_owned();

    XboxLiveData {
        xsts_token,
        user_hash,
    }
}

pub async fn login_to_minecraft(xbox_data: XboxLiveData) -> MinecraftAccount {
    let client = Client::new();

    // Getting token
    let minecraft_data_response_request_data = json!(
        {
        "identityToken": format!("XBL3.0 x={};{}", xbox_data.user_hash, xbox_data.xsts_token)
        }
    );

    let minecraft_data_response = match client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&minecraft_data_response_request_data)
        .send()
        .await
    {
        Ok(ok) => ok.text().await.unwrap(),
        Err(e) => panic!("{e}"),
    };

    let minecraft_data_json: Value = serde_json::from_str(&minecraft_data_response).unwrap();

    let token = minecraft_data_json["access_token"]
        .as_str()
        .unwrap()
        .to_owned();

    // Getting username and uuid

    let mc_profile_response = match client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(token.clone())
        .send()
        .await
    {
        Ok(ok) => ok.text().await.unwrap(),
        Err(e) => panic!("{e}"),
    };

    let mc_profile_json: Value = serde_json::from_str(&mc_profile_response).unwrap();
    let uuid = mc_profile_json["id"].as_str().unwrap().to_owned();
    let username = mc_profile_json["name"].as_str().unwrap().to_owned();

    MinecraftAccount {
        username,
        token,
        uuid,
    }
}

// All in one, using a refresh_token. Used when launching the game.
pub async fn login_with_refresh_token(refresh_token: String) -> Option<MinecraftAccount> {
    let client = Client::new();

    let response = match client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&[
            ("client_id", AZURE_CLIENT_ID),
            ("scope", &"XboxLive.signin offline_access".to_string()),
            ("grant_type", &"refresh_token".to_string()),
            ("refresh_token", &refresh_token),
        ])
        .send()
        .await
    {
        Ok(ok) => ok.text().await.unwrap(),
        Err(_) => return None,
    };

    let response_json: Value = serde_json::from_str(&response).unwrap();

    let access_token = response_json["access_token"].as_str().unwrap().to_owned();

    let xbox_data = login_to_xbox(access_token).await;

    Some(login_to_minecraft(xbox_data).await)
}
