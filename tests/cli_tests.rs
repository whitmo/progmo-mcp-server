#[cfg(test)]
mod cli_tests {
    use p_mo::cli::{Cli, Command};
    use p_mo::config::Config;
    use std::time::Duration;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cli_server_control() {
        // Create CLI instance
        let mut cli = Cli::new();
        
        // Start server
        let result = cli.execute(Command::Start {
            host: Some("127.0.0.1".to_string()),
            port: Some(8081),
            daemon: false,
            config_path: None,
        });
        assert!(result.is_ok(), "Failed to start server: {:?}", result);
        
        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check server status
        let status = cli.execute(Command::Status).expect("Failed to get status");
        assert!(status.contains("running"), "Server should be running");
        
        // Stop server
        let stop_result = cli.execute(Command::Stop);
        assert!(stop_result.is_ok(), "Failed to stop server: {:?}", stop_result);
        
        // Verify server stopped
        tokio::time::sleep(Duration::from_millis(100)).await;
        let status_after = cli.execute(Command::Status).expect("Failed to get status");
        assert!(status_after.contains("stopped"), "Server should be stopped");
    }

    #[test]
    fn test_cli_config_override() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test_config.toml");
        
        // Create a config file with default values
        let config = Config::default();
        config.save(&config_path).expect("Failed to save config");
        
        let mut cli = Cli::new();
        
        // Test that CLI arguments override config values
        let result = cli.execute(Command::Start {
            host: Some("0.0.0.0".to_string()),
            port: Some(9000),
            daemon: true,
            config_path: Some(config_path),
        }).expect("Failed to execute command");
        
        assert!(result.contains("0.0.0.0:9000"));
        assert!(result.contains("daemon mode"));
    }
}
