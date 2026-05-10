use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::future::Future;

use crate::search::vector::{EmbeddingClient, EmbeddingError};

const OPENAI_EMBEDDINGS_URL: &str = "https://api.openai.com/v1/embeddings";

#[derive(Clone)]
pub struct OpenAiEmbeddingClient {
    client: Client,
    api_key: String,
    model: String,
    expected_dimension: usize,
}

impl OpenAiEmbeddingClient {
    pub fn new(api_key: String, model: String, expected_dimension: usize) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            expected_dimension,
        }
    }
}

impl EmbeddingClient for OpenAiEmbeddingClient {
    fn embed<'a>(
        &'a self,
        input: &'a str,
    ) -> impl Future<Output = Result<Vec<f32>, EmbeddingError>> + Send + 'a {
        async move {
            let input = input.trim();
            if input.is_empty() {
                return Err(EmbeddingError::EmptyInput);
            }

            if self.api_key.trim().is_empty() {
                return Err(EmbeddingError::MissingApiKey);
            }

            let request = OpenAiEmbeddingRequest {
                model: self.model.clone(),
                input: input.to_string(),
            };

            let response = self
                .client
                .post(OPENAI_EMBEDDINGS_URL)
                .bearer_auth(&self.api_key)
                .json(&request)
                .send()
                .await
                .map_err(|e| EmbeddingError::RequestError(e.to_string()))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(EmbeddingError::RequestError(format!("{}: {}", status, body)));
            }

            let response: OpenAiEmbeddingResponse = response
                .json()
                .await
                .map_err(|e| EmbeddingError::ParseError(e.to_string()))?;

            let embedding = response
                .data
                .into_iter()
                .next()
                .map(|item| item.embedding)
                .ok_or(EmbeddingError::EmptyResponse)?;

            validate_embedding_dimension(&embedding, self.expected_dimension)?;

            Ok(embedding)
        }
    }
}

#[derive(Serialize)]
struct OpenAiEmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
}

fn validate_embedding_dimension(
    embedding: &[f32],
    expected_dimension: usize,
) -> Result<(), EmbeddingError> {
    if embedding.len() != expected_dimension {
        return Err(EmbeddingError::DimensionMismatch {
            expected: expected_dimension,
            actual: embedding.len(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_input_does_not_call_provider() {
        let client = OpenAiEmbeddingClient::new(
            "unused".to_string(),
            "text-embedding-3-small".to_string(),
            1536,
        );

        let err = client.embed("   ").await.unwrap_err();

        assert!(matches!(err, EmbeddingError::EmptyInput));
    }

    #[tokio::test]
    async fn missing_api_key_fails_before_network_call() {
        let client = OpenAiEmbeddingClient::new(
            "".to_string(),
            "text-embedding-3-small".to_string(),
            1536,
        );

        let err = client.embed("grappled condition").await.unwrap_err();

        assert!(matches!(err, EmbeddingError::MissingApiKey));
    }

    #[test]
    fn dimension_validation_rejects_wrong_size() {
        let err = validate_embedding_dimension(&[0.1, 0.2], 1536).unwrap_err();

        assert!(matches!(
            err,
            EmbeddingError::DimensionMismatch {
                expected: 1536,
                actual: 2
            }
        ));
    }
}
