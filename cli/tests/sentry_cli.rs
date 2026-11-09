use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

use serde_json::Value;
use tempfile::TempDir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_tachyon")
}

fn isolated_command(home: &Path) -> Command {
    let mut cmd = Command::new(bin());
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_TENANT_ID", "tn_test1234567890")
        .env("TACHYON_API_KEY", "test-token")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE");
    cmd
}

fn start_server(body: &'static str) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 16384];
        let n = stream.read(&mut buf).unwrap();
        tx.send(String::from_utf8_lossy(&buf[..n]).to_string())
            .unwrap();

        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, rx, handle)
}

fn request_json_body(request: &str) -> Value {
    let body = request.split("\r\n\r\n").nth(1).unwrap();
    serde_json::from_str(body).unwrap()
}

#[test]
fn sentry_issues_list_sends_project_query_and_limit() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(
        r#"{"issues":[{"id":"12345","shortId":"FIELDADMIN-1","title":"TypeError","count":3}]}"#,
    );

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "ops",
            "sentry",
            "issues",
            "list",
            "--project",
            "fieldadmin",
            "--query",
            "is:unresolved",
            "--limit",
            "10",
        ])
        .output()
        .expect("run tachyon ops sentry issues list");

    assert!(
        output.status.success(),
        "sentry issues list failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    handle.join().unwrap();
    let req = rx.recv().unwrap();
    assert!(req.starts_with("GET /v1/ops/sentry/issues?"));
    assert!(req.contains("project=fieldadmin"));
    assert!(req.contains("query=is%3Aunresolved"));
    assert!(req.contains("limit=10"));
    assert!(req.contains("authorization: Bearer test-token"));
    assert!(req.contains("x-operator-id: tn_test1234567890"));
}

#[test]
fn sentry_issue_view_json_preserves_allowlisted_latest_event() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(
        r#"{
            "id":"7651276254",
            "shortId":"TACHYON-API-L9-1A",
            "title":"money_path_authz_denied",
            "latestEvent":{
                "event_id":"event-latest",
                "timestamp":"2026-08-04T07:33:41Z",
                "level":"error",
                "message":"money_path_authz_denied",
                "tags":{
                    "event.name":"money_path_authz_denied",
                    "authz.money_path":"synthetic-path",
                    "authz.action":"synthetic:Action",
                    "authz.principal_kind":"user",
                    "authz.principal_role":"OWNER",
                    "authz.tenant_id":"tn_synthetic",
                    "authz.target_id":"target_synthetic",
                    "authz.deny_reason":"PermissionDenied",
                    "unrelated":"SYNTHETIC_SECRET_MUST_NOT_LEAK"
                },
                "request":{"data":"SYNTHETIC_SECRET_MUST_NOT_LEAK"},
                "contexts":{"secret":"SYNTHETIC_SECRET_MUST_NOT_LEAK"},
                "extra":{"dsn":"SYNTHETIC_DSN_MUST_NOT_LEAK"}
            }
        }"#,
    );

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args(["ops", "sentry", "issues", "view", "7651276254", "--json"])
        .output()
        .expect("run tachyon ops sentry issues view");

    assert!(
        output.status.success(),
        "sentry issue view failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    handle.join().unwrap();
    let req = rx.recv().unwrap();
    assert!(req.starts_with("GET /v1/ops/sentry/issues/7651276254 HTTP/1.1"));

    let stdout: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        stdout["latest_event"],
        serde_json::json!({
            "id": null,
            "event_id": "event-latest",
            "title": null,
            "message": "money_path_authz_denied",
            "timestamp": "2026-08-04T07:33:41Z",
            "level": "error",
            "tags": {
                "event.name": "money_path_authz_denied",
                "authz.money_path": "synthetic-path",
                "authz.action": "synthetic:Action",
                "authz.principal_kind": "user",
                "authz.principal_role": "OWNER",
                "authz.tenant_id": "tn_synthetic",
                "authz.target_id": "target_synthetic",
                "authz.deny_reason": "PermissionDenied"
            }
        })
    );
    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!combined_output.contains("SYNTHETIC_SECRET_MUST_NOT_LEAK"));
    assert!(!combined_output.contains("SYNTHETIC_DSN_MUST_NOT_LEAK"));
}

