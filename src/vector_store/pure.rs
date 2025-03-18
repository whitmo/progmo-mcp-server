use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::text_processing::EmbeddingProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Optional document ID (will be generated if not provided)
    pub id: Option<String>,
    
    /// Document content
    pub content: String,
    
    /// Vector embedding
    pub embedding: Vec<f32>,
    
    /// Metadata as JSON
    pub metadata: Value,
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
    /// Vector embedding to search for
    pub embedding: Vec<f32>,
    
    /// Maximum number of results to return
    pub limit: usize,
    
    /// Offset for pagination
    pub offset: usize,
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
    /// The matching document
    pub document: Document,
    
    /// Similarity score (higher is more similar)
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct Filter {
    /// Filter conditions (combined with AND logic)
    pub conditions: Vec<FilterCondition>,
}

#[derive(Debug, Clone)]
pub enum FilterCondition {
    /// Field equals value
    Equals(String, Value),
    
    /// Field is in range
    Range(String, RangeValue),
    
    /// Field contains any of the values
    Contains(String, Vec<Value>),
    
    /// Nested conditions with OR logic
    Or(Vec<FilterCondition>),
}

#[derive(Debug, Clone)]
pub struct RangeValue {
    /// Minimum value (inclusive)
    pub min: Option<Value>,
    
    /// Maximum value (inclusive)
    pub max: Option<Value>,
}

/// Calculate cosine similarity between two vectors
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

/// Check if a document matches a filter
pub fn matches_filter(document: &Document, filter: &Filter) -> bool {
    // If there are no conditions, the document matches
    if filter.conditions.is_empty() {
        return true;
    }
    
    // All conditions must match (AND logic)
    filter.conditions.iter().all(|condition| matches_condition(document, condition))
}

/// Check if a document matches a filter condition
fn matches_condition(document: &Document, condition: &FilterCondition) -> bool {
    match condition {
        FilterCondition::Equals(field, value) => {
            // Check if the field exists in metadata and equals the value
            document.metadata.get(field)
                .map(|field_value| field_value == value)
                .unwrap_or(false)
        },
        FilterCondition::Range(field, range_value) => {
            // Check if the field exists in metadata and is in the range
            document.metadata.get(field).map(|field_value| {
                let in_min_range = match &range_value.min {
                    Some(min) => compare_json_values(field_value, min) >= 0,
                    None => true,
                };
                
                let in_max_range = match &range_value.max {
                    Some(max) => compare_json_values(field_value, max) <= 0,
                    None => true,
                };
                
                in_min_range && in_max_range
            }).unwrap_or(false)
        },
        FilterCondition::Contains(field, values) => {
            // Check if the field exists in metadata and contains any of the values
            document.metadata.get(field).map(|field_value| {
                if let Some(array) = field_value.as_array() {
                    // Field is an array, check if it contains any of the values
                    values.iter().any(|value| array.contains(value))
                } else {
                    // Field is not an array, check if it equals any of the values
                    values.contains(field_value)
                }
            }).unwrap_or(false)
        },
        FilterCondition::Or(conditions) => {
            // At least one condition must match (OR logic)
            conditions.iter().any(|condition| matches_condition(document, condition))
        },
    }
}

/// Compare two JSON values
/// Returns -1 if a < b, 0 if a == b, 1 if a > b
fn compare_json_values(a: &Value, b: &Value) -> i8 {
    match (a, b) {
        (Value::Number(a_num), Value::Number(b_num)) => {
            if let (Some(a_f64), Some(b_f64)) = (a_num.as_f64(), b_num.as_f64()) {
                if a_f64 < b_f64 {
                    -1
                } else if a_f64 > b_f64 {
                    1
                } else {
                    0
                }
            } else {
                0
            }
        },
        (Value::String(a_str), Value::String(b_str)) => {
            if a_str < b_str {
                -1
            } else if a_str > b_str {
                1
            } else {
                0
            }
        },
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
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
