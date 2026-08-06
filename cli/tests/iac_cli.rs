use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
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
        .env_remove("TACHYON_PROFILE")
        .env_remove("TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN")
        .env_remove("TACHYON_CHANGE_CONTROL_VERIFICATION_KEY");
    cmd
}

/// Structurally valid fixture accepted by the client-side verifier when no
/// local HMAC key is configured. This is not a real approval credential.
fn production_change_control_token() -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    let payload = r#"{"ref":"PLT-3160-test","env":"production","exp":4102444800}"#;
    format!("tcct.v1.{}.fixture", URL_SAFE_NO_PAD.encode(payload))
}

fn history_response(revision: Option<i32>) -> String {
    let history = revision
        .map(|revision| {
            vec![json!({
                "revision": revision,
                "contentHash": "fixture-content-hash",
                "appliedBy": "test-runner",
                "appliedAt": "2026-08-06T00:00:00Z",
                "manifest": "{}",
            })]
        })
        .unwrap_or_default();
    json!({ "data": { "manifestHistory": history } }).to_string()
}

fn start_graphql_server(
    responses: Vec<String>,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for body in responses {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let mut buf = [0_u8; 16_384];
                        let n = stream.read(&mut buf).unwrap();
                        tx.send(String::from_utf8_lossy(&buf[..n]).to_string())
                            .unwrap();

                        let response = format!(
                            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        stream.write_all(response.as_bytes()).unwrap();
                        break;
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            tx.send(String::new()).unwrap();
                            return;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("accept graphql request: {err}"),
                }
            }
        }
    });
    (url, rx, handle)
}

fn request_body(request: &str) -> &str {
    request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap()
}

fn request_json(request: &str) -> serde_json::Value {
    serde_json::from_str(request_body(request)).unwrap()
}

fn has_change_control_header(request: &str) -> bool {
    request.lines().any(|line| {
        line.to_ascii_lowercase()
            .starts_with("x-tachyon-change-control-token:")
    })
}

fn assert_mutation_contract(
    request: &str,
    operation: &str,
    expected_revision: Option<i32>,
    token: Option<&str>,
) {
    let body = request_json(request);
    assert!(
        body.get("query")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|query| query.contains(operation)),
        "unexpected GraphQL operation"
    );
    assert_eq!(has_change_control_header(request), token.is_some());
    if let Some(token) = token {
        assert!(!request_body(request).contains(token));
    }
    if let Some(expected_revision) = expected_revision {
        assert_eq!(
            body.pointer("/variables/input/expectedRevision")
                .and_then(serde_json::Value::as_i64),
            Some(i64::from(expected_revision))
        );
    }
}

