use p_mo::config::{Config, ConfigError};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_config_defaults() {
    let config = Config::default();
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.timeout_secs, 30);
    assert!(!config.server.daemon);
}

#[test]
fn test_config_load_and_save() -> Result<(), ConfigError> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("test_config.toml");

    // Create a custom config
    let mut config = Config::default();
    config.server.host = "0.0.0.0".to_string();
    config.server.port = 9000;
    
    // Save it
    config.save(&config_path)?;
    
    // Load it back
    let loaded_config = Config::load(&config_path)?;
    
    // Verify values
    assert_eq!(loaded_config.server.host, "0.0.0.0");
    assert_eq!(loaded_config.server.port, 9000);
    assert_eq!(loaded_config.server.timeout_secs, 30); // Default value
    
    Ok(())
}

#[test]
fn test_config_file_not_found() {
    let nonexistent_path = PathBuf::from("/nonexistent/config.toml");
    let result = Config::load(&nonexistent_path);
    assert!(result.is_err());
    
    if let Err(e) = result {
        assert!(matches!(e, ConfigError::ReadError(_)));
    }
}

#[test]
fn test_invalid_config_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("invalid_config.toml");
    
    // Write invalid TOML
    fs::write(&config_path, "invalid { toml content").expect("Failed to write file");
    
    let result = Config::load(&config_path);
    assert!(result.is_err());
    
    if let Err(e) = result {
        assert!(matches!(e, ConfigError::ParseError(_)));
    }
}

#[test]
fn test_config_override_from_file() -> Result<(), ConfigError> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("override_config.toml");
    
    // Write a config file with custom values
    let config_content = r#"
[server]
host = "192.168.1.1"
port = 8888
timeout_secs = 60
daemon = true
"#;
    fs::write(&config_path, config_content).expect("Failed to write config file");
    
    // Load the config
    let config = Config::load(&config_path)?;
    
    // Verify overridden values
    assert_eq!(config.server.host, "192.168.1.1");
    assert_eq!(config.server.port, 8888);
    assert_eq!(config.server.timeout_secs, 60);
    assert!(config.server.daemon);
    
    Ok(())
}
