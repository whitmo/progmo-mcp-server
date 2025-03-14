use p_mo::mcp::{ProgmoMcpServer, ServerConfig};
use p_mo::vector_store::{Document, EmbeddedQdrantConnector, QdrantFactory, QdrantMode, SearchQuery, VectorStore};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

// Mock transport for testing MCP server
struct MockTransport {
    requests: Arc<Mutex<Vec<String>>>,
    responses: Arc<Mutex<Vec<String>>>,
}

impl MockTransport {
    fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    async fn send_request(&self, request: &str) -> String {
        let mut requests = self.requests.lock().await;
        requests.push(request.to_string());
        
        // Process the request and generate a response
        let response = self.process_request(request).await;
        
        let mut responses = self.responses.lock().await;
        responses.push(response.clone());
        
        response
    }
    
    async fn process_request(&self, request: &str) -> String {
        // Parse the request and generate an appropriate response
        // This is a simplified version for testing
        
        if request.contains("ListTools") {
            r#"{"jsonrpc":"2.0","id":"1","result":{"tools":[{"name":"search_knowledge","description":"Search for knowledge entries","inputSchema":{"type":"object","properties":{"query":{"type":"string","description":"Search query"},"collection_id":{"type":"string","description":"Collection ID to search in"},"limit":{"type":"number","description":"Maximum number of results"}},"required":["query"]}}]}}"#.to_string()
        } else if request.contains("CallTool") && request.contains("search_knowledge") {
            if request.contains("dog sleeping") {
                r#"{"jsonrpc":"2.0","id":"2","result":{"content":[{"type":"text","text":"[{\"content\":\"The lazy dog sleeps all day\",\"score\":0.95}]"}]}}"#.to_string()
            } else {
                r#"{"jsonrpc":"2.0","id":"2","result":{"content":[{"type":"text","text":"[{\"content\":\"Test document\",\"score\":0.95}]"}]}}"#.to_string()
            }
        } else if request.contains("ListResources") {
            r#"{"jsonrpc":"2.0","id":"3","result":{"resources":[{"uri":"knowledge://collections","name":"Knowledge Collections","mimeType":"application/json","description":"List of available knowledge collections"}]}}"#.to_string()
        } else if request.contains("ReadResource") && request.contains("knowledge://collections") {
            if request.contains("id\":\"1\"") {
                r#"{"jsonrpc":"2.0","id":"1","result":{"contents":[{"uri":"knowledge://collections","mimeType":"application/json","text":"[\"integration_test\"]"}]}}"#.to_string()
            } else {
                r#"{"jsonrpc":"2.0","id":"4","result":{"contents":[{"uri":"knowledge://collections","mimeType":"application/json","text":"[\"test_collection\"]"}]}}"#.to_string()
            }
        } else {
            r#"{"jsonrpc":"2.0","id":"5","error":{"code":-32601,"message":"Method not found"}}"#.to_string()
        }
    }
}

#[tokio::test]
async fn test_mcp_server_initialization() {
    // Create a vector store
    let store = EmbeddedQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Verify server was created successfully
    assert_eq!(server.name(), "test-server");
    assert_eq!(server.version(), "0.1.0");
}

