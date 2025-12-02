use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;
use tracing_subscriber;

mod config;
mod routes;
mod services;
mod models;
mod handlers;

use config::Settings;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Load settings
    let settings = Settings::from_env().unwrap_or_else(|_| Settings::default());

    // Build application
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/api/proposals", routes::proposals::router())
        .nest("/api/inference", routes::inference::router())
        .nest("/api/governance", routes::governance::router())
        .nest("/api/rewards", routes::rewards::router())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], settings.api_port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "DAO Proposal System API v0.1.0"
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy"
    }))
}

