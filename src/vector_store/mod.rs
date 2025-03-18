mod pure;
pub use pure::*;

use std::time::Duration;
use thiserror::Error;
use async_trait::async_trait;
use deadpool::managed::{Manager, Pool, PoolError, RecycleError};
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use qdrant_client::qdrant::{VectorParams, Distance};
use qdrant_client::{Qdrant, QdrantError};
use qdrant_client::config::QdrantConfig as QdrantClientConfig;
use tracing::error;

#[derive(Debug, Error)]
pub enum VectorStoreError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Pool error: {0}")]
    PoolError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

impl From<PoolError<QdrantError>> for VectorStoreError {
    fn from(err: PoolError<QdrantError>) -> Self {
        VectorStoreError::PoolError(err.to_string())
    }
}

// We'll use QdrantError directly from the qdrant_client crate

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn test_connection(&self) -> Result<(), VectorStoreError>;
    async fn create_collection(&self, name: &str, vector_size: usize) -> Result<(), VectorStoreError>;
    async fn delete_collection(&self, name: &str) -> Result<(), VectorStoreError>;
    async fn insert_document(&self, collection: &str, document: Document) -> Result<(), VectorStoreError>;
    async fn search(&self, collection: &str, query: SearchQuery) -> Result<Vec<SearchResult>, VectorStoreError>;
}

#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub timeout: Duration,
    pub max_connections: usize,
    pub api_key: Option<String>,
    pub retry_max_elapsed_time: Duration,
    pub retry_initial_interval: Duration,
    pub retry_max_interval: Duration,
    pub retry_multiplier: f64,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6333".to_string(),
            timeout: Duration::from_secs(5),
            max_connections: 10,
            api_key: None,
            retry_max_elapsed_time: Duration::from_secs(60),
            retry_initial_interval: Duration::from_millis(100),
            retry_max_interval: Duration::from_secs(10),
            retry_multiplier: 2.0,
        }
    }
}

struct QdrantClientManager {
    config: QdrantConfig,
}

impl QdrantClientManager {
    fn new(config: QdrantConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Manager for QdrantClientManager {
    type Type = Qdrant;
    type Error = QdrantError;

    async fn create(&self) -> Result<Qdrant, QdrantError> {
        let mut config = QdrantClientConfig::from_url(&self.config.url);
        
        // Set timeout
        config.set_timeout(self.config.timeout);
        
        // Set API key if provided
        if let Some(api_key) = &self.config.api_key {
            config.set_api_key(api_key);
        }
        
        Qdrant::new(config)
    }

    async fn recycle(&self, client: &mut Qdrant) -> Result<(), RecycleError<QdrantError>> {
        // Check if the client is still usable
        match client.health_check().await {
            Ok(_) => Ok(()),
            Err(e) => Err(RecycleError::Message(format!("Failed to check health: {}", e))),
        }
    }
}

#[derive(Clone)]
pub struct QdrantConnector {
    client_pool: Pool<QdrantClientManager>,
    config: QdrantConfig,
}

impl QdrantConnector {
    pub async fn new(config: QdrantConfig) -> Result<Self, VectorStoreError> {
        let manager = QdrantClientManager::new(config.clone());
        let pool = Pool::builder(manager)
            .max_size(config.max_connections)
            .build()
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;
        
        Ok(Self {
            client_pool: pool,
            config,
        })
    }
    
    fn create_backoff(&self) -> ExponentialBackoff {
        ExponentialBackoffBuilder::new()
            .with_initial_interval(self.config.retry_initial_interval)
            .with_max_interval(self.config.retry_max_interval)
            .with_multiplier(self.config.retry_multiplier)
            .with_max_elapsed_time(Some(self.config.retry_max_elapsed_time))
            .build()
    }
    
    async fn with_retry<F, Fut, T>(&self, mut operation: F) -> Result<T, VectorStoreError>
    where
        F: FnMut() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, VectorStoreError>> + Send,
    {
        let backoff = self.create_backoff();
        
        let mut current_attempt = 0;
        let max_attempts = 3; // Limit the number of retries
        
        loop {
            match operation().await {
                Ok(value) => return Ok(value),
                Err(err) => {
                    current_attempt += 1;
                    if current_attempt >= max_attempts {
                        return Err(err);
                    }
                    
                    // Log the error
                    error!("Operation failed, will retry (attempt {}/{}): {}", 
                           current_attempt, max_attempts, err);
                    
                    // Wait before retrying
                    let wait_time = backoff.initial_interval * (backoff.multiplier.powf(current_attempt as f64 - 1.0) as u32);
                    tokio::time::sleep(wait_time).await;
                }
            }
        }
    }
}

#[async_trait]
impl VectorStore for QdrantConnector {
    async fn test_connection(&self) -> Result<(), VectorStoreError> {
        self.with_retry(|| async {
            let client = self.client_pool.get().await?;
            client.health_check().await
                .map(|_| ())
                .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))
        }).await
    }
    
