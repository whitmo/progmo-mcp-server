# Vector Store Integration Design

This document outlines the design considerations, implementation details, and testing strategy for the vector store integration in the progmo-mcp-server project, with a focus on the Qdrant reference implementation.

## Architectural Considerations

### Embedding Qdrant vs. External Service

The question of whether to embed Qdrant within the server itself versus using it as an external service is an important architectural decision. Here's an analysis of both approaches:

#### Option 1: Embedded Qdrant

**Pros:**
- **Simplified Deployment**: Users only need to deploy a single binary
- **Reduced Configuration**: No need to configure connection details
- **Guaranteed Availability**: Qdrant starts and stops with the server
- **Reduced Resource Overhead**: Shared resources between server and vector store
- **Simplified Development**: No need to handle connection failures or network issues

**Cons:**
- **Increased Binary Size**: The server binary will be larger
- **Resource Contention**: Server and vector store compete for the same resources
- **Limited Scalability**: Cannot scale vector store independently
- **Upgrade Challenges**: Upgrading Qdrant requires rebuilding the server
- **Potential Licensing Issues**: Need to ensure compatibility with Qdrant's license

#### Option 2: External Qdrant

**Pros:**
- **Independent Scaling**: Can scale Qdrant separately from the server
- **Resource Isolation**: Server and vector store have dedicated resources
- **Flexibility**: Can use existing Qdrant instances or cloud services
- **Independent Upgrades**: Can upgrade Qdrant without rebuilding the server
- **Smaller Binary**: Server binary is smaller and more focused

**Cons:**
- **More Complex Deployment**: Users need to deploy and configure two services
- **Connection Management**: Need to handle connection failures and retries
- **Configuration Overhead**: Need to configure connection details
- **Potential Network Issues**: Network latency and reliability concerns

### Recommendation

For the progmo-mcp-server project, **embedding Qdrant** is recommended for the following reasons:

1. **User Experience**: The primary users are developers running the server locally, who would benefit from a simplified setup
2. **Local Usage**: Since the server will be run locally, network issues are less of a concern
3. **Scale Requirements**: The expected scale for local development is well within Qdrant's capabilities on a single machine
4. **Deployment Simplicity**: A single binary is easier to distribute and install

However, to maintain flexibility, we should implement the vector store integration using a trait-based approach that allows for both embedded and external Qdrant instances, as well as potential future vector store implementations.

## Implementation Design

### Vector Store Trait

The core of the vector store integration will be a `VectorStore` trait that defines the interface for all vector store implementations:

```rust
pub trait VectorStore: Send + Sync + 'static {
    /// Test the connection to the vector store
    async fn test_connection(&self) -> Result<(), VectorStoreError>;
    
    /// Create a new collection
    async fn create_collection(&self, name: &str, vector_size: usize) -> Result<(), VectorStoreError>;
    
    /// Delete a collection
    async fn delete_collection(&self, name: &str) -> Result<(), VectorStoreError>;
    
    /// List all collections
    async fn list_collections(&self) -> Result<Vec<String>, VectorStoreError>;
    
    /// Insert a document into a collection
    async fn insert_document(&self, collection: &str, document: Document) -> Result<String, VectorStoreError>;
    
    /// Batch insert documents into a collection
    async fn batch_insert(&self, collection: &str, documents: Vec<Document>) -> Result<Vec<String>, VectorStoreError>;
    
    /// Get a document by ID
    async fn get_document(&self, collection: &str, id: &str) -> Result<Document, VectorStoreError>;
    
    /// Update a document
    async fn update_document(&self, collection: &str, id: &str, document: Document) -> Result<(), VectorStoreError>;
    
    /// Delete a document
    async fn delete_document(&self, collection: &str, id: &str) -> Result<(), VectorStoreError>;
    
    /// Search for documents
    async fn search(&self, collection: &str, query: SearchQuery) -> Result<Vec<SearchResult>, VectorStoreError>;
    
    /// Search with filtering
    async fn filtered_search(&self, collection: &str, query: SearchQuery, filter: Filter) -> Result<Vec<SearchResult>, VectorStoreError>;
}
```

