use axum::{
    Json,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;

use crate::AppState;

type HmacSha256 = Hmac<Sha256>;

/// POST /webhook — GitHub webhook receiver.
///
/// Validates HMAC-SHA256 signature, dispatches to the transform module,
/// and forwards the ECS event to Elasticsearch asynchronously.
/// Returns 200 immediately (before ES write completes) so GitHub's 10s timeout is never hit.
pub async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Extract and validate X-Hub-Signature-256 header.
    let sig_hex = match headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("sha256="))
    {
        Some(s) => s.to_string(),
        None => {
            tracing::warn!("Rejected webhook: missing X-Hub-Signature-256 header");
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody::new("Missing X-Hub-Signature-256 header")),
            )
                .into_response();
        }
    };

    if !verify_signature(state.settings.webhook_secret.as_bytes(), &body, &sig_hex) {
        tracing::warn!("Rejected webhook: signature mismatch");
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorBody::new("Invalid signature")),
        )
            .into_response();
    }

    // Extract X-GitHub-Event header.
    let event_type = match headers.get("X-GitHub-Event").and_then(|v| v.to_str().ok()) {
        Some(e) => e.to_string(),
        None => {
            tracing::warn!("Rejected webhook: missing X-GitHub-Event header");
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody::new("Missing X-GitHub-Event header")),
            )
                .into_response();
        }
    };

    // Parse JSON body.
    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Rejected webhook: invalid JSON body: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody::new("Invalid JSON")),
            )
                .into_response();
        }
    };

    tracing::debug!(event_type = %event_type, "Received GitHub webhook");

    // Transform to ECS format.
    let ecs_event = crate::transform::transform(&event_type, &payload);

    // Forward to Elasticsearch asynchronously — fire and forget.
    let client = state.es_client.clone();
    let es_url = state.settings.elasticsearch_url.clone();
    let index = state.settings.elasticsearch_index.clone();
    let api_key = state.settings.elasticsearch_api_key.clone();
    tokio::spawn(async move {
        if let Err(e) =
            crate::elasticsearch::send_bulk(&client, &es_url, &index, &api_key, &ecs_event).await
        {
            tracing::error!(
                "Elasticsearch send failed for event '{}': {}",
                event_type,
                e
            );
        }
    });

    StatusCode::OK.into_response()
}

/// Verify the HMAC-SHA256 signature from GitHub.
///
/// Uses `hmac::Mac::verify_slice` which performs constant-time comparison,
/// preventing timing side-channel attacks.
fn verify_signature(secret: &[u8], body: &[u8], expected_hex: &str) -> bool {
    let expected_bytes = match hex::decode(expected_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let mut mac = match HmacSha256::new_from_slice(secret) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);

    mac.verify_slice(&expected_bytes).is_ok()
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ErrorBody {
    fn new(msg: &str) -> Self {
        Self {
            error: msg.to_string(),
        }
    }
}
