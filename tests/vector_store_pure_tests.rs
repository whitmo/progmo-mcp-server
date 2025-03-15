use p_mo::vector_store::{
    Filter, FilterCondition, RangeValue
};
use serde_json::Value;

// Helper functions for testing
mod test_helpers {
    use super::*;
    
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        // If vectors have different lengths, return 0
        if a.len() != b.len() {
            return 0.0;
        }
        
        // If vectors are empty, return 0
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
    
    pub fn matches_filter(metadata: &Value, filter: &Filter) -> bool {
        // If there are no conditions, the document matches
        if filter.conditions.is_empty() {
            return true;
        }
        
        // All conditions must match (AND logic)
        filter.conditions.iter().all(|condition| matches_condition(metadata, condition))
    }
    
    fn matches_condition(metadata: &Value, condition: &FilterCondition) -> bool {
        match condition {
            FilterCondition::Equals(field, value) => {
                // Check if the field exists in metadata and equals the value
                metadata.get(field)
                    .map(|field_value| field_value == value)
                    .unwrap_or(false)
            }
            FilterCondition::Range(field, range_value) => {
                // Check if the field exists in metadata and is in the range
                metadata.get(field).map(|field_value| {
                    let in_min_range = match &range_value.min {
                        Some(min) => compare_json_values(field_value, min) >= 0,
                        None => true
                    };
                    
                    let in_max_range = match &range_value.max {
                        Some(max) => compare_json_values(field_value, max) <= 0,
                        None => true
                    };
                    
                    in_min_range && in_max_range
                }).unwrap_or(false)
            }
            FilterCondition::Contains(field, values) => {
                // Check if the field exists in metadata and contains any of the values
                metadata.get(field).map(|field_value| {
                    if let Some(array) = field_value.as_array() {
                        // Field is an array, check if it contains any of the values
                        values.iter().all(|value| array.contains(value))
                    } else {
                        // Field is not an array, check if it equals any of the values
                        values.contains(field_value)
                    }
                }).unwrap_or(false)
            }
            FilterCondition::Or(conditions) => {
                // At least one condition must match (OR logic)
                conditions.iter().any(|condition| matches_condition(metadata, condition))
            }
        }
    }
    
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
            }
            (Value::String(a_str), Value::String(b_str)) => {
                if a_str < b_str {
                    -1
                } else if a_str > b_str {
                    1
                } else {
                    0
                }
            }
            (Value::Bool(a_bool), Value::Bool(b_bool)) => {
                match (a_bool, b_bool) {
                    (false, true) => -1,
                    (true, false) => 1,
                    _ => 0
                }
            }
            _ => 0
        }
    }
}

use test_helpers::{cosine_similarity, matches_filter};
use serde_json::json;

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
fn test_matches_filter_equals_string() {
    let metadata = json!({
        "category": "books"
    });
    
    let filter = Filter {
        conditions: vec![
            FilterCondition::Equals("category".to_string(), json!("books"))
        ]
    };
    
    assert!(matches_filter(&metadata, &filter));
    
    let filter_no_match = Filter {
        conditions: vec![
            FilterCondition::Equals("category".to_string(), json!("movies"))
        ]
    };
    
    assert!(!matches_filter(&metadata, &filter_no_match));
}

#[test]
fn test_matches_filter_equals_number() {
    let metadata = json!({
        "rating": 5
    });
    
    let filter = Filter {
        conditions: vec![
            FilterCondition::Equals("rating".to_string(), json!(5))
        ]
    };
    
    assert!(matches_filter(&metadata, &filter));
    
    let filter_no_match = Filter {
        conditions: vec![
            FilterCondition::Equals("rating".to_string(), json!(4))
        ]
    };
    
    assert!(!matches_filter(&metadata, &filter_no_match));
}

#[test]
fn test_matches_filter_equals_boolean() {
    let metadata = json!({
        "published": true
    });
    
    let filter = Filter {
        conditions: vec![
            FilterCondition::Equals("published".to_string(), json!(true))
        ]
    };
    
    assert!(matches_filter(&metadata, &filter));
    
    let filter_no_match = Filter {
        conditions: vec![
            FilterCondition::Equals("published".to_string(), json!(false))
        ]
    };
    
    assert!(!matches_filter(&metadata, &filter_no_match));
}

#[test]
fn test_matches_filter_range_min_only() {
    let metadata = json!({
        "price": 50
    });
    
    let filter = Filter {
        conditions: vec![
            FilterCondition::Range(
                "price".to_string(),
                RangeValue {
                    min: Some(json!(30)),
                    max: None
                }
            )
        ]
    };
    
    assert!(matches_filter(&metadata, &filter));
    
    let filter_no_match = Filter {
        conditions: vec![
            FilterCondition::Range(
                "price".to_string(),
                RangeValue {
                    min: Some(json!(60)),
                    max: None
                }
            )
        ]
    };
    
    assert!(!matches_filter(&metadata, &filter_no_match));
}

#[test]
fn test_matches_filter_range_max_only() {
    let metadata = json!({
        "price": 50
    });
    
    let filter = Filter {
        conditions: vec![
            FilterCondition::Range(
                "price".to_string(),
                RangeValue {
                    min: None,
                    max: Some(json!(60))
                }
            )
        ]
    };
    
    assert!(matches_filter(&metadata, &filter));
    
    let filter_no_match = Filter {
        conditions: vec![
            FilterCondition::Range(
                "price".to_string(),
                RangeValue {
                    min: None,
                    max: Some(json!(40))
                }
            )
        ]
    };
    
    assert!(!matches_filter(&metadata, &filter_no_match));
}

