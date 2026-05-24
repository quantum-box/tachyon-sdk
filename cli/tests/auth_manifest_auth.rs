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

fn start_actions_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 8192];
        let n = stream.read(&mut buf).unwrap();
        let req = String::from_utf8_lossy(&buf[..n]).to_string();
        tx.send(req).unwrap();

        let body = r#"{"actions":[]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
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

#[test]
fn auth_manifest_plan_infers_tenant_from_parent_tachyon_yml_for_dot_tachyon_file() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".git")).unwrap();
    fs::create_dir_all(tmp.path().join(".tachyon/manifests")).unwrap();
    fs::write(
        tmp.path().join("tachyon.yml"),
        "apiVersion: tachyon/v1\nkind: CloudApp\nmetadata:\n  name: tachyon-field\n  tenantId: tn_01hjjn348rn3t49zz6hvmfq67p\nspec:\n  framework: static\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join(".tachyon/manifests/tachyon-field-auth.yml"),
        "actions:\n  - context: tachyonfield\n    name: ListRecords\npolicies: []\n",
    )
    .unwrap();
    let (api_url, rx, handle) = start_actions_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .args([
            "auth",
            "manifest",
            "plan",
            "--file",
            ".tachyon/manifests/tachyon-field-auth.yml",
        ])
        .output()
        .expect("run tachyon auth manifest plan");

    handle.join().unwrap();
    let req = rx.recv().unwrap();
    assert!(
        output.status.success(),
        "plan failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(req.starts_with("GET /v1/auth/actions "));
    assert!(req.contains("x-operator-id: tn_01hjjn348rn3t49zz6hvmfq67p"));
}

#[test]
fn auth_manifest_validate_infers_snake_case_tenant_from_parent_when_file_is_nested() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".git")).unwrap();
    fs::create_dir_all(tmp.path().join(".tachyon/manifests")).unwrap();
    fs::create_dir_all(tmp.path().join("work/nested")).unwrap();
    fs::write(
        tmp.path().join("tachyon.yml"),
        "apiVersion: tachyon/v1\nkind: CloudApp\nmetadata:\n  name: snake\n  tenant_id: tn_snake1234567890\nspec:\n  framework: static\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join(".tachyon/manifests/auth.yml"),
        "actions:\n  - context: snake\n    name: Read\npolicies: []\n",
    )
    .unwrap();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path().join("work/nested"))
        .args([
            "auth",
            "manifest",
            "validate",
            "--file",
            "../../.tachyon/manifests/auth.yml",
        ])
        .output()
        .expect("run tachyon auth manifest validate");

    assert!(
        output.status.success(),
        "validate failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("All manifests are valid."));
}

#[test]
fn auth_manifest_reports_actionable_error_when_tenant_cannot_be_resolved() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".git")).unwrap();
    fs::create_dir_all(tmp.path().join(".tachyon/manifests")).unwrap();
    fs::write(
        tmp.path().join(".tachyon/manifests/auth.yml"),
        "actions:\n  - context: missing\n    name: Read\npolicies: []\n",
    )
    .unwrap();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .args([
            "auth",
            "manifest",
            "validate",
            "--file",
            ".tachyon/manifests/auth.yml",
        ])
        .output()
        .expect("run tachyon auth manifest validate");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("tenant could not be resolved for auth manifest"));
    assert!(stderr.contains("--tenant-id"));
    assert!(stderr.contains("TACHYON_TENANT_ID"));
    assert!(stderr.contains("metadata.tenantId/metadata.tenant_id"));
    assert!(stderr.contains(".tachyon/manifests/tachyon.yml"));
    assert!(stderr.contains("tachyon.yml"));
}

#[test]
fn reconcile_infers_tenant_from_project_config() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".git")).unwrap();
    fs::create_dir_all(tmp.path().join(".tachyon/manifests")).unwrap();
    fs::write(
        tmp.path().join("tachyon.yml"),
        "metadata:\n  tenantId: tn_reconcile1234567890\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join(".tachyon/manifests/auth.yml"),
        "actions:\n  - context: reconcile\n    name: Read\npolicies: []\n",
    )
    .unwrap();
    let (api_url, rx, handle) = start_actions_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .args(["reconcile", "--dry-run", "--json"])
        .output()
        .expect("run tachyon reconcile");

    handle.join().unwrap();
    let req = rx.recv().unwrap();
    assert!(
        output.status.success(),
        "reconcile failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(req.starts_with("GET /v1/auth/actions "));
    assert!(req.contains("x-operator-id: tn_reconcile1234567890"));
}