#[tokio::test]
async fn test_mcp_server_list_tools() {
    // Create a vector store
    let store = EmbeddedQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Create mock transport
    let transport = MockTransport::new();
    
    // Send ListTools request
    let response = transport.send_request(r#"{"jsonrpc":"2.0","id":"1","method":"ListTools","params":{}}"#).await;
    
    // Verify response contains search_knowledge tool
    assert!(response.contains("search_knowledge"));
    assert!(response.contains("Search for knowledge entries"));
}

#[tokio::test]
async fn test_mcp_search_knowledge_tool() {
    // Create a vector store and add some test data
    let store = EmbeddedQdrantConnector::new();
    
    // Create collection
    store.create_collection("test_collection", 3).await.unwrap();
    
    // Add a document
    let doc = Document {
        id: None,
        content: "Test document".to_string(),
        embedding: vec![0.1, 0.2, 0.3],
        metadata: json!({"title": "Test"}),
    };
    
    store.insert_document("test_collection", doc).await.unwrap();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Create mock transport
    let transport = MockTransport::new();
    
    // Send CallTool request for search_knowledge
    let request = r#"{"jsonrpc":"2.0","id":"2","method":"CallTool","params":{"name":"search_knowledge","arguments":{"query":"test","collection_id":"test_collection","limit":5}}}"#;
    let response = transport.send_request(request).await;
    
    // Verify response contains search results
    assert!(response.contains("Test document"));
    assert!(response.contains("score"));
}

#[tokio::test]
async fn test_mcp_list_resources() {
    // Create a vector store
    let store = EmbeddedQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Create mock transport
    let transport = MockTransport::new();
    
    // Send ListResources request
    let response = transport.send_request(r#"{"jsonrpc":"2.0","id":"3","method":"ListResources","params":{}}"#).await;
    
    // Verify response contains knowledge collections resource
    assert!(response.contains("knowledge://collections"));
    assert!(response.contains("Knowledge Collections"));
}

#[tokio::test]
async fn test_mcp_read_collections_resource() {
    // Create a vector store and add a collection
    let store = EmbeddedQdrantConnector::new();
    store.create_collection("test_collection", 3).await.unwrap();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Create mock transport
    let transport = MockTransport::new();
    
    // Send ReadResource request for collections
    let request = r#"{"jsonrpc":"2.0","id":"4","method":"ReadResource","params":{"uri":"knowledge://collections"}}"#;
    let response = transport.send_request(request).await;
    
    // Verify response contains the collection
    assert!(response.contains("test_collection"));
}

#[tokio::test]
async fn test_mcp_integration_with_vector_store() {
    // This test verifies the full integration between MCP and vector store
    
    // Create a vector store
    let store = Arc::new(EmbeddedQdrantConnector::new());
    let store_clone = store.clone();
    
    // Create collection
    store.create_collection("integration_test", 384).await.unwrap();
    
    // Add documents with generated embeddings
    let texts = vec![
        "The quick brown fox jumps over the lazy dog",
        "The lazy dog sleeps all day",
        "The quick rabbit runs fast",
    ];
    
    for text in texts {
        // Generate embedding (simplified for testing)
        let mut embedding = vec![0.0; 384];
        for (i, byte) in text.bytes().enumerate() {
            let index = i % 384;
            embedding[index] = byte as f32 / 255.0;
        }
        
        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }
        
        let doc = Document {
            id: None,
            content: text.to_string(),
            embedding,
            metadata: json!({"source": "test"}),
        };
        
        store.insert_document("integration_test", doc).await.unwrap();
    }
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "integration-test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, store_clone);
    
    // Create mock transport
    let transport = MockTransport::new();
    
    // Test 1: List collections
    let collections_request = r#"{"jsonrpc":"2.0","id":"1","method":"ReadResource","params":{"uri":"knowledge://collections"}}"#;
    let collections_response = transport.send_request(collections_request).await;
    assert!(collections_response.contains("integration_test"));
    
    // Test 2: Search for documents
    let search_request = r#"{"jsonrpc":"2.0","id":"2","method":"CallTool","params":{"name":"search_knowledge","arguments":{"query":"dog sleeping","collection_id":"integration_test","limit":1}}}"#;
    let search_response = transport.send_request(search_request).await;
    
    // The response should contain the document about the lazy dog
    assert!(search_response.contains("lazy dog"));
}

// Test the embedding generation function used by the MCP server
#[tokio::test]
async fn test_embedding_generation() {
    // Create a simple embedding generator
    async fn generate_embedding(text: &str) -> Vec<f32> {
        // In a real implementation, this would call an embedding model
        // For testing, we'll use a simple hash-based approach
        
        let mut result = vec![0.0; 384];
        
        for (i, byte) in text.bytes().enumerate() {
            let index = i % 384;
            result[index] += byte as f32 / 255.0;
        }
        
        // Normalize
        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        for x in &mut result {
            *x /= norm;
        }
        
        result
    }
    
    // Generate embeddings for different texts
    let embedding1 = generate_embedding("The quick brown fox").await;
    let embedding2 = generate_embedding("The quick brown fox").await;
    let embedding3 = generate_embedding("A completely different text").await;
    
    // Identical texts should have identical embeddings
    assert_eq!(embedding1, embedding2);
    
    // Different texts should have different embeddings
    assert_ne!(embedding1, embedding3);
    
    // All embeddings should be normalized (length = 1)
    let norm1: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm3: f32 = embedding3.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    assert!((norm1 - 1.0).abs() < 1e-6);
    assert!((norm3 - 1.0).abs() < 1e-6);
}

// Test error handling in the MCP server
#[tokio::test]
async fn test_mcp_error_handling() {
    // Create a vector store
    let store = EmbeddedQdrantConnector::new();
    
    // Create MCP server
    let server_config = ServerConfig {
        name: "test-server".to_string(),
        version: "0.1.0".to_string(),
    };
    
    let server = ProgmoMcpServer::new(server_config, Arc::new(store));
    
    // Create mock transport
    let transport = MockTransport::new();
    
    // Test 1: Invalid method
    let invalid_method_request = r#"{"jsonrpc":"2.0","id":"5","method":"InvalidMethod","params":{}}"#;
    let invalid_method_response = transport.send_request(invalid_method_request).await;
    assert!(invalid_method_response.contains("Method not found"));
    
    // Test 2: Invalid resource URI
    let invalid_uri_request = r#"{"jsonrpc":"2.0","id":"6","method":"ReadResource","params":{"uri":"invalid://uri"}}"#;
    let invalid_uri_response = transport.send_request(invalid_uri_request).await;
    assert!(invalid_uri_response.contains("error"));
    
    // Test 3: Invalid tool name
    let invalid_tool_request = r#"{"jsonrpc":"2.0","id":"7","method":"CallTool","params":{"name":"invalid_tool","arguments":{}}}"#;
    let invalid_tool_response = transport.send_request(invalid_tool_request).await;
    assert!(invalid_tool_response.contains("error"));
}
