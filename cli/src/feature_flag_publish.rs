//! Publishes a host boolean flag to a finite list of tenants through
//! `/v1/feature-flags/system-publications`: preview, single-use approval,
//! apply, then the committed receipt. The server decides who may publish;
//! it accepts only an interactive (Cognito) session of a host administrator.
use crate::client::{http_error_status, ApiClient};
use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Duration, SubsecRound, Utc};
use clap::Args;
use serde_json::{json, Value};
use std::io::{IsTerminal, Write};

/// Production host tenant (`x-operator-id`).
pub const HOST_TENANT_ID: &str = "tn_01jcjtqxah6mhyw4e5mahg02nd";
/// Platform the Cognito session is verified against (`x-platform-id`). The
/// host has no parent, so authentication needs a registered platform scope.
pub const DEFAULT_PLATFORM_ID: &str = "tn_01hjjn348rn3t49zz6hvmfq67p";
const BASE: &str = "/v1/feature-flags/system-publications";
const MAX_TARGETS: usize = 50;
const APPLY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

#[derive(Debug, Args)]
pub struct PublishArgs {
    /// Host boolean flag key, e.g. aip_tool_gate_shadow
    pub key: String,
    /// Tenant to publish to; repeat for each tenant (1-50, no duplicates)
    #[arg(long = "target", value_name = "TENANT_ID", required = true)]
    pub targets: Vec<String>,
    /// When the publication ends: RFC3339, or a duration from now (30m, 24h,
    /// 7d). Omit only for keys the server allows to stay open
    #[arg(long, value_name = "RFC3339|DURATION")]
    pub until: Option<String>,
    /// Why the flag is published; recorded with the approval and receipt
    #[arg(long)]
    pub reason: String,
    /// Close the publication (enabled=false). Needs no approval
    #[arg(long)]
    pub disable: bool,
    /// Apply without asking for confirmation after the preview
    #[arg(long, short = 'y')]
    pub yes: bool,
    /// Show the preview only; request no approval and change nothing
    #[arg(long, conflicts_with = "yes")]
    pub dry_run: bool,
    /// Host tenant that owns the flag (x-operator-id)
    #[arg(long, default_value = HOST_TENANT_ID)]
    pub host: String,
}

/// Result of [`publish`].
#[derive(Debug)]
pub enum Published {
    /// `--dry-run`: the server preview, nothing requested.
    Preview(Value),
    /// The operator declined at the confirmation prompt.
    Declined,
    /// Committed; the apply response or the receipt read after an
    /// unconfirmed apply.
    Applied { receipt: Value, recovered: bool },
}

/// Resolves `--until` once so every request carries the same window, which
/// the approval digest binds.
pub fn parse_until(value: &str, now: DateTime<Utc>) -> Result<DateTime<Utc>> {
    let value = value.trim();
    if let Some(unit) = value.chars().last().filter(|c| "mhd".contains(*c)) {
        if let Ok(amount) = value[..value.len() - 1].parse::<i64>() {
            if amount <= 0 {
                bail!("--until duration must be positive: {value}");
            }
            let duration = match unit {
                'm' => Duration::minutes(amount),
                'h' => Duration::hours(amount),
                _ => Duration::days(amount),
            };
            return Ok((now + duration).trunc_subsecs(0));
        }
    }
    DateTime::parse_from_rfc3339(value)
        .map(|t| t.with_timezone(&Utc))
        .with_context(|| format!("--until must be RFC3339 or a duration like 7d: {value}"))
}

