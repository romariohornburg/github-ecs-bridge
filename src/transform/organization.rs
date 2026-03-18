use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{base_github_fields, extract_common, make_data_stream};
use crate::transform::types::{EcsEvent, EventFields, GithubFields};

pub fn transform(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");

    let (audit_action, ecs_type) = match action {
        "member_added" => ("org.add_member", "user"),
        "member_removed" => ("org.remove_member", "deletion"),
        "member_invited" => ("org.invite_member", "user"),
        "renamed" => ("org.rename", "change"),
        "deleted" => ("org.destroy", "deletion"),
        _ => ("org.update", "change"),
    };

    // Affected user — from membership.user or invitee depending on event action.
    let membership = payload.get("membership").unwrap_or(&Value::Null);
    let invitee = payload.get("invitee").unwrap_or(&Value::Null);

    let affected_id = membership
        .get("user")
        .and_then(|u| u.get("id"))
        .and_then(|v| v.as_i64())
        .or_else(|| invitee.get("id").and_then(|v| v.as_i64()))
        .map(|id| id.to_string());

    let role = membership
        .get("role")
        .and_then(|v| v.as_str())
        .map(String::from);

    let github = GithubFields {
        repo: None,
        repo_id: None,
        user_id: affected_id,
        permission: role,
        ..base_github_fields(&common, audit_action, &now)
    };

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: audit_action.into(),
            category: vec!["iam".into()],
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
