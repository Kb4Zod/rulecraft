use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form,
    Router,
};
use askama::Template;
use serde::Deserialize;
use std::net::SocketAddr;
use pulldown_cmark::{Parser, html};

use super::AppState;
use crate::middleware::extract_client_ip;
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

#[derive(Template)]
#[template(path = "scenario/error.html")]
struct ScenarioErrorTemplate {
    title: String,
    error: String,
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
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Form(query): Form<ScenarioQuery>,
) -> impl IntoResponse {
    // Build a minimal request to extract IP
    let mut req = axum::http::Request::new(());
    *req.headers_mut() = headers;
    req.extensions_mut().insert(ConnectInfo(addr));
    let client_ip = extract_client_ip(&req);

    // Check rate limit for AI endpoint
    if let Err(_) = state.rate_limiter.check_rate_limit(client_ip, "/scenario/ask", "POST").await {
        tracing::warn!("AI rate limit exceeded for IP {}", client_ip);
        let template = ScenarioErrorTemplate {
            title: "Rate Limit Exceeded".to_string(),
            error: "You have exceeded the AI request limit. Please wait before trying again. The limit is 5 requests per hour.".to_string(),
        };
        return Html(template.render().unwrap_or_else(|_| "Rate limit exceeded".to_string()));
    }

    // Validate input
    let question = query.question.trim();
    if question.is_empty() {
        let template = ScenarioErrorTemplate {
            title: "Invalid Question".to_string(),
            error: "Please enter a question.".to_string(),
        };
        return Html(template.render().unwrap_or_else(|_| "Error".to_string()));
    }

    if question.len() > 2000 {
        let template = ScenarioErrorTemplate {
            title: "Question Too Long".to_string(),
            error: "Questions must be 2000 characters or less.".to_string(),
        };
        return Html(template.render().unwrap_or_else(|_| "Error".to_string()));
    }

    // Get relevant rules for context
    let relevant_rules = crate::search::fulltext::search(&state.db, question)
        .await
        .unwrap_or_default();

    // Call Claude API for ruling
    let answer = match &state.config.claude_api_key {
        Some(api_key) => {
            crate::ai::claude::get_ruling(api_key, &state.config.claude_model, question, &relevant_rules)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("Claude API error: {}", e);
                    "Sorry, there was an error processing your question. Please try again later.".to_string()
                })
        }
        None => "AI rulings are currently unavailable. Please contact the administrator.".to_string(),
    };

    // Render markdown to HTML
    let parser = Parser::new(&answer);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let template = ScenarioResponseTemplate {
        question: question.to_string(),
        answer: html_output,
        cited_rules: relevant_rules,
    };
    Html(template.render().unwrap_or_else(|_| "Error rendering template".to_string()))
}
