use chrono::Utc;
use serde_json::Value;

use crate::transform::common::{
    base_github_fields, extract_common, make_data_stream, parse_gh_date,
};
use crate::transform::types::{EcsEvent, EventFields, GithubFields, WorkflowData};

/// X-GitHub-Event: workflow_run
pub fn transform_run(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let action = payload
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("completed");

    let audit_action = match action {
        "completed" => "workflows.completed_workflow_run",
        "requested" => "workflows.requested_workflow_run",
        "in_progress" => "workflows.in_progress_workflow_run",
        _ => "workflows.completed_workflow_run",
    };

    let run = payload.get("workflow_run").unwrap_or(&Value::Null);
    let workflow = payload.get("workflow").unwrap_or(&Value::Null);

    // github.data fields match the Elasticsearch mapping exactly (all keyword/date, NOT integers).
    let workflow_data = WorkflowData {
        workflow_id: run
            .get("workflow_id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string()),
        workflow_run_id: run
            .get("id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string()),
        head_branch: run
            .get("head_branch")
            .and_then(|v| v.as_str())
            .map(String::from),
        head_sha: run
            .get("head_sha")
            .and_then(|v| v.as_str())
            .map(String::from),
        event: run.get("event").and_then(|v| v.as_str()).map(String::from),
        trigger_id: None,
        started_at: run
            .get("run_started_at")
            .and_then(|v| v.as_str())
            .and_then(parse_gh_date),
        workflow_name: workflow
            .get("name")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| run.get("name").and_then(|v| v.as_str()).map(String::from)),
        job_name: None,
        run_number: run
            .get("run_number")
            .and_then(|v| v.as_i64())
            .map(|n| n.to_string()),
        run_attempt: run
            .get("run_attempt")
            .and_then(|v| v.as_i64())
            .map(|n| n.to_string()),
        status: run.get("status").and_then(|v| v.as_str()).map(String::from),
        conclusion: run
            .get("conclusion")
            .and_then(|v| v.as_str())
            .map(String::from),
    };

    let github = GithubFields {
        data: Some(workflow_data),
        ..base_github_fields(&common, audit_action, &now)
    };

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: audit_action.into(),
            category: vec!["configuration".into()],
            event_type: vec!["info".into()],
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

/// X-GitHub-Event: workflow_job
pub fn transform_job(payload: &Value) -> EcsEvent {
    let common = extract_common(payload);
    let now = Utc::now();

    let action = payload
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("completed");

    let audit_action = match action {
        "completed" => "workflows.completed_workflow_job",
        "queued" => "workflows.queued_workflow_job",
        "in_progress" => "workflows.in_progress_workflow_job",
        _ => "workflows.completed_workflow_job",
    };

    let job = payload.get("workflow_job").unwrap_or(&Value::Null);

    let workflow_data = WorkflowData {
        workflow_id: None,
        workflow_run_id: job
            .get("run_id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string()),
        head_branch: job
            .get("head_branch")
            .and_then(|v| v.as_str())
            .map(String::from),
        head_sha: job
            .get("head_sha")
            .and_then(|v| v.as_str())
            .map(String::from),
        event: None,
        trigger_id: None,
        started_at: job
            .get("started_at")
            .and_then(|v| v.as_str())
            .and_then(parse_gh_date),
        workflow_name: job
            .get("workflow_name")
            .and_then(|v| v.as_str())
            .map(String::from),
        job_name: job.get("name").and_then(|v| v.as_str()).map(String::from),
        run_number: None,
        run_attempt: job
            .get("run_attempt")
            .and_then(|v| v.as_i64())
            .map(|n| n.to_string()),
        status: job.get("status").and_then(|v| v.as_str()).map(String::from),
        conclusion: job
            .get("conclusion")
            .and_then(|v| v.as_str())
            .map(String::from),
    };

    let github = GithubFields {
        data: Some(workflow_data),
        ..base_github_fields(&common, audit_action, &now)
    };

    EcsEvent {
        timestamp: now,
        event: EventFields {
            action: audit_action.into(),
            category: vec!["configuration".into()],
            event_type: vec!["info".into()],
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
