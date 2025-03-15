use p_mo::config::Config;
use p_mo::server::ServerConfig;
use std::net::TcpListener;
use std::time::Duration;
use tokio::runtime::Runtime;
use reqwest::blocking::Client;

#[test]
fn test_server_config_from_config() {
    let config = Config::default();
    let server_config = ServerConfig::from(config.server);
    
    assert_eq!(server_config.host, "127.0.0.1");
    assert_eq!(server_config.port, 8080);
    assert_eq!(server_config.timeout, Duration::from_secs(30));
    assert_eq!(server_config.daemon, false);
    assert_eq!(server_config.pid_file, Some(std::path::PathBuf::from("/tmp/p-mo.pid")));
    assert_eq!(server_config.log_file, Some(std::path::PathBuf::from("/tmp/p-mo.log")));
}

#[tokio::test]
async fn test_server_start_and_stop() {
    // Create a config with a random available port
    let port = find_available_port();
    let mut server_config = ServerConfig::default();
    server_config.port = port;
    
    // Start the server
    let server = p_mo::server::Server::new(server_config);
    let server_handle = server.start().await.expect("Failed to start server");
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Check that the server is running by making a request to the health endpoint
    let client = reqwest::Client::new();
    let response = client.get(&format!("http://127.0.0.1:{}/health", port))
        .timeout(Duration::from_secs(2))
        .send()
        .await;
    
    assert!(response.is_ok());
    if let Ok(resp) = response {
        assert!(resp.status().is_success());
        let body = resp.text().await.unwrap();
        assert_eq!(body, "OK");
    }
    
    // Stop the server
    server_handle.shutdown().await.expect("Failed to stop server");
}

#[tokio::test]
async fn test_server_handle_request() {
    // Create a config with a random available port
    let port = find_available_port();
    let mut server_config = ServerConfig::default();
    server_config.port = port;
    
    // Start the server
    let server = p_mo::server::Server::new(server_config);
    let server_handle = server.start().await.expect("Failed to start server");
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Make a request to a non-existent endpoint
    let client = reqwest::Client::new();
    let response = client.get(&format!("http://127.0.0.1:{}/nonexistent", port))
        .timeout(Duration::from_secs(2))
        .send()
        .await;
    
    assert!(response.is_ok());
    if let Ok(resp) = response {
        assert_eq!(resp.status().as_u16(), 404);
    }
    
    // Stop the server
    server_handle.shutdown().await.expect("Failed to stop server");
}

#[tokio::test]
async fn test_server_api_endpoints() {
    // Create a config with a random available port
    let port = find_available_port();
    let mut server_config = ServerConfig::default();
    server_config.port = port;
    
    // Start the server
    let server = p_mo::server::Server::new(server_config);
    let server_handle = server.start().await.expect("Failed to start server");
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Make a request to the knowledge API endpoint
    let client = reqwest::Client::new();
    let response = client.post(&format!("http://127.0.0.1:{}/api/knowledge", port))
        .timeout(Duration::from_secs(2))
        .send()
        .await;
    
    assert!(response.is_ok());
    if let Ok(resp) = response {
        assert_eq!(resp.status().as_u16(), 201);
        let body = resp.text().await.unwrap();
        assert_eq!(body, "\"test-id-123\"");
    }
    
    // Stop the server
    server_handle.shutdown().await.expect("Failed to stop server");
}

// Helper function to find an available port
fn find_available_port() -> u16 {
    // Try to bind to port 0, which will assign a random available port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}
