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

fn assert_ok(output: &std::process::Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
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

fn write_refreshable_profile(home: &Path) {
    let root = home.join(".config").join("tachyon");
    let profiles = root.join("profiles");
    fs::create_dir_all(&profiles).unwrap();
    fs::write(root.join("active_profile"), "default\n").unwrap();
    fs::write(
        profiles.join("default.json"),
        r#"{
  "access_token": "stale-access-token",
  "refresh_token": "refresh-token-for-api",
  "id_token": "stale-id-token",
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

fn start_apply_action_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 8192];
        let n = stream.read(&mut buf).unwrap();
        let req = String::from_utf8_lossy(&buf[..n]).to_string();
        tx.send(req).unwrap();

        let body = r#"{"id":"act_manifest_read","full_name":"manifest:Read"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, rx, handle)
}

fn start_failing_apply_action_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 8192];
        let n = stream.read(&mut buf).unwrap();
        let req = String::from_utf8_lossy(&buf[..n]).to_string();
        tx.send(req).unwrap();

        let body = r#"{"error":"action registry unavailable"}"#;
        let response = format!(
            "HTTP/1.1 500 Internal Server Error\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, rx, handle)
}

fn start_refresh_then_actions_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            tx.send(req.clone()).unwrap();

            let (status, body) = if req.starts_with("GET /v1/auth/actions")
                && req.contains("authorization: Bearer stale-access-token")
            {
                (
                    "401 Unauthorized",
                    r#"{"code":"UNAUTHORIZED","message":"expired"}"#,
                )
            } else if req.starts_with("POST /oauth2/token") {
                (
                    "200 OK",
                    r#"{"access_token":"fresh-access-token","refresh_token":"fresh-refresh-token","id_token":"fresh-id-token","expires_in":3600,"token_type":"Bearer"}"#,
                )
            } else if req.starts_with("GET /v1/auth/actions")
                && req.contains("authorization: Bearer fresh-access-token")
            {
                ("200 OK", r#"{"actions":[]}"#)
            } else {
                (
                    "500 Internal Server Error",
                    r#"{"error":"unexpected request"}"#,
                )
            };

            let response = format!(
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
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
fn auth_manifest_plan_refreshes_access_token_after_401() {
    let tmp = TempDir::new().unwrap();
    write_refreshable_profile(tmp.path());
    fs::write(
        tmp.path().join("auth.yml"),
        "actions:\n  - context: test\n    name: List\npolicies: []\n",
    )
    .unwrap();
    let (api_url, rx, handle) = start_refresh_then_actions_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", &api_url)
        .env("TACHYON_COGNITO_DOMAIN", &api_url)
        .env("TACHYON_COGNITO_CLIENT_ID", "client-id")
        .env("TACHYON_COGNITO_CLIENT_SECRET", "client-secret")
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
    let requests: Vec<String> = rx.try_iter().collect();
    assert_ok(&output, "auth manifest plan with refresh");
    assert_eq!(
        requests
            .iter()
            .filter(|r| r.starts_with("GET /v1/auth/actions"))
            .count(),
        2
    );
    assert!(requests.iter().any(|r| r.starts_with("POST /oauth2/token")));

    let profile =
        fs::read_to_string(tmp.path().join(".config/tachyon/profiles/default.json")).unwrap();
    assert!(profile.contains("fresh-access-token"));
    assert!(profile.contains("fresh-refresh-token"));
    let legacy = fs::read_to_string(tmp.path().join(".config/tachyon/credentials.json")).unwrap();
    assert!(legacy.contains("fresh-access-token"));
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

#[test]
fn manifest_validate_discovers_auth_manifest_without_tenant() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".git")).unwrap();
    fs::create_dir_all(tmp.path().join(".tachyon/manifests")).unwrap();
    fs::write(
        tmp.path().join(".tachyon/manifests/auth.yml"),
        "actions:\n  - context: manifest\n    name: Read\npolicies: []\n",
    )
    .unwrap();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .args(["manifest", "validate"])
        .output()
        .expect("run tachyon manifest validate");

    assert_ok(&output, "manifest validate");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Valid:"));
    assert!(stdout.contains(".tachyon/manifests/auth.yml"));
}

