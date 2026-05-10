use crate::{
    models::Rule,
    search::vector::{rule_to_embedding_text, EmbeddingClient, VectorIndex, VectorSearchError},
};

#[derive(Debug, Default, PartialEq)]
pub struct IndexReport {
    pub indexed: usize,
    pub failed: Vec<IndexFailure>,
}

#[derive(Debug, PartialEq)]
pub struct IndexFailure {
    pub rule_id: String,
    pub error: String,
}

pub async fn index_rules<E, V>(
    rules: &[Rule],
    embedding_client: &E,
    vector_index: &V,
    fail_fast: bool,
) -> Result<IndexReport, VectorSearchError>
where
    E: EmbeddingClient,
    V: VectorIndex,
{
    vector_index.ensure_collection().await?;

    let mut report = IndexReport::default();

    for rule in rules {
        let text = rule_to_embedding_text(rule);
        let result = async {
            let embedding = embedding_client.embed(&text).await?;
            vector_index.upsert_rule(rule, embedding).await?;
            Ok::<(), VectorSearchError>(())
        }
        .await;

        match result {
            Ok(()) => report.indexed += 1,
            Err(error) if fail_fast => return Err(error),
            Err(error) => report.failed.push(IndexFailure {
                rule_id: rule.id.clone(),
                error: error.to_string(),
            }),
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::vector::{EmbeddingError, VectorHit};
    use std::{
        collections::HashSet,
        future::Future,
        sync::{Arc, Mutex},
    };

    fn rule(id: &str) -> Rule {
        Rule {
            id: id.to_string(),
            title: format!("Rule {id}"),
            category: "Combat".to_string(),
            subcategory: None,
            content: "Rule text".to_string(),
            source: "Player's Handbook 2024".to_string(),
            page: None,
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        }
    }

    struct MockEmbeddingClient {
        fail_ids: HashSet<String>,
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl EmbeddingClient for MockEmbeddingClient {
        fn embed<'a>(
            &'a self,
            input: &'a str,
        ) -> impl Future<Output = Result<Vec<f32>, EmbeddingError>> + Send + 'a {
            async move {
                self.calls.lock().unwrap().push(input.to_string());

                if self
                    .fail_ids
                    .iter()
                    .any(|id| input.contains(&format!("Title: Rule {id}")))
                {
                    return Err(EmbeddingError::RequestError("forced failure".to_string()));
                }

                Ok(vec![0.1, 0.2, 0.3])
            }
        }
    }

    #[derive(Default)]
    struct MockVectorIndex {
        ensured: Arc<Mutex<usize>>,
        upserted: Arc<Mutex<Vec<String>>>,
    }

    impl VectorIndex for MockVectorIndex {
        fn ensure_collection(&self) -> impl Future<Output = Result<(), VectorSearchError>> + Send + '_ {
            async move {
                *self.ensured.lock().unwrap() += 1;
                Ok(())
            }
        }

        fn upsert_rule<'a>(
            &'a self,
            rule: &'a Rule,
            _vector: Vec<f32>,
        ) -> impl Future<Output = Result<(), VectorSearchError>> + Send + 'a {
            async move {
                self.upserted.lock().unwrap().push(rule.id.clone());
                Ok(())
            }
        }

        fn search(
            &self,
            _vector: Vec<f32>,
            _limit: usize,
        ) -> impl Future<Output = Result<Vec<VectorHit>, VectorSearchError>> + Send + '_ {
            async { Ok(vec![]) }
        }

        fn delete_rule<'a>(
            &'a self,
            _rule_id: &'a str,
        ) -> impl Future<Output = Result<(), VectorSearchError>> + Send + 'a {
            async { Ok(()) }
        }
    }

    #[tokio::test]
    async fn index_rules_embeds_and_upserts_each_rule() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let embedding_client = MockEmbeddingClient {
            fail_ids: HashSet::new(),
            calls: Arc::clone(&calls),
        };
        let vector_index = MockVectorIndex::default();
        let rules = vec![rule("a"), rule("b")];

        let report = index_rules(&rules, &embedding_client, &vector_index, false)
            .await
            .unwrap();

        assert_eq!(report.indexed, 2);
        assert!(report.failed.is_empty());
        assert_eq!(*vector_index.ensured.lock().unwrap(), 1);
        assert_eq!(*vector_index.upserted.lock().unwrap(), vec!["a", "b"]);
        assert_eq!(calls.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn index_rules_reports_failed_ids_without_aborting() {
        let embedding_client = MockEmbeddingClient {
            fail_ids: HashSet::from(["b".to_string()]),
            calls: Arc::new(Mutex::new(Vec::new())),
        };
        let vector_index = MockVectorIndex::default();
        let rules = vec![rule("a"), rule("b"), rule("c")];

        let report = index_rules(&rules, &embedding_client, &vector_index, false)
            .await
            .unwrap();

        assert_eq!(report.indexed, 2);
        assert_eq!(report.failed.len(), 1);
        assert_eq!(report.failed[0].rule_id, "b");
        assert_eq!(*vector_index.upserted.lock().unwrap(), vec!["a", "c"]);
    }
}
