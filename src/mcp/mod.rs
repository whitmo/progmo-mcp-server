use crate::vector_store::{Document, SearchQuery, VectorStore};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
}

pub struct ProgmoMcpServer {
    config: ServerConfig,
    vector_store: Arc<dyn VectorStore>,
}

impl ProgmoMcpServer {
    pub fn new(config: ServerConfig, vector_store: Arc<dyn VectorStore>) -> Self {
        Self {
            config,
            vector_store,
        }
    }
    
    pub fn name(&self) -> &str {
        &self.config.name
    }
    
    pub fn version(&self) -> &str {
        &self.config.version
    }
    
    pub async fn handle_request(&self, request: &str) -> String {
        // Parse the request
        let request_value: Value = match serde_json::from_str(request) {
            Ok(value) => value,
            Err(e) => return self.create_error_response("1", -32700, &format!("Parse error: {}", e)),
        };
        
        // Extract request fields
        let id = request_value.get("id").and_then(|v| v.as_str()).unwrap_or("1");
        let method = match request_value.get("method").and_then(|v| v.as_str()) {
            Some(method) => method,
            None => return self.create_error_response(id, -32600, "Invalid request: missing method"),
        };
        
        // Handle the request based on the method
        match method {
            "ListTools" => self.handle_list_tools(id).await,
            "CallTool" => self.handle_call_tool(id, request_value.get("params")).await,
            "ListResources" => self.handle_list_resources(id).await,
            "ReadResource" => self.handle_read_resource(id, request_value.get("params")).await,
            _ => self.create_error_response(id, -32601, &format!("Method not found: {}", method)),
        }
    }
    