### Document and Query Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Optional document ID (will be generated if not provided)
    pub id: Option<String>,
    
    /// Document content
    pub content: String,
    
    /// Vector embedding
    pub embedding: Vec<f32>,
    
    /// Metadata as JSON
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Vector embedding to search for
    pub embedding: Vec<f32>,
    
    /// Maximum number of results to return
    pub limit: usize,
    
    /// Offset for pagination
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matching document
    pub document: Document,
    
    /// Similarity score (higher is more similar)
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct Filter {
    /// Filter conditions (combined with AND logic)
    pub conditions: Vec<FilterCondition>,
}

#[derive(Debug, Clone)]
pub enum FilterCondition {
    /// Field equals value
    Equals(String, serde_json::Value),
    
    /// Field is in range
    Range(String, RangeValue),
    
    /// Field contains any of the values
    Contains(String, Vec<serde_json::Value>),
    
    /// Nested conditions with OR logic
    Or(Vec<FilterCondition>),
}

#[derive(Debug, Clone)]
pub struct RangeValue {
    /// Minimum value (inclusive)
    pub min: Option<serde_json::Value>,
    
    /// Maximum value (inclusive)
    pub max: Option<serde_json::Value>,
}
```

### Error Handling

```rust
#[derive(Debug, Error)]
pub enum VectorStoreError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}
```

## Qdrant Implementation

### Embedded Qdrant

For the embedded Qdrant implementation, we'll use the `qdrant-in-memory` crate to run Qdrant in-process:

```rust
use qdrant_in_memory::QdrantInMemory;

pub struct EmbeddedQdrantConnector {
    qdrant: QdrantInMemory,
}

impl EmbeddedQdrantConnector {
    pub fn new() -> Self {
        Self {
            qdrant: QdrantInMemory::new(),
        }
    }
}

impl VectorStore for EmbeddedQdrantConnector {
    // Implementation of VectorStore trait methods
    // ...
}
```

### External Qdrant

For the external Qdrant implementation, we'll use the `qdrant-client` crate:

```rust
use qdrant_client::client::QdrantClient;
use qdrant_client::qdrant::VectorParams;

pub struct QdrantConnector {
    client: QdrantClient,
    config: QdrantConfig,
}

impl QdrantConnector {
    pub async fn new(config: QdrantConfig) -> Result<Self, VectorStoreError> {
        let client = QdrantClient::new(&config.url)
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;
            
        Ok(Self {
            client,
            config,
        })
    }
}

impl VectorStore for QdrantConnector {
    // Implementation of VectorStore trait methods
    // ...
}
```

### Factory Pattern

To support both embedded and external Qdrant, we'll use a factory pattern:

```rust
pub enum QdrantMode {
    Embedded,
    External(QdrantConfig),
}

pub struct QdrantFactory;

