use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_secs: u64,
    pub daemon: bool,
    pub pid_file: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
}

impl ServerConfig {
    pub fn validate(&self) -> Result<(), String> {
        // Pure validation logic
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }
        
        if self.timeout_secs == 0 {
            return Err("Timeout cannot be 0".to_string());
        }
        
        if self.host.is_empty() {
            return Err("Host cannot be empty".to_string());
        }
        
        Ok(())
    }
    
    pub fn with_overrides(self, host: Option<String>, port: Option<u16>, daemon: bool) -> Self {
        Self {
            host: host.unwrap_or(self.host),
            port: port.unwrap_or(self.port),
            daemon: daemon || self.daemon,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_validation() {
        let valid_config = ServerConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_secs: 30,
            daemon: false,
            pid_file: None,
            log_file: None,
        };
        assert!(valid_config.validate().is_ok());
        
        let invalid_config = ServerConfig {
            port: 0,
            ..valid_config.clone()
        };
        assert!(invalid_config.validate().is_err());
    }
    
    #[test]
    fn test_config_overrides() {
        let base_config = ServerConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_secs: 30,
            daemon: false,
            pid_file: None,
            log_file: None,
        };
        
        let overridden = base_config.clone().with_overrides(
            Some("0.0.0.0".to_string()),
            Some(9000),
            true
        );
        
        assert_eq!(overridden.host, "0.0.0.0");
        assert_eq!(overridden.port, 9000);
        assert!(overridden.daemon);
        assert_eq!(overridden.timeout_secs, base_config.timeout_secs);
    }
}
