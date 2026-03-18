use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{base_github_fields, extract_common, make_data_stream};
use crate::transform::types::{EcsEvent, EventFields, GithubFields};

pub fn transform(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");

    let (audit_action, category, ecs_type) = match action {
        "created" => ("team.create", "iam", "creation"),
        "deleted" => ("team.destroy", "iam", "deletion"),
        "edited" => ("team.update", "iam", "change"),
        "added_to_repository" => ("team.add_repository", "configuration", "change"),
        "removed_from_repository" => ("team.remove_repository", "configuration", "change"),
        _ => ("team.update", "iam", "change"),
    };

    let gh_team = payload.get("team").unwrap_or(&Value::Null);
    let team_name = gh_team
        .get("name")
        .and_then(|v| v.as_str())
        .map(String::from);
    let permission = gh_team
        .get("permission")
        .and_then(|v| v.as_str())
        .map(String::from);

    let github = GithubFields {
        team: team_name,
        permission,
        ..base_github_fields(&common, audit_action, &now)
    };

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

/// X-GitHub-Event: team_add (legacy event — always maps to team.add_repository)
pub fn transform_team_add(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let gh_team = payload.get("team").unwrap_or(&Value::Null);
    let team_name = gh_team
        .get("name")
        .and_then(|v| v.as_str())
        .map(String::from);

    let github = GithubFields {
        team: team_name,
        ..base_github_fields(&common, "team.add_repository", &now)
    };

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: "team.add_repository".into(),
            category: vec!["configuration".into()],
            event_type: vec!["change".into()],
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
