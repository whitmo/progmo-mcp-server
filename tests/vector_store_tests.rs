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
            metadata: json!({"animal": "fox"}),
        },
        Document {
            id: None,
            content: "The lazy dog sleeps all day".to_string(),
            embedding: vec![0.2, 0.3, 0.4],
            metadata: json!({"animal": "dog"}),
        },
        Document {
            id: None,
            content: "The quick rabbit runs fast".to_string(),
            embedding: vec![0.3, 0.4, 0.5],
            metadata: json!({"animal": "rabbit"}),
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
            FilterCondition::Equals("animal".to_string(), json!("dog")),
        ],
    };
    
    let query = SearchQuery {
        embedding: vec![0.1, 0.2, 0.3],
        limit: 2,
        offset: 0,
    };
    
    let results = store.filtered_search("test_search", query, filter).await.unwrap();
    
    // Verify filtered results
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].document.metadata["animal"], "dog");
}

#[tokio::test]
async fn test_embedded_qdrant_complex_filters() {
    let store = QdrantFactory::create(QdrantMode::Embedded).await.unwrap();
    
    // Create collection
    store.create_collection("test_filters", 3).await.unwrap();
    
    // Insert documents
    let docs = vec![
        Document {
            id: None,
            content: "Document 1".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: json!({
                "category": "article",
                "views": 100,
                "tags": ["news", "technology"]
            }),
        },
        Document {
            id: None,
            content: "Document 2".to_string(),
            embedding: vec![0.2, 0.3, 0.4],
            metadata: json!({
                "category": "blog",
                "views": 200,
                "tags": ["technology", "programming"]
            }),
        },
        Document {
            id: None,
            content: "Document 3".to_string(),
            embedding: vec![0.3, 0.4, 0.5],
            metadata: json!({
                "category": "article",
                "views": 300,
                "tags": ["science", "research"]
            }),
        },
        Document {
            id: None,
            content: "Document 4".to_string(),
            embedding: vec![0.4, 0.5, 0.6],
            metadata: json!({
                "category": "blog",
                "views": 400,
                "tags": ["programming", "tutorial"]
            }),
        },
    ];
    
    store.batch_insert("test_filters", docs).await.unwrap();
    
    // Test 1: Equals filter
    let filter1 = Filter {
        conditions: vec![
            FilterCondition::Equals("category".to_string(), json!("article")),
        ],
    };
    
    let query = SearchQuery {
        embedding: vec![0.1, 0.2, 0.3],
        limit: 10,
        offset: 0,
    };
    
    let results = store.filtered_search("test_filters", query.clone(), filter1).await.unwrap();
    assert_eq!(results.len(), 2);
    for result in &results {
        assert_eq!(result.document.metadata["category"], "article");
    }
    
    // Test 2: Range filter
    let filter2 = Filter {
        conditions: vec![
            FilterCondition::Range(
                "views".to_string(),
                RangeValue {
                    min: Some(json!(200)),
                    max: Some(json!(300)),
                },
            ),
        ],
    };
    
    let results = store.filtered_search("test_filters", query.clone(), filter2).await.unwrap();
    assert_eq!(results.len(), 2);
    for result in &results {
        let views = result.document.metadata["views"].as_i64().unwrap();
        assert!(views >= 200 && views <= 300);
    }
    
    // Test 3: Contains filter
    let filter3 = Filter {
        conditions: vec![
            FilterCondition::Contains("tags".to_string(), vec![json!("programming")]),
        ],
    };
    
    let results = store.filtered_search("test_filters", query.clone(), filter3).await.unwrap();
    assert_eq!(results.len(), 2);
    for result in &results {
        let tags = result.document.metadata["tags"].as_array().unwrap();
        let has_programming = tags.iter().any(|tag| tag.as_str().unwrap() == "programming");
        assert!(has_programming);
    }
    
    // Test 4: Combined filters (AND logic)
    let filter4 = Filter {
        conditions: vec![
            FilterCondition::Equals("category".to_string(), json!("blog")),
            FilterCondition::Range(
                "views".to_string(),
                RangeValue {
                    min: Some(json!(300)),
                    max: None,
                },
            ),
        ],
    };
    
    let results = store.filtered_search("test_filters", query.clone(), filter4).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].document.metadata["category"], "blog");
    assert!(results[0].document.metadata["views"].as_i64().unwrap() >= 300);
    
    // Test 5: OR logic
    let filter5 = Filter {
        conditions: vec![
            FilterCondition::Or(vec![
                FilterCondition::Equals("category".to_string(), json!("article")),
                FilterCondition::Contains("tags".to_string(), vec![json!("tutorial")]),
            ]),
        ],
    };
    
    let results = store.filtered_search("test_filters", query.clone(), filter5).await.unwrap();
    assert_eq!(results.len(), 3);
    for result in &results {
        let is_article = result.document.metadata["category"] == "article";
        let tags = result.document.metadata["tags"].as_array().unwrap();
        let has_tutorial = tags.iter().any(|tag| tag.as_str().unwrap() == "tutorial");
        assert!(is_article || has_tutorial);
    }
}

