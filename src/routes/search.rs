use axum::{
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use askama::Template;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use super::AppState;
use crate::middleware::extract_client_ip;
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
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    // Build a minimal request to extract IP
    let mut req = axum::http::Request::new(());
    *req.headers_mut() = headers;
    req.extensions_mut().insert(ConnectInfo(addr));
    let client_ip = extract_client_ip(&req);

    // Check rate limit
    if let Err(e) = state.rate_limiter.check_rate_limit(client_ip, "/search", "GET").await {
        tracing::warn!("Search rate limit exceeded for IP {}", client_ip);
        return e.into_response();
    }

    let query = params.q.unwrap_or_default();

    // Validate query length
    let query = if query.len() > 500 {
        query[..500].to_string()
    } else {
        query
    };

    let results = if query.is_empty() {
        vec![]
    } else {
        let fts_results = crate::search::fulltext::search(&state.db, &query)
            .await
            .unwrap_or_default();

        // Fall back to fuzzy search if FTS returns no results
        if fts_results.is_empty() {
            crate::db::fuzzy_search(&state.db, &query, 20)
                .await
                .unwrap_or_default()
        } else {
            fts_results
        }
    };

    let template = SearchResultsTemplate {
        title: format!("Search: {}", query),
        query,
        results,
    };
    Html(template.render().unwrap_or_else(|_| "Error rendering template".to_string())).into_response()
}

async fn api_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    // Build a minimal request to extract IP
    let mut req = axum::http::Request::new(());
    *req.headers_mut() = headers;
    req.extensions_mut().insert(ConnectInfo(addr));
    let client_ip = extract_client_ip(&req);

    // Check rate limit
    if let Err(_) = state.rate_limiter.check_rate_limit(client_ip, "/api/search", "GET").await {
        tracing::warn!("API search rate limit exceeded for IP {}", client_ip);
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({"error": "Rate limit exceeded"})),
        ).into_response();
    }

    let query = params.q.unwrap_or_default();

    // Validate query length
    if query.len() < 2 {
        return Json(Vec::<SearchSuggestion>::new()).into_response();
    }

    let query = if query.len() > 500 {
        query[..500].to_string()
    } else {
        query
    };

    // Use fuzzy search for suggestions
    let results = crate::db::fuzzy_search(&state.db, &query, 8)
        .await
        .unwrap_or_default();

    let suggestions: Vec<SearchSuggestion> = results
        .into_iter()
        .map(|rule| {
            let excerpt = rule.excerpt(100);
            SearchSuggestion {
                id: rule.id,
                title: rule.title,
                category: rule.category,
                excerpt,
            }
        })
        .collect();

    Json(suggestions).into_response()
}
