use anyhow::{Context, Result};
use hs_utils::config::{apply_env_overrides, deser_u16_or_str, prepare_config};
use hs_utils::db::DbConfig;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub log: LogConfig,
    pub db: DbConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(deserialize_with = "deser_u16_or_str")]
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
}

pub fn load() -> Result<AppConfig> {
    let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.json".to_string());
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path))?;
    let mut root: Value =
        serde_json::from_str(&text).context("Failed to parse config.json")?;
    prepare_config(&mut root);
    apply_env_overrides(&mut root);
    serde_json::from_value(root).context("Failed to deserialize config")
}
