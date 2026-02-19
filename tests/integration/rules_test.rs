//! Integration tests for rules functionality

use rulecraft::models::Rule;

#[tokio::test]
async fn test_rule_creation() {
    let rule = Rule::new(
        "Test Rule".to_string(),
        "Combat".to_string(),
        "This is a test rule content.".to_string(),
        "Test Source".to_string(),
    );

    assert!(!rule.id.is_empty());
    assert_eq!(rule.title, "Test Rule");
    assert_eq!(rule.category, "Combat");
    assert!(rule.subcategory.is_none());
}

#[tokio::test]
async fn test_search_sanitization() {
    // Test that special characters are handled
    let query = "attack (melee)";
    let sanitized = query
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>();

    assert_eq!(sanitized, "attack melee");
}
