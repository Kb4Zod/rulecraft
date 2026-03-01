use axum::Router;
use std::net::SocketAddr;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod routes;
mod models;
mod db;
mod search;
mod ai;
mod middleware;

pub use config::Config;
use middleware::{RateLimitConfig, RateLimitState};

#[tokio::main]
async fn main() {
    // Initialize tracing with environment filter
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rulecraft=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::from_env();
    let port = config.port;

    // Log startup configuration (without secrets)
    tracing::info!("Starting RuleCraft server");
    tracing::info!("Port: {}", port);
    tracing::info!("Database: {}", config.database_url);
    tracing::info!("Claude API: {}", if config.claude_api_key.is_some() { "configured" } else { "not configured" });
    tracing::info!("Admin API: {}", if config.admin_api_key.is_some() { "configured" } else { "not configured" });
    tracing::info!("AI rate limit: {} requests/hour", config.ai_rate_limit_per_hour);
    tracing::info!("Search rate limit: {} requests/minute", config.search_rate_limit_per_minute);

    // Initialize database
    let db_pool = db::init_pool(&config.database_url).await
        .expect("Failed to initialize database");

    // Run migrations
    db::run_migrations(&db_pool).await
        .expect("Failed to run migrations");

    // Configure rate limiting
    let rate_limit_state = RateLimitState::new(RateLimitConfig {
        ai_requests_per_hour: config.ai_rate_limit_per_hour,
        search_requests_per_minute: config.search_rate_limit_per_minute,
        general_requests_per_minute: 120,
    });

    // Build application state
    let state = routes::AppState {
        db: db_pool,
        config,
        rate_limiter: rate_limit_state,
    };

    // Build application routes with middleware
    let app = Router::new()
        .merge(routes::router())
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
        // Request tracing/logging
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    // Start server (bind to 0.0.0.0 for Docker compatibility)
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
