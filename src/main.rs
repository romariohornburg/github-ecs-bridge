use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use tower_http::trace::TraceLayer;

mod config;
mod debug;
mod elasticsearch;
mod transform;
mod webhook;

/// Maximum number of concurrent in-flight Elasticsearch send tasks.
const MAX_CONCURRENT_ES_SENDS: usize = 64;
/// Maximum accepted request body size (10 MiB).
const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

#[derive(Clone)]
pub struct AppState {
    pub settings: config::Settings,
    pub es_client: reqwest::Client,
    pub es_semaphore: Arc<Semaphore>,
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
        .timeout(Duration::from_secs(settings.es_timeout_secs))
        .pool_max_idle_per_host(8)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let state = AppState {
        settings: settings.clone(),
        es_client,
        es_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_ES_SENDS)),
    };

    let mut app = Router::new()
        .route("/webhook", post(webhook::handle_webhook))
        .route("/github", post(webhook::handle_webhook))
        .route("/health", get(health));

    if settings.debug_endpoint_enabled {
        tracing::warn!(
            "Endpoint /debug/webhook is enabled and does NOT require HMAC signature validation. \
             Never expose this to the internet."
        );
        app = app.route("/debug/webhook", post(debug::handle_debug));
    }

    let app = app
        .with_state(state)
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = settings
        .listen_addr
        .parse()
        .map_err(|e| format!("Invalid LISTEN_ADDR '{}': {}", settings.listen_addr, e))?;

    tracing::info!("github-webhook-ingester listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
