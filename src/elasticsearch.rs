use reqwest::Client;

use crate::transform::types::EcsEvent;

/// Send a single ECS event to Elasticsearch via the bulk API.
///
/// Uses `POST /_bulk` with a `create` action. The Elasticsearch bulk API requires
/// NDJSON format: two newline-terminated JSON objects per document (action meta + source).
///
/// Returns `Err` on HTTP failure or per-document indexing errors.
/// Callers should `tokio::spawn` this and log errors — do not propagate to the webhook ACK.
pub async fn send_bulk(
    client: &Client,
    es_url: &str,
    index: &str,
    api_key: &str,
    event: &EcsEvent,
) -> Result<(), anyhow::Error> {
    // Action metadata line.
    // Data streams require op_type "create" (append-only); "index" is not allowed.
    let action_meta = serde_json::json!({
        "create": {
            "_index": index
        }
    });

    // Document source line.
    let doc = serde_json::to_string(event)?;
    let meta = serde_json::to_string(&action_meta)?;

    // Bulk body: two newline-separated JSON lines, terminated with a trailing newline.
    // Content-Type must be application/x-ndjson, NOT application/json.
    let bulk_body = format!("{}\n{}\n", meta, doc);

    // Disable ingest pipeline for this write: documents are already ECS-shaped.
    let url = format!("{}/_bulk?pipeline=_none", es_url);

    let response = client
        .post(&url)
        .header("Authorization", format!("ApiKey {}", api_key))
        .header("Content-Type", "application/x-ndjson")
        .body(bulk_body)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Elasticsearch bulk API returned {}: {}", status, body);
    }

    // Elasticsearch returns HTTP 200 even when individual documents fail.
    // Must check the `errors` boolean in the response body.
    let resp_json: serde_json::Value = response.json().await?;
    if resp_json
        .get("errors")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        if let Some(items) = resp_json["items"].as_array() {
            for item in items {
                if let Some(err) = item["create"].get("error") {
                    anyhow::bail!("Elasticsearch per-document error: {}", err);
                }
            }
        }
    }

    tracing::debug!("Indexed 1 github.audit event to {}", index);
    Ok(())
}
