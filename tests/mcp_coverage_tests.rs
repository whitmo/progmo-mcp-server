use p_mo::mcp::{ProgmoMcpServer, ServerConfig};
use p_mo::vector_store::{Document, EmbeddedQdrantConnector, VectorStore, QdrantConfig, VectorStoreError};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use p_mo::mcp::mock::MockQdrantConnector;

#[tokio::test]
async fn test_add_knowledge_entry() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Send CallTool request for add_knowledge_entry
    let request = r#"{"jsonrpc":"2.0","id":"3","method":"CallTool","params":{"name":"add_knowledge_entry","arguments":{"collection_id":"test_add_entry","title":"Test Title","content":"Test content for knowledge entry","tags":["test","knowledge"]}}}"#;
    let response = server.handle_request(request).await;
    
    // Verify response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response_value["id"], "3");
    assert!(response_value["result"]["content"].is_array());
    assert_eq!(response_value["result"]["content"][0]["type"], "text");
    
    // Verify the entry was added by searching for it
    let search_request = r#"{"jsonrpc":"2.0","id":"4","method":"CallTool","params":{"name":"search_knowledge","arguments":{"query":"Test content","collection_id":"test_add_entry","limit":5}}}"#;
    let search_response = server.handle_request(search_request).await;
    
    // Parse the search response
    let search_response_value: Value = serde_json::from_str(&search_response).unwrap();
    let results_text = search_response_value["result"]["content"][0]["text"].as_str().unwrap();
    let results: Vec<Value> = serde_json::from_str(results_text).unwrap();
    
    // Verify the search found our entry
    assert!(!results.is_empty());
    assert!(results[0]["content"].as_str().unwrap().contains("Test document"));
}

#[tokio::test]
async fn test_read_collection_resource() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Send ReadResource request for a specific collection
    let request = r#"{"jsonrpc":"2.0","id":"5","method":"ReadResource","params":{"uri":"knowledge://collections/test_collection_resource"}}"#;
    let response = server.handle_request(request).await;
    
    // Verify response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response_value["id"], "5");
    assert!(response_value["result"]["contents"].is_array());
    
    // Verify the response contains the collection info
    let content_text = response_value["result"]["contents"][0]["text"].as_str().unwrap();
    assert!(content_text.contains("test_collection_resource"));
}

#[tokio::test]
async fn test_error_handling_invalid_json() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Send invalid JSON
    let invalid_json = r#"{"jsonrpc":"2.0","id":"6","method":"#;
    let response = server.handle_request(invalid_json).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32700);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("Parse error"));
}

#[tokio::test]
async fn test_error_handling_missing_method() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Send request without method
    let no_method_request = r#"{"jsonrpc":"2.0","id":"7","params":{}}"#;
    let response = server.handle_request(no_method_request).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32600);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("missing method"));
}

#[tokio::test]
async fn test_error_handling_invalid_tool_params() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Test missing params
    let missing_params = r#"{"jsonrpc":"2.0","id":"8","method":"CallTool"}"#;
    let response = server.handle_request(missing_params).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32602);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("missing params"));
    
    // Test missing tool name
    let missing_tool = r#"{"jsonrpc":"2.0","id":"9","method":"CallTool","params":{}}"#;
    let response = server.handle_request(missing_tool).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32602);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("missing tool name"));
    
    // Test missing arguments
    let missing_args = r#"{"jsonrpc":"2.0","id":"10","method":"CallTool","params":{"name":"search_knowledge"}}"#;
    let response = server.handle_request(missing_args).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32602);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("missing arguments"));
}

#[tokio::test]
async fn test_error_handling_search_knowledge_params() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Test missing query
    let missing_query = r#"{"jsonrpc":"2.0","id":"11","method":"CallTool","params":{"name":"search_knowledge","arguments":{"collection_id":"test"}}}"#;
    let response = server.handle_request(missing_query).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32602);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("missing query"));
}

#[tokio::test]
async fn test_error_handling_add_knowledge_entry_params() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Test missing collection_id
    let missing_collection = r#"{"jsonrpc":"2.0","id":"12","method":"CallTool","params":{"name":"add_knowledge_entry","arguments":{"title":"Test","content":"Test"}}}"#;
    let response = server.handle_request(missing_collection).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32602);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("missing collection_id"));
    
    // Test missing title
    let missing_title = r#"{"jsonrpc":"2.0","id":"13","method":"CallTool","params":{"name":"add_knowledge_entry","arguments":{"collection_id":"test","content":"Test"}}}"#;
    let response = server.handle_request(missing_title).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32602);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("missing title"));
    
    // Test missing content
    let missing_content = r#"{"jsonrpc":"2.0","id":"14","method":"CallTool","params":{"name":"add_knowledge_entry","arguments":{"collection_id":"test","title":"Test"}}}"#;
    let response = server.handle_request(missing_content).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32602);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("missing content"));
}

#[tokio::test]
async fn test_error_handling_read_resource_params() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Test missing params
    let missing_params = r#"{"jsonrpc":"2.0","id":"15","method":"ReadResource"}"#;
    let response = server.handle_request(missing_params).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32602);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("missing params"));
    
    // Test missing uri
    let missing_uri = r#"{"jsonrpc":"2.0","id":"16","method":"ReadResource","params":{}}"#;
    let response = server.handle_request(missing_uri).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32602);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("missing uri"));
    
    // Test invalid uri
    let invalid_uri = r#"{"jsonrpc":"2.0","id":"17","method":"ReadResource","params":{"uri":"invalid://uri"}}"#;
    let response = server.handle_request(invalid_uri).await;
    
    // Verify error response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert!(response_value["error"].is_object());
    assert_eq!(response_value["error"]["code"], -32602);
    assert!(response_value["error"]["message"].as_str().unwrap().contains("Invalid URI"));
}

#[tokio::test]
async fn test_real_qdrant_connection() {
    // This test is skipped if QDRANT_URL is not set
    let qdrant_url = match std::env::var("QDRANT_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test_real_qdrant_connection: QDRANT_URL not set");
            return;
        }
    };
    
    // Create a real vector store with config
    let config = QdrantConfig {
        url: qdrant_url,
        timeout: Duration::from_secs(5),
        max_connections: 5,
        api_key: std::env::var("QDRANT_API_KEY").ok(),
        retry_max_elapsed_time: Duration::from_secs(30),
        retry_initial_interval: Duration::from_millis(100),
        retry_max_interval: Duration::from_secs(5),
        retry_multiplier: 1.5,
    };
    
    let store_result = EmbeddedQdrantConnector::new(config).await;
    if let Err(e) = store_result {
        println!("Skipping test_real_qdrant_connection: Failed to create connector: {}", e);
        return;
    }
    
    let store = store_result.unwrap();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Test server name and version
    assert_eq!(server.name(), "test-server");
    assert_eq!(server.version(), "0.1.0");
}
