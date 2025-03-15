use p_mo::cli::{Command, CommandArgs, CommandResult};
use p_mo::config::{Config, ConfigBuilder};
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_command_args_new() {
    let args = CommandArgs::new(
        Some("start".to_string()),
        Some("127.0.0.1".to_string()),
        Some(8080),
        Some("debug".to_string()),
        Some("/tmp/data".to_string()),
        Some("/tmp/app.pid".to_string()),
        Some("/tmp/config.toml".to_string()),
    );
    
    assert_eq!(args.command, Some("start".to_string()));
    assert_eq!(args.host, Some("127.0.0.1".to_string()));
    assert_eq!(args.port, Some(8080));
    assert_eq!(args.log_level, Some("debug".to_string()));
    assert_eq!(args.data_dir, Some("/tmp/data".to_string()));
    assert_eq!(args.pid_file, Some("/tmp/app.pid".to_string()));
    assert_eq!(args.config_file, Some("/tmp/config.toml".to_string()));
}

#[test]
fn test_command_args_to_config() {
    let args = CommandArgs::new(
        None,
        Some("127.0.0.1".to_string()),
        Some(8080),
        Some("debug".to_string()),
        Some("/tmp/data".to_string()),
        Some("/tmp/app.pid".to_string()),
        None,
    );
    
    let config = args.to_config();
    
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 8080);
    assert_eq!(config.log_level, "debug");
    assert_eq!(config.data_dir, "/tmp/data");
    assert_eq!(config.pid_file, "/tmp/app.pid");
}

#[test]
fn test_command_args_empty() {
    let args = CommandArgs::new(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    
    let config = args.to_config();
    
    // Should use default values
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 3000);
    assert_eq!(config.log_level, "info");
    assert!(config.data_dir.contains("data"));
    assert!(config.pid_file.contains("app.pid"));
}

#[test]
fn test_command_parse_start() {
    let args = CommandArgs::new(
        Some("start".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    
    let command = Command::parse(&args);
    
    match command {
        Command::Start => {}
        _ => panic!("Expected Start command"),
    }
}

#[test]
fn test_command_parse_stop() {
    let args = CommandArgs::new(
        Some("stop".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    
    let command = Command::parse(&args);
    
    match command {
        Command::Stop => {}
        _ => panic!("Expected Stop command"),
    }
}

#[test]
fn test_command_parse_status() {
    let args = CommandArgs::new(
        Some("status".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    
    let command = Command::parse(&args);
    
    match command {
        Command::Status => {}
        _ => panic!("Expected Status command"),
    }
}

#[test]
fn test_command_parse_init_config() {
    let args = CommandArgs::new(
        Some("init-config".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    
    let command = Command::parse(&args);
    
    match command {
        Command::InitConfig => {}
        _ => panic!("Expected InitConfig command"),
    }
}

#[test]
fn test_command_parse_unknown() {
    let args = CommandArgs::new(
        Some("unknown".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    
    let command = Command::parse(&args);
    
    match command {
        Command::Unknown => {}
        _ => panic!("Expected Unknown command"),
    }
}

#[test]
fn test_command_parse_none() {
    let args = CommandArgs::new(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    
    let command = Command::parse(&args);
    
    match command {
        Command::Unknown => {}
        _ => panic!("Expected Unknown command"),
    }
}

#[test]
fn test_command_result_display() {
    let success_result = CommandResult::Success("Command executed successfully".to_string());
    let error_result = CommandResult::Error("Command failed".to_string());
    
    assert_eq!(format!("{}", success_result), "Command executed successfully");
    assert_eq!(format!("{}", error_result), "Error: Command failed");
}