impl QdrantFactory {
    pub async fn create(mode: QdrantMode) -> Result<Box<dyn VectorStore>, VectorStoreError> {
        match mode {
            QdrantMode::Embedded => {
                let connector = EmbeddedQdrantConnector::new();
                Ok(Box::new(connector))
            },
            QdrantMode::External(config) => {
                let connector = QdrantConnector::new(config).await?;
                Ok(Box::new(connector))
            },
        }
    }
}
```

## Documentation

### API Documentation

```rust
/// A trait defining the interface for vector store implementations.
///
/// This trait provides methods for managing collections and documents in a vector store,
/// as well as performing vector similarity searches.
///
/// # Examples
///
/// ```
/// use p_mo::vector_store::{VectorStore, Document, SearchQuery};
///
/// async fn example(store: &dyn VectorStore) -> Result<(), Box<dyn std::error::Error>> {
///     // Create a collection
///     store.create_collection("my_collection", 384).await?;
///
///     // Insert a document
///     let doc = Document {
///         id: None,
///         content: "Example document".to_string(),
///         embedding: vec![0.1, 0.2, 0.3], // In a real app, this would be generated from the content
///         metadata: serde_json::json!({
///             "title": "Example",
///             "tags": ["example", "documentation"]
///         }),
///     };
///
///     let id = store.insert_document("my_collection", doc).await?;
///
///     // Search for similar documents
///     let query = SearchQuery {
///         embedding: vec![0.1, 0.2, 0.3],
///         limit: 10,
///         offset: 0,
///     };
///
///     let results = store.search("my_collection", query).await?;
///     
///     Ok(())
/// }
/// ```
pub trait VectorStore {
    // Method documentation...
}
```

### Usage Examples

```rust
// Example 1: Creating and using an embedded Qdrant store
async fn example_embedded() -> Result<(), Box<dyn std::error::Error>> {
    // Create an embedded Qdrant store
    let store = QdrantFactory::create(QdrantMode::Embedded).await?;
    
    // Create a collection
    store.create_collection("knowledge", 384).await?;
    
    // Insert a document
    let doc = Document {
        id: None,
        content: "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.",
        embedding: generate_embedding("Rust is a systems programming language...").await?,
        metadata: serde_json::json!({
            "title": "About Rust",
            "tags": ["rust", "programming", "systems"]
        }),
    };
    
    let id = store.insert_document("knowledge", doc).await?;
    println!("Inserted document with ID: {}", id);
    
    // Search for similar documents
    let query = SearchQuery {
        embedding: generate_embedding("systems programming").await?,
        limit: 10,
        offset: 0,
    };
    
    let results = store.search("knowledge", query).await?;
    for (i, result) in results.iter().enumerate() {
        println!("Result {}: {} (score: {})", i + 1, result.document.content, result.score);
    }
    
    Ok(())
}

