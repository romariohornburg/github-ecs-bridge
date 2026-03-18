use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{base_github_fields, extract_common, make_data_stream};
use crate::transform::types::{EcsEvent, EventFields};

/// Build an EcsEvent for simple events with a fixed mapping.
pub fn transform_simple(
    payload: &Value,
    audit_action: &str,
    category: &str,
    ecs_type: &str,
) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();
    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: audit_action.into(),
            category: vec![category.into()],
            event_type: vec![ecs_type.into()],
            kind: "event".into(),
            dataset: "github.audit".into(),
            module: "github".into(),
            created: now,
            original: None,
        },
        user: common.user.clone(),
        organization: common.org.clone(),
        source: None,
        related: None,
        github: base_github_fields(&common, audit_action, &now),
        user_agent: None,
        tags: vec!["github-webhook".into()],
        data_stream: make_data_stream(),
    }
}

/// X-GitHub-Event: fork — uses github.forked_repository (top-level per mapping).
pub fn transform_fork(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();
    let forkee_full_name = payload
        .get("forkee")
        .and_then(|f| f.get("full_name"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let mut github = base_github_fields(&common, "repo.create", &now);
    github.forked_repository = forkee_full_name;

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: "repo.create".into(),
            category: vec!["configuration".into()],
            event_type: vec!["creation".into()],
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

/// Events with action-based mapping (deploy_key, secret_scanning_alert, code_scanning_alert).
pub fn transform_with_action(payload: &Value, event_type: &str) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");

    let (audit_action, category, ecs_type) = match (event_type, action) {
        ("deploy_key", "created") => ("deploy_key.create", "configuration", "creation"),
        ("deploy_key", "deleted") => ("deploy_key.destroy", "configuration", "deletion"),
        ("secret_scanning_alert", "created") => {
            ("secret_scanning.alert_create", "configuration", "creation")
        }
        ("secret_scanning_alert", "resolved") => {
            ("secret_scanning.alert_resolve", "configuration", "change")
        }
        ("code_scanning_alert", "created") => {
            ("code_scanning.alert_create", "configuration", "creation")
        }
        ("code_scanning_alert", "closed") => {
            ("code_scanning.alert_close", "configuration", "change")
        }
        _ => {
            let a = if action.is_empty() {
                event_type.to_string()
            } else {
                format!("{}.{}", event_type, action)
            };
            return transform_unknown(payload, &a);
        }
    };

    let user = common.user.clone();
    let org = common.org.clone();

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: audit_action.into(),
            category: vec![category.into()],
            event_type: vec![ecs_type.into()],
            kind: "event".into(),
            dataset: "github.audit".into(),
            module: "github".into(),
            created: now,
            original: None,
        },
        user,
        organization: org,
        source: None,
        related: None,
        github: base_github_fields(&common, audit_action, &now),
        user_agent: None,
        tags: vec!["github-webhook".into()],
        data_stream: make_data_stream(),
    }
}

/// Fallback for completely unrecognized webhook events.
/// Produces a valid ECS document with a best-effort action string.
pub fn transform_unknown(payload: &Value, event_type: &str) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");
    let audit_action = if action.is_empty() {
        event_type.to_string()
    } else {
        format!("{}.{}", event_type, action)
    };

    let user = common.user.clone();
    let org = common.org.clone();

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: audit_action.clone(),
            category: vec!["configuration".into()],
            event_type: vec!["info".into()],
            kind: "event".into(),
            dataset: "github.audit".into(),
            module: "github".into(),
            created: now,
            original: None,
        },
        user,
        organization: org,
        source: None,
        related: None,
        github: base_github_fields(&common, &audit_action, &now),
        user_agent: None,
        tags: vec!["github-webhook".into()],
        data_stream: make_data_stream(),
    }
}