#[test]
fn iac_apply_reconciles_no_change_manifest() {
    let tmp = TempDir::new().unwrap();
    let manifest = json!({
        "apiVersion": "apps.tachy.one/v1alpha",
        "kind": "CloudApp",
        "metadata": {
            "tenantId": "tn_test1234567890",
            "name": "fieldadmin"
        },
        "spec": {
            "envVars": []
        }
    });
    let manifest_path = tmp.path().join("tachyon.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let state_path = tmp.path().join("tachyon.tfstate");
    fs::write(
        &state_path,
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "serial": 1,
            "lineage": "ln_test",
            "resources": [{
                "kind": "CloudApp",
                "name": "fieldadmin",
                "content_hash": "already-applied",
                "manifest": manifest,
                "applied_at": "2026-06-17T00:00:00Z"
            }]
        }))
        .unwrap(),
    )
    .unwrap();

    let (api_url, rx, handle) = start_graphql_server(vec![
        history_response(Some(9)),
        r#"{"data":{"applyManifest":{"success":true,"serviceAccountsCreated":[],"serviceAccountsModified":[],"providersApplied":[],"seedDataTables":[]}}}"#.to_string(),
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "iac",
            "apply",
            "--file",
            manifest_path.to_str().unwrap(),
            "--state",
            state_path.to_str().unwrap(),
        ])
        .output()
        .expect("run tachyon iac apply");

    assert!(
        output.status.success(),
        "iac apply failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let preflight = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let req = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    handle.join().unwrap();
    assert!(request_body(&preflight).contains("manifestHistory"));
    assert!(!has_change_control_header(&preflight));
    assert!(
        req.starts_with("POST /v1/graphql "),
        "unexpected request: {req}"
    );
    assert!(req.contains("applyManifest"));
    assert!(!req.contains("saveManifest"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Reconciled: CloudApp / fieldadmin (no manifest changes)"));
}

fn run_iac_apply_contract(token: Option<&str>) {
    let tmp = TempDir::new().unwrap();
    let manifest_path = tmp.path().join("tachyon.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&json!({
            "apiVersion": "apps.tachy.one/v1alpha",
            "kind": "CloudApp",
            "metadata": { "name": "fieldadmin" },
            "spec": { "envVars": [] }
        }))
        .unwrap(),
    )
    .unwrap();
    let state_path = tmp.path().join("tachyon.tfstate");
    let (api_url, rx, handle) = start_graphql_server(vec![
        history_response(Some(7)),
        r#"{"data":{"saveManifest":{"kind":"CloudApp"}}}"#.to_string(),
        r#"{"data":{"applyManifest":{"success":true,"serviceAccountsCreated":[],"serviceAccountsModified":[],"providersApplied":[],"seedDataTables":[]}}}"#.to_string(),
    ]);

    let mut command = isolated_command(tmp.path());
    command.env("TACHYON_API_URL", api_url).args([
        "iac",
        "apply",
        "--file",
        manifest_path.to_str().unwrap(),
        "--state",
        state_path.to_str().unwrap(),
    ]);
    if let Some(token) = token {
        command.env("TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN", token);
    }
    let output = command.output().expect("run tachyon iac apply");
    assert!(
        output.status.success(),
        "iac apply failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let preflight = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let save = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let apply = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    handle.join().unwrap();
    assert!(request_body(&preflight).contains("manifestHistory"));
    assert!(!has_change_control_header(&preflight));
    assert_mutation_contract(&save, "saveManifest", Some(7), token);
    assert_mutation_contract(&apply, "applyManifest", None, token);
    if let Some(token) = token {
        assert!(!String::from_utf8_lossy(&output.stdout).contains(token));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(token));
    }
}

#[test]
fn iac_apply_forwards_token_and_expected_revision() {
    let token = production_change_control_token();
    run_iac_apply_contract(Some(&token));
}

#[test]
fn iac_apply_without_token_keeps_compatibility_and_sends_revision() {
    run_iac_apply_contract(None);
}

fn run_iac_import_seed_contract(token: Option<&str>) {
    let tmp = TempDir::new().unwrap();
    let seed_path = tmp.path().join("003-iac-manifests.yaml");
    fs::write(
        &seed_path,
        r#"
tables:
  - name: tachyon_apps_iac.manifests
    rows:
      - manifest:
          apiVersion: apps.tachy.one/v1alpha
          kind: CloudApp
          metadata:
            name: seeded-app
          spec:
            envVars: []
      - manifest:
          apiVersion: apps.tachy.one/v1alpha
          kind: ProjectConfig
          metadata:
            name: seeded-project-config
          spec:
            providers: []
"#,
    )
    .unwrap();
    let (api_url, rx, handle) = start_graphql_server(vec![
        history_response(Some(3)),
        history_response(None),
        r#"{"data":{"saveManifest":{"kind":"CloudApp"}}}"#.to_string(),
        r#"{"data":{"saveManifest":{"kind":"ProjectConfig"}}}"#.to_string(),
    ]);

    let mut command = isolated_command(tmp.path());
    command.env("TACHYON_API_URL", api_url).args([
        "iac",
        "import-seed",
        "--file",
        seed_path.to_str().unwrap(),
    ]);
    if let Some(token) = token {
        command.env("TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN", token);
    }
    let output = command.output().expect("run tachyon iac import-seed");
    assert!(
        output.status.success(),
        "iac import-seed failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let preflight = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let second_preflight = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let save = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let second_save = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    handle.join().unwrap();
    assert!(request_body(&preflight).contains("manifestHistory"));
    assert!(request_body(&second_preflight).contains("manifestHistory"));
    assert!(!has_change_control_header(&preflight));
    assert!(!has_change_control_header(&second_preflight));
    assert_mutation_contract(&save, "saveManifest", Some(3), token);
    assert_mutation_contract(&second_save, "saveManifest", Some(0), token);
    if let Some(token) = token {
        assert!(!String::from_utf8_lossy(&output.stdout).contains(token));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(token));
    }
}

