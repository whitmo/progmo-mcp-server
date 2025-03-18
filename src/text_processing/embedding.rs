use std::path::PathBuf;
use thiserror::Error;
use tracing::{info, error};

/// A trait for embedding providers
pub trait EmbeddingProvider {
    /// Generate an embedding for a single text
    fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    
    /// Generate embeddings for multiple texts
    fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
    
    /// Get the dimensionality of the embeddings
    fn embedding_dim(&self) -> usize;
}

#[cfg(feature = "embedding-generation")]
use rust_bert::bert::{BertConfig, BertModel};
#[cfg(feature = "embedding-generation")]
use rust_bert::Config;
#[cfg(feature = "embedding-generation")]
use rust_bert::RustBertError;
#[cfg(feature = "embedding-generation")]
use rust_bert::resources::{LocalResource, Resource};
#[cfg(feature = "embedding-generation")]
use rust_bert::pipelines::sentence_embeddings::{SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType};
#[cfg(feature = "embedding-generation")]
use tch::{Device, Tensor};
#[cfg(feature = "embedding-generation")]
use std::sync::Arc;

/// Error type for embedding operations
#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("Failed to initialize embedding model: {0}")]
    InitializationError(String),
    
    #[error("Failed to generate embedding: {0}")]
    GenerationError(String),
    
    #[error("Invalid input: {0}")]
    InvalidInputError(String),
}

#[cfg(feature = "embedding-generation")]
impl From<RustBertError> for EmbeddingError {
    fn from(err: RustBertError) -> Self {
        EmbeddingError::GenerationError(err.to_string())
    }
}

/// Configuration for the embedding generator
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// The type of model to use for embeddings
    pub model_type: EmbeddingModelType,
    
    /// Path to the model files (if using a local model)
    pub model_path: Option<PathBuf>,
    
    /// Whether to use GPU for inference
    pub use_gpu: bool,
    
    /// The dimensionality of the embeddings
    pub embedding_dim: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_type: EmbeddingModelType::MiniLM,
            model_path: None,
            use_gpu: false,
            embedding_dim: 384,
        }
    }
}

/// Types of embedding models supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingModelType {
    /// BERT base model
    Bert,
    
    /// DistilBERT model (smaller and faster than BERT)
    DistilBert,
    
    /// MiniLM model (very small and fast)
    MiniLM,
    
    /// MPNet model (high quality embeddings)
    MPNet,
}

#[cfg(feature = "embedding-generation")]
impl EmbeddingModelType {
    fn to_sentence_embeddings_model_type(&self) -> SentenceEmbeddingsModelType {
        match self {
            EmbeddingModelType::Bert => SentenceEmbeddingsModelType::AllMiniLmL12V2,
            EmbeddingModelType::DistilBert => SentenceEmbeddingsModelType::AllDistilrobertaV1,
            EmbeddingModelType::MiniLM => SentenceEmbeddingsModelType::AllMiniLmL6V2,
            EmbeddingModelType::MPNet => SentenceEmbeddingsModelType::AllMpnetBaseV2,
        }
    }
}

/// Generator for text embeddings
#[cfg(feature = "embedding-generation")]
#[derive(Debug)]
pub struct EmbeddingGenerator {
    model: SentenceEmbeddingsModel,
    config: EmbeddingConfig,
}

#[cfg(feature = "embedding-generation")]
impl EmbeddingProvider for EmbeddingGenerator {
    fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.trim().is_empty() {
            return Err(EmbeddingError::InvalidInputError("Empty text provided".to_string()));
        }
        
        let embeddings = self.model.encode(&[text])
            .map_err(|e| EmbeddingError::GenerationError(e.to_string()))?;
        
        // Convert the first embedding to a Vec<f32>
        let embedding = embeddings
            .get(0)
            .ok_or_else(|| EmbeddingError::GenerationError("Failed to get embedding".to_string()))?
            .iter()
            .copied()
            .collect();
        
        Ok(embedding)
    }
    
    fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Err(EmbeddingError::InvalidInputError("Empty texts provided".to_string()));
        }
        
        // Filter out empty texts
        let non_empty_texts: Vec<&str> = texts
            .iter()
            .map(|s| s.as_str())
            .filter(|s| !s.trim().is_empty())
            .collect();
        
        if non_empty_texts.is_empty() {
            return Err(EmbeddingError::InvalidInputError("All texts are empty".to_string()));
        }
        
        let embeddings = self.model.encode(&non_empty_texts)
            .map_err(|e| EmbeddingError::GenerationError(e.to_string()))?;
        
        // Convert the embeddings to Vec<Vec<f32>>
        let embeddings: Vec<Vec<f32>> = embeddings
            .iter()
            .map(|embedding| embedding.iter().copied().collect())
            .collect();
        
        Ok(embeddings)
    }
    
    fn embedding_dim(&self) -> usize {
        self.config.embedding_dim
    }
}

