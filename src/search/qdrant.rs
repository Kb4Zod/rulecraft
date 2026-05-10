use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::future::Future;

use crate::{
    models::Rule,
    search::vector::{VectorHit, VectorIndex, VectorSearchError},
};

#[derive(Clone)]
pub struct QdrantVectorIndex {
    client: Client,
    base_url: String,
    collection: String,
    dimension: usize,
}

impl QdrantVectorIndex {
    pub fn new(base_url: String, collection: String, dimension: usize) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            collection,
            dimension,
        }
    }

    fn collection_url(&self) -> String {
        format!("{}/collections/{}", self.base_url, self.collection)
    }

    fn points_url(&self) -> String {
        format!("{}/points", self.collection_url())
    }
}

impl VectorIndex for QdrantVectorIndex {
    fn ensure_collection(&self) -> impl Future<Output = Result<(), VectorSearchError>> + Send + '_ {
        async move {
            let body = json!({
                "vectors": {
                    "size": self.dimension,
                    "distance": "Cosine"
                }
            });

            let response = self
                .client
                .put(self.collection_url())
                .json(&body)
                .send()
                .await
                .map_err(|e| VectorSearchError::SearchError(e.to_string()))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(VectorSearchError::SearchError(format!("{}: {}", status, body)));
            }

            Ok(())
        }
    }

    fn upsert_rule<'a>(
        &'a self,
        rule: &'a Rule,
        vector: Vec<f32>,
    ) -> impl Future<Output = Result<(), VectorSearchError>> + Send + 'a {
        async move {
            if vector.len() != self.dimension {
                return Err(VectorSearchError::SearchError(format!(
                    "vector dimension mismatch: expected {}, got {}",
                    self.dimension,
                    vector.len()
                )));
            }

            let body = json!({
                "points": [{
                    "id": point_id_for_rule_id(&rule.id),
                    "vector": vector,
                    "payload": {
                        "rule_id": rule.id,
                        "category": rule.category,
                        "source": rule.source,
                        "page": rule.page
                    }
                }]
            });

            let response = self
                .client
                .put(format!("{}?wait=true", self.points_url()))
                .json(&body)
                .send()
                .await
                .map_err(|e| VectorSearchError::SearchError(e.to_string()))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(VectorSearchError::SearchError(format!("{}: {}", status, body)));
            }

            Ok(())
        }
    }

    fn search(
        &self,
        vector: Vec<f32>,
        limit: usize,
    ) -> impl Future<Output = Result<Vec<VectorHit>, VectorSearchError>> + Send + '_ {
        async move {
            if vector.len() != self.dimension {
                return Err(VectorSearchError::SearchError(format!(
                    "query vector dimension mismatch: expected {}, got {}",
                    self.dimension,
                    vector.len()
                )));
            }

            let body = json!({
                "vector": vector,
                "limit": limit,
                "with_payload": true,
                "with_vector": false
            });

            let response = self
                .client
                .post(format!("{}/search", self.points_url()))
                .json(&body)
                .send()
                .await
                .map_err(|e| VectorSearchError::SearchError(e.to_string()))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(VectorSearchError::SearchError(format!("{}: {}", status, body)));
            }

            let response: QdrantSearchResponse = response
                .json()
                .await
                .map_err(|e| VectorSearchError::SearchError(e.to_string()))?;

            Ok(hits_from_search_response(response))
        }
    }

    fn delete_rule<'a>(
        &'a self,
        rule_id: &'a str,
    ) -> impl Future<Output = Result<(), VectorSearchError>> + Send + 'a {
        async move {
            let body = json!({
                "points": [point_id_for_rule_id(rule_id)]
            });

            let response = self
                .client
                .post(format!("{}/delete?wait=true", self.points_url()))
                .json(&body)
                .send()
                .await
                .map_err(|e| VectorSearchError::SearchError(e.to_string()))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(VectorSearchError::SearchError(format!("{}: {}", status, body)));
            }

            Ok(())
        }
    }
}

#[derive(Deserialize)]
struct QdrantSearchResponse {
    result: Vec<QdrantScoredPoint>,
}

#[derive(Deserialize)]
struct QdrantScoredPoint {
    score: f32,
    payload: Option<Value>,
}

fn hits_from_search_response(response: QdrantSearchResponse) -> Vec<VectorHit> {
    response
        .result
        .into_iter()
        .filter_map(|point| {
            let rule_id = point
                .payload
                .as_ref()
                .and_then(|payload| payload.get("rule_id"))
                .and_then(|value| value.as_str())?;

            Some(VectorHit {
                rule_id: rule_id.to_string(),
                score: point.score,
            })
        })
        .collect()
}

pub fn point_id_for_rule_id(rule_id: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;

    for byte in rule_id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_id_is_stable_for_rule_id() {
        assert_eq!(
            point_id_for_rule_id("opportunity-attack"),
            point_id_for_rule_id("opportunity-attack")
        );
        assert_ne!(
            point_id_for_rule_id("opportunity-attack"),
            point_id_for_rule_id("grappled")
        );
    }

    #[test]
    fn search_response_uses_payload_rule_ids_and_skips_missing_payloads() {
        let response: QdrantSearchResponse = serde_json::from_value(json!({
            "result": [
                {
                    "id": 1,
                    "score": 0.91,
                    "payload": { "rule_id": "grappled" }
                },
                {
                    "id": 2,
                    "score": 0.5,
                    "payload": { "category": "Combat" }
                }
            ]
        }))
        .unwrap();

        let hits = hits_from_search_response(response);

        assert_eq!(
            hits,
            vec![VectorHit {
                rule_id: "grappled".to_string(),
                score: 0.91
            }]
        );
    }
}
