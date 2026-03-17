use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{extract_common, make_data_stream, to_epoch_millis};
use crate::transform::types::{EcsEvent, EventFields, GithubFields};

/// X-GitHub-Event: create (branch or tag created)
pub fn transform_create(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let ref_name = payload.get("ref").and_then(|v| v.as_str()).map(String::from);

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: "git.create".into(),
            category: vec!["source".into()],
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
        github: GithubFields {
            action: "git.create".into(),
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
            team: None,
            permission: None,
            forked_repository: None,
            commit_id: ref_name,
            data: None,
        },
        user_agent: None,
        tags: vec!["github-webhook".into()],
        data_stream: make_data_stream(),
    }
}

/// X-GitHub-Event: delete (branch or tag deleted)
pub fn transform_delete(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let ref_name = payload.get("ref").and_then(|v| v.as_str()).map(String::from);

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: "git.delete".into(),
            category: vec!["source".into()],
            event_type: vec!["deletion".into()],
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
            action: "git.delete".into(),
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
            team: None,
            permission: None,
            forked_repository: None,
            commit_id: ref_name,
            data: None,
        },
        user_agent: None,
        tags: vec!["github-webhook".into()],
        data_stream: make_data_stream(),
    }
}
