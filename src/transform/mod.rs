pub mod common;
pub mod types;

mod create_delete;
mod generic;
mod issues;
mod member;
mod organization;
mod pull_request;
mod push;
mod release;
mod repository;
mod team;
mod workflow;

use serde_json::Value;
use types::EcsEvent;

const EVENT_ORIGINAL_MAX_BYTES: usize = 8192;

fn truncate_utf8(input: &str, max_bytes: usize) -> String {
    if input.len() <= max_bytes {
        return input.to_string();
    }

    let mut end = max_bytes;
    while end > 0 && !input.is_char_boundary(end) {
        end -= 1;
    }
    input[..end].to_string()
}

/// Main dispatch function. Routes webhook events to the correct transformer.
/// `event_type` is the value of the X-GitHub-Event header.
/// `payload` is the parsed JSON webhook body.
pub fn transform(event_type: &str, payload: &Value) -> EcsEvent {
    let mut ecs = match event_type {
        "push" => push::transform(payload),
        "pull_request" => pull_request::transform(payload),
        "issues" => issues::transform(payload),
        "repository" => repository::transform(payload),
        "member" => member::transform(payload),
        "organization" => organization::transform(payload),
        "team" => team::transform(payload),
        "team_add" => team::transform_team_add(payload),
        "release" => release::transform(payload),
        "workflow_run" => workflow::transform_run(payload),
        "workflow_job" => workflow::transform_job(payload),
        "create" => create_delete::transform_create(payload),
        "delete" => create_delete::transform_delete(payload),
        "deploy_key" => generic::transform_with_action(payload, event_type),
        "secret_scanning_alert" => generic::transform_with_action(payload, event_type),
        "code_scanning_alert" => generic::transform_with_action(payload, event_type),
        "fork" => generic::transform_fork(payload),
        "gollum" => generic::transform_simple(payload, "wiki.gollum", "configuration", "change"),
        "public" => generic::transform_simple(payload, "repo.publicize", "configuration", "change"),
        "watch" => generic::transform_simple(payload, "repo.watch", "configuration", "info"),
        "star" => generic::transform_simple(payload, "repo.star", "configuration", "creation"),
        _ => generic::transform_unknown(payload, event_type),
    };
    // Keep raw payload for audit trail, but cap size to avoid oversized docs.
    let original = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());
    ecs.event.original = Some(truncate_utf8(&original, EVENT_ORIGINAL_MAX_BYTES));
    ecs
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transform_push() {
        let payload = json!({
            "ref": "refs/heads/main",
            "head_commit": {"id": "abc1234567890"},
            "repository": {"id": 1, "full_name": "org/repo"},
            "organization": {"login": "org", "id": 99},
            "sender": {"login": "user", "id": 42}
        });

        let ecs = transform("push", &payload);
        assert_eq!(ecs.event.action, "git.push");
        assert_eq!(ecs.github.actor, "user");
        assert_eq!(ecs.github.repo, Some("org/repo".to_string()));
        assert_eq!(ecs.github.commit_id, Some("abc1234567890".to_string()));
    }

    #[test]
    fn test_transform_pull_request_opened() {
        let payload = json!({
            "action": "opened",
            "pull_request": {
                "number": 10,
                "title": "Fix something",
                "html_url": "https://github.com/org/repo/pull/10",
                "base": {"ref": "main"},
                "head": {"ref": "feature"}
            },
            "repository": {"id": 1, "full_name": "org/repo"},
            "organization": {"login": "org", "id": 99},
            "sender": {"login": "user", "id": 42}
        });

        let ecs = transform("pull_request", &payload);
        assert_eq!(ecs.event.action, "pull_request.create");
        assert_eq!(ecs.github.number, Some(10));
        assert_eq!(ecs.github.target_branch, Some("main".to_string()));
        assert_eq!(ecs.github.source_branch, Some("feature".to_string()));
    }

    #[test]
    fn test_transform_unknown_event() {
        let payload = json!({
            "action": "something",
            "sender": {"login": "user", "id": 42}
        });

        let ecs = transform("unknown_event", &payload);
        assert_eq!(ecs.event.action, "unknown_event.something");
        assert_eq!(ecs.github.actor, "user");
    }

    #[test]
    fn test_truncate_utf8() {
        let s = "Hello, 🌍!"; // 🌍 is 4 bytes
        assert_eq!(truncate_utf8(s, 7), "Hello, ");
        assert_eq!(truncate_utf8(s, 8), "Hello, "); // Cannot cut in middle of 🌍
        assert_eq!(truncate_utf8(s, 11), "Hello, 🌍");
        assert_eq!(truncate_utf8(s, 12), "Hello, 🌍!");
    }
}
