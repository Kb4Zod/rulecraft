use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use askama::Template;

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

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/rules", get(list_rules))
        .route("/rules/:id", get(get_rule))
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
