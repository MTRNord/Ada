use crate::errors::ConfigError;
use async_std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config {
    pub model_path: String,
    pub model_name: String,
}

impl Config {
    pub async fn load(filepath: String) -> Result<Self, ConfigError> {
        let contents = fs::read(filepath).await?;
        let config: Self = serde_yaml::from_slice(&contents)?;
        Ok(config)
    }
}
