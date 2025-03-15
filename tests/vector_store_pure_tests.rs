use p_mo::vector_store::cosine_similarity;

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
