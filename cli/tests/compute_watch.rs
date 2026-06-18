use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

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

fn start_server(
    responses: Vec<&'static str>,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    start_server_with_responses(responses.into_iter().map(|body| (200, body)).collect())
}

fn start_server_with_responses(
    responses: Vec<(u16, &'static str)>,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            tx.send(String::from_utf8_lossy(&buf[..n]).to_string())
                .unwrap();
            let reason = match status {
                200 => "OK",
                404 => "Not Found",
                _ => "Unknown",
            };
            let response = format!(
                "HTTP/1.1 {status} {reason}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (url, rx, handle)
}

#[test]
fn compute_builds_watch_retries_log_404_while_build_is_running() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server_with_responses(vec![
        (
            200,
            r#"{"id":"bld_test1234567890","app_id":"app_test1234567890","status":"running","error_message":null}"#,
        ),
        (404, r#"{"message":"log group not found"}"#),
        (
            200,
            r#"{"id":"bld_test1234567890","app_id":"app_test1234567890","status":"succeeded","error_message":null}"#,
        ),
        (
            200,
            r#"{"lines":[{"timestamp":1767225600000,"message":"build finished"}],"next_token":null,"is_complete":true}"#,
        ),
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "builds",
            "watch",
            "--build-id",
            "bld_test1234567890",
            "--interval-secs",
            "1",
            "--timeout-secs",
            "5",
            "--agent",
        ])
        .output()
        .expect("run tachyon compute builds watch");

    assert!(
        output.status.success(),
        "watch failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let first_req = rx.recv().unwrap();
    let second_req = rx.recv().unwrap();
    let third_req = rx.recv().unwrap();
    let fourth_req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(first_req.starts_with("GET /v1/compute/builds/bld_test1234567890 "));
    assert!(second_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));
    assert!(third_req.starts_with("GET /v1/compute/builds/bld_test1234567890 "));
    assert!(fourth_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));

    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("json line"))
        .collect();
    assert_eq!(lines.len(), 4, "stdout:\n{stdout}");
    assert_eq!(lines[0]["type"], "build");
    assert_eq!(lines[0]["status"], "running");
    assert_eq!(lines[1]["type"], "build");
    assert_eq!(lines[1]["status"], "succeeded");
    assert_eq!(lines[2]["type"], "log");
    assert_eq!(lines[2]["message"], "build finished");
    assert_eq!(lines[3]["type"], "result");
    assert_eq!(lines[3]["exit_code"], 0);
}

#[test]
fn compute_builds_watch_keeps_terminal_log_404_as_error() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server_with_responses(vec![
        (
            200,
            r#"{"id":"bld_test1234567890","app_id":"app_test1234567890","status":"succeeded","error_message":null}"#,
        ),
        (404, r#"{"message":"log group not found"}"#),
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "builds",
            "watch",
            "--build-id",
            "bld_test1234567890",
            "--agent",
        ])
        .output()
        .expect("run tachyon compute builds watch");

    assert!(
        !output.status.success(),
        "watch unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let first_req = rx.recv().unwrap();
    let second_req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(first_req.starts_with("GET /v1/compute/builds/bld_test1234567890 "));
    assert!(second_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("status=404 Not Found"), "stderr:\n{stderr}");
}

#[test]
fn compute_builds_watch_agent_emits_compact_jsonl_and_exits_success() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        r#"{"id":"bld_test1234567890","app_id":"app_test1234567890","status":"succeeded","error_message":null}"#,
        r#"{"lines":[{"timestamp":1767225600000,"message":"build finished"}],"next_token":null,"is_complete":true}"#,
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "builds",
            "watch",
            "--build-id",
            "bld_test1234567890",
            "--agent",
        ])
        .output()
        .expect("run tachyon compute builds watch");

    assert!(
        output.status.success(),
        "watch failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let first_req = rx.recv().unwrap();
    let second_req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(first_req.starts_with("GET /v1/compute/builds/bld_test1234567890 "));
    assert!(first_req.contains("authorization: Bearer test-token"));
    assert!(first_req.contains("x-operator-id: tn_test1234567890"));
    assert!(second_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));

    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("json line"))
        .collect();
    assert_eq!(lines.len(), 3, "stdout:\n{stdout}");
    assert_eq!(lines[0]["type"], "build");
    assert_eq!(lines[0]["status"], "succeeded");
    assert_eq!(lines[1]["type"], "log");
    assert_eq!(lines[1]["message"], "build finished");
    assert_eq!(lines[2]["type"], "result");
    assert_eq!(lines[2]["exit_code"], 0);
}