#[test]
fn test_matches_filter_range_min_and_max() {
    let metadata = json!({
        "price": 50
    });
    
    let filter = Filter {
        conditions: vec![
            FilterCondition::Range(
                "price".to_string(),
                RangeValue {
                    min: Some(json!(30)),
                    max: Some(json!(60))
                }
            )
        ]
    };
    
    assert!(matches_filter(&metadata, &filter));
    
    let filter_no_match = Filter {
        conditions: vec![
            FilterCondition::Range(
                "price".to_string(),
                RangeValue {
                    min: Some(json!(60)),
                    max: Some(json!(70))
                }
            )
        ]
    };
    
    assert!(!matches_filter(&metadata, &filter_no_match));
}

#[test]
fn test_matches_filter_contains_single_value() {
    let metadata = json!({
        "tags": ["fiction", "fantasy", "adventure"]
    });
    
    let filter = Filter {
        conditions: vec![
            FilterCondition::Contains(
                "tags".to_string(),
                vec![json!("fantasy")]
            )
        ]
    };
    
    assert!(matches_filter(&metadata, &filter));
    
    let filter_no_match = Filter {
        conditions: vec![
            FilterCondition::Contains(
                "tags".to_string(),
                vec![json!("horror")]
            )
        ]
    };
    
    assert!(!matches_filter(&metadata, &filter_no_match));
}

#[test]
fn test_matches_filter_contains_multiple_values() {
    let metadata = json!({
        "tags": ["fiction", "fantasy", "adventure"]
    });
    
    let filter = Filter {
        conditions: vec![
            FilterCondition::Contains(
                "tags".to_string(),
                vec![json!("fiction"), json!("fantasy")]
            )
        ]
    };
    
    assert!(matches_filter(&metadata, &filter));
    
    let filter_partial_match = Filter {
        conditions: vec![
            FilterCondition::Contains(
                "tags".to_string(),
                vec![json!("fiction"), json!("horror")]
            )
        ]
    };
    
    assert!(!matches_filter(&metadata, &filter_partial_match));
}

#[test]
fn test_matches_filter_multiple_conditions() {
    let metadata = json!({
        "category": "books",
        "price": 50,
        "published": true
    });
    
    // Multiple conditions in a filter are combined with AND logic
    let filter = Filter {
        conditions: vec![
            FilterCondition::Equals("category".to_string(), json!("books")),
            FilterCondition::Range(
                "price".to_string(),
                RangeValue {
                    min: Some(json!(30)),
                    max: Some(json!(60))
                }
            )
        ]
    };
    
    assert!(matches_filter(&metadata, &filter));
    
    let filter_no_match = Filter {
        conditions: vec![
            FilterCondition::Equals("category".to_string(), json!("books")),
            FilterCondition::Equals("published".to_string(), json!(false))
        ]
    };
    
    assert!(!matches_filter(&metadata, &filter_no_match));
}

#[test]
fn test_matches_filter_or_condition() {
    let metadata = json!({
        "category": "books",
        "price": 50,
        "published": true
    });
    
    let filter = Filter {
        conditions: vec![
            FilterCondition::Or(vec![
                FilterCondition::Equals("category".to_string(), json!("movies")),
                FilterCondition::Range(
                    "price".to_string(),
                    RangeValue {
                        min: Some(json!(30)),
                        max: Some(json!(60))
                    }
                )
            ])
        ]
    };
    
    assert!(matches_filter(&metadata, &filter));
    
    let filter_no_match = Filter {
        conditions: vec![
            FilterCondition::Or(vec![
                FilterCondition::Equals("category".to_string(), json!("movies")),
                FilterCondition::Equals("published".to_string(), json!(false))
            ])
        ]
    };
    
    assert!(!matches_filter(&metadata, &filter_no_match));
}

#[test]
fn test_matches_filter_nested_conditions() {
    let metadata = json!({
        "category": "books",
        "price": 50,
        "published": true,
        "tags": ["fiction", "fantasy"]
    });
    
    // Create a filter with nested conditions
    // First condition: category must be "books"
    // Second condition: either price >= 60 OR tags contains "fantasy"
    let filter = Filter {
        conditions: vec![
            FilterCondition::Equals("category".to_string(), json!("books")),
            FilterCondition::Or(vec![
                FilterCondition::Range(
                    "price".to_string(),
                    RangeValue {
                        min: Some(json!(60)),
                        max: None
                    }
                ),
                FilterCondition::Contains(
                    "tags".to_string(),
                    vec![json!("fantasy")]
                )
            ])
        ]
    };
    
    assert!(matches_filter(&metadata, &filter));
    
    // Create a filter that shouldn't match
    // First condition: category must be "books"
    // Second condition: either price >= 60 OR tags contains "horror"
    let filter_no_match = Filter {
        conditions: vec![
            FilterCondition::Equals("category".to_string(), json!("books")),
            FilterCondition::Or(vec![
                FilterCondition::Range(
                    "price".to_string(),
                    RangeValue {
                        min: Some(json!(60)),
                        max: None
                    }
                ),
                FilterCondition::Contains(
                    "tags".to_string(),
                    vec![json!("horror")]
                )
            ])
        ]
    };
    
    assert!(!matches_filter(&metadata, &filter_no_match));
}

#[test]
fn test_matches_filter_empty_conditions() {
    let metadata = json!({
        "category": "books"
    });
    
    let filter = Filter {
        conditions: vec![]
    };
    
    // Empty filter should match everything
    assert!(matches_filter(&metadata, &filter));
}

#[test]
fn test_matches_filter_non_existent_field() {
    let metadata = json!({
        "category": "books"
    });
    
    let filter = Filter {
        conditions: vec![
            FilterCondition::Equals("author".to_string(), json!("John Doe"))
        ]
    };
    
    // Non-existent field should not match
    assert!(!matches_filter(&metadata, &filter));
}
