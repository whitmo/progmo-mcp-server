use p_mo::text_processing::{EmbeddingProvider, EmbeddingError};

struct MockEmbeddingGenerator {
    embedding_dim: usize,
}

impl MockEmbeddingGenerator {
    fn new(embedding_dim: usize) -> Self {
        Self { embedding_dim }
    }
}

impl EmbeddingProvider for MockEmbeddingGenerator {
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
