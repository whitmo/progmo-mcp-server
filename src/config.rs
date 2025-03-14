use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
    
    #[error("Failed to write config file: {0}")]
    WriteError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_server_config")]
    pub server: ServerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    
    #[serde(default = "default_port")]
    pub port: u16,
    
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    
    #[serde(default)]
    pub daemon: bool,
    
    #[serde(default = "default_pid_file")]
    pub pid_file: Option<PathBuf>,
    
    #[serde(default = "default_log_file")]
    pub log_file: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            timeout_secs: default_timeout_secs(),
            daemon: false,
            pid_file: default_pid_file(),
            log_file: default_log_file(),
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_pid_file() -> Option<PathBuf> {
    Some(PathBuf::from("/tmp/p-mo.pid"))
}

fn default_log_file() -> Option<PathBuf> {
    Some(PathBuf::from("/tmp/p-mo.log"))
}

fn default_server_config() -> ServerConfig {
    ServerConfig::default()
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;
        fs::write(path, content)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;
        Ok(())
    }
    
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("p-mo")
            .join("config.toml")
    }
    
    pub fn ensure_config_dir() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("p-mo");
            
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| ConfigError::WriteError(format!("Failed to create config directory: {}", e)))?;
        }
        
        Ok(config_dir)
    }
    
    pub fn create_default_config() -> Result<PathBuf, ConfigError> {
        let config_dir = Self::ensure_config_dir()?;
        let config_path = config_dir.join("config.toml");
        
        if !config_path.exists() {
            let default_config = Config::default();
            default_config.save(&config_path)?;
        }
        
        Ok(config_path)
    }
}
