use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use askama::Template;
use serde::Deserialize;

use super::AppState;
use crate::models::Rule;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

#[derive(Template)]
#[template(path = "rules/search.html")]
struct SearchResultsTemplate {
    title: String,
    query: String,
    results: Vec<Rule>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_rules))
}

async fn search_rules(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Html<String> {
    let query = params.q.unwrap_or_default();

    let results = if query.is_empty() {
        vec![]
    } else {
        crate::search::fulltext::search(&state.db, &query)
            .await
            .unwrap_or_default()
    };

    let template = SearchResultsTemplate {
        title: format!("Search: {}", query),
        query,
        results,
    };
    Html(template.render().unwrap_or_else(|_| "Error rendering template".to_string()))
}
