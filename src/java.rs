use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{
    launcher::{checker, java_runtime},
    minecraft::{get_minelander_dir, get_minecraft_dir},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JavaType {
    Automatic,
    Java8,
    Java17,
    Java21,
    Java25,
    Custom,
}

impl JavaType {
    pub fn version_number(&self) -> Result<u8> {
        match self {
            Self::Java8 => Ok(8),
            Self::Java17 => Ok(17),
            Self::Java21 => Ok(21),
            Self::Java25 => Ok(25),
            _ => Err(anyhow!("Invalid JavaType"))
        }
    }

    pub fn directory_name(&self) -> Result<&'static str> {
        match self {
            Self::Java8 => Ok("java8"),
            Self::Java17 => Ok("java17"),
            Self::Java21 => Ok("java21"),
            Self::Java25 => Ok("java25"),
            _ => Err(anyhow!("Invalid JavaType"))
        }
    }

    pub fn installation_path(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(get_minelander_dir())
            .join("java")
            .join(self.directory_name()?))
    }
}

pub fn resolve_java_type(java_type: &JavaType, version: &str) -> Result<JavaType> {
    if *java_type != JavaType::Automatic {
        return Ok(java_type.clone());
    }

    let metadata = checker::load_version_metadata(Path::new(&get_minecraft_dir()), version)?;
    Ok(java_runtime::detect_required_java(&metadata.json)
        .or_else(|| {
            metadata
                .vanilla_json
                .as_ref()
                .and_then(java_runtime::detect_required_java)
        })
        .unwrap_or(JavaType::Java25))
}
