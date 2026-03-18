use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{base_github_fields, extract_common, make_data_stream};
use crate::transform::types::{EcsEvent, EventFields, GithubFields};

pub fn transform(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");
    let merged = payload
        .get("pull_request")
        .and_then(|pr| pr.get("merged"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let (audit_action, ecs_type) = match action {
        "opened" => ("pull_request.create", "creation"),
        "closed" if merged => ("pull_request.merge", "change"),
        "closed" => ("pull_request.close", "deletion"),
        "synchronize" => ("pull_request.synchronize", "change"),
        "reopened" => ("pull_request.reopen", "change"),
        _ => ("pull_request.update", "change"),
    };

    let pr = payload.get("pull_request").unwrap_or(&Value::Null);

    // Map to top-level github.* fields per Elasticsearch mapping.
    let number = pr.get("number").and_then(|v| v.as_i64());
    let pull_request_url = pr
        .get("html_url")
        .and_then(|v| v.as_str())
        .map(String::from);
    let pull_request_title = pr.get("title").and_then(|v| v.as_str()).map(String::from);
    let pull_request_id = pr.get("node_id").and_then(|v| v.as_str()).map(String::from);
    let target_branch = pr
        .get("base")
        .and_then(|b| b.get("ref"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let source_branch = pr
        .get("head")
        .and_then(|h| h.get("ref"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let github = GithubFields {
        number,
        pull_request_url,
        pull_request_title,
        pull_request_id,
        target_branch,
        source_branch,
        ..base_github_fields(&common, audit_action, &now)
    };

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: audit_action.into(),
            category: vec!["source".into()],
            event_type: vec![ecs_type.into()],
            kind: "event".into(),
            dataset: "github.audit".into(),
            module: "github".into(),
            created: now,
            original: None,
        },
        user: common.user,
        organization: common.org,
        source: None,
        related: None,
        github,
        user_agent: None,
        tags: vec!["github-webhook".into()],
        data_stream: make_data_stream(),
    }
}
