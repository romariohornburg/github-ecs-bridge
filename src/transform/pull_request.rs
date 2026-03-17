use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{extract_common, make_data_stream, to_epoch_millis};
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
    let pull_request_url = pr.get("html_url").and_then(|v| v.as_str()).map(String::from);
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
        github: GithubFields {
            action: audit_action.into(),
            actor: common.actor,
            actor_id: common.actor_id.map(|id| id.to_string()),
            actor_ip: None,
            org: common.org_name,
            org_id: common.org_id.map(|id| id.to_string()),
            repo: common.repo,
            repo_id: common.repo_id.map(|id| id.to_string()),
            repository: None,
            created_at: to_epoch_millis(&now),
            user_agent: None,
            hashed_token: None,
            programmatic_access_type: None,
            number,
            pull_request_url,
            pull_request_title,
            pull_request_id,
            target_branch,
            source_branch,
            visibility: None,
            public_repo: None,
            user_id: None,
            team: None,
            permission: None,
            forked_repository: None,
            commit_id: None,
            data: None,
        },
        user_agent: None,
        tags: vec!["github-webhook".into()],
        data_stream: make_data_stream(),
    }
}
