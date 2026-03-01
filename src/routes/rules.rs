use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use askama::Template;
use serde::Deserialize;
use pulldown_cmark::{Parser, html};

use super::AppState;
use crate::models::Rule;

#[derive(Template)]
#[template(path = "rules/list.html")]
struct RulesListTemplate {
    title: String,
    rules: Vec<Rule>,
}

#[derive(Template)]
#[template(path = "rules/detail.html")]
struct RuleDetailTemplate {
    title: String,
    rule: Rule,
}

#[derive(Deserialize)]
pub struct CreateRuleRequest {
    pub title: String,
    pub category: String,
    pub subcategory: Option<String>,
    pub content: String,
    pub source: String,
    pub page: Option<i32>,
}

#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl CreateRuleRequest {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Title: 1-200 characters
        if self.title.is_empty() {
            errors.push(ValidationError {
                field: "title".to_string(),
                message: "Title is required".to_string(),
            });
        } else if self.title.len() > 200 {
            errors.push(ValidationError {
                field: "title".to_string(),
                message: "Title must be 200 characters or less".to_string(),
            });
        }

        // Category: 1-100 characters
        if self.category.is_empty() {
            errors.push(ValidationError {
                field: "category".to_string(),
                message: "Category is required".to_string(),
            });
        } else if self.category.len() > 100 {
            errors.push(ValidationError {
                field: "category".to_string(),
                message: "Category must be 100 characters or less".to_string(),
            });
        }

        // Subcategory: optional, but max 100 characters
        if let Some(ref sub) = self.subcategory {
            if sub.len() > 100 {
                errors.push(ValidationError {
                    field: "subcategory".to_string(),
                    message: "Subcategory must be 100 characters or less".to_string(),
                });
            }
        }

        // Content: 1-50000 characters
        if self.content.is_empty() {
            errors.push(ValidationError {
                field: "content".to_string(),
                message: "Content is required".to_string(),
            });
        } else if self.content.len() > 50000 {
            errors.push(ValidationError {
                field: "content".to_string(),
                message: "Content must be 50000 characters or less".to_string(),
            });
        }

        // Source: 1-200 characters
        if self.source.is_empty() {
            errors.push(ValidationError {
                field: "source".to_string(),
                message: "Source is required".to_string(),
            });
        } else if self.source.len() > 200 {
            errors.push(ValidationError {
                field: "source".to_string(),
                message: "Source must be 200 characters or less".to_string(),
            });
        }

        // Page: 1-2000 range
        if let Some(page) = self.page {
            if page < 1 || page > 2000 {
                errors.push(ValidationError {
                    field: "page".to_string(),
                    message: "Page must be between 1 and 2000".to_string(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/rules", get(list_rules))
        .route("/rules/:id", get(get_rule))
        .route("/api/rules", post(create_rule))
}

async fn list_rules(State(state): State<AppState>) -> Html<String> {
    let rules = crate::db::get_all_rules(&state.db).await.unwrap_or_default();

    let template = RulesListTemplate {
        title: "Rules".to_string(),
        rules,
    };
    Html(template.render().unwrap_or_else(|_| "Error rendering template".to_string()))
}

async fn get_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Html<String> {
    match crate::db::get_rule_by_id(&state.db, &id).await {
        Ok(Some(mut rule)) => {
            // Render markdown to HTML for the rule content
            let parser = Parser::new(&rule.content);
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);
            rule.content = html_output;

            let template = RuleDetailTemplate {
                title: rule.title.clone(),
                rule,
            };
            Html(template.render().unwrap_or_else(|_| "Error rendering template".to_string()))
        }
        _ => Html("Rule not found".to_string()),
    }
}

async fn create_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateRuleRequest>,
) -> impl IntoResponse {
    // Check admin API key authentication
    let admin_key = match &state.config.admin_api_key {
        Some(key) => key,
        None => {
            tracing::warn!("POST /api/rules attempted but ADMIN_API_KEY not configured");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "Admin endpoint not configured"
                })),
            );
        }
    };

    let provided_key = headers
        .get("X-Admin-Key")
        .and_then(|v| v.to_str().ok());

    match provided_key {
        Some(key) if key == admin_key => {
            // Authenticated - proceed
        }
        Some(_) => {
            tracing::warn!("POST /api/rules attempted with invalid API key");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid API key"
                })),
            );
        }
        None => {
            tracing::warn!("POST /api/rules attempted without API key");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "API key required. Provide X-Admin-Key header."
                })),
            );
        }
    }

    // Validate input
    if let Err(errors) = payload.validate() {
        let error_messages: Vec<String> = errors
            .iter()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect();
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation failed",
                "details": error_messages
            })),
        );
    }

    let now = chrono::Utc::now().to_rfc3339();
    let rule = Rule {
        id: uuid::Uuid::new_v4().to_string(),
        title: payload.title,
        category: payload.category,
        subcategory: payload.subcategory,
        content: payload.content,
        source: payload.source,
        page: payload.page,
        created_at: now.clone(),
        updated_at: now,
    };

    match crate::db::create_rule(&state.db, &rule).await {
        Ok(_) => {
            tracing::info!("Rule created: {} ({})", rule.title, rule.id);
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "id": rule.id,
                    "message": "Rule created successfully"
                })),
            )
        }
        Err(e) => {
            tracing::error!("Failed to create rule: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to create rule"
                })),
            )
        }
    }
}
