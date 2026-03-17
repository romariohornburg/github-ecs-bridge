use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

mod config;
mod debug;
mod elasticsearch;
mod transform;
mod webhook;

#[derive(Clone)]
pub struct AppState {
    pub settings: config::Settings,
    pub es_client: reqwest::Client,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,github_webhook_ingester=debug".into()),
        )
        .init();

    let settings = config::Settings::from_env().map_err(|e| {
        tracing::error!("Configuration error: {}", e);
        e.to_string()
    })?;

    let es_client = reqwest::Client::builder()
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let state = AppState {
        settings: settings.clone(),
        es_client,
    };

    let app = Router::new()
        .route("/webhook", post(webhook::handle_webhook))
        .route("/github", post(webhook::handle_webhook))
        .route("/debug/webhook", post(debug::handle_debug))
        .route("/health", get(health))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = settings
        .listen_addr
        .parse()
        .map_err(|e| format!("Invalid LISTEN_ADDR '{}': {}", settings.listen_addr, e))?;

    tracing::info!("github-webhook-ingester listening on {}", addr);
    tracing::warn!(
        "Endpoint /debug/webhook is enabled and does NOT require HMAC signature validation. Use with caution in production."
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
