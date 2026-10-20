//! `tachyon ops erp-replays`: a thin client for the audited ERP delivery
//! replay API (`/v1/payment/erp-delivery-replays`, PLT-5518 / PLT-5690).
//!
//! A replay is requested as a dry-run, approved by a second person and
//! executed by someone other than the approver. The server enforces who
//! may do what; this client never works around it. It only fixes the
//! operator header to the Host tenant and refuses API-key auth up front,
//! since the server answers 403 to anything but a human login session.

use std::collections::BTreeMap;

use anyhow::{bail, Result};
use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};

/// Production `SYSTEM_TENANT_ID` (the Host tenant).
pub const DEFAULT_HOST_TENANT_ID: &str = "tn_01jcjtqxah6mhyw4e5mahg02nd";

const BASE_PATH: &str = "/v1/payment/erp-delivery-replays";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayOutbox {
    /// `erp_usage_outbox` (record IDs are `erp_usage_outbox.record_id`)
    Usage,
    /// `erp_accounting_events` (record IDs are `source_id`)
    Accounting,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ErpReplaysCommand {
    /// Dry-run: build the exact bodies of delivered records and store them
    /// for approval. Nothing is sent to Field.
    Request {
        #[arg(long, value_enum)]
        outbox: ReplayOutbox,
        /// Stable record ID to replay, in send order (repeat, 1 to 50)
        #[arg(long = "record-id", required = true)]
        record_ids: Vec<String>,
        #[arg(long)]
        reason: String,
        /// Approval reference, e.g. `PLT-4603 A6b: <link>`
        #[arg(long)]
        change_reference: String,
        #[command(flatten)]
        target: HostTenant,
        #[arg(long)]
        json: bool,
    },
    /// Approve a dry-run's target digest. Must not be the requester.
    Approve {
        /// Replay ID (`edr_...`)
        id: String,
        /// The `targetDigest` reviewed in the dry-run
        #[arg(long)]
        target_digest: String,
        #[command(flatten)]
        target: HostTenant,
        #[arg(long)]
        json: bool,
    },
    /// Send an approved replay. Must not be the approver.
    Execute {
        /// Replay ID (`edr_...`)
        id: String,
        /// The approved `targetDigest`
        #[arg(long)]
        target_digest: String,
        #[command(flatten)]
        target: HostTenant,
        #[arg(long)]
        json: bool,
    },
    /// Revoke a replay that is pending approval or approved.
    Revoke {
        /// Replay ID (`edr_...`)
        id: String,
        #[arg(long)]
        reason: String,
        #[command(flatten)]
        target: HostTenant,
        #[arg(long)]
        json: bool,
    },
    /// Show a replay with its items and attempts.
    Get {
        /// Replay ID (`edr_...`)
        id: String,
        /// Also print each item's exact request body
        #[arg(long)]
        show_body: bool,
        #[command(flatten)]
        target: HostTenant,
        #[arg(long)]
        json: bool,
    },
}

/// The replay API only answers for the Host tenant, so the operator
/// header is not taken from `--tenant-id` or the profile.
#[derive(Debug, Clone, clap::Args)]
pub struct HostTenant {
    /// Host tenant ID sent as `x-operator-id`
    #[arg(
        long,
        env = "TACHYON_HOST_TENANT_ID",
        default_value = DEFAULT_HOST_TENANT_ID
    )]
    pub host_tenant_id: String,
}

