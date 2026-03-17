use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{extract_common, make_data_stream, to_epoch_millis};
use crate::transform::types::{EcsEvent, EventFields, GithubFields};

pub fn transform(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let head_commit_id = payload
        .get("head_commit")
        .and_then(|c| c.get("id"))
        .and_then(|v| v.as_str())
        .map(String::from);

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: "git.push".into(),
            category: vec!["source".into()],
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
            action: "git.push".into(),
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
            commit_id: head_commit_id,
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
            data: None,
        },
        user_agent: None,
        tags: vec!["github-webhook".into()],
        data_stream: make_data_stream(),
    }
}
