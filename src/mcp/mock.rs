use crate::vector_store::{Document, SearchQuery, SearchResult, VectorStore, VectorStoreError};
use async_trait::async_trait;

/// Mock implementation of the EmbeddedQdrantConnector for testing
pub struct MockQdrantConnector;

impl MockQdrantConnector {
    /// Create a new mock connector
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl VectorStore for MockQdrantConnector {
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
    
    async fn search(&self, _collection: &str, _query: SearchQuery) -> Result<Vec<SearchResult>, VectorStoreError> {
        // Return a mock result
        let doc = Document {
            id: "test-id".to_string(),
            content: "Test document".to_string(),
            embedding: vec![0.0; 384],
        };
        
        let result = SearchResult {
            document: doc,
            score: 0.95,
        };
        
        Ok(vec![result])
    }
}
