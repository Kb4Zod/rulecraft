use sqlx::SqlitePool;
use std::collections::HashSet;

use crate::{
    config::VectorSearchConfig,
    models::Rule,
    search::{
        openai_embeddings::OpenAiEmbeddingClient,
        qdrant::QdrantVectorIndex,
        vector::{EmbeddingClient, VectorIndex, VectorSearchError},
    },
};

#[derive(Debug, Clone)]
pub struct ScoredRule {
    pub rule: Rule,
    pub score: f32,
}

pub async fn retrieve_oracle_rules(
    pool: &SqlitePool,
    vector_config: &VectorSearchConfig,
    query: &str,
) -> Vec<Rule> {
    let fts_rules = crate::search::fulltext::search(pool, query)
        .await
        .unwrap_or_default();

    if !vector_config.enabled {
        return fts_rules
            .into_iter()
            .take(vector_config.oracle_max_context_rules)
            .collect();
    }

    let vector_rules = match semantic_search(pool, vector_config, query).await {
        Ok(rules) => rules,
        Err(error) => {
            tracing::warn!("Vector Oracle retrieval failed; falling back to FTS5: {}", error);
            Vec::new()
        }
    };

    merge_oracle_results(
        fts_rules,
        vector_rules,
        vector_config.score_threshold,
        vector_config.oracle_max_context_rules,
    )
}

pub async fn semantic_search(
    pool: &SqlitePool,
    vector_config: &VectorSearchConfig,
    query: &str,
) -> Result<Vec<ScoredRule>, VectorSearchError> {
    if !vector_config.enabled {
        return Ok(vec![]);
    }

    let api_key = vector_config
        .openai_api_key
        .clone()
        .ok_or(VectorSearchError::NotConfigured)?;

    let embedding_client = OpenAiEmbeddingClient::new(
        api_key,
        vector_config.openai_embedding_model.clone(),
        vector_config.openai_embedding_dimension,
    );
    let vector_index = QdrantVectorIndex::new(
        vector_config.qdrant_url.clone(),
        vector_config.qdrant_collection.clone(),
        vector_config.openai_embedding_dimension,
    );

    semantic_search_with_clients(
        pool,
        query,
        vector_config.top_k,
        &embedding_client,
        &vector_index,
    )
    .await
}

pub async fn semantic_search_with_clients<E, V>(
    pool: &SqlitePool,
    query: &str,
    limit: usize,
    embedding_client: &E,
    vector_index: &V,
) -> Result<Vec<ScoredRule>, VectorSearchError>
where
    E: EmbeddingClient,
    V: VectorIndex,
{
    let query_embedding = embedding_client.embed(query).await?;
    let hits = vector_index.search(query_embedding, limit).await?;
    let mut rules = Vec::with_capacity(hits.len());

    for hit in hits {
        match crate::db::get_rule_by_id(pool, &hit.rule_id).await {
            Ok(Some(rule)) => rules.push(ScoredRule {
                rule,
                score: hit.score,
            }),
            Ok(None) => {
                tracing::warn!("Qdrant returned unknown rule id '{}'", hit.rule_id);
            }
            Err(error) => {
                tracing::warn!("Failed to hydrate vector search rule '{}': {}", hit.rule_id, error);
            }
        }
    }

    Ok(rules)
}

pub fn merge_oracle_results(
    fts_rules: Vec<Rule>,
    vector_rules: Vec<ScoredRule>,
    score_threshold: f32,
    limit: usize,
) -> Vec<Rule> {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();

    for rule in fts_rules {
        if seen.insert(rule.id.clone()) {
            merged.push(rule);
        }

        if merged.len() >= limit {
            return merged;
        }
    }

    for scored_rule in vector_rules {
        if scored_rule.score < score_threshold {
            continue;
        }

        if seen.insert(scored_rule.rule.id.clone()) {
            merged.push(scored_rule.rule);
        }

        if merged.len() >= limit {
            break;
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: &str, title: &str) -> Rule {
        Rule {
            id: id.to_string(),
            title: title.to_string(),
            category: "Combat".to_string(),
            subcategory: None,
            content: "Rule text".to_string(),
            source: "Player's Handbook 2024".to_string(),
            page: None,
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        }
    }

    fn scored(rule: Rule, score: f32) -> ScoredRule {
        ScoredRule { rule, score }
    }

    #[test]
    fn merge_keeps_fts_results_before_semantic_results() {
        let merged = merge_oracle_results(
            vec![rule("attack-action", "Attack Action")],
            vec![scored(rule("grappled", "Grappled"), 0.92)],
            0.35,
            10,
        );

        let ids: Vec<_> = merged.into_iter().map(|rule| rule.id).collect();
        assert_eq!(ids, vec!["attack-action", "grappled"]);
    }

    #[test]
    fn merge_dedupes_vector_rules_already_found_by_fts() {
        let merged = merge_oracle_results(
            vec![rule("grappled", "Grappled")],
            vec![
                scored(rule("grappled", "Grappled"), 0.95),
                scored(rule("escape", "Escaping a Grapple"), 0.9),
            ],
            0.35,
            10,
        );

        let ids: Vec<_> = merged.into_iter().map(|rule| rule.id).collect();
        assert_eq!(ids, vec!["grappled", "escape"]);
    }

    #[test]
    fn merge_excludes_low_score_vector_results() {
        let merged = merge_oracle_results(
            vec![],
            vec![
                scored(rule("cover", "Cover"), 0.7),
                scored(rule("swimming", "Swimming"), 0.2),
            ],
            0.35,
            10,
        );

        let ids: Vec<_> = merged.into_iter().map(|rule| rule.id).collect();
        assert_eq!(ids, vec!["cover"]);
    }

    #[test]
    fn merge_respects_context_limit() {
        let merged = merge_oracle_results(
            vec![rule("a", "A"), rule("b", "B")],
            vec![scored(rule("c", "C"), 0.9)],
            0.35,
            2,
        );

        let ids: Vec<_> = merged.into_iter().map(|rule| rule.id).collect();
        assert_eq!(ids, vec!["a", "b"]);
    }
}
