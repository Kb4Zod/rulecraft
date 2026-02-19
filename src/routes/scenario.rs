use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Form,
    Router,
};
use askama::Template;
use serde::Deserialize;

use super::AppState;
use crate::models::Rule;

#[derive(Deserialize)]
pub struct ScenarioQuery {
    question: String,
}

#[derive(Template)]
#[template(path = "scenario/ask.html")]
struct ScenarioAskTemplate {
    title: String,
}

#[derive(Template)]
#[template(path = "scenario/response.html")]
struct ScenarioResponseTemplate {
    question: String,
    answer: String,
    cited_rules: Vec<Rule>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/scenario", get(scenario_form))
        .route("/scenario/ask", post(ask_scenario))
}

async fn scenario_form() -> Html<String> {
    let template = ScenarioAskTemplate {
        title: "Ask a Scenario Question".to_string(),
    };
    Html(template.render().unwrap_or_else(|_| "Error rendering template".to_string()))
}

async fn ask_scenario(
    State(state): State<AppState>,
    Form(query): Form<ScenarioQuery>,
) -> Html<String> {
    // Get relevant rules for context
    let relevant_rules = crate::search::fulltext::search(&state.db, &query.question)
        .await
        .unwrap_or_default();

    // Call Claude API for ruling
    let answer = match &state.config.claude_api_key {
        Some(api_key) => {
            crate::ai::claude::get_ruling(api_key, &state.config.claude_model, &query.question, &relevant_rules)
                .await
                .unwrap_or_else(|e| format!("Error getting ruling: {}", e))
        }
        None => "Claude API key not configured. Please set CLAUDE_API_KEY environment variable.".to_string(),
    };

    let template = ScenarioResponseTemplate {
        question: query.question,
        answer,
        cited_rules: relevant_rules,
    };
    Html(template.render().unwrap_or_else(|_| "Error rendering template".to_string()))
}
