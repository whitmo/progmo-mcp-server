use p_mo::mcp::{mock::MockQdrantConnector, ProgmoMcpServer, ServerConfig};
use serde_json::Value;
use std::sync::Arc;

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
async fn test_delete_knowledge_entry() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Send CallTool request for delete_knowledge_entry
    let request = r#"{"jsonrpc":"2.0","id":"11","method":"CallTool","params":{"name":"delete_knowledge_entry","arguments":{"collection_id":"test_collection","entry_id":"test-id-123"}}}"#;
    let response = server.handle_request(request).await;
    
    // Verify response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response_value["id"], "11");
    assert!(response_value["result"]["content"].is_array());
    assert_eq!(response_value["result"]["content"][0]["type"], "text");
    
    // Verify the entry was deleted
    let text = response_value["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Deleted entry with ID: test-id-123"));
}

#[tokio::test]
async fn test_update_knowledge_entry() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Send CallTool request for update_knowledge_entry
    let request = r#"{"jsonrpc":"2.0","id":"12","method":"CallTool","params":{"name":"update_knowledge_entry","arguments":{"collection_id":"test_collection","entry_id":"test-id-123","content":"Updated content for knowledge entry"}}}"#;
    let response = server.handle_request(request).await;
    
    // Verify response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response_value["id"], "12");
    assert!(response_value["result"]["content"].is_array());
    assert_eq!(response_value["result"]["content"][0]["type"], "text");
    
    // Verify the entry was updated
    let text = response_value["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Updated entry with ID: test-id-123"));
}

#[tokio::test]
async fn test_list_collections() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Send CallTool request for list_collections
    let request = r#"{"jsonrpc":"2.0","id":"13","method":"CallTool","params":{"name":"list_collections","arguments":{}}}"#;
    let response = server.handle_request(request).await;
    
    // Verify response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response_value["id"], "13");
    assert!(response_value["result"]["content"].is_array());
    assert_eq!(response_value["result"]["content"][0]["type"], "text");
    
    // Verify the collections were listed
    let collections_text = response_value["result"]["content"][0]["text"].as_str().unwrap();
    let collections: Vec<String> = serde_json::from_str(collections_text).unwrap();
    assert!(!collections.is_empty());
    assert!(collections.contains(&"general".to_string()));
}

#[tokio::test]
async fn test_create_collection() {
    // Create a mock vector store
    let store = MockQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Send CallTool request for create_collection
    let request = r#"{"jsonrpc":"2.0","id":"14","method":"CallTool","params":{"name":"create_collection","arguments":{"collection_id":"new_test_collection","vector_size":512}}}"#;
    let response = server.handle_request(request).await;
    
    // Verify response
    let response_value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response_value["id"], "14");
    assert!(response_value["result"]["content"].is_array());
    assert_eq!(response_value["result"]["content"][0]["type"], "text");
    
    // Verify the collection was created
    let text = response_value["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Created collection: new_test_collection"));
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
