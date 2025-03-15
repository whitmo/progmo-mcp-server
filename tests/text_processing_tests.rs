use p_mo::text_processing::{
    tokenize, normalize_text, remove_stopwords, stem_words, extract_keywords
};

#[test]
fn test_tokenize() {
    let text = "Hello, world! This is a test.";
    let tokens = tokenize(text);
    assert_eq!(tokens, vec!["Hello", "world", "This", "is", "a", "test"]);
}

#[test]
fn test_normalize_text() {
    let text = "Hello, WORLD! This is a TEST.";
    let normalized = normalize_text(text);
    assert_eq!(normalized, "hello, world! this is a test.");
}

#[test]
fn test_remove_stopwords() {
    let tokens = vec!["this", "is", "a", "test", "of", "stopword", "removal"];
    let filtered = remove_stopwords(&tokens);
    assert_eq!(filtered, vec!["test", "stopword", "removal"]);
}

#[test]
fn test_stem_words() {
    let tokens = vec!["running", "jumps", "easily", "programming"];
    let stemmed = stem_words(&tokens);
    assert_eq!(stemmed, vec!["run".to_string(), "jump".to_string(), "easili".to_string(), "program".to_string()]);
}

#[test]
fn test_extract_keywords() {
    let text = "Natural language processing is a subfield of linguistics, computer science, and artificial intelligence concerned with the interactions between computers and human language.";
    let keywords = extract_keywords(text, 5);
    
    // The exact keywords might vary depending on the implementation,
    // but we can check that we get the expected number of keywords
    assert_eq!(keywords.len(), 5);
    
    // Check that common stopwords are not included
    assert!(!keywords.contains(&"is".to_string()));
    assert!(!keywords.contains(&"a".to_string()));
    assert!(!keywords.contains(&"the".to_string()));
    assert!(!keywords.contains(&"and".to_string()));
    assert!(!keywords.contains(&"between".to_string()));
}

#[test]
fn test_tokenize_empty_string() {
    let text = "";
    let tokens = tokenize(text);
    assert_eq!(tokens, Vec::<&str>::new());
}

#[test]
fn test_normalize_text_empty_string() {
    let text = "";
    let normalized = normalize_text(text);
    assert_eq!(normalized, "");
}

#[test]
fn test_remove_stopwords_empty_list() {
    let tokens = Vec::<&str>::new();
    let filtered = remove_stopwords(&tokens);
    assert_eq!(filtered, Vec::<&str>::new());
}

#[test]
fn test_stem_words_empty_list() {
    let tokens = Vec::<&str>::new();
    let stemmed = stem_words(&tokens);
    assert_eq!(stemmed, Vec::<&str>::new());
}

#[test]
fn test_extract_keywords_empty_string() {
    let text = "";
    let keywords = extract_keywords(text, 5);
    assert_eq!(keywords, Vec::<String>::new());
}

#[test]
fn test_extract_keywords_zero_count() {
    let text = "This is a test of keyword extraction.";
    let keywords = extract_keywords(text, 0);
    assert_eq!(keywords, Vec::<String>::new());
}

#[test]
fn test_tokenize_with_punctuation() {
    let text = "Hello, world! This is a test. What about semi-colons; and dashes-?";
    let tokens = tokenize(text);
    assert_eq!(tokens, vec!["Hello", "world", "This", "is", "a", "test", "What", "about", "semi-colons", "and", "dashes"]);
}

#[test]
fn test_normalize_text_with_numbers() {
    let text = "Testing 123 with numbers 456.";
    let normalized = normalize_text(text);
    assert_eq!(normalized, "testing 123 with numbers 456.");
}

#[test]
fn test_remove_stopwords_all_stopwords() {
    let tokens = vec!["this", "is", "a", "the", "and", "of"];
    let filtered = remove_stopwords(&tokens);
    assert_eq!(filtered, Vec::<&str>::new());
}

#[test]
fn test_stem_words_already_stemmed() {
    let tokens = vec!["run", "jump", "program"];
    let stemmed = stem_words(&tokens);
    assert_eq!(stemmed, vec!["run".to_string(), "jump".to_string(), "program".to_string()]);
}
