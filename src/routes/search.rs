use axum::{
    extract::{Query, State},
    response::{Html, Json},
    routing::get,
    Router,
};
use askama::Template;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
struct SearchSuggestion {
    id: String,
    title: String,
    category: String,
    excerpt: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_rules))
        .route("/api/search", get(api_search))
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

async fn api_search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Json<Vec<SearchSuggestion>> {
    let query = params.q.unwrap_or_default();

    if query.len() < 2 {
        return Json(vec![]);
    }

    // Use fuzzy search for suggestions
    let results = crate::db::fuzzy_search(&state.db, &query, 8)
        .await
        .unwrap_or_default();

    let suggestions: Vec<SearchSuggestion> = results
        .into_iter()
        .map(|rule| {
            let excerpt = if rule.content.len() > 100 {
                format!("{}...", &rule.content[..100])
            } else {
                rule.content.clone()
            };
            SearchSuggestion {
                id: rule.id,
                title: rule.title,
                category: rule.category,
                excerpt,
            }
        })
        .collect();

    Json(suggestions)
}