#[tokio::test]
async fn test_embedded_qdrant_pagination() {
    let store = QdrantFactory::create(QdrantMode::Embedded).await.unwrap();
    
    // Create collection
    store.create_collection("test_pagination", 3).await.unwrap();
    
    // Insert documents
    let mut docs = Vec::with_capacity(10);
    for i in 0..10 {
        docs.push(Document {
            id: None,
            content: format!("Document {}", i),
            embedding: vec![0.1 * i as f32, 0.2 * i as f32, 0.3 * i as f32],
            metadata: json!({"index": i}),
        });
    }
    
    store.batch_insert("test_pagination", docs).await.unwrap();
    
    // Page 1
    let query1 = SearchQuery {
        embedding: vec![0.1, 0.2, 0.3],
        limit: 3,
        offset: 0,
    };
    
    let results1 = store.search("test_pagination", query1).await.unwrap();
    assert_eq!(results1.len(), 3);
    
    // Page 2
    let query2 = SearchQuery {
        embedding: vec![0.1, 0.2, 0.3],
        limit: 3,
        offset: 3,
    };
    
    let results2 = store.search("test_pagination", query2).await.unwrap();
    assert_eq!(results2.len(), 3);
    
    // Page 3
    let query3 = SearchQuery {
        embedding: vec![0.1, 0.2, 0.3],
        limit: 3,
        offset: 6,
    };
    
    let results3 = store.search("test_pagination", query3).await.unwrap();
    assert_eq!(results3.len(), 3);
    
    // Page 4 (partial)
    let query4 = SearchQuery {
        embedding: vec![0.1, 0.2, 0.3],
        limit: 3,
        offset: 9,
    };
    
    let results4 = store.search("test_pagination", query4).await.unwrap();
    assert_eq!(results4.len(), 1);
    
    // Verify no overlap between pages
    let ids1: Vec<String> = results1.iter().map(|r| r.document.id.clone().unwrap()).collect();
    let ids2: Vec<String> = results2.iter().map(|r| r.document.id.clone().unwrap()).collect();
    let ids3: Vec<String> = results3.iter().map(|r| r.document.id.clone().unwrap()).collect();
    let ids4: Vec<String> = results4.iter().map(|r| r.document.id.clone().unwrap()).collect();
    
    for id in &ids1 {
        assert!(!ids2.contains(id));
        assert!(!ids3.contains(id));
        assert!(!ids4.contains(id));
    }
    
    for id in &ids2 {
        assert!(!ids1.contains(id));
        assert!(!ids3.contains(id));
        assert!(!ids4.contains(id));
    }
    
    for id in &ids3 {
        assert!(!ids1.contains(id));
        assert!(!ids2.contains(id));
        assert!(!ids4.contains(id));
    }
    
    for id in &ids4 {
        assert!(!ids1.contains(id));
        assert!(!ids2.contains(id));
        assert!(!ids3.contains(id));
    }
}

#[tokio::test]
async fn test_embedded_qdrant_error_handling() {
    let store = QdrantFactory::create(QdrantMode::Embedded).await.unwrap();
    
    // Test 1: Collection not found
    let result = store.get_document("nonexistent_collection", "123").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VectorStoreError::CollectionNotFound(_)));
    
    // Test 2: Document not found
    store.create_collection("error_test", 3).await.unwrap();
    let result = store.get_document("error_test", "nonexistent_id").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VectorStoreError::DocumentNotFound(_)));
    
    // Test 3: Invalid vector size
    let doc = Document {
        id: None,
        content: "Invalid vector".to_string(),
        embedding: vec![0.1, 0.2], // Only 2 dimensions, but collection expects 3
        metadata: json!({}),
    };
    
    let result = store.insert_document("error_test", doc).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VectorStoreError::InvalidArgument(_)));
    
    // Test 4: Delete nonexistent document
    let result = store.delete_document("error_test", "nonexistent_id").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VectorStoreError::DocumentNotFound(_)));
}

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
            metadata: json!({"index": i}),
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
            metadata: json!({"index": i}),
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

// Helper function to generate simple embeddings for testing
async fn generate_embedding(text: &str) -> Vec<f32> {
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
    
    result
}

#[tokio::test]
async fn test_vector_store_with_generated_embeddings() {
    let store = QdrantFactory::create(QdrantMode::Embedded).await.unwrap();
    
    // Create collection
    store.create_collection("generated_embeddings", 384).await.unwrap();
    
    // Generate embeddings
    let texts = vec![
        "The quick brown fox jumps over the lazy dog",
        "The lazy dog sleeps all day",
        "The quick rabbit runs fast",
    ];
    
    let mut docs = Vec::with_capacity(texts.len());
    for (i, text) in texts.iter().enumerate() {
        let embedding = generate_embedding(text).await;
        
        docs.push(Document {
            id: None,
            content: text.to_string(),
            embedding,
            metadata: json!({"index": i}),
        });
    }
    
    store.batch_insert("generated_embeddings", docs).await.unwrap();
    
    // Search with a generated query embedding
    let query_embedding = generate_embedding("dog sleeping").await;
    
    let query = SearchQuery {
        embedding: query_embedding,
        limit: 3,
        offset: 0,
    };
    
    let results = store.search("generated_embeddings", query).await.unwrap();
    
    // Verify results
    assert_eq!(results.len(), 3);
    
    // The second document should be most relevant to "dog sleeping"
    assert_eq!(results[0].document.content, "The lazy dog sleeps all day");
}
