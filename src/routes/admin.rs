use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form,
    Router,
};
use askama::Template;
use serde::Deserialize;

use super::AppState;
use crate::models::Rule;

// ── Templates ──────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/login.html")]
struct AdminLoginTemplate {
    title: String,
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct AdminDashboardTemplate {
    title: String,
    rules: Vec<Rule>,
    message: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/form.html")]
struct AdminFormTemplate {
    title: String,
    is_edit: bool,
    rule: Option<Rule>,
    categories: Vec<String>,
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/not_configured.html")]
struct AdminNotConfiguredTemplate {
    title: String,
}

// ── Form Data ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginForm {
    admin_key: String,
}

#[derive(Deserialize)]
pub struct RuleForm {
    id: String,
    title: String,
    category: String,
    subcategory: Option<String>,
    source: String,
    page: Option<i32>,
    content: String,
}

// ── Auth Helper ────────────────────────────────────────────

fn get_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';')
                .map(|s| s.trim())
                .find(|s| s.starts_with(&format!("{}=", name)))
                .map(|s| s[name.len() + 1..].to_string())
        })
}

fn is_authenticated(headers: &HeaderMap, admin_key: &Option<String>) -> bool {
    match admin_key {
        Some(key) => {
            get_cookie_value(headers, "admin_token")
                .map(|v| v == *key)
                .unwrap_or(false)
        }
        None => false,
    }
}

// ── Router ─────────────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin", get(admin_index))
        .route("/admin/login", post(admin_login))
        .route("/admin/logout", get(admin_logout))
        .route("/admin/rules/new", get(admin_new_rule))
        .route("/admin/rules", post(admin_create_rule))
        .route("/admin/rules/:id/edit", get(admin_edit_rule))
        .route("/admin/rules/:id/edit", post(admin_update_rule))
        .route("/admin/rules/:id/delete", post(admin_delete_rule))
}

// ── Handlers ───────────────────────────────────────────────

async fn admin_index(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Check if admin is configured
    if state.config.admin_api_key.is_none() {
        let template = AdminNotConfiguredTemplate {
            title: "Admin Not Configured".to_string(),
        };
        return Html(template.render().unwrap_or_else(|_| "Error".to_string())).into_response();
    }

    // Check authentication
    if !is_authenticated(&headers, &state.config.admin_api_key) {
        let template = AdminLoginTemplate {
            title: "Admin Login".to_string(),
            error: None,
        };
        return Html(template.render().unwrap_or_else(|_| "Error".to_string())).into_response();
    }

    // Show dashboard
    let rules = crate::db::get_all_rules(&state.db).await.unwrap_or_default();
    let template = AdminDashboardTemplate {
        title: "Admin Dashboard".to_string(),
        rules,
        message: None,
    };
    Html(template.render().unwrap_or_else(|_| "Error".to_string())).into_response()
}

async fn admin_login(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    match &state.config.admin_api_key {
        Some(key) if form.admin_key == *key => {
            let cookie = format!("admin_token={}; Path=/; HttpOnly; SameSite=Strict", key);
            (
                StatusCode::SEE_OTHER,
                [
                    ("Location", "/admin"),
                    ("Set-Cookie", &cookie),
                ],
                "",
            ).into_response()
        }
        _ => {
            let template = AdminLoginTemplate {
                title: "Admin Login".to_string(),
                error: Some("Invalid admin key.".to_string()),
            };
            Html(template.render().unwrap_or_else(|_| "Error".to_string())).into_response()
        }
    }
}

async fn admin_logout() -> impl IntoResponse {
    (
        StatusCode::SEE_OTHER,
        [
            ("Location", "/admin"),
            ("Set-Cookie", "admin_token=; Path=/; HttpOnly; Max-Age=0"),
        ],
        "",
    )
}

async fn admin_new_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !is_authenticated(&headers, &state.config.admin_api_key) {
        return Redirect::to("/admin").into_response();
    }

    let categories = get_categories(&state).await;
    let template = AdminFormTemplate {
        title: "Add New Rule".to_string(),
        is_edit: false,
        rule: None,
        categories,
        error: None,
    };
    Html(template.render().unwrap_or_else(|_| "Error".to_string())).into_response()
}