// Example 2: Creating and using an external Qdrant store
async fn example_external() -> Result<(), Box<dyn std::error::Error>> {
    // Create an external Qdrant store
    let config = QdrantConfig {
        url: "http://localhost:6333".to_string(),
        timeout: Duration::from_secs(30),
    };
    
    let store = QdrantFactory::create(QdrantMode::External(config)).await?;
    
    // Use the store...
    
    Ok(())
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_embedded_qdrant_connection() {
        let store = QdrantFactory::create(QdrantMode::Embedded).await.unwrap();
        assert!(store.test_connection().await.is_ok());
    }
    
    #[tokio::test]
    async fn test_embedded_qdrant_collection_operations() {
        let store = QdrantFactory::create(QdrantMode::Embedded).await.unwrap();
        
        // Create collection
        assert!(store.create_collection("test_collection", 3).await.is_ok());
        
        // List collections
        let collections = store.list_collections().await.unwrap();
        assert!(collections.contains(&"test_collection".to_string()));
        
        // Delete collection
        assert!(store.delete_collection("test_collection").await.is_ok());
        
        // Verify deletion
        let collections = store.list_collections().await.unwrap();
        assert!(!collections.contains(&"test_collection".to_string()));
    }
    
    #[tokio::test]
    async fn test_embedded_qdrant_document_operations() {
        let store = QdrantFactory::create(QdrantMode::Embedded).await.unwrap();
        
        // Create collection
        store.create_collection("test_docs", 3).await.unwrap();
        
        // Create document
        let doc = Document {
            id: None,
            content: "Test document".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: serde_json::json!({
                "title": "Test",
                "tags": ["test"]
            }),
        };
        
        // Insert document
        let id = store.insert_document("test_docs", doc.clone()).await.unwrap();
        
        // Get document
        let retrieved = store.get_document("test_docs", &id).await.unwrap();
        assert_eq!(retrieved.content, "Test document");
        
        // Update document
        let updated_doc = Document {
            id: Some(id.clone()),
            content: "Updated document".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: serde_json::json!({
                "title": "Updated Test",
                "tags": ["test", "updated"]
            }),
        };
        
        store.update_document("test_docs", &id, updated_doc).await.unwrap();
        
        // Verify update
        let retrieved = store.get_document("test_docs", &id).await.unwrap();
        assert_eq!(retrieved.content, "Updated document");
        
        // Delete document
        store.delete_document("test_docs", &id).await.unwrap();
        
        // Verify deletion
        let result = store.get_document("test_docs", &id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VectorStoreError::DocumentNotFound(_)));
    }
    
    #[tokio::test]
    async fn test_embedded_qdrant_search() {
        let store = QdrantFactory::create(QdrantMode::Embedded).await.unwrap();
        
        // Create collection
        store.create_collection("test_search", 3).await.unwrap();
        
        // Insert documents
        let docs = vec![
            Document {
                id: None,
                content: "The quick brown fox jumps over the lazy dog".to_string(),
                embedding: vec![0.1, 0.2, 0.3],
                metadata: serde_json::json!({"animal": "fox"}),
            },
            Document {
                id: None,
                content: "The lazy dog sleeps all day".to_string(),
                embedding: vec![0.2, 0.3, 0.4],
                metadata: serde_json::json!({"animal": "dog"}),
            },
            Document {
                id: None,
                content: "The quick rabbit runs fast".to_string(),
                embedding: vec![0.3, 0.4, 0.5],
                metadata: serde_json::json!({"animal": "rabbit"}),
            },
        ];
        
        let ids = store.batch_insert("test_search", docs).await.unwrap();
        
        // Search
        let query = SearchQuery {
            embedding: vec![0.1, 0.2, 0.3],
            limit: 2,
            offset: 0,
        };
        
        let results = store.search("test_search", query).await.unwrap();
        
        // Verify results
        assert_eq!(results.len(), 2);
        assert!(results[0].score > results[1].score);
        
        // Filtered search
        let filter = Filter {
            conditions: vec![
                FilterCondition::Equals("animal".to_string(), serde_json::json!("dog")),
            ],
        };
        
        let results = store.filtered_search("test_search", query, filter).await.unwrap();
        
        // Verify filtered results
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document.metadata["animal"], "dog");
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_external_qdrant_connection() {
        // Skip test if QDRANT_URL environment variable is not set
        let qdrant_url = match std::env::var("QDRANT_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping external Qdrant test: QDRANT_URL not set");
                return;
            }
        };
        
        let config = QdrantConfig {
            url: qdrant_url,
            timeout: Duration::from_secs(30),
        };
        
        let store = QdrantFactory::create(QdrantMode::External(config)).await.unwrap();
        assert!(store.test_connection().await.is_ok());
    }
    
    #[tokio::test]
    async fn test_vector_store_with_real_embeddings() {
        // This test uses a real embedding model to generate embeddings
        
        let store = QdrantFactory::create(QdrantMode::Embedded).await.unwrap();
        
        // Create collection
        store.create_collection("real_embeddings", 384).await.unwrap();
        
        // Generate real embeddings
        let embeddings = vec![
            generate_real_embedding("The quick brown fox jumps over the lazy dog").await.unwrap(),
            generate_real_embedding("The lazy dog sleeps all day").await.unwrap(),
            generate_real_embedding("The quick rabbit runs fast").await.unwrap(),
        ];
        
        // Insert documents with real embeddings
        let docs = vec![
            Document {
                id: None,
                content: "The quick brown fox jumps over the lazy dog".to_string(),
                embedding: embeddings[0].clone(),
                metadata: serde_json::json!({"animal": "fox"}),
            },
            Document {
                id: None,
                content: "The lazy dog sleeps all day".to_string(),
                embedding: embeddings[1].clone(),
                metadata: serde_json::json!({"animal": "dog"}),
            },
            Document {
                id: None,
                content: "The quick rabbit runs fast".to_string(),
                embedding: embeddings[2].clone(),
                metadata: serde_json::json!({"animal": "rabbit"}),
            },
        ];
        
        store.batch_insert("real_embeddings", docs).await.unwrap();
        
        // Search with a real query embedding
        let query_embedding = generate_real_embedding("dog sleeping").await.unwrap();
        
        let query = SearchQuery {
            embedding: query_embedding,
            limit: 3,
            offset: 0,
        };
        
        let results = store.search("real_embeddings", query).await.unwrap();
        
        // Verify that the most relevant result is returned first
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].document.metadata["animal"], "dog");
    }
    
    async fn generate_real_embedding(text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
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
        
        Ok(result)
    }
}
```

### Performance Tests

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_batch_insert_performance() {
        let store = QdrantFactory::create(QdrantMode::Embedded).await.unwrap();
        
        // Create collection
        store.create_collection("perf_test", 384).await.unwrap();
        
        // Create a large number of documents
        const NUM_DOCS: usize = 1000;
        let mut docs = Vec::with_capacity(NUM_DOCS);
        
        for i in 0..NUM_DOCS {
            let embedding = vec![0.0; 384]; // Simple embedding for performance testing
            
            docs.push(Document {
                id: None,
                content: format!("Document {}", i),
                embedding,
                metadata: serde_json::json!({"index": i}),
            });
        }
        
        // Measure batch insert performance
        let start = Instant::now();
        store.batch_insert("perf_test", docs).await.unwrap();
        let duration = start.elapsed();
        
        println!("Batch insert of {} documents took {:?}", NUM_DOCS, duration);
        
        // Ensure the operation completes in a reasonable time
        assert!(duration.as_secs() < 10, "Batch insert took too long: {:?}", duration);
    }
    
    #[tokio::test]
    async fn test_search_performance() {
        let store = QdrantFactory::create(QdrantMode::Embedded).await.unwrap();
        
        // Create collection
        store.create_collection("search_perf", 384).await.unwrap();
        
        // Insert a large number of documents
        const NUM_DOCS: usize = 1000;
        let mut docs = Vec::with_capacity(NUM_DOCS);
        
        for i in 0..NUM_DOCS {
            let mut embedding = vec![0.0; 384];
            // Create slightly different embeddings
            for j in 0..384 {
                embedding[j] = (i as f32 * j as f32) % 1.0;
            }
            
            docs.push(Document {
                id: None,
                content: format!("Document {}", i),
                embedding,
                metadata: serde_json::json!({"index": i}),
            });
        }
        
        store.batch_insert("search_perf", docs).await.unwrap();
        
        // Create a query
        let query = SearchQuery {
            embedding: vec![0.5; 384],
            limit: 10,
            offset: 0,
        };
        
        // Measure search performance
        let start = Instant::now();
        let results = store.search("search_perf", query).await.unwrap();
        let duration = start.elapsed();
        
        println!("Search in {} documents took {:?}", NUM_DOCS, duration);
        
        // Ensure the operation completes in a reasonable time
        assert!(duration.as_millis() < 500, "Search took too long: {:?}", duration);
        assert_eq!(results.len(), 10);
    }
}
```

