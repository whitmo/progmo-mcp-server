#[cfg(test)]
mod api_tests {
    use p_mo::server::{Server, ServerConfig};
    use reqwest::Client;
    use serde_json::json;
    use std::time::Duration;

    #[tokio::test]
    async fn test_api_basic_operations() {
        // Start server
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8082,
            timeout: Duration::from_secs(30),
        };
        
        let server = Server::new(config);
        let handle = server.start().await.expect("Failed to start server");
        
        // Give the server a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let client = Client::new();
        
        // Test creating a knowledge entry
        let entry = json!({
            "title": "Test Entry",
            "content": "This is a test knowledge entry",
            "tags": ["test", "knowledge"]
        });
        
        let create_response = client.post("http://127.0.0.1:8082/api/knowledge")
            .json(&entry)
            .send()
            .await
            .expect("Failed to send create request");
        
        assert_eq!(create_response.status().as_u16(), 201);
        
        let entry_id: String = create_response.json().await.expect("Failed to parse response");
        
        // Test retrieving the entry
        let get_response = client.get(format!("http://127.0.0.1:8082/api/knowledge/{}", entry_id))
            .send()
            .await
            .expect("Failed to send get request");
        
        assert_eq!(get_response.status().as_u16(), 200);
        
        // Cleanup
        handle.shutdown().await.expect("Failed to shutdown server");
    }
}
