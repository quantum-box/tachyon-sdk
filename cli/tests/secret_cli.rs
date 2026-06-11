use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Stdio};
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
        .env("TACHYON_API_KEY", "tachyon-token")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_TENANT_ID")
        .env_remove("TACHYON_PROFILE")
        .env_remove("TACHYON_SECRET_VALUE")
        .env_remove("CLOUDFLARE_ACCOUNT_ID")
        .env_remove("CLOUDFLARE_API_TOKEN");
    cmd
}

fn write_cloud_app_config(dir: &Path, name: &str) {
    fs::write(
        dir.join("tachyon.yml"),
        format!(
            "apiVersion: tachyon/v1\nkind: CloudApp\nmetadata:\n  name: {name}\nspec:\n  framework: vite\n"
        ),
    )
    .unwrap();
}

fn write_cloud_apps_config(dir: &Path, app_name: &str) {
    fs::write(
        dir.join("tachyon.yml"),
        format!(
            "apiVersion: apps.tachy.one/v1alpha\nkind: CloudApps\nmetadata:\n  name: collection\nspec:\n  apps:\n  - name: {app_name}\n    framework: vite\n"
        ),
    )
    .unwrap();
}

fn start_cloudflare_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for idx in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            tx.send(req.clone()).unwrap();

            let body = if idx == 0
                && req.starts_with("GET /accounts/acct_123/pages/projects/configured-app ")
            {
                r#"{"success":true,"errors":[],"messages":[],"result":{"deployment_configs":{"production":{"env_vars":{"EXISTING":{"type":"plain_text","value":"plain"}}},"preview":{"env_vars":{}}}}}"#
            } else if idx == 1
                && req.starts_with("PATCH /accounts/acct_123/pages/projects/configured-app ")
            {
                r#"{"success":true,"errors":[],"messages":[],"result":{"deployment_configs":{"production":{"env_vars":{}}}}}"#
            } else {
                r#"{"success":false,"errors":[{"message":"unexpected request"}],"messages":[],"result":null}"#
            };
            let status = if body.contains("unexpected request") {
                "HTTP/1.1 500 Internal Server Error"
            } else {
                "HTTP/1.1 200 OK"
            };
            let response = format!(
                "{status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (url, rx, handle)
}