## Implementation Plan

### Phase 1: Core Vector Store Trait (Week 1)

1. Define the `VectorStore` trait
2. Implement data structures (Document, SearchQuery, etc.)
3. Define error types
4. Create unit tests for the trait

### Phase 2: Embedded Qdrant Implementation (Week 2)

1. Add the `qdrant-in-memory` dependency
2. Implement the `EmbeddedQdrantConnector` struct
3. Implement the `VectorStore` trait for `EmbeddedQdrantConnector`
4. Write unit tests for the embedded implementation

### Phase 3: External Qdrant Implementation (Week 3)

1. Add the `qdrant-client` dependency
2. Implement the `QdrantConnector` struct
3. Implement the `VectorStore` trait for `QdrantConnector`
4. Write unit tests for the external implementation
5. Implement connection pooling and retry logic

### Phase 4: Factory and Integration (Week 4)

1. Implement the `QdrantFactory` for creating vector store instances
2. Create integration tests
3. Implement performance tests
4. Document the API and usage examples
5. Integrate with the rest of the system

## Conclusion

The vector store integration with Qdrant is a critical component of the progmo-mcp-server project. By implementing both embedded and external Qdrant connectors, we provide flexibility while maintaining ease of use for local development.

The trait-based design allows for future extensions to support other vector stores, while the comprehensive testing strategy ensures reliability and performance.
