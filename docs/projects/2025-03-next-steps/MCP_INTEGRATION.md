# MCP Integration with modelcontextprotocol/rust-sdk

This document outlines how to leverage the [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk) for implementing the MCP integration in the progmo-mcp-server project.

## Overview

The Model Context Protocol (MCP) is a standardized protocol for communication between AI models and external tools or resources. The `modelcontextprotocol/rust-sdk` provides Rust implementations of the MCP specification, making it easier to create MCP-compatible servers and clients.

For the progmo-mcp-server project, we'll use this SDK to:

1. Expose the vector store and knowledge management capabilities as MCP resources and tools
2. Enable seamless integration with Cline and other MCP clients
3. Implement standardized request/response handling for AI model interactions

## Getting Started

### Adding the Dependency

Add the MCP SDK to your `Cargo.toml`:

```toml
[dependencies]
mcp-sdk = { git = "https://github.com/modelcontextprotocol/rust-sdk", branch = "main" }
```

Or if you prefer to use a specific version:

```toml
[dependencies]
mcp-sdk = "0.1.0"  # Replace with the actual version
```

### Basic Server Implementation

Here's a basic example of how to implement an MCP server using the SDK:

```rust
use mcp_sdk::server::{Server, ServerConfig};
use mcp_sdk::transport::StdioTransport;
use mcp_sdk::types::{
    CallToolRequestSchema, ErrorCode, ListResourcesRequestSchema,
    ListResourceTemplatesRequestSchema, ListToolsRequestSchema, McpError,
    ReadResourceRequestSchema,
};

struct ProgmoMcpServer {
    server: Server,
    vector_store: Arc<dyn VectorStore>,
}

impl ProgmoMcpServer {
    pub fn new(vector_store: Arc<dyn VectorStore>) -> Self {
        let server = Server::new(
            ServerConfig {
                name: "progmo-mcp-server",
                version: "0.1.0",
            },
            {
                capabilities: {
                    resources: {},
                    tools: {},
                },
            },
        );

        let instance = Self {
            server,
            vector_store,
        };

        instance.setup_resource_handlers();
        instance.setup_tool_handlers();
        
        instance
    }

    fn setup_resource_handlers(&self) {
        // Implement resource handlers
        self.server.set_request_handler(ListResourcesRequestSchema, async move |_| {
            // Return list of available resources
            Ok({
                resources: [
                    {
                        uri: "knowledge://collections",
                        name: "Knowledge Collections",
                        mimeType: "application/json",
                        description: "List of available knowledge collections",
                    },
                ],
            })
        });

        // Implement resource template handlers
        self.server.set_request_handler(
            ListResourceTemplatesRequestSchema,
            async move |_| {
                Ok({
                    resourceTemplates: [
                        {
                            uriTemplate: "knowledge://collections/{collection_id}",
                            name: "Knowledge Collection",
                            mimeType: "application/json",
                            description: "Information about a specific knowledge collection",
                        },
                        {
                            uriTemplate: "knowledge://collections/{collection_id}/entries/{entry_id}",
                            name: "Knowledge Entry",
                            mimeType: "application/json",
                            description: "A specific knowledge entry",
                        },
                    ],
                })
            },
        );

        // Implement resource read handler
        self.server.set_request_handler(
            ReadResourceRequestSchema,
            async move |request| {
                let uri = request.params.uri;
                
                // Parse the URI and return the appropriate resource
                // Example: knowledge://collections/my_collection/entries/123
                
                // Return error if URI is invalid
                if !uri.starts_with("knowledge://") {
                    return Err(McpError {
                        code: ErrorCode::InvalidRequest,
                        message: format!("Invalid URI: {}", uri),
                    });
                }
                
                // Handle different URI patterns
                // ...
                
                Ok({
                    contents: [
                        {
                            uri: request.params.uri,
                            mimeType: "application/json",
                            text: json_content,
                        },
                    ],
                })
            },
        );
    }

    fn setup_tool_handlers(&self) {
        // Implement tool handlers
        self.server.set_request_handler(ListToolsRequestSchema, async move |_| {
            Ok({
                tools: [
                    {
                        name: "search_knowledge",
                        description: "Search for knowledge entries",
                        inputSchema: {
                            type: "object",
                            properties: {
                                query: {
                                    type: "string",
                                    description: "Search query",
                                },
                                collection_id: {
                                    type: "string",
                                    description: "Collection ID to search in",
                                },
                                limit: {
                                    type: "number",
                                    description: "Maximum number of results",
                                },
                            },
                            required: ["query"],
                        },
                    },
                    {
                        name: "add_knowledge_entry",
                        description: "Add a new knowledge entry",
                        inputSchema: {
                            type: "object",
                            properties: {
                                collection_id: {
                                    type: "string",
                                    description: "Collection ID",
                                },
                                title: {
                                    type: "string",
                                    description: "Entry title",
                                },
                                content: {
                                    type: "string",
                                    description: "Entry content",
                                },
                                tags: {
                                    type: "array",
                                    items: {
                                        type: "string",
                                    },
                                    description: "Tags for the entry",
                                },
                            },
                            required: ["collection_id", "title", "content"],
                        },
                    },
                ],
            })
        });

        // Implement tool call handler
        self.server.set_request_handler(CallToolRequestSchema, async move |request| {
            match request.params.name.as_str() {
                "search_knowledge" => {
                    // Parse arguments
                    let query = request.params.arguments.get("query").unwrap().as_str().unwrap();
                    let collection_id = request.params.arguments.get("collection_id").map(|v| v.as_str().unwrap());
                    let limit = request.params.arguments.get("limit").map(|v| v.as_u64().unwrap()).unwrap_or(10);
                    
                    // Perform search using vector store
                    // ...
                    
                    Ok({
                        content: [
                            {
                                type: "text",
                                text: search_results_json,
                            },
                        ],
                    })
                },
                "add_knowledge_entry" => {
                    // Parse arguments
                    let collection_id = request.params.arguments.get("collection_id").unwrap().as_str().unwrap();
                    let title = request.params.arguments.get("title").unwrap().as_str().unwrap();
                    let content = request.params.arguments.get("content").unwrap().as_str().unwrap();
                    let tags = request.params.arguments.get("tags").map(|v| {
                        v.as_array().unwrap().iter().map(|tag| tag.as_str().unwrap().to_string()).collect::<Vec<String>>()
                    }).unwrap_or_else(Vec::new);
                    
                    // Add entry using vector store
                    // ...
                    
                    Ok({
                        content: [
                            {
                                type: "text",
                                text: format!("Added entry with ID: {}", entry_id),
                            },
                        ],
                    })
                },
                _ => Err(McpError {
                    code: ErrorCode::MethodNotFound,
                    message: format!("Unknown tool: {}", request.params.name),
                }),
            }
        });
    }

    pub async fn run(&self) -> Result<(), McpError> {
        let transport = StdioTransport::new();
        self.server.connect(transport).await?;
        Ok(())
    }
}
```