#[test]
fn compute_builds_watch_latest_build_uses_app_builds_endpoint() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        r#"{"builds":[{"id":"bld_latest1234567890","app_id":"app_test1234567890","status":"running"}]}"#,
        r#"{"id":"bld_latest1234567890","app_id":"app_test1234567890","status":"succeeded","error_message":null}"#,
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "builds",
            "watch",
            "app_test1234567890",
            "--no-logs",
            "--agent",
        ])
        .output()
        .expect("run tachyon compute builds watch latest");

    assert!(
        output.status.success(),
        "watch latest failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let first_req = rx.recv().unwrap();
    let second_req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(first_req.starts_with("GET /v1/compute/apps/app_test1234567890/builds "));
    assert!(second_req.starts_with("GET /v1/compute/builds/bld_latest1234567890 "));

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""type":"result""#), "stdout:\n{stdout}");
    assert!(!stdout.contains(r#""type":"log""#), "stdout:\n{stdout}");
}

#[test]
fn compute_builds_list_prefers_public_preview_url() {
    let tmp = TempDir::new().unwrap();
    let app_id = "app_01kp4vm07tr3d4375597d15gkp";
    let (api_url, rx, handle) = start_server(vec![
        r#"{"builds":[{"id":"bld_01kp4vm07tr3d4375597d15gka","app_id":"app_01kp4vm07tr3d4375597d15gkp","status":"succeeded","source_branch":"feature/fix-mcp-write-tool-jsonrpc","commit_sha":"abcdef123456","created_at":"2026-05-07T00:00:00Z"}]}"#,
        r#"{"deployments":[{"id":"dep_01kp4vm07tr3d4375597d15gkb","app_id":"app_01kp4vm07tr3d4375597d15gkp","build_id":"bld_01kp4vm07tr3d4375597d15gka","status":"active","source_branch":"feature/fix-mcp-write-tool-jsonrpc","url":"https://8383df2f.moverent.pages.dev","public_url":"https://pr158--moverent.txcloud.app","created_at":"2026-05-07T00:00:00Z"}]}"#,
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args(["compute", "builds", "list", app_id])
        .output()
        .expect("run tachyon compute builds list");

    assert!(
        output.status.success(),
        "builds list failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let first_req = rx.recv().unwrap();
    let second_req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(first_req.starts_with(&format!("GET /v1/compute/apps/{app_id}/builds ")));
    assert!(second_req.starts_with(&format!(
        "GET /v1/compute/apps/{app_id}/deployments?environment=preview "
    )));

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Preview URLs:"), "stdout:\n{stdout}");
    assert!(
        stdout.contains("https://pr158--moverent.txcloud.app"),
        "stdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("https://8383df2f.moverent.pages.dev"),
        "stdout:\n{stdout}"
    );
}

#[test]
fn compute_builds_list_converts_pages_dev_preview_url_using_build_pr_number() {
    let tmp = TempDir::new().unwrap();
    let app_id = "app_01kp4vm07tr3d4375597d15gkp";
    let (api_url, rx, handle) = start_server(vec![
        r#"{"builds":[{"id":"bld_01kp4vm07tr3d4375597d15gka","app_id":"app_01kp4vm07tr3d4375597d15gkp","status":"succeeded","source_branch":"feature/fix-mcp-write-tool-jsonrpc","commit_sha":"abcdef123456","pr_number":158,"created_at":"2026-05-07T00:00:00Z"}]}"#,
        r#"{"deployments":[{"id":"dep_01kp4vm07tr3d4375597d15gkb","app_id":"app_01kp4vm07tr3d4375597d15gkp","build_id":"bld_01kp4vm07tr3d4375597d15gka","status":"active","source_branch":"feature/fix-mcp-write-tool-jsonrpc","url":"https://8383df2f.moverent.pages.dev","created_at":"2026-05-07T00:00:00Z"}]}"#,
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args(["compute", "builds", "list", app_id])
        .output()
        .expect("run tachyon compute builds list");

    assert!(
        output.status.success(),
        "builds list failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let first_req = rx.recv().unwrap();
    let second_req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(first_req.starts_with(&format!("GET /v1/compute/apps/{app_id}/builds ")));
    assert!(second_req.starts_with(&format!(
        "GET /v1/compute/apps/{app_id}/deployments?environment=preview "
    )));

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Preview URLs:"), "stdout:\n{stdout}");
    assert!(
        stdout.contains("https://pr158--moverent.txcloud.app"),
        "stdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("https://8383df2f.moverent.pages.dev"),
        "stdout:\n{stdout}"
    );
}

#[test]
fn compute_preview_create_posts_manual_branch_build() {
    let tmp = TempDir::new().unwrap();
    let app_id = "app_01kp4vm07tr3d4375597d15gkp";
    let (api_url, rx, handle) = start_server(vec![
        r#"{"id":"bld_01kp4vm07tr3d4375597d15gka","app_id":"app_01kp4vm07tr3d4375597d15gkp","status":"queued","source_branch":"feature/manual-preview","created_at":"2026-05-07T00:00:00Z"}"#,
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "preview",
            "create",
            "--app",
            app_id,
            "--branch",
            "feature/manual-preview",
        ])
        .output()
        .expect("run tachyon compute preview create");

    assert!(
        output.status.success(),
        "preview create failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let request = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(request.starts_with(&format!("POST /v1/apps/{app_id}/builds ")));
    assert!(request.contains("authorization: Bearer test-token"));
    assert!(request.contains("x-operator-id: tn_test1234567890"));
    assert!(
        request.contains(r#""source_branch":"feature/manual-preview""#),
        "request:\n{request}"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Preview build created: bld_01kp4vm07tr3d4375597d15gka"),
        "stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Branch: feature/manual-preview"),
        "stdout:\n{stdout}"
    );
}
