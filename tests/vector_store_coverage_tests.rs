use p_mo::vector_store::{
    Document, EmbeddedQdrantConnector, Filter, FilterCondition, RangeValue, SearchQuery, VectorStore,
    VectorStoreError,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_vector_store_error_handling() {
    // Create a vector store
    let store = EmbeddedQdrantConnector::new();
    
    // Test getting a document from a non-existent collection
    let result = store.get_document("non_existent_collection", "123").await;
    assert!(result.is_err());
    match result {
        Err(VectorStoreError::CollectionNotFound(_)) => {}
        _ => panic!("Expected CollectionNotFound error"),
    }
    
    // Create a collection
    store.create_collection("error_test", 3).await.unwrap();
    
    // Test getting a non-existent document
    let result = store.get_document("error_test", "non_existent_doc").await;
    assert!(result.is_err());
    match result {
        Err(VectorStoreError::DocumentNotFound(_)) => {}
        _ => panic!("Expected DocumentNotFound error"),
    }
    
    // Test updating a non-existent document
    let doc = Document {
        id: Some("non_existent_doc".to_string()),
        content: "Test".to_string(),
        embedding: vec![0.1, 0.2, 0.3],
        metadata: json!({}),
    };
    let result = store.update_document("error_test", "non_existent_doc", doc).await;
    assert!(result.is_err());
    match result {
        Err(VectorStoreError::DocumentNotFound(_)) => {}
        _ => panic!("Expected DocumentNotFound error"),
    }
    
    // Test deleting a non-existent document
    let result = store.delete_document("error_test", "non_existent_doc").await;
    assert!(result.is_err());
    match result {
        Err(VectorStoreError::DocumentNotFound(_)) => {}
        _ => panic!("Expected DocumentNotFound error"),
    }
    
    // Test invalid embedding size
    let doc = Document {
        id: None,
        content: "Test".to_string(),
        embedding: vec![0.1, 0.2], // Only 2 dimensions, but collection expects 3
        metadata: json!({}),
    };
    let result = store.insert_document("error_test", doc).await;
    assert!(result.is_err());
    match result {
        Err(VectorStoreError::InvalidArgument(_)) => {}
        _ => panic!("Expected InvalidArgument error"),
    }
    
    // Test search with invalid embedding size
    let query = SearchQuery {
        embedding: vec![0.1, 0.2], // Only 2 dimensions, but collection expects 3
        limit: 10,
        offset: 0,
    };
    let result = store.search("error_test", query).await;
    assert!(result.is_err());
    match result {
        Err(VectorStoreError::InvalidArgument(_)) => {}
        _ => panic!("Expected InvalidArgument error"),
    }
}

#[tokio::test]
async fn test_vector_store_complex_operations() {
    // Create a vector store
    let store = EmbeddedQdrantConnector::new();
    
    // Create a collection
    store.create_collection("complex_test", 3).await.unwrap();
    
    // Insert documents with metadata
    let docs = vec![
        Document {
            id: None,
            content: "Document about cats".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: json!({
                "category": "animals",
                "tags": ["cat", "pet"],
                "views": 100
            }),
        },
        Document {
            id: None,
            content: "Document about dogs".to_string(),
            embedding: vec![0.2, 0.3, 0.4],
            metadata: json!({
                "category": "animals",
                "tags": ["dog", "pet"],
                "views": 200
            }),
        },
        Document {
            id: None,
            content: "Document about cars".to_string(),
            embedding: vec![0.3, 0.4, 0.5],
            metadata: json!({
                "category": "vehicles",
                "tags": ["car", "transportation"],
                "views": 150
            }),
        },
    ];
    
    let ids = store.batch_insert("complex_test", docs).await.unwrap();
    assert_eq!(ids.len(), 3);
    
    // Test filtered search with equals condition
    let filter = Filter {
        conditions: vec![FilterCondition::Equals(
            "category".to_string(),
            json!("animals"),
        )],
    };
    
    let query = SearchQuery {
        embedding: vec![0.1, 0.2, 0.3],
        limit: 10,
        offset: 0,
    };
    
    let results = store.filtered_search("complex_test", query.clone(), filter).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.document.metadata["category"] == "animals"));
    
    // Test filtered search with range condition
    let filter = Filter {
        conditions: vec![FilterCondition::Range(
            "views".to_string(),
            RangeValue {
                min: Some(json!(150)),
                max: None,
            },
        )],
    };
    
    let results = store.filtered_search("complex_test", query.clone(), filter).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.document.metadata["views"].as_i64().unwrap() >= 150));
    
    // Test filtered search with contains condition
    let filter = Filter {
        conditions: vec![FilterCondition::Contains(
            "tags".to_string(),
            vec![json!("pet")],
        )],
    };
    
    let results = store.filtered_search("complex_test", query.clone(), filter).await.unwrap();
    assert_eq!(results.len(), 2);
    
    // Test filtered search with OR condition
    let filter = Filter {
        conditions: vec![FilterCondition::Or(vec![
            FilterCondition::Equals("category".to_string(), json!("vehicles")),
            FilterCondition::Range(
                "views".to_string(),
                RangeValue {
                    min: Some(json!(200)),
                    max: None,
                },
            ),
        ])],
    };
    
    let results = store.filtered_search("complex_test", query.clone(), filter).await.unwrap();
    assert_eq!(results.len(), 2);
    
    // Test pagination
    let query_with_offset = SearchQuery {
        embedding: vec![0.1, 0.2, 0.3],
        limit: 1,
        offset: 1,
    };
    
    let results = store.search("complex_test", query_with_offset).await.unwrap();
    assert_eq!(results.len(), 1);
    
    // Test document update
    let doc_id = &ids[0];
    let updated_doc = Document {
        id: Some(doc_id.clone()),
        content: "Updated document about cats".to_string(),
        embedding: vec![0.1, 0.2, 0.3],
        metadata: json!({
            "category": "animals",
            "tags": ["cat", "pet", "updated"],
            "views": 150
        }),
    };
    
    store.update_document("complex_test", doc_id, updated_doc).await.unwrap();
    
    // Verify update
    let retrieved = store.get_document("complex_test", doc_id).await.unwrap();
    assert_eq!(retrieved.content, "Updated document about cats");
    assert_eq!(retrieved.metadata["views"], 150);
    
    // Test document deletion
    store.delete_document("complex_test", doc_id).await.unwrap();
    
    // Verify deletion
    let result = store.get_document("complex_test", doc_id).await;
    assert!(result.is_err());
    
    // Test collection deletion
    store.delete_collection("complex_test").await.unwrap();
    
    // Verify collection deletion
    let collections = store.list_collections().await.unwrap();
    assert!(!collections.contains(&"complex_test".to_string()));
}

#[tokio::test]
async fn test_as_any_method() {
    // Test the as_any method which is used for downcasting
    let store = EmbeddedQdrantConnector::new();
    
    // Get a reference to the store as a trait object
    let store_trait: &dyn VectorStore = &store;
    
    // Use as_any to downcast to the concrete type
    let downcast_result = store_trait.as_any().downcast_ref::<EmbeddedQdrantConnector>();
    
    // Verify downcast succeeded
    assert!(downcast_result.is_some());
}

#[tokio::test]
async fn test_empty_vector_handling() {
    // Create a vector store
    let store = EmbeddedQdrantConnector::new();
    
    // Create a collection with a non-zero vector size
    store.create_collection("empty_test", 3).await.unwrap();
    
    // Insert a document with an empty embedding
    let doc = Document {
        id: None,
        content: "Empty embedding".to_string(),
        embedding: vec![],
        metadata: json!({}),
    };
    
    // This should fail with an InvalidArgument error
    let result = store.insert_document("empty_test", doc).await;
    assert!(result.is_err());
    match result {
        Err(VectorStoreError::InvalidArgument(_)) => {}
        _ => panic!("Expected InvalidArgument error"),
    }
}