#[test]
fn sentry_issue_assign_posts_user_body() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(
        r#"{"id":"12345","shortId":"FIELDADMIN-1","title":"TypeError","assignedTo":{"email":"user@example.com"}}"#,
    );

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "ops",
            "sentry",
            "issue",
            "assign",
            "12345",
            "user@example.com",
        ])
        .output()
        .expect("run tachyon ops sentry issue assign");

    assert!(
        output.status.success(),
        "sentry issue assign failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    handle.join().unwrap();
    let req = rx.recv().unwrap();
    assert!(req.starts_with("POST /v1/ops/sentry/issues/12345/assign "));
    assert_eq!(
        request_json_body(&req),
        serde_json::json!({"user": "user@example.com"})
    );
}

fn run_sentry_issue_command(args: &[&str], body: &'static str) -> (String, String) {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(body);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args(["ops", "sentry", "issues"])
        .args(args)
        .output()
        .expect("run tachyon ops sentry issues");

    assert!(
        output.status.success(),
        "sentry issues {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    handle.join().unwrap();
    (
        rx.recv().unwrap(),
        String::from_utf8_lossy(&output.stdout).to_string(),
    )
}

#[test]
fn sentry_issue_resolve_accepts_short_id() {
    let (req, stdout) = run_sentry_issue_command(
        &["resolve", "TACHYON-API-1A2"],
        r#"{"id":"12345","shortId":"TACHYON-API-1A2","status":"resolved"}"#,
    );

    assert!(req.starts_with("POST /v1/ops/sentry/issues/TACHYON-API-1A2/resolve "));
    assert!(stdout.contains("Sentry issue TACHYON-API-1A2 resolved."));
}

#[test]
fn sentry_issue_unresolve_posts_unresolved_status() {
    let (req, stdout) = run_sentry_issue_command(
        &["reopen", "12345"],
        r#"{"id":"12345","shortId":"FIELDADMIN-1","status":"unresolved"}"#,
    );

    assert!(req.starts_with("POST /v1/ops/sentry/issues/12345/status "));
    assert_eq!(
        request_json_body(&req),
        serde_json::json!({"status": "unresolved"})
    );
    assert!(stdout.contains("Sentry issue FIELDADMIN-1 reopened."));
}

#[test]
fn sentry_issue_archive_posts_ignored_status() {
    let (req, stdout) = run_sentry_issue_command(
        &["archive", "12345"],
        r#"{"id":"12345","shortId":"FIELDADMIN-1","status":"ignored"}"#,
    );

    assert!(req.starts_with("POST /v1/ops/sentry/issues/12345/status "));
    assert_eq!(
        request_json_body(&req),
        serde_json::json!({"status": "ignored"})
    );
    assert!(stdout.contains("Sentry issue FIELDADMIN-1 archived."));
}

#[test]
fn sentry_issue_unassign_posts_to_unassign_endpoint() {
    let (req, stdout) = run_sentry_issue_command(
        &["unassign", "12345", "--json"],
        r#"{"id":"12345","shortId":"FIELDADMIN-1","assignedTo":null}"#,
    );

    assert!(req.starts_with("POST /v1/ops/sentry/issues/12345/unassign "));
    let stdout: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(stdout["id"], "12345");
}

#[test]
fn sentry_issues_list_table_shows_numeric_id() {
    let (_, stdout) = run_sentry_issue_command(
        &["list"],
        r#"{"issues":[{"id":"7651276254","shortId":"FIELDADMIN-1","title":"TypeError","count":3}]}"#,
    );

    assert!(stdout.contains("FIELDADMIN-1"));
    assert!(stdout.contains("7651276254"));
}
