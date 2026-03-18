use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{base_github_fields, extract_common, make_data_stream};
use crate::transform::types::{EcsEvent, EventFields, GithubFields};

pub fn transform(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");

    let (audit_action, ecs_type) = match action {
        "published" | "released" | "created" => ("repo.create_tag", "creation"),
        "deleted" => ("repo.destroy_tag", "deletion"),
        "edited" => ("repo.update_tag", "change"),
        _ => ("repo.create_tag", "creation"),
    };

    let release = payload.get("release").unwrap_or(&Value::Null);
    // Use the tag name as commit_id for alignment with the mapped field.
    let tag_name = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .map(String::from);

    let github = GithubFields {
        commit_id: tag_name,
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
