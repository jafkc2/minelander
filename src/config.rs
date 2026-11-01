use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{accounts::AccountInfo, java::JavaType, minecraft, theme::Themes};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub path: String,

    // minelander
    pub theme: Themes,
    pub check_for_updates: bool,
    // minecraft
    pub ram_mib: u32,
    pub java: JavaType,
    pub current_instance: String,
    pub current_account: String,
    pub current_version: String,
    pub current_java: String,
    //accounts
    pub accounts: Vec<AccountInfo>,
    //system
    pub environment_variables: Vec<(String, String)>,
    pub wrapper_commands: Vec<String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let mut config: Self = serde_json::from_str(&contents)?;
        config.path = path.to_string();
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let serialized = serde_json::to_string(self)?;
        if let Some(parent) = std::path::Path::new(&self.path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, serialized)?;
        Ok(())
    }

    pub fn new() -> Self {
        println!("Creating new config at {}", default_config_path());
        Self {
            path: default_config_path(),
            theme: Themes::CatppuccinMocha,
            check_for_updates: false,
            ram_mib: 2048,
            java: JavaType::Java17,
            current_instance: String::new(),
            current_account: "".to_string(),
            current_version: "".to_string(),
            current_java: "".to_string(),
            accounts: vec![],
            environment_variables: vec![],
            wrapper_commands: vec![],
        }
    }
}

pub fn default_config_path() -> String {
    format!("{}/config.json", minecraft::get_minelander_dir())
}