#[test]
fn manifest_apply_delegates_auth_manifest_file() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".git")).unwrap();
    fs::write(
        tmp.path().join("tachyon.yml"),
        "metadata:\n  tenantId: tn_manifestapply123456\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join(".tachyon/manifests")).unwrap();
    fs::write(
        tmp.path().join(".tachyon/manifests/auth.yml"),
        "actions:\n  - context: manifest\n    name: Read\npolicies: []\n",
    )
    .unwrap();
    let (api_url, rx, handle) = start_apply_action_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .args(["manifest", "apply", "--file", ".tachyon/manifests/auth.yml"])
        .output()
        .expect("run tachyon manifest apply auth");

    handle.join().unwrap();
    let req = rx.recv().unwrap();
    assert_ok(&output, "manifest apply auth");
    assert!(req.starts_with("POST /v1/auth/actions "));
    assert!(req.contains("x-operator-id: tn_manifestapply123456"));
    assert!(req.contains("\"context\":\"manifest\""));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("=== Auth Manifest Apply ==="));
    assert!(stdout.contains("manifest:Read"));
}

#[test]
fn manifest_apply_fails_when_auth_apply_reports_resource_errors() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".git")).unwrap();
    fs::write(
        tmp.path().join("tachyon.yml"),
        "metadata:\n  tenantId: tn_manifestapply123456\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join(".tachyon/manifests")).unwrap();
    fs::write(
        tmp.path().join(".tachyon/manifests/auth.yml"),
        "actions:\n  - context: manifest\n    name: Read\npolicies: []\n",
    )
    .unwrap();
    let (api_url, rx, handle) = start_failing_apply_action_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .args(["manifest", "apply", "--file", ".tachyon/manifests/auth.yml"])
        .output()
        .expect("run tachyon manifest apply auth");

    handle.join().unwrap();
    let req = rx.recv().unwrap();
    assert!(req.starts_with("POST /v1/auth/actions "));
    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("=== Auth Manifest Apply ==="));
    assert!(stdout.contains("manifest:Read"));
    assert!(stdout.contains("ERROR"));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Auth manifest apply: some resources failed."));
    assert!(stderr.contains("Manifest apply completed with 1 error(s):"));
    assert!(stderr.contains("1 manifest step(s) failed"));
}

#[test]
fn manifest_apply_reports_unsupported_iac_manifest_without_api_call() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".git")).unwrap();
    fs::write(
        tmp.path().join("operator.yml"),
        "apiVersion: apps.tachy.one/v1alpha\nkind: Operator\nmetadata:\n  name: test\n",
    )
    .unwrap();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_TENANT_ID", "tn_test1234567890")
        .args(["manifest", "apply", "--file", "operator.yml"])
        .output()
        .expect("run tachyon manifest apply iac");

    assert_ok(&output, "manifest apply unsupported iac");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("IaC manifest:"));
    assert!(stdout.contains("apply/reconcile is not supported yet"));
}

#[test]
fn manifest_apply_aggregates_invalid_cloud_apps_after_iac_skip() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".git")).unwrap();
    fs::write(
        tmp.path().join("tachyon.yml"),
        r#"apiVersion: apps.tachy.one/v1alpha
kind: Operator
metadata:
  name: test
---
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: broken
"#,
    )
    .unwrap();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_TENANT_ID", "tn_test1234567890")
        .args(["manifest", "apply", "--file", "tachyon.yml"])
        .output()
        .expect("run tachyon manifest apply mixed invalid manifests");

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("IaC manifest:"));
    assert!(stdout.contains("apply/reconcile is not supported yet"));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid Cloud Apps manifest"));
    assert!(stderr.contains("Manifest apply completed with 1 error(s):"));
    assert!(stderr.contains("1 manifest step(s) failed"));
}
