use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{base_github_fields, extract_common, make_data_stream};
use crate::transform::types::{EcsEvent, EventFields, GithubFields};

pub fn transform(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");

    let (audit_action, ecs_type) = match action {
        "created" => ("repo.create", "creation"),
        "deleted" => ("repo.destroy", "deletion"),
        "publicized" => ("repo.publicize", "change"),
        "privatized" => ("repo.privatize", "change"),
        "renamed" => ("repo.rename", "change"),
        "transferred" => ("repo.transfer", "change"),
        "archived" => ("repo.archived", "change"),
        "unarchived" => ("repo.unarchived", "change"),
        "edited" => ("repo.update", "change"),
        _ => ("repo.update", "change"),
    };

    let repo = payload.get("repository").unwrap_or(&Value::Null);

    let is_private = repo
        .get("private")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let visibility = repo
        .get("visibility")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            Some(if is_private {
                "private".into()
            } else {
                "public".into()
            })
        });

    let github = GithubFields {
        visibility,
        public_repo: Some(!is_private),
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