/// Builds the proposal body sent unchanged to preview, approvals and apply.
pub fn proposal(args: &PublishArgs, now: DateTime<Utc>) -> Result<Value> {
    if args.targets.len() > MAX_TARGETS {
        bail!("at most {MAX_TARGETS} --target values are allowed");
    }
    let mut seen = std::collections::HashSet::new();
    if let Some(duplicate) = args.targets.iter().find(|t| !seen.insert(t.as_str())) {
        bail!("duplicate --target {duplicate}");
    }
    let mut body = json!({
        "key": args.key,
        "targets": args.targets,
        "enabled": !args.disable,
        "reason": args.reason,
    });
    if let Some(until) = &args.until {
        body["validUntil"] =
            json!(parse_until(until, now)?.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    }
    Ok(body)
}

/// A 4xx apply was refused before commit; anything else may have committed.
fn refused_before_commit(error: &anyhow::Error) -> bool {
    http_error_status(error).is_some_and(|s| s.is_client_error() && s.as_u16() != 408)
}

/// Runs preview → (approval) → apply. An apply that times out or fails
/// without a definite refusal is never resent; the receipt is read instead.
pub async fn publish(
    api: &ApiClient,
    proposal: &Value,
    dry_run: bool,
    apply_timeout: std::time::Duration,
    confirm: &mut dyn FnMut(&Value) -> Result<bool>,
) -> Result<Published> {
    let preview: Value = api.post(&format!("{BASE}/preview"), proposal).await?;
    if dry_run {
        return Ok(Published::Preview(preview));
    }
    if !confirm(&preview)? {
        return Ok(Published::Declined);
    }
    let approval_id = if proposal["enabled"] == true {
        let approval: Value = api.post(&format!("{BASE}/approvals"), proposal).await?;
        Some(
            approval["approvalId"]
                .as_str()
                .ok_or_else(|| anyhow!("approval response has no approvalId"))?
                .to_string(),
        )
    } else {
        None
    };
    let mut body = json!({ "proposal": proposal });
    if let Some(id) = &approval_id {
        body["approvalId"] = json!(id);
    }
    let cause = match tokio::time::timeout(
        apply_timeout,
        api.post_once::<_, Value>(&format!("{BASE}/apply"), &body),
    )
    .await
    {
        Ok(Ok(receipt)) => {
            return Ok(Published::Applied {
                receipt,
                recovered: false,
            })
        }
        Ok(Err(error)) if refused_before_commit(&error) => return Err(error),
        Ok(Err(error)) => error,
        Err(_) => anyhow!("apply did not answer within {}s", apply_timeout.as_secs()),
    };
    let Some(id) = approval_id else {
        return Err(cause.context(
            "apply was not confirmed; it was not resent. Check the flag state before running it again",
        ));
    };
    match api.get::<Value>(&format!("{BASE}/receipts/{id}")).await {
        Ok(receipt) => Ok(Published::Applied {
            receipt,
            recovered: true,
        }),
        Err(receipt_error) => Err(anyhow!(
            "apply was not confirmed ({cause:#}) and no receipt was found for {id} ({receipt_error:#}). \
             It was not resent. Read the receipt again before retrying: GET {BASE}/receipts/{id}"
        )),
    }
}

fn state_line(state: &Value) -> String {
    if state.is_null() {
        return "(no row)".into();
    }
    format!(
        "{} until {}",
        if state["enabled"] == true {
            "ON"
        } else {
            "OFF"
        },
        state["valid_until"].as_str().unwrap_or("(none)")
    )
}

fn print_preview(preview: &Value) {
    let proposal = &preview["proposal"];
    let list = |v: &Value| {
        v.as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    };
    eprintln!(
        "key:       {}",
        proposal["key"].as_str().unwrap_or_default()
    );
    eprintln!(
        "host:      {}",
        preview["host"].as_str().unwrap_or_default()
    );
    eprintln!("targets:   {}", list(&proposal["targets"]));
    eprintln!(
        "platforms: {} (inherited: {})",
        list(&preview["platforms"]),
        list(&preview["inheritedPlatforms"])
    );
    eprintln!("before:    {}", state_line(&preview["beforeState"]));
    eprintln!("after:     {}", state_line(&preview["afterState"]));
    eprintln!(
        "reason:    {}",
        proposal["reason"].as_str().unwrap_or_default()
    );
}

fn prompt(preview: &Value, yes: bool) -> Result<bool> {
    print_preview(preview);
    if yes {
        return Ok(true);
    }
    if !std::io::stdin().is_terminal() {
        bail!("refusing to apply without confirmation; pass --yes to apply non-interactively");
    }
    eprint!("Apply this publication? [y/N] ");
    std::io::stderr().flush()?;
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}

pub async fn run(
    args: &PublishArgs,
    config: &tachyon_sdk::apis::configuration::Configuration,
    json: bool,
) -> Result<()> {
    let platform = crate::client::platform_id().unwrap_or(DEFAULT_PLATFORM_ID);
    let api = ApiClient::new_scoped(config, &args.host, Some(platform))?;
    let body = proposal(args, Utc::now())?;
    let yes = args.yes;
    let outcome = publish(&api, &body, args.dry_run, APPLY_TIMEOUT, &mut |p| {
        prompt(p, yes)
    })
    .await?;
    match outcome {
        Published::Preview(preview) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&preview)?);
            } else {
                print_preview(&preview);
                eprintln!("dry run: no approval requested, nothing changed");
            }
        }
        Published::Declined => eprintln!("not applied"),
        Published::Applied { receipt, recovered } => {
            if json {
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            } else {
                if recovered {
                    eprintln!("apply was not confirmed in time; the committed receipt was found");
                }
                for field in ["changeId", "approvalId", "rowRevision", "freshRowMatches"] {
                    if !receipt[field].is_null() {
                        println!("{field}: {}", receipt[field]);
                    }
                }
                eprintln!(
                    "freshRowMatches covers the flag row only; check the consumer separately"
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::Path,
        http::HeaderMap,
        response::IntoResponse,
        routing::{get, post},
        Json, Router,
    };
    use std::sync::{Arc, Mutex};
    use tachyon_sdk::apis::configuration::Configuration;

    #[derive(Clone, Copy)]
    enum Apply {
        Ok,
        Slow,
        Unconfirmed,
        Refused,
    }

    type Calls = Arc<Mutex<Vec<(String, Value)>>>;

    async fn server(apply: Apply) -> (ApiClient, Calls, tokio::task::JoinHandle<()>) {
        let calls: Calls = Arc::default();
        let record = |name: &'static str, calls: Calls| {
            move |headers: HeaderMap, Json(body): Json<Value>| {
                let calls = calls.clone();
                async move {
                    assert_eq!(headers["x-operator-id"], HOST_TENANT_ID);
                    assert_eq!(headers["x-platform-id"], DEFAULT_PLATFORM_ID);
                    calls.lock().unwrap().push((name.into(), body.clone()));
                    match name {
                        "preview" => Json(json!({"proposal": body, "beforeState": null, "afterState": {"enabled": true}})).into_response(),
                        "approvals" => Json(json!({"approvalId": "ap_test", "preview": {}})).into_response(),
                        _ => match apply {
                            Apply::Ok => Json(json!({"changeId": "fc_1", "approvalId": body["approvalId"]})).into_response(),
                            Apply::Slow => {
                                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                Json(json!({})).into_response()
                            }
                            Apply::Unconfirmed => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "not confirmed").into_response(),
                            Apply::Refused => (axum::http::StatusCode::FORBIDDEN, "approval expired").into_response(),
                        },
                    }
                }
            }
        };
        let receipts = calls.clone();
        let app = Router::new()
            .route(
                &format!("{BASE}/preview"),
                post(record("preview", calls.clone())),
            )
            .route(
                &format!("{BASE}/approvals"),
                post(record("approvals", calls.clone())),
            )
            .route(
                &format!("{BASE}/apply"),
                post(record("apply", calls.clone())),
            )
            .route(
                &format!("{BASE}/receipts/{{id}}"),
                get(move |Path(id): Path<String>| {
                    let receipts = receipts.clone();
                    async move {
                        receipts.lock().unwrap().push(("receipt".into(), json!(id)));
                        Json(json!({"approvalId": id, "consumerVerified": false}))
                    }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let mut config = Configuration::new();
        config.base_path = format!("http://{address}");
        let api =
            ApiClient::new_scoped(&config, HOST_TENANT_ID, Some(DEFAULT_PLATFORM_ID)).unwrap();
        (api, calls, handle)
    }

    fn args(disable: bool) -> PublishArgs {
        PublishArgs {
            key: "aip_tool_gate_shadow".into(),
            targets: vec!["tn_field".into()],
            until: Some("7d".into()),
            reason: "shadow".into(),
            disable,
            yes: true,
            dry_run: false,
            host: HOST_TENANT_ID.into(),
        }
    }

    fn names(calls: &Calls) -> Vec<String> {
        calls
            .lock()
            .unwrap()
            .iter()
            .map(|(n, _)| n.clone())
            .collect()
    }

    const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

    #[tokio::test]
    async fn enabling_previews_approves_then_applies_with_the_approval_id() {
        let (api, calls, handle) = server(Apply::Ok).await;
        let body = proposal(&args(false), Utc::now()).unwrap();
        let outcome = publish(&api, &body, false, TIMEOUT, &mut |_| Ok(true))
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            Published::Applied {
                recovered: false,
                ..
            }
        ));
        assert_eq!(names(&calls), ["preview", "approvals", "apply"]);
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].1, body);
        assert_eq!(calls[1].1, body);
        assert_eq!(
            calls[2].1,
            json!({"proposal": body, "approvalId": "ap_test"})
        );
        handle.abort();
    }

    #[tokio::test]
    async fn disabling_applies_without_an_approval() {
        let (api, calls, handle) = server(Apply::Ok).await;
        let body = proposal(&args(true), Utc::now()).unwrap();
        publish(&api, &body, false, TIMEOUT, &mut |_| Ok(true))
            .await
            .unwrap();
        assert_eq!(names(&calls), ["preview", "apply"]);
        assert_eq!(calls.lock().unwrap()[1].1, json!({"proposal": body}));
        handle.abort();
    }

    #[tokio::test]
    async fn dry_run_and_declining_stop_after_the_preview() {
        let (api, calls, handle) = server(Apply::Ok).await;
        let body = proposal(&args(false), Utc::now()).unwrap();
        let dry = publish(&api, &body, true, TIMEOUT, &mut |_| unreachable!())
            .await
            .unwrap();
        assert!(matches!(dry, Published::Preview(_)));
        let declined = publish(&api, &body, false, TIMEOUT, &mut |_| Ok(false))
            .await
            .unwrap();
        assert!(matches!(declined, Published::Declined));
        assert_eq!(names(&calls), ["preview", "preview"]);
        handle.abort();
    }

    #[tokio::test]
    async fn a_timed_out_apply_is_not_resent_and_the_receipt_is_read() {
        let (api, calls, handle) = server(Apply::Slow).await;
        let body = proposal(&args(false), Utc::now()).unwrap();
        let outcome = publish(
            &api,
            &body,
            false,
            std::time::Duration::from_millis(200),
            &mut |_| Ok(true),
        )
        .await
        .unwrap();
        assert!(matches!(
            outcome,
            Published::Applied {
                recovered: true,
                ..
            }
        ));
        assert_eq!(names(&calls), ["preview", "approvals", "apply", "receipt"]);
        assert_eq!(calls.lock().unwrap()[3].1, "ap_test");
        handle.abort();
    }

    #[tokio::test]
    async fn an_unconfirmed_apply_reads_the_receipt_and_a_refused_one_does_not() {
        let (api, calls, handle) = server(Apply::Unconfirmed).await;
        let body = proposal(&args(false), Utc::now()).unwrap();
        let outcome = publish(&api, &body, false, TIMEOUT, &mut |_| Ok(true))
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            Published::Applied {
                recovered: true,
                ..
            }
        ));
        assert_eq!(names(&calls), ["preview", "approvals", "apply", "receipt"]);
        handle.abort();

        let (api, calls, handle) = server(Apply::Refused).await;
        let error = publish(&api, &body, false, TIMEOUT, &mut |_| Ok(true))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("approval expired"));
        assert_eq!(names(&calls), ["preview", "approvals", "apply"]);
        handle.abort();
    }

    #[tokio::test]
    async fn an_unconfirmed_disable_is_not_resent() {
        let (api, calls, handle) = server(Apply::Unconfirmed).await;
        let body = proposal(&args(true), Utc::now()).unwrap();
        let error = publish(&api, &body, false, TIMEOUT, &mut |_| Ok(true))
            .await
            .unwrap_err();
        assert!(format!("{error:#}").contains("not resent"));
        assert_eq!(names(&calls), ["preview", "apply"]);
        handle.abort();
    }

    #[test]
    fn until_accepts_durations_and_rfc3339_and_targets_must_be_unique() {
        let now = DateTime::parse_from_rfc3339("2026-11-25T07:00:00.5Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(
            parse_until("7d", now).unwrap().to_rfc3339(),
            "2026-12-02T07:00:00+00:00"
        );
        assert_eq!(
            parse_until("24h", now).unwrap().to_rfc3339(),
            "2026-11-26T07:00:00+00:00"
        );
        assert_eq!(
            parse_until("2026-12-01T00:00:00+09:00", now)
                .unwrap()
                .to_rfc3339(),
            "2026-11-30T15:00:00+00:00"
        );
        assert!(parse_until("0d", now).is_err());
        assert!(parse_until("soon", now).is_err());
        let body = proposal(&args(false), now).unwrap();
        assert_eq!(body["validUntil"], "2026-12-02T07:00:00Z");
        assert_eq!(body["enabled"], true);
        let mut duplicate = args(false);
        duplicate.targets.push("tn_field".into());
        assert!(proposal(&duplicate, now).is_err());
    }
}
