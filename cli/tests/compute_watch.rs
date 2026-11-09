use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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

fn start_server_with_body_delays(
    responses: Vec<(Duration, &'static str)>,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for (body_delay, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            tx.send(String::from_utf8_lossy(&buf[..n]).to_string())
                .unwrap();
            let headers = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(headers.as_bytes()).unwrap();
            stream.flush().unwrap();
            thread::sleep(body_delay);
            let _ = stream.write_all(body.as_bytes());
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
        r#"{"id":"bld_test1234567890","app_id":"app_test1234567890","status":"succeeded","artifact_status":"succeeded","deploy_hook_status":"succeeded","error_message":null}"#,
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
    assert_eq!(lines[0]["artifact_status"], "succeeded");
    assert_eq!(lines[0]["deploy_hook_status"], "succeeded");
    assert_eq!(lines[1]["type"], "log");
    assert_eq!(lines[1]["message"], "build finished");
    assert_eq!(lines[2]["type"], "result");
    assert_eq!(lines[2]["exit_code"], 0);
}

#[test]
fn compute_builds_watch_uses_effective_status_for_deploy_hook_failure_exit() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        r#"{"id":"bld_test1234567890","app_id":"app_test1234567890","status":"failed","artifact_status":"succeeded","deploy_hook_status":"failed","error_message":"deploy hook failed"}"#,
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "builds",
            "watch",
            "--build-id",
            "bld_test1234567890",
            "--no-logs",
            "--agent",
        ])
        .output()
        .expect("run tachyon compute builds watch");

    assert!(
        !output.status.success(),
        "deploy hook failure must produce a failing exit status\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let request = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(request.starts_with("GET /v1/compute/builds/bld_test1234567890 "));

    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("json line"))
        .collect();
    assert_eq!(lines.len(), 2, "stdout:\n{stdout}");
    assert_eq!(lines[0]["artifact_status"], "succeeded");
    assert_eq!(lines[0]["deploy_hook_status"], "failed");
    assert_eq!(lines[1]["type"], "result");
    assert_eq!(lines[1]["status"], "failed");
    assert_eq!(lines[1]["exit_code"], 1);
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
fn compute_builds_logs_follow_stops_after_repeated_none_token_no_progress() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        r#"{"lines":[],"next_token":null,"is_complete":false}"#,
        r#"{"lines":[],"next_token":null,"is_complete":false}"#,
        r#"{"lines":[],"next_token":null,"is_complete":false}"#,
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_COMPUTE_BUILD_LOGS_FOLLOW_INTERVAL_MS", "1")
        .args([
            "compute",
            "builds",
            "logs",
            "--build-id",
            "bld_test1234567890",
            "--follow",
        ])
        .output()
        .expect("run tachyon compute builds logs --follow");

    assert!(
        output.status.success(),
        "logs follow failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let first_req = rx.recv().unwrap();
    let second_req = rx.recv().unwrap();
    let third_req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(first_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));
    assert!(second_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));
    assert!(third_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));
    assert!(
        String::from_utf8(output.stdout).unwrap().is_empty(),
        "stdout should not include duplicate no-progress logs",
    );
}

#[test]
fn compute_builds_logs_follow_continues_when_new_logs_arrive_without_token() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        r#"{"lines":[],"next_token":null,"is_complete":false}"#,
        r#"{"lines":[{"timestamp":1767225600000,"message":"installing"}],"next_token":null,"is_complete":false}"#,
        r#"{"lines":[{"timestamp":1767225600000,"message":"installing"},{"timestamp":1767225601000,"message":"build finished"}],"next_token":null,"is_complete":true}"#,
        r#"{"id":"bld_test1234567890","app_id":"app_test1234567890","status":"succeeded","error_message":null}"#,
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_COMPUTE_BUILD_LOGS_FOLLOW_INTERVAL_MS", "1")
        .args([
            "compute",
            "builds",
            "logs",
            "--build-id",
            "bld_test1234567890",
            "--follow",
        ])
        .output()
        .expect("run tachyon compute builds logs --follow");

    assert!(
        output.status.success(),
        "logs follow failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let first_req = rx.recv().unwrap();
    let second_req = rx.recv().unwrap();
    let third_req = rx.recv().unwrap();
    let fourth_req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(first_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));
    assert!(second_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));
    assert!(third_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));
    assert!(fourth_req.starts_with("GET /v1/compute/builds/bld_test1234567890 "));

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("installing"), "stdout:\n{stdout}");
    assert!(stdout.contains("build finished"), "stdout:\n{stdout}");
}

#[test]
fn compute_builds_logs_follow_continues_when_next_token_advances() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        r#"{"lines":[],"next_token":"n1","is_complete":false}"#,
        r#"{"lines":[],"next_token":"n2","is_complete":false}"#,
        r#"{"lines":[{"timestamp":1767225600000,"message":"build finished"}],"next_token":null,"is_complete":true}"#,
        r#"{"id":"bld_test1234567890","app_id":"app_test1234567890","status":"succeeded","error_message":null}"#,
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_COMPUTE_BUILD_LOGS_FOLLOW_INTERVAL_MS", "1")
        .args([
            "compute",
            "builds",
            "logs",
            "--build-id",
            "bld_test1234567890",
            "--follow",
        ])
        .output()
        .expect("run tachyon compute builds logs --follow");

    assert!(
        output.status.success(),
        "logs follow failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let first_req = rx.recv().unwrap();
    let second_req = rx.recv().unwrap();
    let third_req = rx.recv().unwrap();
    let fourth_req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(first_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));
    assert!(second_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs?next_token=n1 "));
    assert!(third_req.starts_with("GET /v1/compute/builds/bld_test1234567890/logs?next_token=n2 "));
    assert!(fourth_req.starts_with("GET /v1/compute/builds/bld_test1234567890 "));

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("build finished"), "stdout:\n{stdout}");
}

