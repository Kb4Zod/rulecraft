use crate::models::Rule;
use std::future::Future;

#[derive(Debug, Clone, PartialEq)]
pub struct VectorHit {
    pub rule_id: String,
    pub score: f32,
}

pub trait EmbeddingClient: Send + Sync {
    fn embed<'a>(
        &'a self,
        input: &'a str,
    ) -> impl Future<Output = Result<Vec<f32>, EmbeddingError>> + Send + 'a;
}

pub trait VectorIndex: Send + Sync {
    fn ensure_collection(&self) -> impl Future<Output = Result<(), VectorSearchError>> + Send + '_;
    fn upsert_rule(
        &self,
        rule: &Rule,
        vector: Vec<f32>,
    ) -> impl Future<Output = Result<(), VectorSearchError>> + Send + '_;
    fn search(
        &self,
        vector: Vec<f32>,
        limit: usize,
    ) -> impl Future<Output = Result<Vec<VectorHit>, VectorSearchError>> + Send + '_;
    fn delete_rule(
        &self,
        rule_id: &str,
    ) -> impl Future<Output = Result<(), VectorSearchError>> + Send + '_;
}

pub fn rule_to_embedding_text(rule: &Rule) -> String {
    let subcategory = rule.subcategory.as_deref().unwrap_or("N/A");
    let page = rule
        .page
        .map(|page| page.to_string())
        .unwrap_or_else(|| "N/A".to_string());

    format!(
        "Title: {}\nCategory: {}\nSubcategory: {}\nContent: {}\nSource: {}\nPage: {}",
        rule.title, rule.category, subcategory, rule.content, rule.source, page
    )
}

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("embedding input is empty")]
    EmptyInput,

    #[error("embedding API key is not configured")]
    MissingApiKey,

    #[error("embedding request failed: {0}")]
    RequestError(String),

    #[error("embedding response could not be parsed: {0}")]
    ParseError(String),

    #[error("embedding dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("embedding provider returned no vector")]
    EmptyResponse,
}

#[derive(Debug, thiserror::Error)]
pub enum VectorSearchError {
    #[error("Vector database not configured")]
    NotConfigured,

    #[error("Embedding generation failed: {0}")]
    EmbeddingError(#[from] EmbeddingError),

    #[error("Search failed: {0}")]
    SearchError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rule() -> Rule {
        Rule {
            id: "opportunity-attack".to_string(),
            title: "Opportunity Attack".to_string(),
            category: "Combat".to_string(),
            subcategory: Some("Actions".to_string()),
            content: "You can make an opportunity attack when a hostile creature leaves your reach.".to_string(),
            source: "Player's Handbook 2024".to_string(),
            page: Some(195),
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        }
    }

    #[test]
    fn embedding_text_contains_canonical_rule_fields() {
        let text = rule_to_embedding_text(&test_rule());

        assert!(text.contains("Title: Opportunity Attack"));
        assert!(text.contains("Category: Combat"));
        assert!(text.contains("Subcategory: Actions"));
        assert!(text.contains("Content: You can make an opportunity attack"));
        assert!(text.contains("Source: Player's Handbook 2024"));
        assert!(text.contains("Page: 195"));
    }

    #[test]
    fn embedding_text_handles_missing_optional_fields() {
        let mut rule = test_rule();
        rule.subcategory = None;
        rule.page = None;

        let text = rule_to_embedding_text(&rule);

        assert!(text.contains("Subcategory: N/A"));
        assert!(text.contains("Page: N/A"));
    }
}
