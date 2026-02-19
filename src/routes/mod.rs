use axum::{
    extract::State,
    response::Html,
    routing::get,
    Router,
};
use askama::Template;
use sqlx::SqlitePool;

use crate::Config;

pub mod rules;
pub mod search;
pub mod scenario;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title: String,
}

#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate {
    title: String,
    content: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .merge(rules::router())
        .merge(search::router())
        .merge(scenario::router())
}

pub async fn index() -> Html<String> {
    let template = IndexTemplate {
        title: "Rulecraft - D&D 2024 Rules".to_string(),
    };
    Html(template.render().unwrap_or_else(|_| "Error rendering template".to_string()))
}

pub async fn health() -> &'static str {
    "OK"
}
