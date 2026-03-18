use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{base_github_fields, extract_common, make_data_stream};
use crate::transform::types::{EcsEvent, EventFields, GithubFields};

pub fn transform(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let head_commit_id = payload
        .get("head_commit")
        .and_then(|c| c.get("id"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let github = GithubFields {
        commit_id: head_commit_id,
        ..base_github_fields(&common, "git.push", &now)
    };

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
        github,
        user_agent: None,
        tags: vec!["github-webhook".into()],
        data_stream: make_data_stream(),
    }
}
