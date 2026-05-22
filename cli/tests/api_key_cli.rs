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
    status: &'static str,
    body: &'static str,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 8192];
        let n = stream.read(&mut buf).unwrap();
        tx.send(String::from_utf8_lossy(&buf[..n]).to_string())
            .unwrap();

        let response = if body.is_empty() {
            format!("HTTP/1.1 {status}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n")
        } else {
            format!(
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            )
        };
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, rx, handle)
}

#[test]
fn api_key_create_posts_service_account_request() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(
        "201 Created",
        r#"{"id":"key_123456789012","serviceAccountId":"sa_123456789012","name":"CEO key","value":"tchy_live_secret_value","createdAt":"2026-05-21T00:00:00Z"}"#,
    );

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "api-key",
            "create",
            "sa_123456789012",
            "--name",
            "CEO key",
            "--json",
        ])
        .output()
        .expect("run tachyon api-key create");

    assert!(
        output.status.success(),
        "api-key create failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(req.starts_with("POST /v1/auth/service-accounts/sa_123456789012/api-keys "));
    assert!(req.contains("authorization: Bearer test-token"));
    assert!(req.contains("x-operator-id: tn_test1234567890"));
    assert!(req.contains(r#""name":"CEO key""#));
    assert!(req.contains(r#""operatorId":"tn_test1234567890""#));

    let key: serde_json::Value = serde_json::from_slice(&output.stdout).expect("key json");
    assert_eq!(key["id"], "key_123456789012");
    assert_eq!(key["value"], "tchy_live_secret_value");
}

#[test]
fn api_key_list_decodes_openapi_envelope() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(
        "200 OK",
        r#"{"apiKeys":[{"id":"key_123456789012","serviceAccountId":"sa_123456789012","name":"CEO key","createdAt":"2026-05-21T00:00:00Z"}]}"#,
    );

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args(["api-key", "list", "sa_123456789012", "--json"])
        .output()
        .expect("run tachyon api-key list");

    assert!(
        output.status.success(),
        "api-key list failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(req.starts_with(
        "GET /v1/auth/service-accounts/sa_123456789012/api-keys?operator_id=tn_test1234567890 "
    ));

    let keys: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).expect("keys json");
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0]["id"], "key_123456789012");
}

#[test]
fn api_key_revoke_posts_documented_revoke_contract() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server("204 No Content", "");

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "api-key",
            "revoke",
            "sa_123456789012",
            "key_123456789012",
            "--json",
        ])
        .output()
        .expect("run tachyon api-key revoke");

    assert!(
        output.status.success(),
        "api-key revoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(req.starts_with(
        "POST /v1/auth/service-accounts/sa_123456789012/api-keys/key_123456789012/revoke "
    ));
    assert!(req.contains(r#""operatorId":"tn_test1234567890""#));

    let body: serde_json::Value = serde_json::from_slice(&output.stdout).expect("revoke json");
    assert_eq!(body["id"], "key_123456789012");
    assert_eq!(body["revoked"], true);
}
