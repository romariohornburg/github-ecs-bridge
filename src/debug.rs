use axum::{
    Json,
    body::Bytes,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;

/// POST /debug/webhook
///
/// Recebe o mesmo payload e headers de um webhook real, mas em vez de
/// encaminhar ao Elasticsearch, retorna o documento ECS gerado como JSON.
/// Não valida a assinatura HMAC — apenas para uso local.
pub async fn handle_debug(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    let event_type = match headers.get("X-GitHub-Event").and_then(|v| v.to_str().ok()) {
        Some(e) => e.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(DebugError {
                    error: "Missing X-GitHub-Event header".into(),
                    ecs: None,
                }),
            )
                .into_response();
        }
    };

    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(DebugError {
                    error: format!("Invalid JSON: {}", e),
                    ecs: None,
                }),
            )
                .into_response();
        }
    };

    let ecs_event = crate::transform::transform(&event_type, &payload);
    let ecs_value = serde_json::to_value(&ecs_event).unwrap_or(serde_json::Value::Null);

    Json(DebugResponse {
        event_type,
        ecs: ecs_value,
    })
    .into_response()
}

#[derive(Serialize)]
struct DebugResponse {
    event_type: String,
    ecs: serde_json::Value,
}

#[derive(Serialize)]
struct DebugError {
    error: String,
    ecs: Option<serde_json::Value>,
}