    async fn handle_list_tools(&self, id: &str) -> String {
        // Define the available tools
        let tools = json!({
            "tools": [
                {
                    "name": "search_knowledge",
                    "description": "Search for knowledge entries",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query"
                            },
                            "collection_id": {
                                "type": "string",
                                "description": "Collection ID to search in"
                            },
                            "limit": {
                                "type": "number",
                                "description": "Maximum number of results"
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "add_knowledge_entry",
                    "description": "Add a new knowledge entry",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "collection_id": {
                                "type": "string",
                                "description": "Collection ID"
                            },
                            "title": {
                                "type": "string",
                                "description": "Entry title"
                            },
                            "content": {
                                "type": "string",
                                "description": "Entry content"
                            },
                            "tags": {
                                "type": "array",
                                "items": {
                                    "type": "string"
                                },
                                "description": "Tags for the entry"
                            }
                        },
                        "required": ["collection_id", "title", "content"]
                    }
                }
            ]
        });
        
        self.create_success_response(id, tools)
    }
    
    async fn handle_call_tool(&self, id: &str, params: Option<&Value>) -> String {
        // Extract tool name and arguments
        let params = match params {
            Some(params) => params,
            None => return self.create_error_response(id, -32602, "Invalid params: missing params"),
        };
        
        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => return self.create_error_response(id, -32602, "Invalid params: missing tool name"),
        };
        
        let arguments = match params.get("arguments") {
            Some(args) => args,
            None => return self.create_error_response(id, -32602, "Invalid params: missing arguments"),
        };
        
        // Handle the tool call based on the tool name
        match tool_name {
            "search_knowledge" => self.handle_search_knowledge(id, arguments).await,
            "add_knowledge_entry" => self.handle_add_knowledge_entry(id, arguments).await,
            _ => self.create_error_response(id, -32601, &format!("Tool not found: {}", tool_name)),
        }
    }
    
    async fn handle_search_knowledge(&self, id: &str, arguments: &Value) -> String {
        // Extract search parameters
        let query = match arguments.get("query").and_then(|v| v.as_str()) {
            Some(query) => query,
            None => return self.create_error_response(id, -32602, "Invalid params: missing query"),
        };
        
        let collection_id = arguments.get("collection_id").and_then(|v| v.as_str()).unwrap_or("default");
        let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
        
        // Generate embedding for the query
        let embedding = self.generate_embedding(query).await;
        
        // Create search query
        let search_query = SearchQuery {
            embedding,
            limit,
            offset: 0,
        };
        
        // Perform search
        let results = match self.vector_store.search(collection_id, search_query).await {
            Ok(results) => results,
            Err(e) => return self.create_error_response(id, -32000, &format!("Search failed: {}", e)),
        };
        
        // Format results
        let formatted_results: Vec<Value> = results.into_iter().map(|result| {
            json!({
                "content": result.document.content,
                "score": result.score
            })
        }).collect();
        
        // Create response
        let response = json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string(&formatted_results).unwrap()
                }
            ]
        });
        
        self.create_success_response(id, response)
    }
    
    async fn handle_add_knowledge_entry(&self, id: &str, arguments: &Value) -> String {
        // Extract entry parameters
        let collection_id = match arguments.get("collection_id").and_then(|v| v.as_str()) {
            Some(collection_id) => collection_id,
            None => return self.create_error_response(id, -32602, "Invalid params: missing collection_id"),
        };
        
        let title = match arguments.get("title").and_then(|v| v.as_str()) {
            Some(title) => title,
            None => return self.create_error_response(id, -32602, "Invalid params: missing title"),
        };
        
        let content = match arguments.get("content").and_then(|v| v.as_str()) {
            Some(content) => content,
            None => return self.create_error_response(id, -32602, "Invalid params: missing content"),
        };
        
        let tags = arguments.get("tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter().filter_map(|v| v.as_str()).map(String::from).collect::<Vec<String>>()
        }).unwrap_or_else(Vec::new);
        
        // Generate embedding for the content
        let embedding = self.generate_embedding(content).await;
        
        // Create document
        let document = Document {
            id: None,
            content: content.to_string(),
            embedding,
            metadata: json!({
                "title": title,
                "tags": tags
            }),
        };
        
        // Insert document
        let entry_id = match self.vector_store.insert_document(collection_id, document).await {
            Ok(id) => id,
            Err(e) => return self.create_error_response(id, -32000, &format!("Failed to add entry: {}", e)),
        };
        
        // Create response
        let response = json!({
            "content": [
                {
                    "type": "text",
                    "text": format!("Added entry with ID: {}", entry_id)
                }
            ]
        });
        
        self.create_success_response(id, response)
    }
    
    async fn handle_list_resources(&self, id: &str) -> String {
        // Check if we can list collections
        let _ = self.vector_store.list_collections().await.map_err(|e| {
            return self.create_error_response(id, -32000, &format!("Failed to list collections: {}", e));
        });
        
        // Define the available resources
        let resources = json!({
            "resources": [
                {
                    "uri": "knowledge://collections",
                    "name": "Knowledge Collections",
                    "mimeType": "application/json",
                    "description": "List of available knowledge collections"
                }
            ]
        });
        
        self.create_success_response(id, resources)
    }
    
    async fn handle_read_resource(&self, id: &str, params: Option<&Value>) -> String {
        // Extract URI
        let params = match params {
            Some(params) => params,
            None => return self.create_error_response(id, -32602, "Invalid params: missing params"),
        };
        
        let uri = match params.get("uri").and_then(|v| v.as_str()) {
            Some(uri) => uri,
            None => return self.create_error_response(id, -32602, "Invalid params: missing uri"),
        };
        
        // Handle different resource URIs
        if uri == "knowledge://collections" {
            // List collections
            let collections = match self.vector_store.list_collections().await {
                Ok(collections) => collections,
                Err(e) => return self.create_error_response(id, -32000, &format!("Failed to list collections: {}", e)),
            };
            
            // Create response
            let response = json!({
                "contents": [
                    {
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": serde_json::to_string(&collections).unwrap()
                    }
                ]
            });
            
            self.create_success_response(id, response)
        } else if let Some(collection_id) = uri.strip_prefix("knowledge://collections/") {
            // Get collection info
            // In a real implementation, this would return more information about the collection
            
            // Create response
            let response = json!({
                "contents": [
                    {
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": format!("{{\"id\":\"{}\",\"name\":\"{}\"}}", collection_id, collection_id)
                    }
                ]
            });
            
            self.create_success_response(id, response)
        } else {
            self.create_error_response(id, -32602, &format!("Invalid URI: {}", uri))
        }
    }
    
    fn create_success_response(&self, id: &str, result: Value) -> String {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }).to_string()
    }
    
    fn create_error_response(&self, id: &str, code: i32, message: &str) -> String {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message
            }
        }).to_string()
    }
    
    async fn generate_embedding(&self, text: &str) -> Vec<f32> {
        // In a real implementation, this would call an embedding model
        // For now, we'll use a simple hash-based approach
        
        let mut result = vec![0.0; 384];
        
        for (i, byte) in text.bytes().enumerate() {
            let index = i % 384;
            result[index] += byte as f32 / 255.0;
        }
        
        // Normalize
        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut result {
                *x /= norm;
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector_store::EmbeddedQdrantConnector;
    
    #[tokio::test]
    async fn test_server_initialization() {
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
    async fn test_list_tools() {
        // Create a vector store
        let store = EmbeddedQdrantConnector::new();
        
        // Create MCP server
        let server_config = ServerConfig {
            name: "test-server".to_string(),
            version: "0.1.0".to_string(),
        };
        
        let server = ProgmoMcpServer::new(server_config, Arc::new(store));
        
        // Send ListTools request
        let request = r#"{"jsonrpc":"2.0","id":"1","method":"ListTools","params":{}}"#;
        let response = server.handle_request(request).await;
        
        // Verify response
        let response_value: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response_value["id"], "1");
        assert!(response_value["result"]["tools"].is_array());
        assert!(response_value["result"]["tools"].as_array().unwrap().len() > 0);
    }
    
    #[tokio::test]
    async fn test_search_knowledge() {
        // Create a vector store
        let store = EmbeddedQdrantConnector::new();
        
        // Create collection
        store.create_collection("test_collection", 384).await.unwrap();
        
        // Add a document
        let embedding = vec![0.1; 384];
        let doc = Document {
            id: None,
            content: "Test document".to_string(),
            embedding,
            metadata: json!({"title": "Test"}),
        };
        
        store.insert_document("test_collection", doc).await.unwrap();
        
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
}
