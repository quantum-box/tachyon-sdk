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
    let mut command = Command::new(bin());
    command
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_TENANT_ID", "tn_test1234567890")
        .env("TACHYON_API_KEY", "test-token")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE")
        .env_remove("TACHYON_SLACK_BOT_TOKEN")
        .env_remove("SLACK_BOT_TOKEN");
    command
}

fn start_server(body: &'static str) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    start_server_sequence(vec![body])
}

fn start_server_sequence(
    bodies: Vec<&'static str>,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for body in bodies {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0_u8; 16384];
            let bytes_read = stream.read(&mut buffer).unwrap();
            tx.send(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
                .unwrap();

            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\
                 content-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (url, rx, handle)
}

fn request_json_body(request: &str) -> Value {
    let body = request.split("\r\n\r\n").nth(1).unwrap();
    serde_json::from_str(body).unwrap()
}

#[test]
fn notify_users_uses_tenant_api_without_slack_token() {
    let temp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(
        r#"{"users":[{"id":"U123","name":"taka","email":"taka@example.com","display_name":"Taka","real_name":"Takanori Fukuyama"}]}"#,
    );

    let output = isolated_command(temp.path())
        .env("TACHYON_API_URL", api_url)
        .args(["ops", "notify", "users", "--json"])
        .output()
        .expect("run tachyon ops notify users");

    assert!(
        output.status.success(),
        "notify users failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    handle.join().unwrap();
    let request = rx.recv().unwrap();
    assert!(request.starts_with("GET /v1/chat/users "));
    assert!(request.contains("x-operator-id: tn_test1234567890"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("taka@example.com"));
}

#[test]
fn notify_send_passes_email_mention_to_server() {
    let temp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(r#"{"accepted":true}"#);

    let output = isolated_command(temp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "ops",
            "notify",
            "send",
            "--text",
            "deploy complete",
            "--mention",
            "taka@example.com",
        ])
        .output()
        .expect("run tachyon ops notify send");

    assert!(
        output.status.success(),
        "notify send failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    handle.join().unwrap();
    let request = rx.recv().unwrap();
    assert!(request.starts_with("POST /v1/chat/send "));
    assert_eq!(
        request_json_body(&request),
        serde_json::json!({
            "text": "deploy complete",
            "mentions": ["taka@example.com"]
        })
    );
}

#[test]
fn notify_send_validates_and_preserves_configured_project_group_alias() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("tachyon.yml");
    std::fs::write(
        &config_path,
        r#"apiVersion: apps.tachy.one/v1alpha
kind: ProjectConfig
metadata:
  name: default
  tenantId: tn_test1234567890
spec:
  chat:
    slack:
      workspaceId: T123
      mentionAliases:
        - alias: on-call
          kind: user_group
          groupId: S123
"#,
    )
    .unwrap();
    let (api_url, rx, handle) = start_server_sequence(vec![
        r#"{"user_groups":[{"id":"S123","name":"On-call Team","handle":"on-call-team","disabled":false}]}"#,
        r#"{"accepted":true}"#,
    ]);

    let output = isolated_command(temp.path())
        .env("TACHYON_API_URL", api_url)
        .arg("--config")
        .arg(&config_path)
        .args([
            "ops",
            "notify",
            "send",
            "--text",
            "deploy complete",
            "--mention",
            "on-call",
        ])
        .output()
        .expect("run tachyon ops notify send with ProjectConfig alias");

    assert!(
        output.status.success(),
        "notify send failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    handle.join().unwrap();
    let directory_request = rx.recv().unwrap();
    assert!(directory_request.starts_with("GET /v1/chat/user-groups HTTP/1.1"));
    let send_request = rx.recv().unwrap();
    assert!(send_request.starts_with("POST /v1/chat/send "));
    assert_eq!(
        request_json_body(&send_request),
        serde_json::json!({
            "text": "deploy complete",
            "mentions": ["on-call"]
        })
    );
}

#[test]
fn ops_slack_send_normalizes_mixed_mentions_and_dedupes() {
    let temp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server_sequence(vec![
        r#"{"user_groups":[{"id":"S123","name":"Platform Team","handle":"platform-team","disabled":false}]}"#,
        r#"{"accepted":true}"#,
    ]);

    let output = isolated_command(temp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "ops",
            "slack",
            "send",
            "--text",
            "deploy complete",
            "--mention",
            "<@U123|taka>",
            "--mention",
            "U123",
            "--mention",
            "Platform Team",
            "--mention",
            "@platform-team",
            "--mention",
            "<!subteam^S123|platform>",
            "--mention",
            "S123",
            "--mention",
            "@here",
            "--mention",
            "on-call",
        ])
        .output()
        .expect("run tachyon ops slack send");

    assert!(
        output.status.success(),
        "ops slack send failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    handle.join().unwrap();
    let directory_request = rx.recv().unwrap();
    assert!(directory_request.starts_with("GET /v1/chat/user-groups HTTP/1.1"));
    assert!(!directory_request.starts_with("GET /v1/chat/user-groups?"));

    let send_request = rx.recv().unwrap();
    assert!(send_request.starts_with("POST /v1/chat/send "));
    assert_eq!(
        request_json_body(&send_request),
        serde_json::json!({
            "text": "deploy complete",
            "mentions": [
                "<@U123>",
                "<!subteam^S123>",
                "<!here>",
                "on-call"
            ]
        })
    );
}

#[test]
fn notify_send_help_documents_slack_team_selectors() {
    let temp = TempDir::new().unwrap();
    let output = isolated_command(temp.path())
        .args(["ops", "notify", "send", "--help"])
        .output()
        .expect("show tachyon ops notify send help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Team selectors accept a display name"));
    assert!(stdout.contains("@handle"));
    assert!(stdout.contains("User Group ID (S...)"));
    assert!(stdout.contains("<!subteam^S...>"));
}

#[test]
fn notify_send_rejects_ambiguous_team_name_before_posting() {
    let temp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(
        r#"{"user_groups":[{"id":"S100","name":"Platform","handle":"platform-apac","disabled":false},{"id":"S200","name":"Platform","handle":"platform-emea","disabled":false}]}"#,
    );

    let output = isolated_command(temp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "ops",
            "notify",
            "send",
            "--text",
            "deploy complete",
            "--mention",
            "Platform",
        ])
        .output()
        .expect("run tachyon ops notify send");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("is ambiguous"));
    assert!(stderr.contains("S100"));
    assert!(stderr.contains("S200"));
    assert!(stderr.contains("use @handle or a User Group ID"));

    handle.join().unwrap();
    let request = rx.recv().unwrap();
    assert!(request.starts_with("GET /v1/chat/user-groups HTTP/1.1"));
}

#[test]
fn notify_send_rejects_disabled_raw_team_id_before_posting() {
    let temp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(
        r#"{"user_groups":[{"id":"S123","name":"Former Team","handle":"former-team","disabled":true}]}"#,
    );

    let output = isolated_command(temp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "ops",
            "slack",
            "send",
            "--text",
            "deploy complete",
            "--mention",
            "<!subteam^S123>",
        ])
        .output()
        .expect("run tachyon ops slack send");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Former Team"));
    assert!(stderr.contains("is disabled"));
    assert!(stderr.contains("notification was not sent"));

    handle.join().unwrap();
    let request = rx.recv().unwrap();
    assert!(request.starts_with("GET /v1/chat/user-groups HTTP/1.1"));
}
