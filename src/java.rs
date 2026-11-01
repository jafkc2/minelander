use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::minecraft::get_minelander_dir;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JavaType {
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
            Self::Custom => Err(anyhow!("custom Java installations are not managed")),
        }
    }

    pub fn directory_name(&self) -> Result<&'static str> {
        match self {
            Self::Java8 => Ok("java8"),
            Self::Java17 => Ok("java17"),
            Self::Java21 => Ok("java21"),
            Self::Java25 => Ok("java25"),
            Self::Custom => Err(anyhow!("custom Java installations are not managed")),
        }
    }

    pub fn installation_path(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(get_minelander_dir())
            .join("java")
            .join(self.directory_name()?))
    }
}
