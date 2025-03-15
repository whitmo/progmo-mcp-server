use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Vector embedding to search for
    pub embedding: Vec<f32>,
    
    /// Maximum number of results to return
    pub limit: usize,
    
    /// Offset for pagination
    pub offset: usize,
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
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
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
        assert!((cosine_similarity(&e, &f) - 0.5).abs() < 1e-6);
    }
    
    #[test]
    fn test_matches_filter_equals() {
        let document = Document {
            id: Some("test".to_string()),
            content: "Test document".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: json!({
                "category": "article",
                "views": 100
            }),
        };
        
        let filter = Filter {
            conditions: vec![
                FilterCondition::Equals("category".to_string(), json!("article")),
            ],
        };
        
        assert!(matches_filter(&document, &filter));
        
        let filter2 = Filter {
            conditions: vec![
                FilterCondition::Equals("category".to_string(), json!("blog")),
            ],
        };
        
        assert!(!matches_filter(&document, &filter2));
    }
    
    #[test]
    fn test_matches_filter_range() {
        let document = Document {
            id: Some("test".to_string()),
            content: "Test document".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: json!({
                "views": 100
            }),
        };
        
        let filter = Filter {
            conditions: vec![
                FilterCondition::Range(
                    "views".to_string(),
                    RangeValue {
                        min: Some(json!(50)),
                        max: Some(json!(150)),
                    },
                ),
            ],
        };
        
        assert!(matches_filter(&document, &filter));
        
        let filter2 = Filter {
            conditions: vec![
                FilterCondition::Range(
                    "views".to_string(),
                    RangeValue {
                        min: Some(json!(150)),
                        max: None,
                    },
                ),
            ],
        };
        
        assert!(!matches_filter(&document, &filter2));
    }
    
    #[test]
    fn test_matches_filter_contains() {
        let document = Document {
            id: Some("test".to_string()),
            content: "Test document".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: json!({
                "tags": ["news", "technology"]
            }),
        };
        
        let filter = Filter {
            conditions: vec![
                FilterCondition::Contains("tags".to_string(), vec![json!("technology")]),
            ],
        };
        
        assert!(matches_filter(&document, &filter));
        
        let filter2 = Filter {
            conditions: vec![
                FilterCondition::Contains("tags".to_string(), vec![json!("programming")]),
            ],
        };
        
        assert!(!matches_filter(&document, &filter2));
    }
    
    #[test]
    fn test_matches_filter_or() {
        let document = Document {
            id: Some("test".to_string()),
            content: "Test document".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: json!({
                "category": "article",
                "views": 100,
                "tags": ["news", "technology"]
            }),
        };
        
        let filter = Filter {
            conditions: vec![
                FilterCondition::Or(vec![
                    FilterCondition::Equals("category".to_string(), json!("blog")),
                    FilterCondition::Contains("tags".to_string(), vec![json!("news")]),
                ]),
            ],
        };
        
        assert!(matches_filter(&document, &filter));
        
        let filter2 = Filter {
            conditions: vec![
                FilterCondition::Or(vec![
                    FilterCondition::Equals("category".to_string(), json!("blog")),
                    FilterCondition::Contains("tags".to_string(), vec![json!("programming")]),
                ]),
            ],
        };
        
        assert!(!matches_filter(&document, &filter2));
    }
    
    #[test]
    fn test_matches_filter_and() {
        let document = Document {
            id: Some("test".to_string()),
            content: "Test document".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: json!({
                "category": "article",
                "views": 100,
                "tags": ["news", "technology"]
            }),
        };
        
        let filter = Filter {
            conditions: vec![
                FilterCondition::Equals("category".to_string(), json!("article")),
                FilterCondition::Range(
                    "views".to_string(),
                    RangeValue {
                        min: Some(json!(50)),
                        max: Some(json!(150)),
                    },
                ),
            ],
        };
        
        assert!(matches_filter(&document, &filter));
        
        let filter2 = Filter {
            conditions: vec![
                FilterCondition::Equals("category".to_string(), json!("article")),
                FilterCondition::Range(
                    "views".to_string(),
                    RangeValue {
                        min: Some(json!(150)),
                        max: None,
                    },
                ),
            ],
        };
        
        assert!(!matches_filter(&document, &filter2));
    }
}
