use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub log: LogConfig,
    pub db: DbConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssl: Option<DbSslConfig>,
    #[allow(dead_code)]
    pub debug: Option<bool>,
    pub minpool: Option<u32>,
    pub maxpool: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DbSslConfig {
    pub enabled: Option<bool>,
    pub verify: Option<bool>,
    pub ca_cert_file: Option<String>,
}

pub fn load() -> Result<AppConfig> {
    let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.json".to_string());
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path))?;
    let mut root: serde_json::Value =
        serde_json::from_str(&text).context("Failed to parse config.json")?;

    // Apply env var overrides using __ as path separator.
    // Example: db__host=postgres sets root["db"]["host"] = "postgres"
    // Key names are exact-case to match config.json keys.
    for (key, value) in std::env::vars() {
        let parts: Vec<&str> = key.split("__").collect();
        if parts.len() < 2 {
            continue;
        }
        apply_override(&mut root, &parts, &value);
    }

    serde_json::from_value(root).context("Failed to deserialize config")
}

fn apply_override(node: &mut serde_json::Value, parts: &[&str], value: &str) {
    if parts.is_empty() {
        return;
    }
    let key = parts[0];
    if parts.len() == 1 {
        if let Some(obj) = node.as_object_mut() {
            let new_val = if let Some(existing) = obj.get(key) {
                coerce_value(existing, value)
            } else {
                serde_json::Value::String(value.to_string())
            };
            obj.insert(key.to_string(), new_val);
        }
    } else if let Some(obj) = node.as_object_mut() {
        let child = obj
            .entry(key.to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        apply_override(child, &parts[1..], value);
    }
}

fn coerce_value(existing: &serde_json::Value, new_str: &str) -> serde_json::Value {
    match existing {
        serde_json::Value::Bool(_) => {
            serde_json::Value::Bool(new_str.eq_ignore_ascii_case("true"))
        }
        serde_json::Value::Number(n) => {
            if n.is_f64() {
                new_str
                    .parse::<f64>()
                    .ok()
                    .and_then(serde_json::Number::from_f64)
                    .map(serde_json::Value::Number)
                    .unwrap_or_else(|| serde_json::Value::String(new_str.to_string()))
            } else {
                new_str
                    .parse::<i64>()
                    .map(|v| serde_json::Value::Number(v.into()))
                    .unwrap_or_else(|_| serde_json::Value::String(new_str.to_string()))
            }
        }
        _ => serde_json::Value::String(new_str.to_string()),
    }
}
