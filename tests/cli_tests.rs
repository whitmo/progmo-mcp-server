#[cfg(test)]
mod cli_tests {
    use p_mo::cli::{Cli, Command};
    use std::time::Duration;

    #[tokio::test]
    async fn test_cli_server_control() {
        // Create CLI instance
        let cli = Cli::new();
        
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
}
