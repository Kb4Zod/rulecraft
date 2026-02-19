use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::models::Rule;

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: String,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

/// Get a ruling from Claude based on a scenario question and relevant rules
pub async fn get_ruling(
    api_key: &str,
    model: &str,
    question: &str,
    relevant_rules: &[Rule],
) -> Result<String, ClaudeError> {
    let client = Client::new();

    // Build context from relevant rules
    let rules_context = if relevant_rules.is_empty() {
        "No specific rules found for context.".to_string()
    } else {
        relevant_rules
            .iter()
            .map(|r| format!("## {}\n{}\n(Source: {}, Page {})\n",
                r.title,
                r.content,
                r.source,
                r.page.map(|p| p.to_string()).unwrap_or_else(|| "N/A".to_string())
            ))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let system_prompt = format!(
        r#"You are a D&D 2024 rules expert. Your role is to provide accurate rulings based on the official 2024 Player's Handbook and Dungeon Master's Guide.

IMPORTANT GUIDELINES:
1. Only cite rules from D&D 2024 (not 2014 or earlier editions)
2. When uncertain, clearly state the ambiguity
3. Distinguish between RAW (Rules as Written) and RAI (Rules as Intended)
4. If homebrew or DM discretion is needed, say so clearly
5. Cite specific page numbers when possible

RELEVANT RULES FOR CONTEXT:
{rules_context}

Provide clear, concise rulings that a DM can use at the table."#
    );

    let request = ClaudeRequest {
        model: model.to_string(),
        max_tokens: 1024,
        messages: vec![Message {
            role: "user".to_string(),
            content: question.to_string(),
        }],
        system: system_prompt,
    };

    let response = client
        .post(CLAUDE_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| ClaudeError::RequestError(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(ClaudeError::ApiError(format!("{}: {}", status, body)));
    }

    let claude_response: ClaudeResponse = response
        .json()
        .await
        .map_err(|e| ClaudeError::ParseError(e.to_string()))?;

    claude_response
        .content
        .first()
        .map(|c| c.text.clone())
        .ok_or_else(|| ClaudeError::EmptyResponse)
}

#[derive(Debug, thiserror::Error)]
pub enum ClaudeError {
    #[error("Request failed: {0}")]
    RequestError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Empty response from Claude")]
    EmptyResponse,
}
