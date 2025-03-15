use p_mo::cli::{Cli, Command};
use p_mo::config::Config;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_cli_new() {
    let cli = Cli::new();
    // Just verify we can create a new CLI instance
    assert!(true);
}

#[test]
fn test_cli_execute_start() {
    let mut cli = Cli::new();
    
    let command = Command::Start {
        host: Some("127.0.0.1".to_string()),
        port: Some(8080),
        daemon: false,
        config_path: None,
    };
    
    let result = cli.execute(command);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "127.0.0.1:8080");
}

#[test]
fn test_cli_execute_start_with_daemon() {
    let mut cli = Cli::new();
    
    let command = Command::Start {
        host: Some("127.0.0.1".to_string()),
        port: Some(8080),
        daemon: true,
        config_path: None,
    };
    
    let result = cli.execute(command);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "127.0.0.1:8080 in daemon mode");
}

#[test]
fn test_cli_execute_start_with_config() {
    // Create a temporary directory and config file
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    // Create a config
    let mut config = Config::default();
    config.server.host = "192.168.1.1".to_string();
    config.server.port = 9090;
    
    // Save the config
    config.save(&config_path).unwrap();
    
    let mut cli = Cli::new();
    
    let command = Command::Start {
        host: None,
        port: None,
        daemon: false,
        config_path: Some(config_path),
    };
    
    let result = cli.execute(command);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "192.168.1.1:9090");
}

#[test]
fn test_cli_execute_stop() {
    let mut cli = Cli::new();
    
    // First start the server
    let start_command = Command::Start {
        host: Some("127.0.0.1".to_string()),
        port: Some(8080),
        daemon: false,
        config_path: None,
    };
    
    let _ = cli.execute(start_command);
    
    // Then stop it
    let stop_command = Command::Stop;
    let result = cli.execute(stop_command);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Server stopped");
}

#[test]
fn test_cli_execute_status_running() {
    let mut cli = Cli::new();
    
    // First start the server
    let start_command = Command::Start {
        host: Some("127.0.0.1".to_string()),
        port: Some(8080),
        daemon: false,
        config_path: None,
    };
    
    let _ = cli.execute(start_command);
    
    // Then check status
    let status_command = Command::Status;
    let result = cli.execute(status_command);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Server status: running");
}

#[test]
fn test_cli_execute_status_stopped() {
    let mut cli = Cli::new();
    
    // Check status without starting
    let status_command = Command::Status;
    let result = cli.execute(status_command);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Server status: stopped");
}

#[test]
fn test_cli_execute_init_config() {
    let mut cli = Cli::new();
    
    // Create a temporary directory for the config
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("init_config.toml");
    
    let command = Command::InitConfig {
        config_path: Some(config_path.clone()),
    };
    
    let result = cli.execute(command);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Created default configuration");
    assert!(config_path.exists());
}

#[test]
fn test_command_variants() {
    // Test that we can create all command variants
    let start_cmd = Command::Start {
        host: Some("localhost".to_string()),
        port: Some(8080),
        daemon: true,
        config_path: None,
    };
    
    let stop_cmd = Command::Stop;
    let status_cmd = Command::Status;
    
    let init_cmd = Command::InitConfig {
        config_path: Some(PathBuf::from("/tmp/config.toml")),
    };
    
    assert!(matches!(start_cmd, Command::Start { .. }));
    assert!(matches!(stop_cmd, Command::Stop));
    assert!(matches!(status_cmd, Command::Status));
    assert!(matches!(init_cmd, Command::InitConfig { .. }));
}
