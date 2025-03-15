use p_mo::config::Config;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_config_default() {
    let config = Config::default();
    
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.timeout_secs, 30);
    assert_eq!(config.server.daemon, false);
    assert_eq!(config.server.pid_file, Some(std::path::PathBuf::from("/tmp/p-mo.pid")));
    assert_eq!(config.server.log_file, Some(std::path::PathBuf::from("/tmp/p-mo.log")));
}

#[test]
fn test_config_save_and_load() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    // Create a config
    let mut config = Config::default();
    config.server.host = "192.168.1.1".to_string();
    config.server.port = 9090;
    
    // Save the config
    config.save(&config_path).unwrap();
    
    // Load the config
    let loaded_config = Config::load(&config_path).unwrap();
    
    // Verify the loaded config matches the original
    assert_eq!(loaded_config.server.host, config.server.host);
    assert_eq!(loaded_config.server.port, config.server.port);
    assert_eq!(loaded_config.server.timeout_secs, config.server.timeout_secs);
    assert_eq!(loaded_config.server.daemon, config.server.daemon);
    assert_eq!(loaded_config.server.pid_file, config.server.pid_file);
    assert_eq!(loaded_config.server.log_file, config.server.log_file);
}

#[test]
fn test_config_default_path() {
    let path = Config::default_path();
    assert!(path.to_string_lossy().contains("config.toml"));
}

#[test]
fn test_config_ensure_config_dir() {
    let result = Config::ensure_config_dir();
    assert!(result.is_ok());
    let dir = result.unwrap();
    assert!(dir.exists());
}

#[test]
fn test_config_create_default_config() {
    let result = Config::create_default_config();
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.exists());
}

#[test]
fn test_config_invalid_toml() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("invalid_config.toml");
    
    // Write invalid TOML to the file
    fs::write(&config_path, "server = { host = 'localhost' port = 8080 }").unwrap();
    
    // Try to load the config
    let result = Config::load(&config_path);
    
    // Verify that loading failed
    assert!(result.is_err());
}
