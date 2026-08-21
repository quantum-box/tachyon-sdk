use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;

use tempfile::TempDir;

const TENANT_ID: &str = "tn_test1234567890";
const APP_ID: &str = "app_test1234567890";

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_tachyon")
}

fn isolated_command(home: &Path) -> Command {
    let mut cmd = Command::new(bin());
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_TENANT_ID", TENANT_ID)
        .env("TACHYON_API_KEY", "test-token")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE");
    cmd
}

struct MockResponse {
    status: &'static str,
    body: &'static str,
}

fn start_server(
    responses: Vec<MockResponse>,
) -> (String, mpsc::Receiver<Vec<String>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut requests = Vec::with_capacity(responses.len());
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            requests.push(String::from_utf8_lossy(&buf[..n]).to_string());

            let raw_response = format!(
                "HTTP/1.1 {}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                response.status,
                response.body.len(),
                response.body
            );
            stream.write_all(raw_response.as_bytes()).unwrap();
        }
        tx.send(requests).unwrap();
    });
    (url, rx, handle)
}

fn finish_requests(rx: mpsc::Receiver<Vec<String>>, handle: thread::JoinHandle<()>) -> Vec<String> {
    let requests = rx.recv().unwrap();
    handle.join().unwrap();
    requests
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn request_body(request: &str) -> serde_json::Value {
    let body = request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("request body separator");
    serde_json::from_str(body).expect("request JSON body")
}

#[test]
fn rollback_requires_a_target() {
    let tmp = TempDir::new().unwrap();

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", "http://127.0.0.1:1")
        .args(["compute", "deployments", "rollback", APP_ID])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--to-previous"), "stderr was:\n{stderr}");
    assert!(
        stderr.contains("rollback-candidates"),
        "stderr was:\n{stderr}"
    );
}

#[test]
fn rollback_rejects_both_targets() {
    let tmp = TempDir::new().unwrap();

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", "http://127.0.0.1:1")
        .args([
            "compute",
            "deployments",
            "rollback",
            APP_ID,
            "--deployment-id",
            "dep_a",
            "--to-previous",
        ])
        .output()
        .unwrap();

    // clap-level conflict: no request is ever sent.
    assert!(!output.status.success());
}

#[test]
fn rollback_to_previous_posts_the_flag() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "201 Created",
        body: r#"{"id":"dep_new123","status":"active","build_id":"bld_prev456"}"#,
    }]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "deployments",
            "rollback",
            APP_ID,
            "--to-previous",
        ])
        .output()
        .unwrap();
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Rolled back to previous production version (build bld_prev456)"),
        "stdout was:\n{stdout}"
    );
    assert!(stdout.contains("dep_new123"), "stdout was:\n{stdout}");

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0].starts_with(&format!("POST /v1/compute/apps/{APP_ID}/rollback ")),
        "request was:\n{}",
        requests[0]
    );
    let body = request_body(&requests[0]);
    assert_eq!(body, serde_json::json!({ "to_previous": true }));
}

#[test]
fn rollback_with_deployment_id_keeps_existing_shape() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "201 Created",
        body: r#"{"id":"dep_new123","status":"active"}"#,
    }]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "deployments",
            "rollback",
            APP_ID,
            "--deployment-id",
            "dep_target789",
        ])
        .output()
        .unwrap();
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 1);
    let body = request_body(&requests[0]);
    assert_eq!(
        body,
        serde_json::json!({
            "deployment_id": "dep_target789",
            "to_previous": false,
        })
    );
}

#[test]
fn rollback_candidates_lists_targets_newest_first() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"candidates":[
            {"id":"dep_newer","status":"superseded","build_id":"bld_b","source_branch":"main","created_at":"2026-08-20T10:00:00Z"},
            {"id":"dep_older","status":"superseded","build_id":"bld_a","source_branch":"main","created_at":"2026-08-13T10:00:00Z"}
        ]}"#,
    }]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "deployments",
            "rollback-candidates",
            APP_ID,
            "--limit",
            "2",
        ])
        .output()
        .unwrap();
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dep_newer"), "stdout was:\n{stdout}");
    assert!(stdout.contains("dep_older"), "stdout was:\n{stdout}");
    assert!(
        stdout.contains("`rollback --to-previous` rolls back to the first candidate (dep_newer)"),
        "stdout was:\n{stdout}"
    );

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0].starts_with(&format!(
            "GET /v1/compute/apps/{APP_ID}/rollback/candidates?limit=2 "
        )),
        "request was:\n{}",
        requests[0]
    );
}

#[test]
fn rollback_candidates_reports_empty_result() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"candidates":[]}"#,
    }]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args(["compute", "deployments", "rollback-candidates", APP_ID])
        .output()
        .unwrap();
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No rollback candidates"),
        "stdout was:\n{stdout}"
    );

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 1);
}
