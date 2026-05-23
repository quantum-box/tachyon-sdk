use std::fs;
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
        .env_remove("TACHYON_API_KEY")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE")
        .env_remove("TACHYON_TENANT_ID");
    cmd
}

fn write_profile(home: &Path) {
    let root = home.join(".config").join("tachyon");
    let profiles = root.join("profiles");
    fs::create_dir_all(&profiles).unwrap();
    fs::write(root.join("active_profile"), "default\n").unwrap();
    fs::write(
        profiles.join("default.json"),
        r#"{
  "access_token": "access-token-for-api",
  "refresh_token": null,
  "id_token": "id-token-not-selected",
  "expires_at": null,
  "token_type": "Bearer",
  "operator_id": "tn_test1234567890"
}"#,
    )
    .unwrap();
}

fn start_unauthorized_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 8192];
        let n = stream.read(&mut buf).unwrap();
        let req = String::from_utf8_lossy(&buf[..n]).to_string();
        tx.send(req).unwrap();

        let body = r#"{"code":"UNAUTHORIZED","message":"verify token failed"}"#;
        let response = format!(
            "HTTP/1.1 401 Unauthorized\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, rx, handle)
}

#[test]
fn auth_manifest_plan_uses_access_token_and_explains_unauthorized() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path());
    fs::write(
        tmp.path().join("auth.yml"),
        "actions:\n  - context: test\n    name: List\npolicies: []\n",
    )
    .unwrap();
    let (api_url, rx, handle) = start_unauthorized_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "--tenant-id",
            "tn_test1234567890",
            "auth",
            "manifest",
            "plan",
            "--file",
            "auth.yml",
        ])
        .output()
        .expect("run tachyon auth manifest plan");

    handle.join().unwrap();
    let req = rx.recv().unwrap();
    assert!(req.starts_with("GET /v1/auth/actions "));
    assert!(req.contains("authorization: Bearer access-token-for-api"));
    assert!(!req.contains("id-token-not-selected"));

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Authentication diagnostics"));
    assert!(stderr.contains("profile='default'"));
    assert!(stderr.contains("token_kind='access_token'"));
    assert!(stderr.contains("issuer"));
    assert!(stderr.contains("audience"));
    assert!(stderr.contains("COGNITO_ALLOWED_CLIENT_IDS"));
}
