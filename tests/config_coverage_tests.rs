use p_mo::config::{Config, ConfigBuilder};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_config_builder_with_all_options() {
    let config = ConfigBuilder::new()
        .with_host("127.0.0.1".to_string())
        .with_port(8080)
        .with_log_level("debug".to_string())
        .with_data_dir("/tmp/data".to_string())
        .with_pid_file("/tmp/app.pid".to_string())
        .build();
    
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 8080);
    assert_eq!(config.log_level, "debug");
    assert_eq!(config.data_dir, "/tmp/data");
    assert_eq!(config.pid_file, "/tmp/app.pid");
}

#[test]
fn test_config_to_string() {
    let config = ConfigBuilder::new()
        .with_host("127.0.0.1".to_string())
        .with_port(8080)
        .build();
    
    let config_str = config.to_string();
    assert!(config_str.contains("host: 127.0.0.1"));
    assert!(config_str.contains("port: 8080"));
}

#[test]
fn test_config_save_and_load() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    // Create a config
    let config = ConfigBuilder::new()
        .with_host("127.0.0.1".to_string())
        .with_port(8080)
        .build();
    
    // Save the config
    config.save(&config_path).unwrap();
    
    // Load the config
    let loaded_config = Config::load(&config_path).unwrap();
    
    // Verify the loaded config matches the original
    assert_eq!(loaded_config.host, config.host);
    assert_eq!(loaded_config.port, config.port);
    assert_eq!(loaded_config.log_level, config.log_level);
    assert_eq!(loaded_config.data_dir, config.data_dir);
    assert_eq!(loaded_config.pid_file, config.pid_file);
}

#[test]
fn test_config_merge() {
    // Create base config
    let base_config = ConfigBuilder::new()
        .with_host("127.0.0.1".to_string())
        .with_port(8080)
        .with_log_level("info".to_string())
        .build();
    
    // Create override config with some different values
    let override_config = ConfigBuilder::new()
        .with_port(9090)
        .with_log_level("debug".to_string())
        .build();
    
    // Merge the configs
    let merged_config = base_config.merge(&override_config);
    
    // Verify the merged config has the expected values
    assert_eq!(merged_config.host, "127.0.0.1"); // From base
    assert_eq!(merged_config.port, 9090); // From override
    assert_eq!(merged_config.log_level, "debug"); // From override
}

#[test]
fn test_config_from_env() {
    // Set environment variables
    std::env::set_var("P_MO_HOST", "192.168.1.1");
    std::env::set_var("P_MO_PORT", "9000");
    
    // Create config from environment
    let config = Config::from_env();
    
    // Verify the config has values from environment
    assert_eq!(config.host, "192.168.1.1");
    assert_eq!(config.port, 9000);
    
    // Clean up
    std::env::remove_var("P_MO_HOST");
    std::env::remove_var("P_MO_PORT");
}

#[test]
fn test_config_invalid_toml() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("invalid_config.toml");
    
    // Write invalid TOML to the file
    fs::write(&config_path, "host = 'localhost' port = 8080").unwrap();
    
    // Try to load the config
    let result = Config::load(&config_path);
    
    // Verify that loading failed
    assert!(result.is_err());
}
