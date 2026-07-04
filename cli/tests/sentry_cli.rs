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
