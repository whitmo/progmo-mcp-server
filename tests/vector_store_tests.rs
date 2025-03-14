#[cfg(test)]
mod vector_store_tests {
    use p_mo::vector_store::{QdrantConnector, VectorStore, QdrantConfig, VectorStoreError, Document, SearchQuery, cosine_similarity};
    use std::time::Duration;
    use uuid::Uuid;
    use tokio::test;

    #[tokio::test]
    async fn test_qdrant_connection() {
        // Skip test if QDRANT_URL environment variable is not set
        let qdrant_url = match std::env::var("QDRANT_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping Qdrant test: QDRANT_URL not set");
                return;
            }
        };
        
        // Initialize Qdrant connector with config
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
        
        let connector = QdrantConnector::new(config).await
            .expect("Failed to create Qdrant connector");
        
        // Test connection
        assert!(connector.test_connection().await.is_ok(), "Failed to connect to Qdrant");
        
        // Create test collection
        let collection_name = format!("test_collection_{}", chrono::Utc::now().timestamp());
        let create_result = connector.create_collection(&collection_name, 384).await;
        assert!(create_result.is_ok(), "Failed to create collection: {:?}", create_result);
        
        // Clean up
        let delete_result = connector.delete_collection(&collection_name).await;
        assert!(delete_result.is_ok(), "Failed to delete collection: {:?}", delete_result);
    }
    
    #[tokio::test]
    async fn test_qdrant_retry_logic() {
        // This test is more of an integration test and requires a real Qdrant instance
        // Skip if QDRANT_URL is not set
        let qdrant_url = match std::env::var("QDRANT_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping Qdrant retry test: QDRANT_URL not set");
                return;
            }
        };
        
        // Initialize Qdrant connector with retry config
        let config = QdrantConfig {
            url: qdrant_url,
            timeout: Duration::from_secs(1), // Short timeout to trigger retries
            max_connections: 3,
            api_key: std::env::var("QDRANT_API_KEY").ok(),
            retry_max_elapsed_time: Duration::from_secs(10),
            retry_initial_interval: Duration::from_millis(100),
            retry_max_interval: Duration::from_secs(1),
            retry_multiplier: 1.5,
        };
        
        let connector = QdrantConnector::new(config).await
            .expect("Failed to create Qdrant connector");
        
        // Test connection with retry
        let result = connector.test_connection().await;
        assert!(result.is_ok(), "Failed to connect to Qdrant with retry: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_qdrant_connection_pooling() {
        // Skip if QDRANT_URL is not set
        let qdrant_url = match std::env::var("QDRANT_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping Qdrant connection pooling test: QDRANT_URL not set");
                return;
            }
        };
        
        // Initialize Qdrant connector with connection pooling
        let config = QdrantConfig {
            url: qdrant_url,
            timeout: Duration::from_secs(5),
            max_connections: 5, // Set pool size
            api_key: std::env::var("QDRANT_API_KEY").ok(),
            retry_max_elapsed_time: Duration::from_secs(30),
            retry_initial_interval: Duration::from_millis(100),
            retry_max_interval: Duration::from_secs(5),
            retry_multiplier: 1.5,
        };
        
        let connector = QdrantConnector::new(config).await
            .expect("Failed to create Qdrant connector");
        
        // Run multiple operations concurrently to test connection pooling
        let mut handles = Vec::new();
        for i in 0..10 {
            let connector_clone = connector.clone();
            let handle = tokio::spawn(async move {
                let collection_name = format!("test_pool_{}_{}", i, chrono::Utc::now().timestamp());
                let create_result = connector_clone.create_collection(&collection_name, 384).await;
                assert!(create_result.is_ok(), "Failed to create collection in thread {}: {:?}", i, create_result);
                
                let delete_result = connector_clone.delete_collection(&collection_name).await;
                assert!(delete_result.is_ok(), "Failed to delete collection in thread {}: {:?}", i, delete_result);
                
                Ok::<_, VectorStoreError>(())
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_ok(), "Task {} failed: {:?}", i, result);
        }
    }
    
    #[tokio::test]
    async fn test_document_insertion_and_search() {
        // Skip if QDRANT_URL is not set
        let qdrant_url = match std::env::var("QDRANT_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping Qdrant document test: QDRANT_URL not set");
                return;
            }
        };
        
        // Initialize Qdrant connector
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
        
        let connector = QdrantConnector::new(config).await
            .expect("Failed to create Qdrant connector");
        
        // Create test collection
        let collection_name = format!("test_docs_{}", chrono::Utc::now().timestamp());
        let vector_size = 3; // Small size for testing
        connector.create_collection(&collection_name, vector_size).await
            .expect("Failed to create collection");
        
        // Create test documents
        let documents = vec![
            Document {
                id: Uuid::new_v4().to_string(),
                content: "This is a test document about artificial intelligence".to_string(),
                embedding: vec![1.0, 0.5, 0.1],
            },
            Document {
                id: Uuid::new_v4().to_string(),
                content: "Document about machine learning and neural networks".to_string(),
                embedding: vec![0.9, 0.4, 0.2],
            },
            Document {
                id: Uuid::new_v4().to_string(),
                content: "Information about databases and storage systems".to_string(),
                embedding: vec![0.1, 0.2, 0.9],
            },
        ];
        
        // Insert documents
        for document in &documents {
            connector.insert_document(&collection_name, document.clone()).await
                .expect("Failed to insert document");
        }
        
        // Search for documents similar to the first document
        let query = SearchQuery {
            embedding: documents[0].embedding.clone(),
            limit: 2,
        };
        
        let results = connector.search(&collection_name, query).await
            .expect("Failed to search for documents");
        
        // Verify results
        assert!(!results.is_empty(), "Search returned no results");
        assert!(results.len() <= 2, "Search returned too many results");
        
        // The first result should be the document itself or very similar
        if !results.is_empty() {
            let first_result = &results[0];
            let similarity = cosine_similarity(&first_result.document.embedding, &documents[0].embedding);
            assert!(similarity > 0.9, "First result is not similar enough to query");
        }
        
        // Clean up
        connector.delete_collection(&collection_name).await
            .expect("Failed to delete collection");
    }
}
