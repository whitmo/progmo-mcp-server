mod pure;

use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

pub use pure::{Document, Filter, FilterCondition, RangeValue, SearchQuery, SearchResult};

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

#[async_trait::async_trait]
pub trait VectorStore: Send + Sync + 'static {
    /// Get a reference to self as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
    
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

pub struct QdrantConfig {
    pub url: String,
    pub timeout: Duration,
}

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
                let connector = ExternalQdrantConnector::new(config).await?;
                Ok(Box::new(connector))
            },
        }
    }
}

#[derive(Clone)]
pub struct EmbeddedQdrantConnector {
    collections: Arc<tokio::sync::Mutex<std::collections::HashMap<String, EmbeddedCollection>>>,
}

struct EmbeddedCollection {
    vector_size: usize,
    documents: std::collections::HashMap<String, Document>,
}

impl EmbeddedQdrantConnector {
    pub fn new() -> Self {
        Self {
            collections: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl VectorStore for EmbeddedQdrantConnector {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    async fn test_connection(&self) -> Result<(), VectorStoreError> {
        // For embedded connector, connection is always successful
        Ok(())
    }
    
    async fn create_collection(&self, name: &str, vector_size: usize) -> Result<(), VectorStoreError> {
        let mut collections = self.collections.lock().await;
        
        if collections.contains_key(name) {
            return Err(VectorStoreError::OperationFailed(format!("Collection '{}' already exists", name)));
        }
        
        collections.insert(name.to_string(), EmbeddedCollection {
            vector_size,
            documents: std::collections::HashMap::new(),
        });
        
        Ok(())
    }
    
    async fn delete_collection(&self, name: &str) -> Result<(), VectorStoreError> {
        let mut collections = self.collections.lock().await;
        
        if !collections.contains_key(name) {
            return Err(VectorStoreError::CollectionNotFound(name.to_string()));
        }
        
        collections.remove(name);
        
        Ok(())
    }
    
    async fn list_collections(&self) -> Result<Vec<String>, VectorStoreError> {
        let collections = self.collections.lock().await;
        
        let names = collections.keys().cloned().collect();
        
        Ok(names)
    }
    
    async fn insert_document(&self, collection: &str, document: Document) -> Result<String, VectorStoreError> {
        let mut collections = self.collections.lock().await;
        
        let collection_data = collections.get_mut(collection)
            .ok_or_else(|| VectorStoreError::CollectionNotFound(collection.to_string()))?;
        
        // Validate vector size
        if document.embedding.len() != collection_data.vector_size {
            return Err(VectorStoreError::InvalidArgument(format!(
                "Document embedding size ({}) does not match collection vector size ({})",
                document.embedding.len(),
                collection_data.vector_size
            )));
        }
        
        // Generate ID if not provided
        let id = document.id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        // Create document with ID
        let doc_with_id = Document {
            id: Some(id.clone()),
            ..document
        };
        
        // Insert document
        collection_data.documents.insert(id.clone(), doc_with_id);
        
        Ok(id)
    }
    
    async fn batch_insert(&self, collection: &str, documents: Vec<Document>) -> Result<Vec<String>, VectorStoreError> {
        let mut ids = Vec::with_capacity(documents.len());
        
        for document in documents {
            let id = self.insert_document(collection, document).await?;
            ids.push(id);
        }
        
        Ok(ids)
    }
    
    async fn get_document(&self, collection: &str, id: &str) -> Result<Document, VectorStoreError> {
        let collections = self.collections.lock().await;
        
        let collection_data = collections.get(collection)
            .ok_or_else(|| VectorStoreError::CollectionNotFound(collection.to_string()))?;
        
        let document = collection_data.documents.get(id)
            .ok_or_else(|| VectorStoreError::DocumentNotFound(id.to_string()))?;
        
        Ok(document.clone())
    }
    
    async fn update_document(&self, collection: &str, id: &str, document: Document) -> Result<(), VectorStoreError> {
        let mut collections = self.collections.lock().await;
        
        let collection_data = collections.get_mut(collection)
            .ok_or_else(|| VectorStoreError::CollectionNotFound(collection.to_string()))?;
        
        if !collection_data.documents.contains_key(id) {
            return Err(VectorStoreError::DocumentNotFound(id.to_string()));
        }
        
        // Validate vector size
        if document.embedding.len() != collection_data.vector_size {
            return Err(VectorStoreError::InvalidArgument(format!(
                "Document embedding size ({}) does not match collection vector size ({})",
                document.embedding.len(),
                collection_data.vector_size
            )));
        }
        
        // Create document with ID
        let doc_with_id = Document {
            id: Some(id.to_string()),
            ..document
        };
        
        // Update document
        collection_data.documents.insert(id.to_string(), doc_with_id);
        
        Ok(())
    }
    
    async fn delete_document(&self, collection: &str, id: &str) -> Result<(), VectorStoreError> {
        let mut collections = self.collections.lock().await;
        
        let collection_data = collections.get_mut(collection)
            .ok_or_else(|| VectorStoreError::CollectionNotFound(collection.to_string()))?;
        
        if !collection_data.documents.contains_key(id) {
            return Err(VectorStoreError::DocumentNotFound(id.to_string()));
        }
        
        collection_data.documents.remove(id);
        
        Ok(())
    }
    
    async fn search(&self, collection: &str, query: SearchQuery) -> Result<Vec<SearchResult>, VectorStoreError> {
        let collections = self.collections.lock().await;
        
        let collection_data = collections.get(collection)
            .ok_or_else(|| VectorStoreError::CollectionNotFound(collection.to_string()))?;
        
        // Validate query vector size
        if query.embedding.len() != collection_data.vector_size {
            return Err(VectorStoreError::InvalidArgument(format!(
                "Query embedding size ({}) does not match collection vector size ({})",
                query.embedding.len(),
                collection_data.vector_size
            )));
        }
        
        // Calculate similarity scores
        let mut results: Vec<SearchResult> = collection_data.documents.values()
            .map(|doc| {
                let score = pure::cosine_similarity(&query.embedding, &doc.embedding);
                SearchResult {
                    document: doc.clone(),
                    score,
                }
            })
            .collect();
        
        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Apply pagination
        let start = query.offset.min(results.len());
        let end = (query.offset + query.limit).min(results.len());
        
        Ok(results[start..end].to_vec())
    }
    
    async fn filtered_search(&self, collection: &str, query: SearchQuery, filter: Filter) -> Result<Vec<SearchResult>, VectorStoreError> {
        let collections = self.collections.lock().await;
        
        let collection_data = collections.get(collection)
            .ok_or_else(|| VectorStoreError::CollectionNotFound(collection.to_string()))?;
        
        // Validate query vector size
        if query.embedding.len() != collection_data.vector_size {
            return Err(VectorStoreError::InvalidArgument(format!(
                "Query embedding size ({}) does not match collection vector size ({})",
                query.embedding.len(),
                collection_data.vector_size
            )));
        }
        
        // Calculate similarity scores and apply filter
        let mut results: Vec<SearchResult> = collection_data.documents.values()
            .filter(|doc| pure::matches_filter(doc, &filter))
            .map(|doc| {
                let score = pure::cosine_similarity(&query.embedding, &doc.embedding);
                SearchResult {
                    document: doc.clone(),
                    score,
                }
            })
            .collect();
        
        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Apply pagination
        let start = query.offset.min(results.len());
        let end = (query.offset + query.limit).min(results.len());
        
        Ok(results[start..end].to_vec())
    }
}

struct ExternalQdrantConnector {
    config: QdrantConfig,
}

impl ExternalQdrantConnector {
    async fn new(config: QdrantConfig) -> Result<Self, VectorStoreError> {
        // In a real implementation, this would connect to an external Qdrant instance
        // For now, we'll just return a new connector
        
        Ok(Self {
            config,
        })
    }
}

#[async_trait::async_trait]
impl VectorStore for ExternalQdrantConnector {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    async fn test_connection(&self) -> Result<(), VectorStoreError> {
        // In a real implementation, this would test the connection to the external Qdrant instance
        // For now, we'll just return Ok
        
        Ok(())
    }
    
    async fn create_collection(&self, _name: &str, _vector_size: usize) -> Result<(), VectorStoreError> {
        // Not implemented for this example
        Err(VectorStoreError::OperationFailed("Not implemented".to_string()))
    }
    
    async fn delete_collection(&self, _name: &str) -> Result<(), VectorStoreError> {
        // Not implemented for this example
        Err(VectorStoreError::OperationFailed("Not implemented".to_string()))
    }
    
    async fn list_collections(&self) -> Result<Vec<String>, VectorStoreError> {
        // Not implemented for this example
        Ok(Vec::new())
    }
    
    async fn insert_document(&self, _collection: &str, _document: Document) -> Result<String, VectorStoreError> {
        // Not implemented for this example
        Err(VectorStoreError::OperationFailed("Not implemented".to_string()))
    }
    
    async fn batch_insert(&self, _collection: &str, _documents: Vec<Document>) -> Result<Vec<String>, VectorStoreError> {
        // Not implemented for this example
        Err(VectorStoreError::OperationFailed("Not implemented".to_string()))
    }
    
    async fn get_document(&self, _collection: &str, _id: &str) -> Result<Document, VectorStoreError> {
        // Not implemented for this example
        Err(VectorStoreError::OperationFailed("Not implemented".to_string()))
    }
    
    async fn update_document(&self, _collection: &str, _id: &str, _document: Document) -> Result<(), VectorStoreError> {
        // Not implemented for this example
        Err(VectorStoreError::OperationFailed("Not implemented".to_string()))
    }
    
    async fn delete_document(&self, _collection: &str, _id: &str) -> Result<(), VectorStoreError> {
        // Not implemented for this example
        Err(VectorStoreError::OperationFailed("Not implemented".to_string()))
    }
    
    async fn search(&self, _collection: &str, _query: SearchQuery) -> Result<Vec<SearchResult>, VectorStoreError> {
        // Not implemented for this example
        Err(VectorStoreError::OperationFailed("Not implemented".to_string()))
    }
    
    async fn filtered_search(&self, _collection: &str, _query: SearchQuery, _filter: Filter) -> Result<Vec<SearchResult>, VectorStoreError> {
        // Not implemented for this example
        Err(VectorStoreError::OperationFailed("Not implemented".to_string()))
    }
}
