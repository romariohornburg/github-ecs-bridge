use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{base_github_fields, extract_common, make_data_stream};
use crate::transform::types::{EcsEvent, EventFields, GithubFields};

/// X-GitHub-Event: create (branch or tag created)
pub fn transform_create(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let ref_name = payload
        .get("ref")
        .and_then(|v| v.as_str())
        .map(String::from);

    let github = GithubFields {
        commit_id: ref_name,
        ..base_github_fields(&common, "git.create", &now)
    };

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
        github,
        user_agent: None,
        tags: vec!["github-webhook".into()],
        data_stream: make_data_stream(),
    }
}

/// X-GitHub-Event: delete (branch or tag deleted)
pub fn transform_delete(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let ref_name = payload
        .get("ref")
        .and_then(|v| v.as_str())
        .map(String::from);

    let github = GithubFields {
        commit_id: ref_name,
        ..base_github_fields(&common, "git.delete", &now)
    };

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
        github,
        user_agent: None,
        tags: vec!["github-webhook".into()],
        data_stream: make_data_stream(),
    }
}