## Integration with Vector Store

To integrate the MCP server with the vector store, you'll need to:

1. Create an adapter between the vector store and MCP interfaces
2. Implement resource handlers that expose vector store collections and entries
3. Implement tool handlers that allow operations on the vector store

### Vector Store Adapter

```rust
struct VectorStoreAdapter {
    vector_store: Arc<dyn VectorStore>,
}

impl VectorStoreAdapter {
    pub fn new(vector_store: Arc<dyn VectorStore>) -> Self {
        Self { vector_store }
    }
    
    pub async fn search(&self, collection: &str, query: &str, limit: usize) -> Result<Vec<SearchResult>, VectorStoreError> {
        // Convert query to embedding
        let embedding = self.generate_embedding(query).await?;
        
        // Create search query
        let search_query = SearchQuery {
            embedding,
            limit,
            offset: 0,
        };
        
        // Perform search
        self.vector_store.search(collection, search_query).await
    }
    
    pub async fn add_entry(&self, collection: &str, title: &str, content: &str, tags: Vec<String>) -> Result<String, VectorStoreError> {
        // Generate embedding for content
        let embedding = self.generate_embedding(content).await?;
        
        // Create document
        let document = Document {
            id: None,
            content: content.to_string(),
            embedding,
            metadata: json!({
                "title": title,
                "tags": tags,
            }),
        };
        
        // Insert document
        self.vector_store.insert_document(collection, document).await
    }
    
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, VectorStoreError> {
        // In a real implementation, this would call an embedding model
        // For now, we'll return a dummy embedding
        Ok(vec![0.1, 0.2, 0.3])
    }
}
```

