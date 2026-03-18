use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{base_github_fields, extract_common, make_data_stream};
use crate::transform::types::{EcsEvent, EventFields, GithubFields};

pub fn transform(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");

    let (audit_action, ecs_type) = match action {
        "opened" => ("issue.create", "creation"),
        "closed" => ("issue.close", "change"),
        "deleted" => ("issue.destroy", "deletion"),
        "edited" => ("issue.update", "change"),
        "reopened" => ("issue.reopen", "change"),
        "labeled" | "unlabeled" => ("issue.update", "change"),
        "assigned" | "unassigned" => ("issue.update", "change"),
        _ => ("issue.update", "change"),
    };

    let issue = payload.get("issue").unwrap_or(&Value::Null);
    let number = issue.get("number").and_then(|v| v.as_i64());

    let github = GithubFields {
        number,
        ..base_github_fields(&common, audit_action, &now)
    };

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: audit_action.into(),
            category: vec!["configuration".into()],
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
