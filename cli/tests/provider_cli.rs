use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

use tempfile::TempDir;

const SECRET_KEY: &str = "sk_test_abcdefghijklmnopqrstuvwxyz";
const PUBLISHABLE_KEY: &str = "pk_test_abcdefghijklmnopqrstuvwxyz";
const WEBHOOK_SECRET: &str = "whsec_account";
const CONNECT_WEBHOOK_SECRET: &str = "whsec_connect";
const TENANT_ID: &str = "tn_01m2ae5hbtwqf42wshj7r4efqt";

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_tachyon")
}

fn isolated_command(home: &Path) -> Command {
    let mut command = Command::new(bin());
    command
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_API_KEY", "test-token")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_TENANT_ID")
        .env_remove("TACHYON_PLATFORM_ID")
        .env_remove("TACHYON_PROFILE");
    command
}

fn start_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 8192];
        let n = stream.read(&mut buf).unwrap();
        tx.send(String::from_utf8_lossy(&buf[..n]).to_string())
            .unwrap();

        let body = format!(
            r#"{{"tenant_id":"{TENANT_ID}","environment":"test","has_webhook_secret":true,"has_connect_webhook_secret":true}}"#
        );
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, rx, handle)
}

fn request_body(request: &str) -> &str {
    request.split("\r\n\r\n").nth(1).unwrap_or_default()
}

#[test]
fn stripe_credentials_set_sends_secrets_once_without_printing_them() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server();
    let input = serde_json::json!({
        "secret_key": SECRET_KEY,
        "publishable_key": PUBLISHABLE_KEY,
        "webhook_secret": WEBHOOK_SECRET,
        "connect_webhook_secret": CONNECT_WEBHOOK_SECRET,
    })
    .to_string();

    let mut command = isolated_command(tmp.path());
    command
        .env("TACHYON_API_URL", api_url)
        .args([
            "provider",
            "stripe",
            "credentials",
            "set",
            "--tenant-id",
            TENANT_ID,
            "--platform-id",
            TENANT_ID,
            "--from-stdin",
            "--yes",
            "--json",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("spawn provider credential command");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().expect("run provider command");

    assert!(
        output.status.success(),
        "provider command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let request = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(request.starts_with("PUT /v1/providers/stripe/credentials "));
    assert!(request.contains("authorization: Bearer test-token"));
    assert!(request.contains(&format!("x-operator-id: {TENANT_ID}")));
    assert!(request.contains(&format!("x-platform-id: {TENANT_ID}")));
    let body: serde_json::Value = serde_json::from_str(request_body(&request)).unwrap();
    assert_eq!(body["secret_key"], SECRET_KEY);
    assert_eq!(body["publishable_key"], PUBLISHABLE_KEY);
    assert_eq!(body["webhook_secret"], WEBHOOK_SECRET);
    assert_eq!(body["connect_webhook_secret"], CONNECT_WEBHOOK_SECRET);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    for secret in [
        SECRET_KEY,
        PUBLISHABLE_KEY,
        WEBHOOK_SECRET,
        CONNECT_WEBHOOK_SECRET,
    ] {
        assert!(!stdout.contains(secret));
        assert!(!stderr.contains(secret));
    }
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["tenant_id"], TENANT_ID);
    assert_eq!(result["environment"], "test");
}

#[test]
fn stripe_credentials_set_reads_tenant_from_manifest() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("tachyon.yml"),
        format!(
            "apiVersion: apps.tachy.one/v1alpha\nkind: ProjectConfig\nmetadata:\n  name: default\n  tenantId: {TENANT_ID}\nspec:\n  providers: []\n"
        ),
    )
    .unwrap();
    let (api_url, rx, handle) = start_server();
    let input = serde_json::json!({
        "secret_key": SECRET_KEY,
        "publishable_key": PUBLISHABLE_KEY,
    })
    .to_string();

    let mut command = isolated_command(tmp.path());
    command
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "provider",
            "stripe",
            "credentials",
            "set",
            "--from-stdin",
            "--yes",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("spawn provider credential command");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().expect("run provider command");

    assert!(
        output.status.success(),
        "provider command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let request = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(request.contains(&format!("x-operator-id: {TENANT_ID}")));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Key material was not stored by the CLI.")
    );
}

#[test]
fn stripe_credentials_set_requires_explicit_tenant_context() {
    let tmp = TempDir::new().unwrap();
    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .args([
            "provider",
            "stripe",
            "credentials",
            "set",
            "--from-stdin",
            "--yes",
        ])
        .output()
        .expect("run provider command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("tenant could not be resolved for provider credentials"));
}

#[test]
fn stripe_credentials_set_rejects_stdin_without_confirmation() {
    let tmp = TempDir::new().unwrap();
    let output = isolated_command(tmp.path())
        .args([
            "provider",
            "stripe",
            "credentials",
            "set",
            "--tenant-id",
            TENANT_ID,
            "--from-stdin",
        ])
        .output()
        .expect("run provider command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--from-stdin requires --yes"));
}
