use anyhow::{anyhow, Result};
use minecraft_msa_auth::{MinecraftAuthenticationResponse, MinecraftAuthorizationFlow};
use oauth2::{
    basic::BasicClient, reqwest::Client as OAuthHttpClient, AuthUrl, ClientId,
    DeviceAuthorizationUrl, EndpointNotSet, EndpointSet, RefreshToken, Scope,
    StandardDeviceAuthorizationResponse, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    time::{Duration, Instant},
};

use crate::minecraft::get_minelander_dir;

const AZURE_CLIENT_ID: &str = "7f8e9d75-ca8f-4603-b2ab-ae7fc0f871d9";
const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const MSA_AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MSA_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const MSA_SCOPE: &str = "XboxLive.signin offline_access";

fn accounts_file_path() -> String {
    format!("{}/accounts.json", get_minelander_dir())
}

fn load_accounts_file() -> HashMap<String, String> {
    let path = accounts_file_path();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

fn save_accounts_file(accounts: &HashMap<String, String>) -> Result<()> {
    let dir = get_minelander_dir();
    fs::create_dir_all(&dir)?;
    let path = accounts_file_path();
    let contents = serde_json::to_string_pretty(accounts)?;
    fs::write(&path, contents)?;
    Ok(())
}

pub fn save_refresh_token(username: &str, refresh_token: &str) -> Result<()> {
    let mut accounts = load_accounts_file();
    accounts.insert(username.to_string(), refresh_token.to_string());
    save_accounts_file(&accounts)
}

pub fn get_refresh_token(username: &str) -> Result<String> {
    let accounts = load_accounts_file();
    accounts
        .get(username)
        .cloned()
        .ok_or_else(|| anyhow!("no refresh token stored for account '{username}'"))
}

pub fn delete_refresh_token(username: &str) -> Result<()> {
    let mut accounts = load_accounts_file();
    accounts.remove(username);
    save_accounts_file(&accounts)
}

#[derive(Debug, Clone)]
pub struct DeviceCode {
    pub verification_uri: String,
    pub user_code: String,
    pub expires_in: std::time::Duration,
    pub interval: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct DeviceFlow {
    client: reqwest::Client,
    device_code: String,
    interval: Duration,
    expires_at: Instant,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedAccount {
    pub minecraft_username: String,
    pub uuid: String,
    pub minecraft_access_token: String,
    pub minecraft_token_expires_in: u32,
    pub refresh_token: String,
}

#[derive(Debug, Clone)]
pub enum AuthState {
    Code(DeviceCode),
    Waiting(Duration),
    Authenticated(AuthenticatedAccount),
    Error(String),
    Finished,
}

type MicrosoftClient =
    BasicClient<EndpointSet, EndpointSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

fn microsoft_client() -> Result<MicrosoftClient> {
    Ok(BasicClient::new(ClientId::new(AZURE_CLIENT_ID.to_owned()))
        .set_auth_uri(AuthUrl::new(MSA_AUTHORIZE_URL.to_owned())?)
        .set_token_uri(TokenUrl::new(MSA_TOKEN_URL.to_owned())?)
        .set_device_authorization_url(DeviceAuthorizationUrl::new(DEVICE_CODE_URL.to_owned())?))
}

#[derive(Debug, Deserialize)]
struct DeviceTokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

fn form_component(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                vec![byte as char]
            }
            byte => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

pub async fn begin_device_flow() -> Result<(DeviceCode, DeviceFlow)> {
    let client = microsoft_client()?;
    let http_client = OAuthHttpClient::new();
    let details: StandardDeviceAuthorizationResponse = client
        .exchange_device_code()
        .add_scope(Scope::new(MSA_SCOPE.to_owned()))
        .request_async(&http_client)
        .await?;

    let code = DeviceCode {
        verification_uri: details.verification_uri().to_string(),
        user_code: details.user_code().secret().to_owned(),
        expires_in: details.expires_in(),
        interval: details.interval(),
    };

    let flow = DeviceFlow {
        client: reqwest::Client::new(),
        device_code: details.device_code().secret().to_owned(),
        interval: details.interval(),
        expires_at: Instant::now() + details.expires_in(),
    };

    Ok((code, flow))
}

pub async fn poll_device_flow(flow: &mut DeviceFlow) -> Result<AuthState> {
    if Instant::now() >= flow.expires_at {
        return Err(anyhow!("Microsoft device code has expired"));
    }

    let response = flow
        .client
        .post(MSA_TOKEN_URL)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(format!(
            "grant_type={}&client_id={}&device_code={}",
            form_component("urn:ietf:params:oauth:grant-type:device_code"),
            form_component(AZURE_CLIENT_ID),
            form_component(&flow.device_code),
        ))
        .send()
        .await?;
    let status = response.status();
    let token: DeviceTokenResponse = response.json().await?;

    if let Some(error) = token.error.as_deref() {
        match error {
            "authorization_pending" => {
                return Ok(AuthState::Waiting(flow.interval));
            }
            "slow_down" => {
                flow.interval += Duration::from_secs(5);
                return Ok(AuthState::Waiting(flow.interval));
            }
            "access_denied" => return Err(anyhow!("Microsoft device authorization was denied")),
            "expired_token" => return Err(anyhow!("Microsoft device code has expired")),
            _ => {
                let description = token.error_description.as_deref().unwrap_or(error);
                return Err(anyhow!("Microsoft token request failed: {description}"));
            }
        }
    }

    if !status.is_success() {
        return Err(anyhow!(
            "Microsoft token request failed with status {status}"
        ));
    }

    let access_token = token
        .access_token
        .ok_or_else(|| anyhow!("Microsoft response did not contain an access token"))?;
    let refresh_token = token
        .refresh_token
        .ok_or_else(|| anyhow!("Microsoft did not return a refresh token"))?;

    Ok(AuthState::Authenticated(
        minecraft_account(&access_token, refresh_token).await?,
    ))
}

pub async fn authenticate_with_refresh_token(refresh_token: &str) -> Result<AuthenticatedAccount> {
    if refresh_token.is_empty() {
        return Err(anyhow!("refresh token is empty"));
    }

    let client = microsoft_client()?;
    let http_client = OAuthHttpClient::new();
    let token = client
        .exchange_refresh_token(&RefreshToken::new(refresh_token.to_owned()))
        .add_scope(Scope::new(MSA_SCOPE.to_owned()))
        .request_async(&http_client)
        .await?;
    let next_refresh_token = token
        .refresh_token()
        .map(|value| value.secret().to_owned())
        .unwrap_or_else(|| refresh_token.to_owned());

    minecraft_account(token.access_token().secret(), next_refresh_token).await
}

pub async fn poll_device_flow_until_complete(mut flow: DeviceFlow) -> Result<AuthenticatedAccount> {
    loop {
        match poll_device_flow(&mut flow).await? {
            AuthState::Code(_) => {}
            AuthState::Waiting(interval) => {
                tokio::time::sleep(interval).await;
            }
            AuthState::Authenticated(account) => {
                return Ok(account);
            }
            AuthState::Error(err) => return Err(anyhow!("{err}")),
            AuthState::Finished => return Err(anyhow!("Device flow finished unexpectedly")),
        }
    }
}

#[derive(Debug, Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
}

async fn minecraft_account(
    microsoft_access_token: &str,
    refresh_token: String,
) -> Result<AuthenticatedAccount> {
    let http = reqwest::Client::new();
    let flow = MinecraftAuthorizationFlow::new(http.clone());
    let account: MinecraftAuthenticationResponse = flow
        .exchange_microsoft_token(microsoft_access_token)
        .await?;

    let profile_response = http
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(account.access_token().as_ref())
        .send()
        .await?;

    if !profile_response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch Minecraft profile (HTTP {}). Ensure this account owns Minecraft.",
            profile_response.status()
        ));
    }

    let profile: MinecraftProfile = profile_response.json().await?;
    let uuid = uuid::Uuid::parse_str(&profile.id)
        .map(|parsed| parsed.to_string())
        .unwrap_or(profile.id);

    Ok(AuthenticatedAccount {
        minecraft_username: profile.name,
        uuid,
        minecraft_access_token: account.access_token().as_ref().to_owned(),
        minecraft_token_expires_in: account.expires_in(),
        refresh_token,
    })
}
