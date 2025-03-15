use p_mo::vector_store::{
    Document, EmbeddedQdrantConnector, SearchQuery, VectorStore, VectorStoreError, QdrantConfig
};
use std::time::Duration;

#[tokio::test]
async fn test_vector_store_error_handling() {
    // Create a vector store with config
    let config = QdrantConfig {
        url: "http://localhost:6333".to_string(),
        timeout: Duration::from_secs(5),
        max_connections: 5,
        api_key: None,
        retry_max_elapsed_time: Duration::from_secs(30),
        retry_initial_interval: Duration::from_millis(100),
        retry_max_interval: Duration::from_secs(5),
        retry_multiplier: 1.5,
    };
    
    let store = EmbeddedQdrantConnector::new(config).await.expect("Failed to create connector");
    
    // Test connection
    let result = store.test_connection().await;
    // This might fail if Qdrant is not running, which is expected in a test environment
    if result.is_err() {
        println!("Skipping test_vector_store_error_handling: Qdrant connection failed");
        return;
    }
    
    // Create a test collection
    let collection_name = format!("test_collection_{}", chrono::Utc::now().timestamp());
    let create_result = store.create_collection(&collection_name, 384).await;
    
    if create_result.is_err() {
        println!("Skipping test: Failed to create collection");
        return;
    }
    
    // Test search with invalid embedding size
    let query = SearchQuery {
        embedding: vec![0.1, 0.2], // Only 2 dimensions, but collection expects 384
        limit: 10,
    };
    
    let result = store.search(&collection_name, query).await;
    assert!(result.is_err());
    
    // Clean up
    let _ = store.delete_collection(&collection_name).await;
}

#[tokio::test]
async fn test_document_operations() {
    // Create a vector store with config
    let config = QdrantConfig {
        url: "http://localhost:6333".to_string(),
        timeout: Duration::from_secs(5),
        max_connections: 5,
        api_key: None,
        retry_max_elapsed_time: Duration::from_secs(30),
        retry_initial_interval: Duration::from_millis(100),
        retry_max_interval: Duration::from_secs(5),
        retry_multiplier: 1.5,
    };
    
    let store = EmbeddedQdrantConnector::new(config).await.expect("Failed to create connector");
    
    // Test connection
    let result = store.test_connection().await;
    // This might fail if Qdrant is not running, which is expected in a test environment
    if result.is_err() {
        println!("Skipping test_document_operations: Qdrant connection failed");
        return;
    }
    
    // Create a test collection
    let collection_name = format!("test_collection_{}", chrono::Utc::now().timestamp());
    let create_result = store.create_collection(&collection_name, 3).await;
    
    if create_result.is_err() {
        println!("Skipping test: Failed to create collection");
        return;
    }
    
    // Insert a document
    let doc = Document {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Test document".to_string(),
        embedding: vec![0.1, 0.2, 0.3],
    };
    
    let insert_result = store.insert_document(&collection_name, doc.clone()).await;
    assert!(insert_result.is_ok());
    
    // Search for the document
    let query = SearchQuery {
        embedding: vec![0.1, 0.2, 0.3],
        limit: 10,
    };
    
    let search_result = store.search(&collection_name, query).await;
    assert!(search_result.is_ok());
    
    let results = search_result.unwrap();
    assert!(!results.is_empty());
    
    // Clean up
    let _ = store.delete_collection(&collection_name).await;
}