#[test]
fn iac_import_seed_forwards_token_and_expected_revision() {
    let token = production_change_control_token();
    run_iac_import_seed_contract(Some(&token));
}

#[test]
fn iac_import_seed_without_token_keeps_compatibility_and_sends_revision() {
    run_iac_import_seed_contract(None);
}

fn run_iac_rollback_contract(token: Option<&str>) {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_graphql_server(vec![
        history_response(Some(11)),
        r#"{"data":{"rollbackManifest":{"kind":"CloudApp"}}}"#.to_string(),
    ]);

    let mut command = isolated_command(tmp.path());
    command.env("TACHYON_API_URL", api_url).args([
        "iac",
        "rollback",
        "--kind",
        "CloudApp",
        "--name",
        "fieldadmin",
        "--revision",
        "4",
    ]);
    if let Some(token) = token {
        command.env("TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN", token);
    }
    let output = command.output().expect("run tachyon iac rollback");
    assert!(
        output.status.success(),
        "iac rollback failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let preflight = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let rollback = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    handle.join().unwrap();
    assert!(request_body(&preflight).contains("manifestHistory"));
    assert!(!has_change_control_header(&preflight));
    assert_mutation_contract(&rollback, "rollbackManifest", Some(11), token);
    if let Some(token) = token {
        assert!(!String::from_utf8_lossy(&output.stdout).contains(token));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(token));
    }
}

#[test]
fn iac_rollback_forwards_token_and_expected_revision() {
    let token = production_change_control_token();
    run_iac_rollback_contract(Some(&token));
}

#[test]
fn iac_rollback_without_token_keeps_compatibility_and_sends_revision() {
    run_iac_rollback_contract(None);
}

#[test]
fn iac_mutation_help_documents_safe_token_input() {
    let tmp = TempDir::new().unwrap();
    for command in ["apply", "import-seed", "rollback"] {
        let output = isolated_command(tmp.path())
            .env(
                "TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN",
                "help-secret-marker",
            )
            .args(["iac", command, "--help"])
            .output()
            .expect("show iac mutation help");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("--change-control-token"));
        assert!(stdout.contains("TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN"));
        assert!(stdout.contains("Optional during the compatibility rollout"));
        assert!(!stdout.contains("help-secret-marker"));
    }
}

#[test]
fn iac_apply_rejects_invalid_token_before_api_access() {
    let tmp = TempDir::new().unwrap();
    let manifest_path = tmp.path().join("tachyon.json");
    fs::write(
        &manifest_path,
        r#"{"kind":"CloudApp","metadata":{"name":"fieldadmin"},"spec":{}}"#,
    )
    .unwrap();

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", "http://127.0.0.1:9")
        .env(
            "TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN",
            "invalid-token-marker",
        )
        .args(["iac", "apply", "--file", manifest_path.to_str().unwrap()])
        .output()
        .expect("run tachyon iac apply with invalid token");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a valid approval token"));
    assert!(!stderr.contains("invalid-token-marker"));
    assert!(!stderr.contains("connection refused"));
}

#[test]
fn iac_apply_cas_conflict_does_not_update_local_state() {
    let tmp = TempDir::new().unwrap();
    let manifest_path = tmp.path().join("tachyon.json");
    fs::write(
        &manifest_path,
        r#"{"kind":"CloudApp","metadata":{"name":"fieldadmin"},"spec":{}}"#,
    )
    .unwrap();
    let state_path = tmp.path().join("tachyon.tfstate");
    let (api_url, rx, handle) = start_graphql_server(vec![
        history_response(Some(7)),
        r#"{"errors":[{"message":"manifest serving revision changed; re-plan"}]}"#.to_string(),
    ]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "iac",
            "apply",
            "--file",
            manifest_path.to_str().unwrap(),
            "--state",
            state_path.to_str().unwrap(),
        ])
        .output()
        .expect("run tachyon iac apply with CAS conflict");

    assert!(!output.status.success());
    let preflight = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let save = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    handle.join().unwrap();
    assert!(request_body(&preflight).contains("manifestHistory"));
    assert_mutation_contract(&save, "saveManifest", Some(7), None);
    assert!(!state_path.exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("re-plan"));
}
