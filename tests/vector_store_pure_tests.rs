use p_mo::vector_store::{cosine_similarity, Document, SearchQuery};
use p_mo::text_processing::{EmbeddingProvider, EmbeddingError};

// Mock embedding provider for testing
#[derive(Debug)]
struct MockEmbeddingProvider {
    embedding_dim: usize,
}

impl MockEmbeddingProvider {
    fn new(embedding_dim: usize) -> Self {
        Self { embedding_dim }
    }
}

impl EmbeddingProvider for MockEmbeddingProvider {
    fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        // Generate a deterministic embedding based on text length
        let mut embedding = vec![0.0; self.embedding_dim];
        let text_len = text.len() as f32;
        
        for i in 0..self.embedding_dim {
            embedding[i] = (i as f32) / text_len;
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

#[test]
fn test_cosine_similarity_identical_vectors() {
    let vec1 = vec![1.0, 2.0, 3.0];
    let vec2 = vec![1.0, 2.0, 3.0];
    
    let similarity = cosine_similarity(&vec1, &vec2);
    
    // Identical vectors should have similarity of 1.0
    assert!((similarity - 1.0).abs() < 1e-6);
}

#[test]
fn test_cosine_similarity_orthogonal_vectors() {
    let vec1 = vec![1.0, 0.0, 0.0];
    let vec2 = vec![0.0, 1.0, 0.0];
    
    let similarity = cosine_similarity(&vec1, &vec2);
    
    // Orthogonal vectors should have similarity of 0.0
    assert!(similarity.abs() < 1e-6);
}

#[test]
fn test_cosine_similarity_opposite_vectors() {
    let vec1 = vec![1.0, 2.0, 3.0];
    let vec2 = vec![-1.0, -2.0, -3.0];
    
    let similarity = cosine_similarity(&vec1, &vec2);
    
    // Opposite vectors should have similarity of -1.0
    assert!((similarity + 1.0).abs() < 1e-6);
}

#[test]
fn test_cosine_similarity_different_lengths() {
    let vec1 = vec![1.0, 2.0, 3.0];
    let vec2 = vec![1.0, 2.0];
    
    let similarity = cosine_similarity(&vec1, &vec2);
    
    // Different length vectors should return 0.0
    assert_eq!(similarity, 0.0);
}

#[test]
fn test_cosine_similarity_empty_vectors() {
    let vec1: Vec<f32> = vec![];
    let vec2: Vec<f32> = vec![];
    
    let similarity = cosine_similarity(&vec1, &vec2);
    
    // Empty vectors should return 0.0
    assert_eq!(similarity, 0.0);
}

#[test]
fn test_document_new_with_embedding_provider() {
    let embedding_provider = MockEmbeddingProvider::new(384);
    let content = "This is a test document.";
    
    let document = Document::new(content.to_string(), &embedding_provider).unwrap();
    
    // Check that the document has the expected properties
    assert!(!document.id.is_empty());
    assert_eq!(document.content, content);
    assert_eq!(document.embedding.len(), 384);
    
    // Check that the embedding is not all zeros
    assert!(document.embedding.iter().any(|&x| x != 0.0));
}

#[test]
fn test_document_with_id_and_embedding_provider() {
    let embedding_provider = MockEmbeddingProvider::new(384);
    let id = "test-id-123";
    let content = "This is a test document with a specific ID.";
    
    let document = Document::with_id(id.to_string(), content.to_string(), &embedding_provider).unwrap();
    
    // Check that the document has the expected properties
    assert_eq!(document.id, id);
    assert_eq!(document.content, content);
    assert_eq!(document.embedding.len(), 384);
    
    // Check that the embedding is not all zeros
    assert!(document.embedding.iter().any(|&x| x != 0.0));
}

#[test]
fn test_document_with_placeholder_embedding() {
    let content = "This is a test document with a placeholder embedding.";
    let embedding_dim = 384;
    
    let document = Document::with_placeholder_embedding(content.to_string(), embedding_dim);
    
    // Check that the document has the expected properties
    assert!(!document.id.is_empty());
    assert_eq!(document.content, content);
    assert_eq!(document.embedding.len(), embedding_dim);
    
    // Check that the embedding is all zeros
    assert!(document.embedding.iter().all(|&x| x == 0.0));
}

#[test]
fn test_search_query_from_text() {
    let embedding_provider = MockEmbeddingProvider::new(384);
    let text = "This is a test search query.";
    let limit = 10;
    
    let query = SearchQuery::from_text(text, limit, &embedding_provider).unwrap();
    
    // Check that the query has the expected properties
    assert_eq!(query.embedding.len(), 384);
    assert_eq!(query.limit, limit);
    
    // Check that the embedding is not all zeros
    assert!(query.embedding.iter().any(|&x| x != 0.0));
}

#[test]
fn test_search_query_with_placeholder_embedding() {
    let embedding_dim = 384;
    let limit = 10;
    
    let query = SearchQuery::with_placeholder_embedding(embedding_dim, limit);
    
    // Check that the query has the expected properties
    assert_eq!(query.embedding.len(), embedding_dim);
    assert_eq!(query.limit, limit);
    
    // Check that the embedding is all zeros
    assert!(query.embedding.iter().all(|&x| x == 0.0));
}
