use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::transform::types::{DataStreamFields, GithubFields, OrgFields, UserFields};

/// Fields extracted from every GitHub webhook payload.
pub struct CommonFields {
    pub actor: String,
    /// Raw integer from sender.id — convert to String when setting github.actor_id (keyword).
    pub actor_id: Option<i64>,
    pub user: Option<UserFields>,
    pub org: Option<OrgFields>,
    pub org_name: Option<String>,
    /// Raw integer — convert to String when setting github.org_id (keyword).
    pub org_id: Option<i64>,
    pub repo: Option<String>,
    /// Raw integer — convert to String when setting github.repo_id (keyword).
    pub repo_id: Option<i64>,
}

/// Extract common fields present in every GitHub webhook payload.
pub fn extract_common(payload: &Value) -> CommonFields {
    let actor = payload
        .get("sender")
        .and_then(|s| s.get("login"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let actor_id = payload
        .get("sender")
        .and_then(|s| s.get("id"))
        .and_then(|v| v.as_i64());

    let user = Some(UserFields {
        name: actor.clone(),
        id: actor_id.map(|id| id.to_string()),
    });

    let org_name = payload
        .get("organization")
        .and_then(|o| o.get("login"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let org_id = payload
        .get("organization")
        .and_then(|o| o.get("id"))
        .and_then(|v| v.as_i64());

    let org = org_name.as_ref().map(|name| OrgFields {
        name: name.clone(),
        id: org_id.map(|id| id.to_string()),
    });

    let repo = payload
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let repo_id = payload
        .get("repository")
        .and_then(|r| r.get("id"))
        .and_then(|v| v.as_i64());

    CommonFields {
        actor,
        actor_id,
        user,
        org,
        org_name,
        org_id,
        repo,
        repo_id,
    }
}

/// Build a `GithubFields` populated with all common fields and every optional field set to `None`.
/// Transformers use struct update syntax (`..base_github_fields(...)`) to override specific fields.
pub fn base_github_fields(
    common: &CommonFields,
    action: &str,
    now: &DateTime<Utc>,
) -> GithubFields {
    GithubFields {
        action: action.to_string(),
        actor: common.actor.clone(),
        actor_id: common.actor_id.map(|id| id.to_string()),
        actor_ip: None,
        org: common.org_name.clone(),
        org_id: common.org_id.map(|id| id.to_string()),
        repo: common.repo.clone(),
        repo_id: common.repo_id.map(|id| id.to_string()),
        repository: None,
        created_at: to_epoch_millis(now),
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
        commit_id: None,
        data: None,
    }
}

/// Build the standard data_stream fields for the github.audit dataset.
pub fn make_data_stream() -> DataStreamFields {
    DataStreamFields {
        ds_type: "logs".into(),
        dataset: "github.audit".into(),
        namespace: "default".into(),
    }
}

/// Convert a DateTime<Utc> to epoch milliseconds (i64), matching the Audit Log API format.
pub fn to_epoch_millis(dt: &DateTime<Utc>) -> i64 {
    dt.timestamp_millis()
}

/// Parse a GitHub ISO 8601 timestamp string into DateTime<Utc>.
pub fn parse_gh_date(s: &str) -> Option<DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}
