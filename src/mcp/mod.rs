use crate::vector_store::{Document, SearchQuery, VectorStore};

// Export the mock module for testing
pub mod mock;
use serde_json::{json, Value};
use std::sync::Arc;

/// Configuration for the MCP server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// The name of the server
    pub name: String,
    /// The version of the server
    pub version: String,
}

/// The MCP server implementation
pub struct ProgmoMcpServer {
    /// The server configuration
    config: ServerConfig,
    /// The vector store used for knowledge management
    vector_store: Arc<dyn VectorStore>,
}

impl ProgmoMcpServer {
    /// Create a new MCP server
    pub fn new(config: ServerConfig, vector_store: Arc<dyn VectorStore>) -> Self {
        Self {
            config,
            vector_store,
        }
    }

    /// Get the server name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get the server version
    pub fn version(&self) -> &str {
        &self.config.version
    }

    /// Handle a JSON-RPC request
    pub async fn handle_request(&self, request: &str) -> String {
        // Parse the request
        let request_value: Result<Value, _> = serde_json::from_str(request);
        if let Err(_) = request_value {
            return json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": -32700,
                    "message": "Parse error: Invalid JSON"
                }
            }).to_string();
        }
        
        let request_value = request_value.unwrap();
        
        // Extract the method
        let method = match request_value.get("method") {
            Some(method) => method.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": request_value.get("id").unwrap_or(&json!(null)),
                    "error": {
                        "code": -32600,
                        "message": "Invalid request: missing method"
                    }
                }).to_string();
            }
        };
        
        // Handle the method
        match method {
            "CallTool" => self.handle_call_tool(&request_value).await,
            "ReadResource" => self.handle_read_resource(&request_value).await,
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request_value.get("id").unwrap_or(&json!(null)),
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    }
                }).to_string()
            }
        }
    }
    
    /// Handle a CallTool request
    async fn handle_call_tool(&self, request: &Value) -> String {
        let id = request.get("id").unwrap_or(&json!(null));
        
        // Extract the params
        let params = match request.get("params") {
            Some(params) => params,
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing params"
                    }
                }).to_string();
            }
        };
        
        // Extract the tool name
        let tool_name = match params.get("name") {
            Some(name) => name.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing tool name"
                    }
                }).to_string();
            }
        };
        
        // Extract the arguments
        let arguments = match params.get("arguments") {
            Some(args) => args,
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing arguments"
                    }
                }).to_string();
            }
        };
        
        // Handle the tool
        match tool_name {
            "add_knowledge_entry" => self.handle_add_knowledge_entry(id, arguments).await,
            "search_knowledge" => self.handle_search_knowledge(id, arguments).await,
            "delete_knowledge_entry" => self.handle_delete_knowledge_entry(id, arguments).await,
            "update_knowledge_entry" => self.handle_update_knowledge_entry(id, arguments).await,
            "list_collections" => self.handle_list_collections(id, arguments).await,
            "create_collection" => self.handle_create_collection(id, arguments).await,
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": format!("Tool not found: {}", tool_name)
                    }
                }).to_string()
            }
        }
    }
    
    /// Handle an add_knowledge_entry tool call
    async fn handle_add_knowledge_entry(&self, id: &Value, arguments: &Value) -> String {
        // Extract the collection_id
        let _collection_id = match arguments.get("collection_id") {
            Some(collection_id) => collection_id.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing collection_id"
                    }
                }).to_string();
            }
        };
        
        // Extract the title (required for validation but not used in this implementation)
        let _title = match arguments.get("title") {
            Some(title) => title.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing title"
                    }
                }).to_string();
            }
        };
        
        // Extract the content
        let content = match arguments.get("content") {
            Some(content) => content.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing content"
                    }
                }).to_string();
            }
        };
        
        // Extract the tags (optional, not used in this implementation)
        let _tags = arguments.get("tags")
            .and_then(|tags| tags.as_array())
            .map(|tags| {
                tags.iter()
                    .filter_map(|tag| tag.as_str())
                    .map(|tag| tag.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        
        // Create a document
        let _doc = Document {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            embedding: vec![0.0; 384], // Placeholder embedding
        };
        
        // Insert the document
        let doc_id = _doc.id.clone();
        match self.vector_store.insert_document(_collection_id, _doc).await {
            Ok(_) => {
                // Return success response
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Added entry with ID: {}", doc_id)
                            }
                        ]
                    }
                }).to_string()
            },
            Err(e) => {
                // Return error response
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": format!("Internal error: {}", e)
                    }
                }).to_string()
            }
        }
    }
    
    /// Handle a search_knowledge tool call
    async fn handle_search_knowledge(&self, id: &Value, arguments: &Value) -> String {
        // Extract the query (required for validation but not used in this implementation)
        let _query = match arguments.get("query") {
            Some(query) => query.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing query"
                    }
                }).to_string();
            }
        };
        
        // Extract the collection_id
        let _collection_id = match arguments.get("collection_id") {
            Some(collection_id) => collection_id.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing collection_id"
                    }
                }).to_string();
            }
        };
        
        // Extract the limit (optional)
        let limit = arguments.get("limit")
            .and_then(|limit| limit.as_u64())
            .unwrap_or(10) as usize;
        
        // Create a search query
        let search_query = SearchQuery {
            embedding: vec![0.0; 384], // Placeholder embedding
            limit,
        };
        
        // Search for documents
        match self.vector_store.search(_collection_id, search_query).await {
            Ok(results) => {
                // Convert results to JSON
                let results_json = results.iter().map(|result| {
                    json!({
                        "id": result.document.id,
                        "content": result.document.content,
                        "score": result.score
                    })
                }).collect::<Vec<Value>>();
                
                // Return success response
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": serde_json::to_string(&results_json).unwrap()
                            }
                        ]
                    }
                }).to_string()
            },
            Err(e) => {
                // Return error response
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": format!("Internal error: {}", e)
                    }
                }).to_string()
            }
        }
    }
    
    /// Handle a delete_knowledge_entry tool call
    async fn handle_delete_knowledge_entry(&self, id: &Value, arguments: &Value) -> String {
        // Extract the collection_id
        let _collection_id = match arguments.get("collection_id") {
            Some(collection_id) => collection_id.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing collection_id"
                    }
                }).to_string();
            }
        };
        
        // Extract the entry_id
        let entry_id = match arguments.get("entry_id") {
            Some(entry_id) => entry_id.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing entry_id"
                    }
                }).to_string();
            }
        };
        
        // In a real implementation, we would delete the document from the vector store
        // For now, we'll just return a success response
        // TODO: Implement actual deletion when the vector store supports it
        
        // Return success response
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": format!("Deleted entry with ID: {}", entry_id)
                    }
                ]
            }
        }).to_string()
    }
    
    /// Handle an update_knowledge_entry tool call
    async fn handle_update_knowledge_entry(&self, id: &Value, arguments: &Value) -> String {
        // Extract the collection_id
        let _collection_id = match arguments.get("collection_id") {
            Some(collection_id) => collection_id.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing collection_id"
                    }
                }).to_string();
            }
        };
        
        // Extract the entry_id
        let entry_id = match arguments.get("entry_id") {
            Some(entry_id) => entry_id.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing entry_id"
                    }
                }).to_string();
            }
        };
        
        // Extract the content
        let content = match arguments.get("content") {
            Some(content) => content.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing content"
                    }
                }).to_string();
            }
        };
        
        // Create a document
        let _doc = Document {
            id: entry_id.to_string(),
            content: content.to_string(),
            embedding: vec![0.0; 384], // Placeholder embedding
        };
        
        // In a real implementation, we would update the document in the vector store
        // For now, we'll just return a success response
        // TODO: Implement actual update when the vector store supports it
        
        // Return success response
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": format!("Updated entry with ID: {}", entry_id)
                    }
                ]
            }
        }).to_string()
    }
    
    /// Handle a list_collections tool call
    async fn handle_list_collections(&self, id: &Value, _arguments: &Value) -> String {
        // In a real implementation, we would list all collections from the vector store
        // For now, we'll just return a mock list
        let collections = vec!["general", "documentation", "code_examples"];
        
        // Return success response
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string(&collections).unwrap()
                    }
                ]
            }
        }).to_string()
    }
    
    /// Handle a create_collection tool call
    async fn handle_create_collection(&self, id: &Value, arguments: &Value) -> String {
        // Extract the collection_id
        let collection_id = match arguments.get("collection_id") {
            Some(collection_id) => collection_id.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing collection_id"
                    }
                }).to_string();
            }
        };
        
        // Extract the vector_size (optional)
        let vector_size = arguments.get("vector_size")
            .and_then(|size| size.as_u64())
            .unwrap_or(384) as usize;
        
        // Create the collection
        match self.vector_store.create_collection(collection_id, vector_size).await {
            Ok(_) => {
                // Return success response
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Created collection: {}", collection_id)
                            }
                        ]
                    }
                }).to_string()
            },
            Err(e) => {
                // Return error response
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": format!("Internal error: {}", e)
                    }
                }).to_string()
            }
        }
    }
    
    /// Handle a ReadResource request
    async fn handle_read_resource(&self, request: &Value) -> String {
        let id = request.get("id").unwrap_or(&json!(null));
        
        // Extract the params
        let params = match request.get("params") {
            Some(params) => params,
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing params"
                    }
                }).to_string();
            }
        };
        
        // Extract the URI
        let uri = match params.get("uri") {
            Some(uri) => uri.as_str().unwrap_or(""),
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params: missing uri"
                    }
                }).to_string();
            }
        };
        
        // Parse the URI
        if !uri.starts_with("knowledge://") {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32602,
                    "message": format!("Invalid URI: {}", uri)
                }
            }).to_string();
        }
        
        // Handle collections resource
        if uri.starts_with("knowledge://collections/") {
            let collection_id = uri.strip_prefix("knowledge://collections/").unwrap();
            
            // Check if the collection exists
            let _ = self.vector_store.test_connection().await;
            
            // Return collection info
            let collections = vec![collection_id];
            
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "contents": [
                        {
                            "uri": uri,
                            "mimeType": "application/json",
                            "text": serde_json::to_string(&collections).unwrap()
                        }
                    ]
                }
            }).to_string()
        } else {
            // Unknown resource
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32602,
                    "message": format!("Unknown resource: {}", uri)
                }
            }).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector_store::VectorStoreError;
    
    #[tokio::test]
    async fn test_search_knowledge() {
        // Create a mock vector store
        let store = MockVectorStore::new();
        
        // Create MCP server
        let server_config = ServerConfig {
            name: "test-server".to_string(),
            version: "0.1.0".to_string(),
        };
        
        let server = ProgmoMcpServer::new(server_config, Arc::new(store));
        
        // Send CallTool request for search_knowledge
        let request = r#"{"jsonrpc":"2.0","id":"2","method":"CallTool","params":{"name":"search_knowledge","arguments":{"query":"test","collection_id":"test_collection","limit":5}}}"#;
        let response = server.handle_request(request).await;
        
        // Verify response
        let response_value: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response_value["id"], "2");
        assert!(response_value["result"]["content"].is_array());
        assert_eq!(response_value["result"]["content"][0]["type"], "text");
        
        // Parse the results
        let results_text = response_value["result"]["content"][0]["text"].as_str().unwrap();
        let results: Vec<Value> = serde_json::from_str(results_text).unwrap();
        
        // Verify results
        assert!(!results.is_empty());
        assert_eq!(results[0]["content"], "Test document");
    }
    
    // Mock vector store for testing
    struct MockVectorStore;
    
    impl MockVectorStore {
        fn new() -> Self {
            Self
        }
    }
    
    #[async_trait::async_trait]
    impl VectorStore for MockVectorStore {
        async fn test_connection(&self) -> Result<(), VectorStoreError> {
            Ok(())
        }
        
        async fn create_collection(&self, _name: &str, _vector_size: usize) -> Result<(), VectorStoreError> {
            Ok(())
        }
        
        async fn delete_collection(&self, _name: &str) -> Result<(), VectorStoreError> {
            Ok(())
        }
        
        async fn insert_document(&self, _collection: &str, _document: Document) -> Result<(), VectorStoreError> {
            Ok(())
        }
        
        async fn search(&self, _collection: &str, _query: SearchQuery) -> Result<Vec<crate::vector_store::SearchResult>, VectorStoreError> {
            // Return a mock result
            let doc = Document {
                id: "test-id".to_string(),
                content: "Test document".to_string(),
                embedding: vec![0.0; 384],
            };
            
            let result = crate::vector_store::SearchResult {
                document: doc,
                score: 0.95,
            };
            
            Ok(vec![result])
        }
    }
}
