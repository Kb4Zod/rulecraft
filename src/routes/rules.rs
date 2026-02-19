use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use askama::Template;
use serde::Deserialize;

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
        Ok(Some(rule)) => {
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
    Json(payload): Json<CreateRuleRequest>,
) -> impl IntoResponse {
    let rule = Rule {
        id: uuid::Uuid::new_v4().to_string(),
        title: payload.title,
        category: payload.category,
        subcategory: payload.subcategory,
        content: payload.content,
        source: payload.source,
        page: payload.page,
        created_at: chrono_now(),
        updated_at: chrono_now(),
    };

    match crate::db::create_rule(&state.db, &rule).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({
            "id": rule.id,
            "message": "Rule created successfully"
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}
