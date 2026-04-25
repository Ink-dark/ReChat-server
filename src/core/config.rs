use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub database: DatabaseConfig,
    pub sender: SenderConfig,
    pub adapters: Vec<AdapterConfig>,
    pub plugins: Vec<PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub web_ui: bool,
    pub web_ui_port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub queue_name: String,
    pub max_connections: usize,
    pub connection_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
    pub max_connections: usize,
    pub connection_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SenderConfig {
    pub max_retries: u32,
    pub retry_interval: u64, // in seconds
    pub batch_size: usize,
    pub concurrency: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdapterConfig {
    pub name: String,
    pub type_: String, // qq, wechat, telegram, etc.
    pub enabled: bool,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: 4,
                web_ui: true,
                web_ui_port: 8081,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                queue_name: "rechat_messages".to_string(),
                max_connections: 10,
                connection_timeout: 5,
            },
            database: DatabaseConfig {
                path: "./rechat.db".to_string(),
                max_connections: 5,
                connection_timeout: 3,
            },
            sender: SenderConfig {
                max_retries: 3,
                retry_interval: 5,
                batch_size: 10,
                concurrency: 5,
            },
            adapters: vec![],
            plugins: vec![],
        }
    }
}
