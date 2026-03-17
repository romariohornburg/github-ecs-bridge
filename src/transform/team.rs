use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{extract_common, make_data_stream, to_epoch_millis};
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
            number: None,
            pull_request_url: None,
            pull_request_title: None,
            pull_request_id: None,
            target_branch: None,
            source_branch: None,
            visibility: None,
            public_repo: None,
            user_id: None,
            team: team_name,
            permission,
            forked_repository: None,
            commit_id: None,
            data: None,
        },
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
        github: GithubFields {
            action: "team.add_repository".into(),
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
            number: None,
            pull_request_url: None,
            pull_request_title: None,
            pull_request_id: None,
            target_branch: None,
            source_branch: None,
            visibility: None,
            public_repo: None,
            user_id: None,
            team: team_name,
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
