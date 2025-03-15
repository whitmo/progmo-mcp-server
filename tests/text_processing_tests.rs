#[cfg(test)]
mod text_processing_tests {
    use p_mo::text_processing::{TextProcessor, ChunkingStrategy, TokenizerConfig};

    #[test]
    fn test_tokenization() {
        let config = TokenizerConfig::default();
        let processor = TextProcessor::new(config, ChunkingStrategy::FixedSize(100));
        
        let text = "This is a test sentence. This is another test sentence.";
        let tokens = processor.tokenize(text);
        
        assert!(tokens.len() > 0);
        assert!(tokens.contains(&"test".to_string()));
        assert!(tokens.contains(&"sentence".to_string()));
    }
    
    #[test]
    fn test_fixed_size_chunking() {
        let config = TokenizerConfig::default();
        let processor = TextProcessor::new(config, ChunkingStrategy::FixedSize(10));
        
        let text = "This is a test sentence. This is another test sentence.";
        let chunks = processor.chunk(text);
        
        // With a token limit of 10, we should have at least 2 chunks
        assert!(chunks.len() >= 2);
        
        // Each chunk should have no more than 10 tokens
        for chunk in &chunks {
            let tokens = processor.tokenize(&chunk.content);
            assert!(tokens.len() <= 10);
        }
        
        // The combined content of all chunks should equal the original text
        let combined = chunks.iter()
            .map(|c| c.content.clone())
            .collect::<Vec<String>>()
            .join("");
        assert_eq!(combined, text);
    }
    
    #[test]
    fn test_paragraph_chunking() {
        let config = TokenizerConfig::default();
        let processor = TextProcessor::new(config, ChunkingStrategy::Paragraph);
        
        let text = "This is paragraph one.\n\nThis is paragraph two.\n\nThis is paragraph three.";
        let chunks = processor.chunk(text);
        
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].content, "This is paragraph one.");
        assert_eq!(chunks[1].content, "This is paragraph two.");
        assert_eq!(chunks[2].content, "This is paragraph three.");
    }
    
    #[test]
    fn test_semantic_chunking() {
        let config = TokenizerConfig::default();
        let processor = TextProcessor::new(config, ChunkingStrategy::Semantic);
        
        let text = "# Introduction\nThis is an introduction.\n\n# Methods\nThese are the methods.\n\n# Results\nThese are the results.";
        let chunks = processor.chunk(text);
        
        assert_eq!(chunks.len(), 3);
        assert!(chunks[0].content.contains("Introduction"));
        assert!(chunks[1].content.contains("Methods"));
        assert!(chunks[2].content.contains("Results"));
    }
    
    #[test]
    fn test_metadata_extraction() {
        let config = TokenizerConfig::default();
        let processor = TextProcessor::new(config, ChunkingStrategy::FixedSize(100));
        
        let text = "Title: Test Document\nAuthor: Test Author\nDate: 2025-03-14\n\nThis is the content of the document.";
        let metadata = processor.extract_metadata(text);
        
        assert_eq!(metadata.get("title"), Some(&"Test Document".to_string()));
        assert_eq!(metadata.get("author"), Some(&"Test Author".to_string()));
        assert_eq!(metadata.get("date"), Some(&"2025-03-14".to_string()));
    }
    
    #[test]
    fn test_chunk_with_metadata() {
        let config = TokenizerConfig::default();
        let processor = TextProcessor::new(config, ChunkingStrategy::FixedSize(100));
        
        let text = "Title: Test Document\nAuthor: Test Author\nDate: 2025-03-14\n\nThis is the content of the document.";
        let chunks = processor.chunk_with_metadata(text);
        
        assert!(chunks.len() > 0);
        
        // Each chunk should have the same metadata
        for chunk in &chunks {
            assert_eq!(chunk.metadata.get("title"), Some(&"Test Document".to_string()));
            assert_eq!(chunk.metadata.get("author"), Some(&"Test Author".to_string()));
            assert_eq!(chunk.metadata.get("date"), Some(&"2025-03-14".to_string()));
        }
    }
    
    #[test]
    fn test_custom_tokenizer_config() {
        let config = TokenizerConfig {
            lowercase: true,
            remove_punctuation: true,
            remove_stopwords: true,
            ..Default::default()
        };
        let processor = TextProcessor::new(config, ChunkingStrategy::FixedSize(100));
        
        let text = "This is a test sentence with some punctuation!";
        let tokens = processor.tokenize(text);
        
        // Stopwords like "this", "is", "a", "with", "some" should be removed
        assert!(!tokens.contains(&"this".to_string()));
        assert!(!tokens.contains(&"is".to_string()));
        assert!(!tokens.contains(&"a".to_string()));
        
        // Punctuation should be removed
        assert!(!tokens.contains(&"punctuation!".to_string()));
        assert!(tokens.contains(&"punctuation".to_string()));
    }
}