#[test]
fn secret_set_patches_cloudflare_pages_secret_text_binding() {
    let tmp = TempDir::new().unwrap();
    let (cf_url, rx, handle) = start_preview_cloudflare_server();

    let mut cmd = isolated_command(tmp.path());
    cmd.current_dir(tmp.path())
        .env("TACHYON_CLOUDFLARE_API_URL", cf_url)
        .args([
            "secret",
            "set",
            "API_KEY",
            "--from-stdin",
            "--account-id",
            "acct",
            "--project-name",
            "proj",
            "--environment",
            "preview",
            "--api-token",
            "cf-token",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn tachyon secret set");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"test-value\n")
        .unwrap();
    let output = child.wait_with_output().expect("run tachyon secret set");

    assert!(
        output.status.success(),
        "secret set failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let get_req = rx.recv().unwrap();
    let patch_req = rx.recv().unwrap();
    handle.join().unwrap();

    assert!(get_req.starts_with("GET /accounts/acct/pages/projects/proj "));
    assert!(get_req.contains("authorization: Bearer cf-token"));
    assert!(patch_req.starts_with("PATCH /accounts/acct/pages/projects/proj "));
    assert!(patch_req.contains("authorization: Bearer cf-token"));

    let patch_body = http_body(&patch_req);
    let patch_json: serde_json::Value = serde_json::from_str(patch_body).unwrap();
    assert_eq!(
        patch_json["deployment_configs"]["preview"]["env_vars"]["API_KEY"]["type"],
        "secret_text"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("Set secret API_KEY"));
    assert!(!stdout.contains("test-value"));
    assert!(!stderr.contains("test-value"));
}

fn start_preview_cloudflare_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for idx in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            tx.send(req.clone()).unwrap();

            let body = if idx == 0 && req.starts_with("GET /accounts/acct/pages/projects/proj ") {
                r#"{"success":true,"errors":[],"messages":[],"result":{"deployment_configs":{"preview":{"env_vars":{"EXISTING":{"type":"plain_text","value":"plain"}}}}}}"#
            } else if idx == 1 && req.starts_with("PATCH /accounts/acct/pages/projects/proj ") {
                r#"{"success":true,"errors":[],"messages":[],"result":{"deployment_configs":{"preview":{"env_vars":{}}}}}"#
            } else {
                r#"{"success":false,"errors":[{"message":"unexpected request"}],"messages":[],"result":null}"#
            };
            let status = if body.contains("unexpected request") {
                "HTTP/1.1 500 Internal Server Error"
            } else {
                "HTTP/1.1 200 OK"
            };
            let response = format!(
                "{status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (url, rx, handle)
}

fn http_body(request: &str) -> &str {
    request.split("\r\n\r\n").nth(1).unwrap_or_default()
}

#[test]
fn secret_set_with_env_value_patches_cloudflare_pages_secret_text_binding() {
    let tmp = TempDir::new().unwrap();
    write_cloud_app_config(tmp.path(), "configured-app");
    let (cf_url, rx, handle) = start_cloudflare_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_CLOUDFLARE_API_URL", cf_url)
        .env("CLOUDFLARE_ACCOUNT_ID", "acct_123")
        .env("CLOUDFLARE_API_TOKEN", "cf-token")
        .env("TACHYON_SECRET_VALUE", "test-value")
        .args(["secret", "set", "RESEND_API_KEY"])
        .output()
        .expect("run tachyon secret set");

    assert!(
        output.status.success(),
        "secret set failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let get_req = rx.recv().unwrap();
    let patch_req = rx.recv().unwrap();
    handle.join().unwrap();

    assert!(get_req.starts_with("GET /accounts/acct_123/pages/projects/configured-app "));
    assert!(get_req.contains("authorization: Bearer cf-token"));
    assert!(patch_req.starts_with("PATCH /accounts/acct_123/pages/projects/configured-app "));
    let patch_body = http_body(&patch_req);
    let patch_json: serde_json::Value = serde_json::from_str(patch_body).unwrap();
    let env_vars = &patch_json["deployment_configs"]["production"]["env_vars"];
    assert_eq!(env_vars["EXISTING"]["type"], "plain_text");
    assert_eq!(env_vars["RESEND_API_KEY"]["type"], "secret_text");
    assert!(env_vars["RESEND_API_KEY"]["value"].is_string());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("Set secret RESEND_API_KEY"));
    assert!(!stdout.contains("test-value"));
    assert!(!stderr.contains("test-value"));
}

#[test]
fn secret_set_rejects_value_and_stdin_before_api_call() {
    let tmp = TempDir::new().unwrap();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .args([
            "secret",
            "set",
            "API_KEY",
            "--value",
            "test-value",
            "--from-stdin",
            "--account-id",
            "acct",
            "--project-name",
            "proj",
            "--api-token",
            "cf-token",
        ])
        .output()
        .expect("run tachyon secret set");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot be used with"));
    assert!(!stderr.contains("test-value"));
}

#[test]
fn secret_set_fails_before_api_call_without_account_id() {
    let tmp = TempDir::new().unwrap();
    write_cloud_app_config(tmp.path(), "configured-app");

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("CLOUDFLARE_API_TOKEN", "cf-token")
        .env("TACHYON_SECRET_VALUE", "test-value")
        .args(["secret", "set", "RESEND_API_KEY"])
        .output()
        .expect("run tachyon secret set");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cloudflare account ID is required"));
    assert!(!stderr.contains("test-value"));
}

#[test]
fn secret_set_help_does_not_print_cloudflare_api_token_env_value() {
    let tmp = TempDir::new().unwrap();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("CLOUDFLARE_API_TOKEN", "cf-token-from-env")
        .args(["secret", "set", "--help"])
        .output()
        .expect("run tachyon secret set help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("CLOUDFLARE_API_TOKEN"));
    assert!(!stdout.contains("cf-token-from-env"));
    assert!(!stderr.contains("cf-token-from-env"));
}

#[test]
fn secret_set_resolves_single_cloud_apps_app_name() {
    let tmp = TempDir::new().unwrap();
    write_cloud_apps_config(tmp.path(), "configured-app");
    let (cf_url, rx, handle) = start_cloudflare_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_CLOUDFLARE_API_URL", cf_url)
        .env("CLOUDFLARE_ACCOUNT_ID", "acct_123")
        .env("CLOUDFLARE_API_TOKEN", "cf-token")
        .env("TACHYON_SECRET_VALUE", "test-value")
        .args(["secret", "set", "RESEND_API_KEY"])
        .output()
        .expect("run tachyon secret set");

    assert!(
        output.status.success(),
        "secret set failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let get_req = rx.recv().unwrap();
    let _patch_req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(get_req.starts_with("GET /accounts/acct_123/pages/projects/configured-app "));
}
