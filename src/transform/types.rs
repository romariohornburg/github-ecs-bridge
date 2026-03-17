use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Top-level ECS event document sent to Elasticsearch.
#[derive(Debug, Serialize, Deserialize)]
pub struct EcsEvent {
    #[serde(rename = "@timestamp")]
    pub timestamp: DateTime<Utc>,

    pub event: EventFields,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserFields>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<OrgFields>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceFields>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub related: Option<RelatedFields>,

    pub github: GithubFields,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<UserAgentFields>,

    pub tags: Vec<String>,

    pub data_stream: DataStreamFields,
}

/// ECS `event.*` fields.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventFields {
    pub action: String,
    pub category: Vec<String>,
    #[serde(rename = "type")]
    pub event_type: Vec<String>,
    /// Always "event" for audit logs.
    pub kind: String,
    /// Always "github.audit".
    pub dataset: String,
    /// Always "github" — required constant_keyword in the Elastic mapping.
    pub module: String,
    pub created: DateTime<Utc>,
    /// Raw event payload — required by ECS / Fleet logs-github.audit mapping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<String>,
}

/// ECS `user.*` fields — represents the actor who triggered the event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFields {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// ECS `organization.*` fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgFields {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// ECS `source.*` fields — actor IP (absent for webhook-sourced events).
#[derive(Debug, Serialize, Deserialize)]
pub struct SourceFields {
    pub ip: String,
}

/// ECS `related.*` fields.
#[derive(Debug, Serialize, Deserialize)]
pub struct RelatedFields {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ip: Vec<String>,
}

/// GitHub-specific fields nested under `github.*`.
/// All field types match the explicit Elasticsearch mapping for the github.audit dataset.
/// Fields not present in the mapping are omitted to avoid dynamic mapping conflicts.
#[derive(Debug, Serialize, Deserialize)]
pub struct GithubFields {
    /// Audit log action string (e.g. "repo.create", "git.push").
    pub action: String,
    /// Login of the user who triggered the event.
    pub actor: String,

    /// Mapped as keyword — must be String, NOT integer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,

    /// Actor IP address. Absent for webhook-sourced events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_ip: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,

    /// Mapped as keyword — must be String, NOT integer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,

    /// Repo full name, e.g. "org/repo".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,

    /// Mapped as keyword — must be String, NOT integer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_id: Option<String>,

    /// `github.repository` in the mapping — same as `repo`, kept for schema alignment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    /// Event timestamp as epoch milliseconds (matches Elastic Audit Log API output).
    pub created_at: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashed_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub programmatic_access_type: Option<String>,

    // ── Pull request fields (top-level per mapping) ──────────────────────────

    /// PR / issue number — mapped as `long`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_request_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_request_title: Option<String>,

    /// Mapped as keyword — PR node ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_request_id: Option<String>,

    /// Base branch (PR target). Maps to `github.target_branch`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_branch: Option<String>,

    /// Head branch (PR source). Maps to `github.source_branch`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_branch: Option<String>,

    // ── Repository fields ─────────────────────────────────────────────────────

    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_repo: Option<bool>,

    // ── Member / IAM fields ───────────────────────────────────────────────────

    /// User affected by the action (e.g. the member added/removed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Team name for team-related events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<String>,

    // ── Misc ──────────────────────────────────────────────────────────────────

    /// Full name of the forked-from repository (fork events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forked_repository: Option<String>,

    /// HEAD commit SHA for push/create events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>,

    // ── Workflow sub-object ───────────────────────────────────────────────────

    /// github.data — typed sub-object used only for workflow_run / workflow_job events.
    /// Field types match the explicit mapping (all keyword/date, never integer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<WorkflowData>,
}

/// Typed `github.data` sub-object for GitHub Actions workflow events.
/// Field names and types match the Elasticsearch mapping exactly.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowData {
    /// Mapped as keyword — must be String.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,

    /// Mapped as keyword — must be String.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_run_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_branch: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_sha: Option<String>,

    /// The event that triggered the workflow (e.g. "push", "pull_request").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_id: Option<String>,

    /// Mapped as date type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,

    /// Workflow display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,

    /// Workflow job name (for workflow_job events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_name: Option<String>,

    /// Run number as string for keyword-friendly filtering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_number: Option<String>,

    /// Run attempt as string for keyword-friendly filtering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_attempt: Option<String>,

    /// Current status (queued, in_progress, completed, ...).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Final conclusion (success, failure, skipped, ...).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
}

/// ECS `user_agent.*` fields.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserAgentFields {
    pub original: String,
}

/// ECS `data_stream.*` fields required for ILM-managed data streams.
#[derive(Debug, Serialize, Deserialize)]
pub struct DataStreamFields {
    #[serde(rename = "type")]
    pub ds_type: String,
    pub dataset: String,
    pub namespace: String,
}