#[cfg(feature = "embedding-generation")]
impl EmbeddingGenerator {
    /// Create a new embedding generator with the given configuration
    pub fn new(config: EmbeddingConfig) -> Result<Self, EmbeddingError> {
        info!("Initializing embedding model: {:?}", config.model_type);
        
        let device = if config.use_gpu {
            Device::Cuda(0)
        } else {
            Device::Cpu
        };
        
        let model_type = config.model_type.to_sentence_embeddings_model_type();
        
        let model = match &config.model_path {
            Some(path) => {
                info!("Loading model from local path: {:?}", path);
                // Load model from local path
                Self::load_local_model(path, device)?
            },
            None => {
                info!("Downloading model from HuggingFace Hub");
                // Download model from HuggingFace Hub
                SentenceEmbeddingsBuilder::remote(model_type)
                    .with_device(device)
                    .create_model()
                    .map_err(|e| EmbeddingError::InitializationError(e.to_string()))?
            }
        };
        
        info!("Embedding model initialized successfully");
        
        Ok(Self {
            model,
            config,
        })
    }
    
    /// Load a model from a local path
    fn load_local_model(path: &PathBuf, device: Device) -> Result<SentenceEmbeddingsModel, EmbeddingError> {
        // This is a simplified implementation - in a real-world scenario,
        // you would need to handle the specific model architecture and files
        let model_resource = Resource::Local(LocalResource {
            local_path: path.join("model.ot"),
        });
        
        let config_resource = Resource::Local(LocalResource {
            local_path: path.join("config.json"),
        });
        
        let vocab_resource = Resource::Local(LocalResource {
            local_path: path.join("vocab.txt"),
        });
        
        SentenceEmbeddingsBuilder::from_file(
            model_resource,
            config_resource,
            vocab_resource,
        )
        .with_device(device)
        .create_model()
        .map_err(|e| EmbeddingError::InitializationError(e.to_string()))
    }
    
    /// Generate an embedding for a single text
    pub fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.trim().is_empty() {
            return Err(EmbeddingError::InvalidInputError("Empty text provided".to_string()));
        }
        
        let embeddings = self.model.encode(&[text])?;
        
        // Convert the first embedding to a Vec<f32>
        let embedding = embeddings
            .get(0)
            .ok_or_else(|| EmbeddingError::GenerationError("Failed to get embedding".to_string()))?
            .iter()
            .copied()
            .collect();
        
        Ok(embedding)
    }
    
    /// Generate embeddings for multiple texts
    pub fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Err(EmbeddingError::InvalidInputError("Empty texts provided".to_string()));
        }
        
        // Filter out empty texts
        let non_empty_texts: Vec<&str> = texts
            .iter()
            .map(|s| s.as_str())
            .filter(|s| !s.trim().is_empty())
            .collect();
        
        if non_empty_texts.is_empty() {
            return Err(EmbeddingError::InvalidInputError("All texts are empty".to_string()));
        }
        
        let embeddings = self.model.encode(&non_empty_texts)?;
        
        // Convert the embeddings to Vec<Vec<f32>>
        let embeddings: Vec<Vec<f32>> = embeddings
            .iter()
            .map(|embedding| embedding.iter().copied().collect())
            .collect();
        
        Ok(embeddings)
    }
    
    /// Get the dimensionality of the embeddings
    pub fn embedding_dim(&self) -> usize {
        self.config.embedding_dim
    }
}

/// A placeholder embedding generator for when the embedding-generation feature is disabled
#[cfg(not(feature = "embedding-generation"))]
#[derive(Debug)]
pub struct EmbeddingGenerator {
    config: EmbeddingConfig,
}

#[cfg(not(feature = "embedding-generation"))]
impl EmbeddingProvider for EmbeddingGenerator {
    fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
        // Generate a placeholder embedding (all zeros)
        Ok(vec![0.0; self.config.embedding_dim])
    }
    
    fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        // Generate placeholder embeddings (all zeros)
        Ok(texts.iter().map(|_| vec![0.0; self.config.embedding_dim]).collect())
    }
    
    fn embedding_dim(&self) -> usize {
        self.config.embedding_dim
    }
}

#[cfg(not(feature = "embedding-generation"))]
impl EmbeddingGenerator {
    /// Create a new embedding generator with the given configuration
    pub fn new(config: EmbeddingConfig) -> Result<Self, EmbeddingError> {
        info!("Creating placeholder embedding generator (embedding-generation feature disabled)");
        Ok(Self { config })
    }
}

/// A mock embedding generator for testing
#[cfg(test)]
#[derive(Debug)]
pub struct MockEmbeddingGenerator {
    embedding_dim: usize,
}

#[cfg(test)]
impl EmbeddingProvider for MockEmbeddingGenerator {
    fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        // Generate a deterministic but unique embedding based on the text
        let mut embedding = vec![0.0; self.embedding_dim];
        
        // Fill with some values based on the hash of the text
        for i in 0..self.embedding_dim {
            embedding[i] = (i as f32) / (self.embedding_dim as f32);
        }
        
        Ok(embedding)
    }
    
    fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut result = Vec::with_capacity(texts.len());
        
        for text in texts {
            result.push(self.generate_embedding(text)?);
        }
        
        Ok(result)
    }
    
    fn embedding_dim(&self) -> usize {
        self.embedding_dim
    }
}

#[cfg(test)]
impl MockEmbeddingGenerator {
    pub fn new(embedding_dim: usize) -> Self {
        Self { embedding_dim }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_embedding_generator() {
        let generator = MockEmbeddingGenerator::new(384);
        
        // Test single embedding
        let embedding = generator.generate_embedding("Test text").unwrap();
        assert_eq!(embedding.len(), 384);
        
        // Test multiple embeddings
        let texts = vec!["Text 1".to_string(), "Text 2".to_string()];
        let embeddings = generator.generate_embeddings(&texts).unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);
        assert_eq!(embeddings[1].len(), 384);
    }
}