    async fn create_collection(&self, name: &str, vector_size: usize) -> Result<(), VectorStoreError> {
        self.with_retry(|| async {
            let client = self.client_pool.get().await?;
            
            // Create a collection with the given name and vector size
            let vector_params = VectorParams {
                size: vector_size as u64,
                distance: Distance::Cosine as i32,
                ..Default::default()
            };
            
            // Create vectors config
            let vectors_config = qdrant_client::qdrant::VectorsConfig {
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(vector_params)),
            };
            
            // Create collection request
            let create_collection = qdrant_client::qdrant::CreateCollection {
                collection_name: name.to_string(),
                vectors_config: Some(vectors_config),
                ..Default::default()
            };
            
            client.create_collection(create_collection).await
                .map(|_| ())
                .map_err(|e| VectorStoreError::OperationFailed(format!("Failed to create collection: {}", e)))
        }).await
    }
    
    async fn delete_collection(&self, name: &str) -> Result<(), VectorStoreError> {
        self.with_retry(|| async {
            let client = self.client_pool.get().await?;
            
            client.delete_collection(name).await
                .map(|_| ())
                .map_err(|e| VectorStoreError::OperationFailed(format!("Failed to delete collection: {}", e)))
        }).await
    }
    
    async fn insert_document(&self, collection: &str, document: Document) -> Result<(), VectorStoreError> {
        self.with_retry(|| async {
            let client = self.client_pool.get().await?;
            
            use qdrant_client::qdrant::{PointId, PointStruct, Vectors, Vector};
            use std::collections::HashMap;
            
            // Create point ID
            let point_id = PointId {
                point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(
                    document.id.clone(),
                )),
            };
            
            // Create vector
            let vector = Vector {
                data: document.embedding.clone(),
                vector: None,
                indices: None,
                vectors_count: None,
            };
            
            // Create vectors
            let vectors = Vectors {
                vectors_options: Some(qdrant_client::qdrant::vectors::VectorsOptions::Vector(vector)),
            };
            
            // Create payload
            let mut payload = HashMap::new();
            payload.insert(
                "content".to_string(),
                qdrant_client::qdrant::Value {
                    kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                        document.content.clone(),
                    )),
                },
            );
            
            // Create point
            let point = PointStruct {
                id: Some(point_id),
                vectors: Some(vectors),
                payload,
            };
            
            // Create upsert points request
            let upsert_points = qdrant_client::qdrant::UpsertPoints {
                collection_name: collection.to_string(),
                wait: Some(true),
                points: vec![point],
                ..Default::default()
            };
            
            // Insert point into collection
            client.upsert_points(upsert_points).await
                .map(|_| ())
                .map_err(|e| VectorStoreError::OperationFailed(format!("Failed to insert document: {}", e)))
        }).await
    }
    
    async fn search(&self, collection: &str, query: SearchQuery) -> Result<Vec<SearchResult>, VectorStoreError> {
        self.with_retry(|| async {
            let client = self.client_pool.get().await?;
            
            use qdrant_client::qdrant::{SearchParams, WithPayloadSelector, WithVectorsSelector, SearchPoints};
            
            // Create search request
            let search_request = SearchPoints {
                collection_name: collection.to_string(),
                vector: query.embedding.clone(),
                limit: query.limit as u64,
                with_payload: Some(WithPayloadSelector::from(true)),
                with_vectors: Some(WithVectorsSelector::from(true)),
                params: Some(SearchParams {
                    hnsw_ef: Some(128),
                    exact: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            };
            
            // Execute search
            let search_result = client.search_points(search_request).await
                .map_err(|e| VectorStoreError::OperationFailed(format!("Failed to search: {}", e)))?;
            
            // Convert search results to our format
            let results = search_result.result
                .into_iter()
                .filter_map(|point| {
                    let id = match point.id.and_then(|id| id.point_id_options) {
                        Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) => uuid,
                        _ => return None,
                    };
                    
                    let content = point.payload.get("content").and_then(|value| {
                        if let Some(qdrant_client::qdrant::value::Kind::StringValue(content)) = &value.kind {
                            Some(content.clone())
                        } else {
                            None
                        }
                    }).unwrap_or_default();
                    
                    let embedding = point.vectors.and_then(|v| {
                        if let Some(qdrant_client::qdrant::vectors_output::VectorsOptions::Vector(vector)) = v.vectors_options {
                            Some(vector.data)
                        } else {
                            None
                        }
                    }).unwrap_or_default();
                    
                    Some(SearchResult {
                        document: Document {
                            id,
                            content,
                            embedding,
                        },
                        score: point.score,
                    })
                })
                .collect();
            
            Ok(results)
        }).await
    }
}

// Re-export the QdrantConnector for backward compatibility
pub use self::QdrantConnector as EmbeddedQdrantConnector;