#[test]
fn compute_builds_logs_without_follow_fetches_once() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        r#"{"lines":[{"timestamp":1767225600000,"message":"first page"}],"next_token":"n1","is_complete":false}"#,
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "builds",
            "logs",
            "--build-id",
            "bld_test1234567890",
        ])
        .output()
        .expect("run tachyon compute builds logs");

    assert!(
        output.status.success(),
        "logs failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let request = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(request.starts_with("GET /v1/compute/builds/bld_test1234567890/logs "));

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("first page"), "stdout:\n{stdout}");
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
    assert!(first_req.starts_with(&format!("GET /v1/compute/apps/{app_id}/builds?limit=10 ")));
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
    assert!(first_req.starts_with(&format!("GET /v1/compute/apps/{app_id}/builds?limit=10 ")));
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
fn compute_builds_list_sends_limit_and_reports_stage_timings() {
    let tmp = TempDir::new().unwrap();
    let app_id = "app_01kp4vm07tr3d4375597d15gkp";
    let (api_url, rx, handle) = start_server(vec![
        r#"{"builds":[{"id":"bld_01kp4vm07tr3d4375597d15gka","app_id":"app_01kp4vm07tr3d4375597d15gkp","status":"succeeded"}]}"#,
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "builds",
            "list",
            app_id,
            "--limit",
            "3",
            "--timeout-secs",
            "1",
            "--verbose",
            "--json",
        ])
        .output()
        .expect("run tachyon compute builds list");

    assert!(
        output.status.success(),
        "builds list failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let request = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(request.starts_with(&format!("GET /v1/compute/apps/{app_id}/builds?limit=3 ")));

    let stderr = String::from_utf8(output.stderr).unwrap();
    for stage in [
        "[timing] resolve app:",
        "[timing] builds response headers:",
        "[timing] builds response body:",
        "[timing] builds response bytes:",
        "[timing] builds JSON decode:",
        "[timing] render output:",
        "[timing] command total:",
    ] {
        assert!(stderr.contains(stage), "missing {stage}\nstderr:\n{stderr}");
    }
    assert!(
        !stderr.contains("Still waiting"),
        "fast request should not emit progress\nstderr:\n{stderr}"
    );
}

#[test]
fn compute_builds_list_times_out_slow_body_with_progress() {
    let tmp = TempDir::new().unwrap();
    let app_id = "app_01kp4vm07tr3d4375597d15gkp";
    let (api_url, rx, handle) = start_server_with_body_delays(vec![(
        Duration::from_secs(4),
        r#"{"builds":[{"id":"bld_01kp4vm07tr3d4375597d15gka","app_id":"app_01kp4vm07tr3d4375597d15gkp","status":"succeeded"}]}"#,
    )]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "builds",
            "list",
            app_id,
            "--limit",
            "3",
            "--timeout-secs",
            "3",
            "--verbose",
            "--json",
        ])
        .output()
        .expect("run tachyon compute builds list");

    assert!(
        !output.status.success(),
        "slow builds response unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let request = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(request.starts_with(&format!("GET /v1/compute/apps/{app_id}/builds?limit=3 ")));

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("[timing] builds response headers:"),
        "stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("Still waiting for builds response body... 2s elapsed"),
        "stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("timed out waiting for builds response body after 3s"),
        "stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("[timing] command total:"),
        "stderr:\n{stderr}"
    );
}

#[test]
fn compute_builds_list_warns_and_succeeds_when_preview_body_times_out() {
    let tmp = TempDir::new().unwrap();
    let app_id = "app_01kp4vm07tr3d4375597d15gkp";
    let build_id = "bld_01kp4vm07tr3d4375597d15gka";
    let (api_url, rx, handle) = start_server_with_body_delays(vec![
        (
            Duration::ZERO,
            r#"{"builds":[{"id":"bld_01kp4vm07tr3d4375597d15gka","app_id":"app_01kp4vm07tr3d4375597d15gkp","status":"succeeded"}]}"#,
        ),
        (Duration::from_secs(2), r#"{"deployments":[]}"#),
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute",
            "builds",
            "list",
            app_id,
            "--limit",
            "3",
            "--preview-timeout-secs",
            "1",
        ])
        .output()
        .expect("run tachyon compute builds list");

    assert!(
        output.status.success(),
        "preview timeout should preserve the build list\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let first_request = rx.recv().unwrap();
    let second_request = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(first_request.starts_with(&format!("GET /v1/compute/apps/{app_id}/builds?limit=3 ")));
    assert!(second_request.starts_with(&format!(
        "GET /v1/compute/apps/{app_id}/deployments?environment=preview "
    )));

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(build_id), "stdout:\n{stdout}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains(
            "Warning: preview URLs unavailable: timed out waiting for preview deployments response body after 1s"
        ),
        "stderr:\n{stderr}"
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
