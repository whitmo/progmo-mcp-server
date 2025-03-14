use tempfile::TempDir;
use p_mo::cli::{Cli, Command};
use p_mo::config::Config;

#[tokio::test]
async fn test_cli_with_config_file() {
    // Create a temporary directory for config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("test_config.toml");
    
    // Create test config
    let mut config = Config::default();
    config.server.host = "127.0.0.1".to_string();
    config.server.port = 9999;
    config.save(&config_path).expect("Failed to save config");
    
    let mut cli = Cli::new();
    
    // Test start with config
    let result = cli.execute(Command::Start {
        host: None,
        port: None,
        daemon: false,
        config_path: Some(config_path.clone()),
    }).expect("Failed to execute start command");
    
    assert!(result.contains("127.0.0.1:9999"));
    
    // Test status shows running
    let status = cli.execute(Command::Status).expect("Failed to get status");
    assert!(status.contains("running"));
    
    // Test stop
    let stop_result = cli.execute(Command::Stop).expect("Failed to stop server");
    assert!(stop_result.contains("stopped"));
    
    // Test status shows stopped
    let final_status = cli.execute(Command::Status).expect("Failed to get final status");
    assert!(final_status.contains("stopped"));
}

#[tokio::test]
async fn test_cli_config_initialization() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("init_config.toml");
    
    let mut cli = Cli::new();
    
    // Test config initialization
    let result = cli.execute(Command::InitConfig {
        config_path: Some(config_path.clone()),
    }).expect("Failed to initialize config");
    
    assert!(result.contains("Created default configuration"));
    assert!(config_path.exists());
    
    // Verify config content
    let config = Config::load(&config_path).expect("Failed to load config");
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
}

#[tokio::test]
async fn test_cli_command_line_override() {
    // Create a temporary directory for config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("override_config.toml");
    
    // Create test config with default values
    let config = Config::default();
    config.save(&config_path).expect("Failed to save config");
    
    let mut cli = Cli::new();
    
    // Test that command line arguments override config
    let result = cli.execute(Command::Start {
        host: Some("0.0.0.0".to_string()),
        port: Some(7777),
        daemon: true,
        config_path: Some(config_path),
    }).expect("Failed to execute start command");
    
    assert!(result.contains("0.0.0.0:7777"));
    assert!(result.contains("daemon mode"));
}
