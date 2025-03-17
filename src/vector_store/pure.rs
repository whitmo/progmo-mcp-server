use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::text_processing::EmbeddingProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
}

impl Document {
    pub fn new(content: String, embedding_provider: &impl EmbeddingProvider) -> Result<Self, crate::text_processing::EmbeddingError> {
        let embedding = embedding_provider.generate_embedding(&content)?;
        
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            content,
            embedding,
        })
    }
    
    pub fn with_id(id: String, content: String, embedding_provider: &impl EmbeddingProvider) -> Result<Self, crate::text_processing::EmbeddingError> {
        let embedding = embedding_provider.generate_embedding(&content)?;
        
        Ok(Self {
            id,
            content,
            embedding,
        })
    }
    
    pub fn with_placeholder_embedding(content: String, embedding_dim: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            embedding: vec![0.0; embedding_dim],
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub embedding: Vec<f32>,
    pub limit: usize,
}

impl SearchQuery {
    pub fn from_text(text: &str, limit: usize, embedding_provider: &impl EmbeddingProvider) -> Result<Self, crate::text_processing::EmbeddingError> {
        let embedding = embedding_provider.generate_embedding(text)?;
        
        Ok(Self {
            embedding,
            limit,
        })
    }
    
    pub fn with_placeholder_embedding(embedding_dim: usize, limit: usize) -> Self {
        Self {
            embedding: vec![0.0; embedding_dim],
            limit,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
}

// Pure functions for vector operations
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    
    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    
    norm_a = norm_a.sqrt();
    norm_b = norm_b.sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
        
        let c = vec![1.0, 0.0, 0.0];
        let d = vec![1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&c, &d), 1.0);
        
        let e = vec![1.0, 1.0, 0.0];
        let f = vec![1.0, 0.0, 1.0];
        assert!((cosine_similarity(&e, &f) - 0.5).abs() < 0.0001);
    }
}
