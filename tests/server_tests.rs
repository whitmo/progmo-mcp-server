#[cfg(test)]
mod server_tests {
    use p_mo::server::{Server, ServerConfig};
    use std::time::Duration;
    use reqwest::Client;

    #[tokio::test]
    async fn test_server_health_check() {
        // Create a server with test configuration
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            timeout: Duration::from_secs(30),
        };
        
        let server = Server::new(config);
        let handle = server.start().await.expect("Failed to start server");
        
        // Give the server a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Send request to health check endpoint
        let client = Client::new();
        let response = client.get("http://127.0.0.1:8080/health")
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .expect("Failed to send request");
        
        // Verify 200 OK response
        assert_eq!(response.status().as_u16(), 200);
        
        // Cleanup
        handle.shutdown().await.expect("Failed to shutdown server");
    }
}