async fn admin_create_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<RuleForm>,
) -> impl IntoResponse {
    if !is_authenticated(&headers, &state.config.admin_api_key) {
        return Redirect::to("/admin").into_response();
    }

    // Validate
    if form.title.trim().is_empty() || form.category.trim().is_empty() || form.content.trim().is_empty() || form.source.trim().is_empty() {
        let categories = get_categories(&state).await;
        let template = AdminFormTemplate {
            title: "Add New Rule".to_string(),
            is_edit: false,
            rule: Some(form_to_rule(&form)),
            categories,
            error: Some("Title, Category, Content, and Source are required.".to_string()),
        };
        return Html(template.render().unwrap_or_else(|_| "Error".to_string())).into_response();
    }

    let rule = form_to_rule(&form);
    match crate::db::upsert_rule(&state.db, &rule).await {
        Ok(_) => {
            tracing::info!("Admin created rule: {} ({})", rule.title, rule.id);
            Redirect::to("/admin").into_response()
        }
        Err(e) => {
            tracing::error!("Error creating rule: {}", e);
            let categories = get_categories(&state).await;
            let template = AdminFormTemplate {
                title: "Add New Rule".to_string(),
                is_edit: false,
                rule: Some(rule),
                categories,
                error: Some(format!("Database error: {}", e)),
            };
            Html(template.render().unwrap_or_else(|_| "Error".to_string())).into_response()
        }
    }
}

async fn admin_edit_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !is_authenticated(&headers, &state.config.admin_api_key) {
        return Redirect::to("/admin").into_response();
    }

    match crate::db::get_rule_by_id(&state.db, &id).await {
        Ok(Some(rule)) => {
            let categories = get_categories(&state).await;
            let template = AdminFormTemplate {
                title: format!("Edit: {}", rule.title),
                is_edit: true,
                rule: Some(rule),
                categories,
                error: None,
            };
            Html(template.render().unwrap_or_else(|_| "Error".to_string())).into_response()
        }
        _ => {
            Redirect::to("/admin").into_response()
        }
    }
}

async fn admin_update_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<RuleForm>,
) -> impl IntoResponse {
    if !is_authenticated(&headers, &state.config.admin_api_key) {
        return Redirect::to("/admin").into_response();
    }

    if form.title.trim().is_empty() || form.category.trim().is_empty() || form.content.trim().is_empty() || form.source.trim().is_empty() {
        let categories = get_categories(&state).await;
        let template = AdminFormTemplate {
            title: format!("Edit: {}", form.title),
            is_edit: true,
            rule: Some(form_to_rule(&form)),
            categories,
            error: Some("Title, Category, Content, and Source are required.".to_string()),
        };
        return Html(template.render().unwrap_or_else(|_| "Error".to_string())).into_response();
    }

    let mut rule = form_to_rule(&form);
    rule.id = id; // Preserve original ID

    match crate::db::upsert_rule(&state.db, &rule).await {
        Ok(_) => {
            tracing::info!("Admin updated rule: {} ({})", rule.title, rule.id);
            Redirect::to("/admin").into_response()
        }
        Err(e) => {
            tracing::error!("Error updating rule: {}", e);
            let categories = get_categories(&state).await;
            let template = AdminFormTemplate {
                title: format!("Edit: {}", rule.title),
                is_edit: true,
                rule: Some(rule),
                categories,
                error: Some(format!("Database error: {}", e)),
            };
            Html(template.render().unwrap_or_else(|_| "Error".to_string())).into_response()
        }
    }
}

async fn admin_delete_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !is_authenticated(&headers, &state.config.admin_api_key) {
        return Redirect::to("/admin").into_response();
    }

    match crate::db::delete_rule(&state.db, &id).await {
        Ok(_) => {
            tracing::info!("Admin deleted rule: {}", id);
        }
        Err(e) => {
            tracing::error!("Error deleting rule {}: {}", id, e);
        }
    }

    Redirect::to("/admin").into_response()
}

// ── Helpers ────────────────────────────────────────────────

fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

fn form_to_rule(form: &RuleForm) -> Rule {
    let id = if form.id.trim().is_empty() {
        slugify(&form.title)
    } else {
        form.id.clone()
    };

    Rule {
        id,
        title: form.title.trim().to_string(),
        category: form.category.trim().to_string(),
        subcategory: form.subcategory.as_ref().and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
        }),
        content: form.content.clone(),
        source: form.source.trim().to_string(),
        page: form.page,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

async fn get_categories(state: &AppState) -> Vec<String> {
    let rules = crate::db::get_all_rules(&state.db).await.unwrap_or_default();
    let mut categories: Vec<String> = rules.iter().map(|r| r.category.clone()).collect();
    categories.sort();
    categories.dedup();
    categories
}
