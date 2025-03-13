#[cfg(test)]
mod vector_store_tests {
    use p_mo::vector_store::{QdrantConnector, VectorStore};
    use std::time::Duration;

    #[test]
    fn test_qdrant_connection() {
        // Skip test if QDRANT_URL environment variable is not set
        let qdrant_url = match std::env::var("QDRANT_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping Qdrant test: QDRANT_URL not set");
                return;
            }
        };
        
        // Initialize Qdrant connector
        let connector = QdrantConnector::new(&qdrant_url, Duration::from_secs(5))
            .expect("Failed to create Qdrant connector");
        
        // Test connection
        assert!(connector.test_connection().is_ok(), "Failed to connect to Qdrant");
        
        // Create test collection
        let collection_name = format!("test_collection_{}", chrono::Utc::now().timestamp());
        let create_result = connector.create_collection(&collection_name, 384);
        assert!(create_result.is_ok(), "Failed to create collection: {:?}", create_result);
        
        // Clean up
        let delete_result = connector.delete_collection(&collection_name);
        assert!(delete_result.is_ok(), "Failed to delete collection: {:?}", delete_result);
    }
}