impl ErpReplaysCommand {
    fn host_tenant_id(&self) -> &str {
        match self {
            Self::Request { target, .. }
            | Self::Approve { target, .. }
            | Self::Execute { target, .. }
            | Self::Revoke { target, .. }
            | Self::Get { target, .. } => &target.host_tenant_id,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateRequest<'a> {
    outbox: ReplayOutbox,
    record_ids: &'a [String],
    reason: &'a str,
    change_reference: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DigestRequest<'a> {
    target_digest: &'a str,
}

#[derive(Debug, Serialize)]
struct RevokeRequest<'a> {
    reason: &'a str,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayView {
    pub id: String,
    pub outbox: String,
    pub status: String,
    pub target_digest: String,
    pub item_count: u32,
    pub reason: String,
    pub change_reference: String,
    pub requested_by: String,
    pub requested_at: String,
    pub expires_at: String,
    pub expired: bool,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub executed_by: Option<String>,
    pub execution_started_at: Option<String>,
    pub execution_finished_at: Option<String>,
    pub revoked_by: Option<String>,
    pub revoked_at: Option<String>,
    pub revoke_reason: Option<String>,
    #[serde(default)]
    pub items: Vec<ReplayItem>,
    #[serde(default)]
    pub attempts: Vec<ReplayAttempt>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayItem {
    pub position: u32,
    pub record_id: String,
    pub payload_body: String,
    pub payload_sha256: String,
    pub source_tenant_id: Option<String>,
    pub billing_account_id: Option<String>,
    pub original_delivered_at: Option<String>,
    pub original_remote_entry_id: Option<String>,
    pub grant_change_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayAttempt {
    pub id: String,
    pub record_id: String,
    pub payload_sha256: String,
    pub approved_by: String,
    pub executed_by: String,
    pub outcome: String,
    pub field_status: Option<String>,
    pub http_status: Option<u16>,
    pub remote_entry_id: Option<String>,
    pub remote_entry_id_matches: Option<bool>,
    pub grant_change_id: Option<String>,
    pub detail: Option<String>,
    pub sent_at: Option<String>,
    pub finished_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionView {
    pub request: ReplayView,
    pub attempts: Vec<ReplayAttempt>,
    pub outcome_counts: BTreeMap<String, usize>,
    pub sent_count: usize,
    pub refused_count: usize,
    pub all_duplicate: bool,
}

pub async fn run(
    command: &ErpReplaysCommand,
    config: &Configuration,
    uses_api_key: bool,
) -> Result<()> {
    if uses_api_key {
        bail!(
            "ERP delivery replays need a human login session; the server \
             refuses API keys and service accounts (403). Unset \
             TACHYON_API_KEY / --api-key and run `tachyon auth login`."
        );
    }
    let api = ApiClient::new(config, command.host_tenant_id())?;

    match command {
        ErpReplaysCommand::Request {
            outbox,
            record_ids,
            reason,
            change_reference,
            json,
            ..
        } => {
            // Not idempotent: a 401 retry could create a second request.
            let view: ReplayView = api
                .post_once(
                    BASE_PATH,
                    &CreateRequest {
                        outbox: *outbox,
                        record_ids,
                        reason,
                        change_reference,
                    },
                )
                .await?;
            output_view(&view, *json, true)
        }
        ErpReplaysCommand::Approve {
            id,
            target_digest,
            json,
            ..
        } => {
            let view: ReplayView = api
                .post_once(
                    &format!("{BASE_PATH}/{}/approve", segment(id)?),
                    &DigestRequest { target_digest },
                )
                .await?;
            output_view(&view, *json, false)
        }
        ErpReplaysCommand::Execute {
            id,
            target_digest,
            json,
            ..
        } => {
            // Sent once: a resend after a lost response is refused (409);
            // read the replay with `get` instead.
            let execution: ExecutionView = api
                .post_once(
                    &format!("{BASE_PATH}/{}/execute", segment(id)?),
                    &DigestRequest { target_digest },
                )
                .await?;
            if *json {
                return print_json(&execution);
            }
            print!("{}", render_execution(&execution));
            Ok(())
        }
        ErpReplaysCommand::Revoke {
            id, reason, json, ..
        } => {
            let view: ReplayView = api
                .post_once(
                    &format!("{BASE_PATH}/{}/revoke", segment(id)?),
                    &RevokeRequest { reason },
                )
                .await?;
            output_view(&view, *json, false)
        }
        ErpReplaysCommand::Get {
            id,
            show_body,
            json,
            ..
        } => {
            let view: Value = api.get(&format!("{BASE_PATH}/{}", segment(id)?)).await?;
            if *json {
                return print_json(&view);
            }
            let view: ReplayView = serde_json::from_value(view)?;
            output_view(&view, false, *show_body)
        }
    }
}

/// Replay IDs go into the path; refuse anything that could leave it.
fn segment(id: &str) -> Result<&str> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    if !valid {
        bail!("invalid replay ID: {id:?}");
    }
    Ok(id)
}

fn output_view(view: &ReplayView, json: bool, show_body: bool) -> Result<()> {
    if json {
        return print_json(view);
    }
    print!("{}", render_view(view, show_body));
    Ok(())
}

fn render_view(view: &ReplayView, show_body: bool) -> String {
    let mut out = String::new();
    let mut line = |label: &str, value: &str| {
        out.push_str(&format!("{label:<17}{value}\n"));
    };
    line("ID:", &view.id);
    line("Outbox:", &view.outbox);
    let status = if view.expired {
        format!("{} (expired)", view.status)
    } else {
        view.status.clone()
    };
    line("Status:", &status);
    line("Target digest:", &view.target_digest);
    line("Reason:", &view.reason);
    line("Change ref:", &view.change_reference);
    line(
        "Requested:",
        &format!("{} by {}", view.requested_at, view.requested_by),
    );
    line("Expires:", &view.expires_at);
    if let (Some(by), Some(at)) = (&view.approved_by, &view.approved_at) {
        line("Approved:", &format!("{at} by {by}"));
    }
    if let Some(by) = &view.executed_by {
        let started = view.execution_started_at.as_deref().unwrap_or("-");
        let finished = view.execution_finished_at.as_deref().unwrap_or("-");
        line("Executed:", &format!("{started} .. {finished} by {by}"));
    }
    if let (Some(by), Some(at)) = (&view.revoked_by, &view.revoked_at) {
        let reason = view.revoke_reason.as_deref().unwrap_or("");
        line("Revoked:", &format!("{at} by {by}: {reason}"));
    }

    out.push_str(&format!("\nItems ({}):\n", view.item_count));
    for item in &view.items {
        out.push_str(&format!(
            "  #{:<3} {}\n       sha256 {}\n",
            item.position, item.record_id, item.payload_sha256
        ));
        let mut scope = Vec::new();
        if let Some(tenant) = &item.source_tenant_id {
            scope.push(format!("tenant {tenant}"));
        }
        if let Some(ba) = &item.billing_account_id {
            scope.push(format!("BA {ba}"));
        }
        if let Some(grant) = &item.grant_change_id {
            scope.push(format!("grant {grant}"));
        }
        if let Some(at) = &item.original_delivered_at {
            scope.push(format!("delivered {at}"));
        }
        if !scope.is_empty() {
            out.push_str(&format!("       {}\n", scope.join(", ")));
        }
        if show_body {
            out.push_str(&format!(
                "       body {}\n",
                pretty_body(&item.payload_body)
            ));
        }
    }

    if !view.attempts.is_empty() {
        out.push_str(&format!("\nAttempts ({}):\n", view.attempts.len()));
        out.push_str(&render_attempts(&view.attempts));
    }
    out
}

/// The body is shown as sent; pretty-printing is only for reading.
fn pretty_body(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| serde_json::to_string_pretty(&value).ok())
        .map(|pretty| pretty.replace('\n', "\n       "))
        .unwrap_or_else(|| body.to_string())
}

fn render_attempts(attempts: &[ReplayAttempt]) -> String {
    let mut out = String::new();
    for attempt in attempts {
        let http = attempt
            .http_status
            .map(|status| status.to_string())
            .unwrap_or_else(|| "-".to_string());
        out.push_str(&format!(
            "  {:<16} {}  http {}  approved_by {}  executed_by {}\n",
            attempt.outcome, attempt.record_id, http, attempt.approved_by, attempt.executed_by
        ));
        if let Some(detail) = &attempt.detail {
            out.push_str(&format!("                   {detail}\n"));
        }
    }
    out
}

fn render_execution(execution: &ExecutionView) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Replay {}: {}\n",
        execution.request.id, execution.request.status
    ));
    out.push_str(&format!(
        "Sent {}  refused {}\n",
        execution.sent_count, execution.refused_count
    ));
    let counts: Vec<String> = execution
        .outcome_counts
        .iter()
        .map(|(outcome, count)| format!("{outcome}={count}"))
        .collect();
    out.push_str(&format!("Outcomes: {}\n", counts.join(" ")));
    out.push_str(&render_attempts(&execution.attempts));
    if execution.all_duplicate {
        out.push_str("allDuplicate: true\n");
    } else {
        out.push_str(
            "allDuplicate: false -- not every record came back `duplicate`; \
             stop the drill (PLT-4603 §5.1)\n",
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> ReplayView {
        serde_json::from_value(serde_json::json!({
            "id": "edr_01",
            "outbox": "usage",
            "status": "pending_approval",
            "targetDigest": "d1g",
            "itemCount": 1,
            "reason": "drill",
            "changeReference": "PLT-4603 A6b: link",
            "requestedBy": "us_a",
            "requestedAt": "2026-10-01T00:00:00Z",
            "expiresAt": "2026-10-02T00:00:00Z",
            "expired": false,
            "approvedBy": null,
            "approvedAt": null,
            "executedBy": null,
            "executionStartedAt": null,
            "executionFinishedAt": null,
            "revokedBy": null,
            "revokedAt": null,
            "revokeReason": null,
            "items": [{
                "position": 1,
                "recordId": "credit_consumption_attempts:cca_1:input-token",
                "payloadBody": "{\"billingAccountId\":\"ba_1\",\"quantity\":\"10\"}",
                "payloadSha256": "abc",
                "sourceTenantId": "tn_1",
                "billingAccountId": "ba_1",
                "originalDeliveredAt": "2026-09-30T00:00:00Z",
                "originalRemoteEntryId": null,
                "grantChangeId": "chg_1"
            }],
            "attempts": []
        }))
        .unwrap()
    }

    #[test]
    fn dry_run_shows_digest_and_item_hashes() {
        let text = render_view(&view(), true);
        assert!(text.contains("Target digest:   d1g"));
        assert!(text.contains("credit_consumption_attempts:cca_1:input-token"));
        assert!(text.contains("sha256 abc"));
        assert!(text.contains("BA ba_1"));
        assert!(text.contains("grant chg_1"));
        assert!(text.contains("\"quantity\": \"10\""));
        assert!(!render_view(&view(), false).contains("body "));
    }

    #[test]
    fn execution_flags_anything_but_all_duplicate() {
        let attempt = |outcome: &str| ReplayAttempt {
            id: "era_1".into(),
            record_id: "r1".into(),
            payload_sha256: "abc".into(),
            approved_by: "us_b".into(),
            executed_by: "us_a".into(),
            outcome: outcome.into(),
            field_status: None,
            http_status: Some(200),
            remote_entry_id: None,
            remote_entry_id_matches: None,
            grant_change_id: None,
            detail: None,
            sent_at: None,
            finished_at: "2026-10-01T00:00:00Z".into(),
        };
        let mut execution = ExecutionView {
            request: view(),
            attempts: vec![attempt("duplicate")],
            outcome_counts: BTreeMap::from([("duplicate".to_string(), 1)]),
            sent_count: 1,
            refused_count: 0,
            all_duplicate: true,
        };
        let text = render_execution(&execution);
        assert!(text.contains("duplicate=1"));
        assert!(text.contains("approved_by us_b  executed_by us_a"));
        assert!(text.contains("allDuplicate: true"));

        execution.attempts = vec![attempt("accepted")];
        execution.all_duplicate = false;
        assert!(render_execution(&execution).contains("stop the drill"));
    }

    #[test]
    fn replay_ids_cannot_leave_the_path() {
        assert!(segment("edr_01jc0aph0v6rwx3h9caxepm800").is_ok());
        assert!(segment("../v1/secrets").is_err());
        assert!(segment("").is_err());
    }

    #[test]
    fn create_body_uses_the_api_field_names() {
        let ids = vec!["r1".to_string()];
        let body = serde_json::to_value(CreateRequest {
            outbox: ReplayOutbox::Usage,
            record_ids: &ids,
            reason: "drill",
            change_reference: "ref",
        })
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({
                "outbox": "usage",
                "recordIds": ["r1"],
                "reason": "drill",
                "changeReference": "ref"
            })
        );
        assert_eq!(
            serde_json::to_value(DigestRequest { target_digest: "d" }).unwrap(),
            serde_json::json!({"targetDigest": "d"})
        );
    }

    #[tokio::test]
    async fn api_keys_are_refused_before_any_request() {
        let command = ErpReplaysCommand::Get {
            id: "edr_01".into(),
            show_body: false,
            target: HostTenant {
                host_tenant_id: DEFAULT_HOST_TENANT_ID.into(),
            },
            json: false,
        };
        // Unroutable base path: a request would fail differently.
        let mut config = Configuration::new();
        config.base_path = "http://127.0.0.1:9".into();
        let err = run(&command, &config, true).await.unwrap_err();
        assert!(err.to_string().contains("human login session"), "{err}");
    }
}
