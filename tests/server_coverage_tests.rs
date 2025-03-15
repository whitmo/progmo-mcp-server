use p_mo::config::ConfigBuilder;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use reqwest::blocking::Client;

#[test]
fn test_server_start_and_stop() {
    // Create a config with a random available port
    let port = find_available_port();
    let config = ConfigBuilder::new()
        .with_host("127.0.0.1".to_string())
        .with_port(port)
        .build();
    
    // Start the server in a separate thread
    let config_clone = config.clone();
    let server_thread = thread::spawn(move || {
        let server = p_mo::server::Server::new(config_clone);
        let _ = server.start();
    });
    
    // Give the server time to start
    thread::sleep(Duration::from_millis(500));
    
    // Check that the server is running by making a request to the health endpoint
    let client = Client::new();
    let response = client.get(&format!("http://127.0.0.1:{}/health", port))
        .timeout(Duration::from_secs(2))
        .send();
    
    assert!(response.is_ok());
    if let Ok(resp) = response {
        assert!(resp.status().is_success());
        let body = resp.text().unwrap();
        assert!(body.contains("status"));
        assert!(body.contains("ok"));
    }
    
    // Stop the server thread
    // In a real scenario, we would call server.stop(), but for this test
    // we'll just let the thread terminate when the test ends
}

#[test]
fn test_server_handle_request() {
    // Create a config with a random available port
    let port = find_available_port();
    let config = ConfigBuilder::new()
        .with_host("127.0.0.1".to_string())
        .with_port(port)
        .build();
    
    // Start the server in a separate thread
    let config_clone = config.clone();
    let server_thread = thread::spawn(move || {
        let server = p_mo::server::Server::new(config_clone);
        let _ = server.start();
    });
    
    // Give the server time to start
    thread::sleep(Duration::from_millis(500));
    
    // Make a request to a non-existent endpoint
    let client = Client::new();
    let response = client.get(&format!("http://127.0.0.1:{}/nonexistent", port))
        .timeout(Duration::from_secs(2))
        .send();
    
    assert!(response.is_ok());
    if let Ok(resp) = response {
        assert_eq!(resp.status().as_u16(), 404);
    }
}

#[test]
fn test_server_config_endpoint() {
    // Create a config with a random available port
    let port = find_available_port();
    let config = ConfigBuilder::new()
        .with_host("127.0.0.1".to_string())
        .with_port(port)
        .with_log_level("debug".to_string())
        .build();
    
    // Start the server in a separate thread
    let config_clone = config.clone();
    let server_thread = thread::spawn(move || {
        let server = p_mo::server::Server::new(config_clone);
        let _ = server.start();
    });
    
    // Give the server time to start
    thread::sleep(Duration::from_millis(500));
    
    // Make a request to the config endpoint
    let client = Client::new();
    let response = client.get(&format!("http://127.0.0.1:{}/config", port))
        .timeout(Duration::from_secs(2))
        .send();
    
    assert!(response.is_ok());
    if let Ok(resp) = response {
        assert!(resp.status().is_success());
        let body = resp.text().unwrap();
        assert!(body.contains("host"));
        assert!(body.contains("127.0.0.1"));
        assert!(body.contains("port"));
        assert!(body.contains(&port.to_string()));
        assert!(body.contains("log_level"));
        assert!(body.contains("debug"));
    }
}

// Helper function to find an available port
fn find_available_port() -> u16 {
    // Try to bind to port 0, which will assign a random available port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}