## MCP Resource URIs

Define a clear URI structure for your MCP resources:

- `knowledge://collections` - List all collections
- `knowledge://collections/{collection_id}` - Information about a specific collection
- `knowledge://collections/{collection_id}/entries` - List entries in a collection
- `knowledge://collections/{collection_id}/entries/{entry_id}` - Get a specific entry

## MCP Tools

Implement the following MCP tools:

1. `search_knowledge` - Search for knowledge entries
2. `add_knowledge_entry` - Add a new knowledge entry
3. `update_knowledge_entry` - Update an existing entry
4. `delete_knowledge_entry` - Delete an entry
5. `create_collection` - Create a new collection
6. `delete_collection` - Delete a collection

## Testing MCP Integration

Create tests for your MCP server:

```rust
#[tokio::test]
async fn test_mcp_search_knowledge() {
    // Create a mock vector store
    let vector_store = Arc::new(MockVectorStore::new());
    
    // Create MCP server with the mock vector store
    let mcp_server = ProgmoMcpServer::new(vector_store.clone());
    
    // Create a test transport
    let (client_transport, server_transport) = TestTransport::new_pair();
    
    // Connect the server to the transport
    tokio::spawn(async move {
        mcp_server.server.connect(server_transport).await.unwrap();
    });
    
    // Create a client
    let client = Client::new(client_transport);
    
    // Call the search_knowledge tool
    let result = client.call_tool(
        "search_knowledge",
        json!({
            "query": "test query",
            "collection_id": "test_collection",
            "limit": 5,
        }),
    ).await.unwrap();
    
    // Verify the result
    assert!(result.content.len() > 0);
    assert_eq!(result.content[0].type_, "text");
    // Further assertions...
}
```

## Integration with Cline

To integrate with Cline, you'll need to:

1. Implement the MCP server as described above
2. Run the server as a subprocess that Cline can communicate with
3. Register the server with Cline using the MCP configuration

### Running as a Subprocess

Cline will typically launch your MCP server as a subprocess and communicate with it via stdin/stdout. Ensure your server uses the `StdioTransport` for communication.

### Registering with Cline

Users will need to add your server to their Cline configuration:

```json
{
  "mcpServers": {
    "progmo": {
      "command": "path/to/progmo-mcp-server",
      "args": [],
      "env": {
        "QDRANT_URL": "http://localhost:6333"
      }
    }
  }
}
```

## Resources

- [Model Context Protocol Specification](https://github.com/modelcontextprotocol/mcp)
- [MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [Cline MCP Documentation](https://docs.anthropic.com/claude/docs/model-context-protocol)

## Implementation Timeline

1. **Week 1**: Set up basic MCP server structure
2. **Week 2**: Implement resource handlers
3. **Week 3**: Implement tool handlers
4. **Week 4**: Integrate with vector store
5. **Week 5**: Test with Cline
6. **Week 6**: Optimize and refine

## Conclusion

Using the `modelcontextprotocol/rust-sdk` will significantly simplify the implementation of MCP integration in the progmo-mcp-server project. By following the approach outlined in this document, you can create a robust MCP server that exposes your vector store and knowledge management capabilities to Cline and other MCP clients.
