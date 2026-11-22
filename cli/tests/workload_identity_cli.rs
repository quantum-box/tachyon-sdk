//! `tachyon auth workload-identity` and the workload identity credential
//! mode, against a local mock of the Tachyon API and the GitHub Actions
//! OIDC token service.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;

use tempfile::TempDir;

const TENANT: &str = "tn_01hjryxysgey07h5jz5wagqj0m";
const APP: &str = "app_01km2dr0f6hvgj0qvcteyydfbe";
const EXCHANGED: &str = "eyJ.exchanged.jwt";

fn isolated_command(home: &Path, api_url: &str) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tachyon"));
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_TENANT_ID", TENANT)
        .env_remove("TACHYON_API_KEY")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE")
        .env_remove("GITHUB_ACTIONS")
        .env_remove("ACTIONS_ID_TOKEN_REQUEST_URL")
        .env_remove("ACTIONS_ID_TOKEN_REQUEST_TOKEN")
        .current_dir(home);
    for (name, _) in std::env::vars() {
        if name.starts_with("TACHYON_WORKLOAD_IDENTITY_") {
            cmd.env_remove(name);
        }
    }
    cmd
}

struct Request {
    head: String,
    body: String,
}

fn read_request(stream: &mut TcpStream) -> Request {
    let mut data = Vec::new();
    let mut buf = [0_u8; 8192];
    loop {
        let n = stream.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        data.extend_from_slice(&buf[..n]);
        let text = String::from_utf8_lossy(&data).to_string();
        if let Some(split) = text.find("\r\n\r\n") {
            let head = text[..split].to_string();
            let length = head
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            if data.len() >= split + 4 + length {
                return Request {
                    head,
                    body: text[split + 4..split + 4 + length].to_string(),
                };
            }
        }
    }
    let text = String::from_utf8_lossy(&data).to_string();
    Request {
        head: text,
        body: String::new(),
    }
}

fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).unwrap();
}

/// Answers each incoming request with the next `(status, body)` and sends
/// the request to the returned channel.
fn start_server(
    responses: Vec<(&'static str, String)>,
) -> (String, mpsc::Receiver<Request>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_request(&mut stream);
            tx.send(request).unwrap();
            respond(&mut stream, status, &body);
        }
    });
    (url, rx, handle)
}

fn exchange_body(expires_in: i64) -> String {
    format!(r#"{{"tokenType":"Bearer","accessToken":"{EXCHANGED}","expiresIn":{expires_in}}}"#)
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn token_uses_github_actions_oidc_and_masks_the_result() {
    let tmp = TempDir::new().unwrap();
    let (url, rx, handle) = start_server(vec![
        ("200 OK", r#"{"value":"gh-oidc-jwt"}"#.to_string()),
        ("200 OK", exchange_body(300)),
    ]);

    let output = isolated_command(tmp.path(), &url)
        .env("GITHUB_ACTIONS", "true")
        .env(
            "ACTIONS_ID_TOKEN_REQUEST_URL",
            format!("{url}/_apis/oidc?api-version=2.0"),
        )
        .env("ACTIONS_ID_TOKEN_REQUEST_TOKEN", "gh-request-token")
        .args([
            "auth",
            "workload-identity",
            "token",
            "--app-id",
            APP,
            "--environment",
            "production",
            "--action",
            "compute:CreateDeployment",
            "--ttl",
            "300",
        ])
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let oidc = rx.recv().unwrap();
    assert!(
        oidc.head.starts_with(
            "GET /_apis/oidc?api-version=2.0&audience=tachyon-api%2Fworkload-identity "
        ),
        "{}",
        oidc.head
    );
    assert!(oidc.head.contains("authorization: Bearer gh-request-token"));

    let exchange = rx.recv().unwrap();
    assert!(exchange
        .head
        .starts_with("POST /v1/auth/workload-identities/token/exchange "));
    assert!(!exchange
        .head
        .to_ascii_lowercase()
        .contains("authorization:"));
    let body: serde_json::Value = serde_json::from_str(&exchange.body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({
            "appId": APP,
            "operatorId": TENANT,
            "environment": "production",
            "action": "compute:CreateDeployment",
            "subjectToken": "gh-oidc-jwt",
            "ttlSeconds": 300,
        })
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stdout.trim(), EXCHANGED);
    // The only place the token appears on stderr is the mask command.
    assert_eq!(stderr.matches(EXCHANGED).count(), 1);
    assert!(stderr.contains(&format!("::add-mask::{EXCHANGED}")));
    assert!(!stderr.contains("gh-oidc-jwt"));
}

#[test]
fn token_reads_subject_from_file_and_prints_json() {
    let tmp = TempDir::new().unwrap();
    let token_file = tmp.path().join("oidc-token");
    std::fs::write(&token_file, "file-oidc-jwt\n").unwrap();
    let (url, rx, handle) = start_server(vec![("200 OK", exchange_body(900))]);

    let output = isolated_command(tmp.path(), &url)
        .args([
            "auth",
            "wif",
            "token",
            "--app-id",
            APP,
            "--environment",
            "staging",
            "--action",
            "deploy",
            "--format",
            "json",
        ])
        .arg("--subject-token-file")
        .arg(&token_file)
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let exchange = rx.recv().unwrap();
    let body: serde_json::Value = serde_json::from_str(&exchange.body).unwrap();
    assert_eq!(body["subjectToken"], "file-oidc-jwt");
    assert!(body.get("ttlSeconds").is_none());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["accessToken"], EXCHANGED);
    assert_eq!(json["expiresIn"], 900);
    assert!(!String::from_utf8_lossy(&output.stderr).contains(EXCHANGED));
}

#[test]
fn token_without_subject_source_explains_the_permission() {
    let tmp = TempDir::new().unwrap();
    let output = isolated_command(tmp.path(), "http://127.0.0.1:9")
        .args([
            "auth",
            "workload-identity",
            "token",
            "--app-id",
            APP,
            "--environment",
            "production",
            "--action",
            "deploy",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("id-token: write"));
}

#[test]
fn token_reports_exchange_errors_without_the_subject_token() {
    let tmp = TempDir::new().unwrap();
    let (url, _rx, handle) = start_server(vec![(
        "403 Forbidden",
        r#"{"message":"action is not allowed"}"#.to_string(),
    )]);
    let output = isolated_command(tmp.path(), &url)
        .env("MY_OIDC", "secret-oidc-jwt")
        .args([
            "auth",
            "workload-identity",
            "token",
            "--app-id",
            APP,
            "--environment",
            "production",
            "--action",
            "deploy",
            "--subject-token-env",
            "MY_OIDC",
        ])
        .output()
        .unwrap();
    handle.join().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("status=403"), "{stderr}");
    assert!(stderr.contains("action is not allowed"));
    assert!(!stderr.contains("secret-oidc-jwt"));
}

#[test]
fn credential_mode_authenticates_other_commands_with_the_exchanged_token() {
    let tmp = TempDir::new().unwrap();
    let (url, rx, handle) = start_server(vec![
        ("200 OK", exchange_body(900)),
        ("200 OK", "[]".to_string()),
    ]);

    let output = isolated_command(tmp.path(), &url)
        .env("TACHYON_WORKLOAD_IDENTITY_APP_ID", APP)
        .env("TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT", "production")
        .env(
            "TACHYON_WORKLOAD_IDENTITY_ACTION",
            "auth:FindAllServiceAccounts",
        )
        .env("TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_ENV", "MY_OIDC")
        .env("MY_OIDC", "env-oidc-jwt")
        .args(["auth", "workload-identity", "provider", "list", "--json"])
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let exchange = rx.recv().unwrap();
    assert!(exchange
        .head
        .starts_with("POST /v1/auth/workload-identities/token/exchange "));
    let body: serde_json::Value = serde_json::from_str(&exchange.body).unwrap();
    assert_eq!(body["subjectToken"], "env-oidc-jwt");
    assert_eq!(body["action"], "auth:FindAllServiceAccounts");

    let list = rx.recv().unwrap();
    assert!(list
        .head
        .starts_with("GET /v1/auth/workload-identities/federation-providers "));
    assert!(list
        .head
        .contains(&format!("authorization: Bearer {EXCHANGED}")));
    assert!(list.head.contains(&format!("x-operator-id: {TENANT}")));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(EXCHANGED));
}

#[test]
fn credential_mode_re_exchanges_after_401() {
    let tmp = TempDir::new().unwrap();
    let (url, rx, handle) = start_server(vec![
        ("200 OK", exchange_body(900)),
        ("401 Unauthorized", r#"{"message":"expired"}"#.to_string()),
        ("200 OK", exchange_body(900)),
        ("200 OK", "[]".to_string()),
    ]);

    let output = isolated_command(tmp.path(), &url)
        .env("TACHYON_WORKLOAD_IDENTITY_APP_ID", APP)
        .env("TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT", "production")
        .env(
            "TACHYON_WORKLOAD_IDENTITY_ACTION",
            "auth:FindAllServiceAccounts",
        )
        .env("TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_ENV", "MY_OIDC")
        .env("MY_OIDC", "env-oidc-jwt")
        .args(["auth", "workload-identity", "list", "--json"])
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let paths: Vec<String> = rx
        .try_iter()
        .map(|request| request.head.lines().next().unwrap_or_default().to_string())
        .collect();
    assert_eq!(paths.len(), 4, "{paths:?}");
    assert!(paths[0].starts_with("POST /v1/auth/workload-identities/token/exchange "));
    assert!(paths[1].starts_with("GET /v1/auth/workload-identities "));
    assert!(paths[2].starts_with("POST /v1/auth/workload-identities/token/exchange "));
    assert!(paths[3].starts_with("GET /v1/auth/workload-identities "));
}

#[test]
fn api_key_takes_precedence_over_workload_identity() {
    let tmp = TempDir::new().unwrap();
    let (url, rx, handle) = start_server(vec![("200 OK", "[]".to_string())]);

    let output = isolated_command(tmp.path(), &url)
        .env("TACHYON_API_KEY", "test-api-key")
        .env("TACHYON_WORKLOAD_IDENTITY_APP_ID", APP)
        .env("TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT", "production")
        .env("TACHYON_WORKLOAD_IDENTITY_ACTION", "deploy")
        .args(["auth", "workload-identity", "list", "--json"])
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let list = rx.recv().unwrap();
    assert!(list.head.starts_with("GET /v1/auth/workload-identities "));
    assert!(list.head.contains("authorization: Bearer test-api-key"));
}

#[test]
fn credential_mode_requires_environment_and_action() {
    let tmp = TempDir::new().unwrap();
    let output = isolated_command(tmp.path(), "http://127.0.0.1:9")
        .env("TACHYON_WORKLOAD_IDENTITY_APP_ID", APP)
        .args(["auth", "workload-identity", "list"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT")
    );
}

#[test]
fn provider_add_sends_github_defaults_and_claims() {
    let tmp = TempDir::new().unwrap();
    let response = serde_json::json!({
        "id": "wif_01",
        "appId": APP,
        "operatorId": TENANT,
        "environment": "production",
        "issuer": "https://token.actions.githubusercontent.com",
        "audience": "tachyon-api/workload-identity",
        "subjectPattern": "repo:my-org/my-repo:ref:refs/heads/main",
        "claimConditions": {"repository_owner": ["my-org"]},
        "replayProtectionMode": "required",
        "enabled": true,
    })
    .to_string();
    let (url, rx, handle) = start_server(vec![("200 OK", response)]);

    let output = isolated_command(tmp.path(), &url)
        .env("TACHYON_API_KEY", "test-api-key")
        .args([
            "auth",
            "workload-identity",
            "provider",
            "add",
            "--app-id",
            APP,
            "--environment",
            "production",
            "--subject-pattern",
            "repo:my-org/my-repo:ref:refs/heads/main",
            "--claim",
            "repository_owner=my-org",
            "--json",
        ])
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let put = rx.recv().unwrap();
    assert!(put
        .head
        .starts_with("PUT /v1/auth/workload-identities/federation-providers "));
    let body: serde_json::Value = serde_json::from_str(&put.body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({
            "appId": APP,
            "operatorId": TENANT,
            "environment": "production",
            "issuer": "https://token.actions.githubusercontent.com",
            "audience": "tachyon-api/workload-identity",
            "subjectPattern": "repo:my-org/my-repo:ref:refs/heads/main",
            "claimConditions": {"repository_owner": ["my-org"]},
            "replayProtectionMode": "required",
        })
    );
}

#[test]
fn provider_delete_calls_delete_endpoint() {
    let tmp = TempDir::new().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        stream
            .write_all(b"HTTP/1.1 204 No Content\r\ncontent-length: 0\r\nconnection: close\r\n\r\n")
            .unwrap();
        request
    });

    let output = isolated_command(tmp.path(), &url)
        .env("TACHYON_API_KEY", "test-api-key")
        .args([
            "auth",
            "workload-identity",
            "provider",
            "delete",
            "wif_01abc",
            "--json",
        ])
        .output()
        .unwrap();
    assert_success(&output);
    let request = handle.join().unwrap();
    assert!(request
        .head
        .starts_with("DELETE /v1/auth/workload-identities/federation-providers/wif_01abc "));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["deleted"], true);
}

fn exchange_body_with(token: &str) -> String {
    format!(r#"{{"tokenType":"Bearer","accessToken":"{token}","expiresIn":900}}"#)
}

#[test]
fn token_flags_are_used_even_when_credential_mode_variables_are_incomplete() {
    let tmp = TempDir::new().unwrap();
    let (url, rx, handle) = start_server(vec![("200 OK", exchange_body(300))]);

    // Only the app ID comes from the environment; credential mode would
    // reject this, but the explicit exchange has everything it needs.
    let output = isolated_command(tmp.path(), &url)
        .env("TACHYON_WORKLOAD_IDENTITY_APP_ID", APP)
        .env("MY_OIDC", "env-oidc-jwt")
        .args([
            "auth",
            "workload-identity",
            "token",
            "--environment",
            "production",
            "--action",
            "deploy",
            "--subject-token-env",
            "MY_OIDC",
        ])
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let body: serde_json::Value = serde_json::from_str(&rx.recv().unwrap().body).unwrap();
    assert_eq!(body["appId"], APP);
    assert_eq!(body["environment"], "production");
    assert_eq!(body["subjectToken"], "env-oidc-jwt");
}

#[test]
fn credential_mode_reuses_the_re_exchanged_token_for_later_requests() {
    let tmp = TempDir::new().unwrap();
    let sa_list =
        r#"{"serviceAccounts":[{"id":"sa_01jz0aaaaaaaaaaaaaaaaaaaaa","name":"gha.deploy"}]}"#;
    let binding = serde_json::json!({
        "id": "wi_01",
        "appId": APP,
        "operatorId": TENANT,
        "environment": "production",
        "serviceAccountId": "sa_01jz0aaaaaaaaaaaaaaaaaaaaa",
        "allowedActions": ["deploy"],
        "enabled": true,
    })
    .to_string();
    let (url, rx, handle) = start_server(vec![
        ("200 OK", exchange_body_with("first-token")),
        ("401 Unauthorized", r#"{"message":"expired"}"#.to_string()),
        ("200 OK", exchange_body_with("second-token")),
        ("200 OK", sa_list.to_string()),
        ("200 OK", binding),
    ]);

    let output = isolated_command(tmp.path(), &url)
        .env("TACHYON_WORKLOAD_IDENTITY_APP_ID", APP)
        .env("TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT", "production")
        .env(
            "TACHYON_WORKLOAD_IDENTITY_ACTION",
            "auth:UpdateServiceAccount",
        )
        .env("TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_ENV", "MY_OIDC")
        .env("MY_OIDC", "env-oidc-jwt")
        .args([
            "auth",
            "workload-identity",
            "ensure",
            "--app-id",
            APP,
            "--environment",
            "production",
            "--service-account",
            "gha.deploy",
            "--allowed-action",
            "deploy",
            "--json",
        ])
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let requests: Vec<Request> = rx.try_iter().collect();
    assert_eq!(requests.len(), 5);
    assert!(requests[1]
        .head
        .contains("authorization: Bearer first-token"));
    assert!(requests[3]
        .head
        .contains("authorization: Bearer second-token"));
    // The request after the retry carries the cached token instead of the
    // rejected one, so no further 401 or exchange happens.
    assert!(requests[4]
        .head
        .starts_with("PUT /v1/auth/workload-identities "));
    assert!(requests[4]
        .head
        .contains("authorization: Bearer second-token"));
    assert!(!requests[4].head.contains("first-token"));
}

#[test]
fn credential_mode_runtime_log_tail_reconnects_with_the_refreshed_token() {
    let tmp = TempDir::new().unwrap();
    let (url, rx, handle) = start_server(vec![
        ("200 OK", exchange_body_with("first-token")),
        ("401 Unauthorized", r#"{"message":"expired"}"#.to_string()),
        ("200 OK", exchange_body_with("second-token")),
        // The stream ends at once, so the tail reconnects after its backoff.
        ("200 OK", String::new()),
        ("200 OK", String::new()),
    ]);

    let mut child = isolated_command(tmp.path(), &url)
        .env("TACHYON_WORKLOAD_IDENTITY_APP_ID", APP)
        .env("TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT", "production")
        .env("TACHYON_WORKLOAD_IDENTITY_ACTION", "compute:GetApp")
        .env("TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_ENV", "MY_OIDC")
        .env("MY_OIDC", "env-oidc-jwt")
        .args(["compute", "logs", "--tail", APP])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    handle.join().unwrap();
    child.kill().ok();
    child.wait().ok();

    let requests: Vec<Request> = rx.try_iter().collect();
    assert_eq!(requests.len(), 5);
    let tail = format!("GET /v1/compute/apps/{APP}/logs/tail ");
    assert!(requests[1].head.starts_with(&tail));
    assert!(requests[1]
        .head
        .contains("authorization: Bearer first-token"));
    assert!(requests[3].head.starts_with(&tail));
    assert!(requests[3]
        .head
        .contains("authorization: Bearer second-token"));
    // The reconnect uses the cached refreshed token, not the rejected one.
    assert!(requests[4].head.starts_with(&tail));
    assert!(requests[4]
        .head
        .contains("authorization: Bearer second-token"));
    assert!(!requests[4].head.contains("first-token"));
}

#[test]
fn credential_mode_uses_the_tenant_next_to_the_manifest() {
    const MANIFEST_TENANT: &str = "tn_01manifesttenant000000000";
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join(".git")).unwrap();
    // The working directory belongs to another tenant; the manifest's own
    // tachyon.yml must win, as it does for the command itself.
    std::fs::write(
        tmp.path().join("tachyon.yml"),
        "metadata:\n  tenantId: tn_01workdirtenant0000000000\n",
    )
    .unwrap();
    let service = tmp.path().join("svc");
    std::fs::create_dir_all(service.join(".tachyon/manifests")).unwrap();
    std::fs::write(
        service.join("tachyon.yml"),
        format!("metadata:\n  tenantId: {MANIFEST_TENANT}\n"),
    )
    .unwrap();
    std::fs::write(
        service.join(".tachyon/manifests/auth.yml"),
        "actions:\n  - context: manifest\n    name: Read\npolicies: []\n",
    )
    .unwrap();
    let (url, rx, handle) = start_server(vec![
        ("200 OK", exchange_body(900)),
        ("200 OK", "{}".to_string()),
    ]);

    let _ = isolated_command(tmp.path(), &url)
        .env_remove("TACHYON_TENANT_ID")
        .env("TACHYON_WORKLOAD_IDENTITY_APP_ID", APP)
        .env("TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT", "production")
        .env("TACHYON_WORKLOAD_IDENTITY_ACTION", "auth:CreateAction")
        .env("TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_ENV", "MY_OIDC")
        .env("MY_OIDC", "env-oidc-jwt")
        .args([
            "manifest",
            "apply",
            "--file",
            "svc/.tachyon/manifests/auth.yml",
        ])
        .output()
        .unwrap();
    handle.join().unwrap();

    let exchange = rx.recv().unwrap();
    assert!(exchange
        .head
        .starts_with("POST /v1/auth/workload-identities/token/exchange "));
    let body: serde_json::Value = serde_json::from_str(&exchange.body).unwrap();
    assert_eq!(body["operatorId"], MANIFEST_TENANT);
    let next = rx.recv().unwrap();
    assert!(next
        .head
        .contains(&format!("x-operator-id: {MANIFEST_TENANT}")));
}

#[test]
fn credential_mode_policy_update_uses_the_refreshed_token() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(
        tmp.path().join("auth.yml"),
        "actions: []\npolicies:\n  - name: ManifestPolicy\n    description: Updated\n    actions:\n      - action: manifest:Read\n        effect: allow\n",
    )
    .unwrap();
    let policies = format!(
        r#"{{"policies":[{{"id":"pol_existing","name":"ManifestPolicy","isSystem":false,"tenantId":"{TENANT}"}}]}}"#
    );
    let updated = format!(
        r#"{{"createdAt":"2026-11-21T00:00:00Z","description":"Updated","id":"pol_existing","isSystem":false,"name":"ManifestPolicy","tenantId":"{TENANT}","updatedAt":"2026-11-21T00:00:01Z"}}"#
    );
    let (url, rx, handle) = start_server(vec![
        ("200 OK", exchange_body_with("first-token")),
        ("401 Unauthorized", r#"{"message":"expired"}"#.to_string()),
        ("200 OK", exchange_body_with("second-token")),
        ("200 OK", policies),
        ("200 OK", updated),
    ]);

    let output = isolated_command(tmp.path(), &url)
        .env("TACHYON_WORKLOAD_IDENTITY_APP_ID", APP)
        .env("TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT", "production")
        .env("TACHYON_WORKLOAD_IDENTITY_ACTION", "auth:UpdatePolicy")
        .env("TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_ENV", "MY_OIDC")
        .env("MY_OIDC", "env-oidc-jwt")
        .args(["auth", "manifest", "apply", "--file", "auth.yml", "--json"])
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let requests: Vec<Request> = rx.try_iter().collect();
    assert_eq!(requests.len(), 5);
    assert!(requests[3].head.starts_with("GET /v1/auth/policies "));
    // The generated-SDK PATCH carries the refreshed token, not the one the
    // SDK configuration was built with.
    assert!(requests[4]
        .head
        .starts_with("PATCH /v1/auth/policies/pol_existing "));
    assert!(requests[4]
        .head
        .contains("authorization: Bearer second-token"));
    assert!(requests[4]
        .head
        .contains(&format!("x-operator-id: {TENANT}")));
}

#[test]
fn credential_mode_keeps_an_explicit_iac_apply_key_for_writes() {
    let tmp = TempDir::new().unwrap();
    let manifest = tmp.path().join("tachyon.json");
    std::fs::write(
        &manifest,
        r#"{"apiVersion":"apps.tachy.one/v1alpha","kind":"CloudApp","metadata":{"name":"fieldadmin"},"spec":{"envVars":[]}}"#,
    )
    .unwrap();
    let history = r#"{"data":{"manifestHistory":[{"revision":7,"contentHash":"h","appliedBy":"t","appliedAt":"2026-11-21T00:00:00Z","manifest":"{}"}]}}"#;
    let (url, rx, handle) = start_server(vec![
        ("200 OK", exchange_body(900)),
        ("200 OK", history.to_string()),
        (
            "200 OK",
            r#"{"data":{"saveManifest":{"kind":"CloudApp"}}}"#.to_string(),
        ),
        (
            "200 OK",
            r#"{"data":{"applyManifest":{"success":true,"serviceAccountsCreated":[],"serviceAccountsModified":[],"providersApplied":[],"seedDataTables":[]}}}"#.to_string(),
        ),
    ]);

    let output = isolated_command(tmp.path(), &url)
        .env("TACHYON_WORKLOAD_IDENTITY_APP_ID", APP)
        .env("TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT", "production")
        .env("TACHYON_WORKLOAD_IDENTITY_ACTION", "iac:GetManifestHistory")
        .env("TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_ENV", "MY_OIDC")
        .env("MY_OIDC", "env-oidc-jwt")
        .env("TACHYON_IAC_APPLY_API_KEY", "write-key")
        .env_remove("TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN")
        .arg("iac")
        .arg("apply")
        .arg("--file")
        .arg(&manifest)
        .arg("--state")
        .arg(tmp.path().join("tachyon.tfstate"))
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let requests: Vec<Request> = rx.try_iter().collect();
    assert_eq!(requests.len(), 4);
    assert!(requests[1]
        .head
        .contains(&format!("authorization: Bearer {EXCHANGED}")));
    for write in &requests[2..] {
        assert!(write.head.starts_with("POST /v1/graphql "));
        assert!(write.head.contains("authorization: Bearer write-key"));
        assert!(!write.head.contains(EXCHANGED));
    }
}

#[test]
fn token_without_action_omits_the_field() {
    let tmp = TempDir::new().unwrap();
    let (url, rx, handle) = start_server(vec![("200 OK", exchange_body(300))]);

    let output = isolated_command(tmp.path(), &url)
        .env("MY_OIDC", "env-oidc-jwt")
        .args([
            "auth",
            "workload-identity",
            "token",
            "--app-id",
            APP,
            "--environment",
            "production",
            "--subject-token-env",
            "MY_OIDC",
        ])
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let body: serde_json::Value = serde_json::from_str(&rx.recv().unwrap().body).unwrap();
    assert!(body.get("action").is_none(), "{body}");
    assert_eq!(body["environment"], "production");
}

#[test]
fn credential_mode_works_without_an_action() {
    let tmp = TempDir::new().unwrap();
    let (url, rx, handle) = start_server(vec![
        ("200 OK", exchange_body(900)),
        ("200 OK", "[]".to_string()),
    ]);

    let output = isolated_command(tmp.path(), &url)
        .env("TACHYON_WORKLOAD_IDENTITY_APP_ID", APP)
        .env("TACHYON_WORKLOAD_IDENTITY_ENVIRONMENT", "production")
        .env("TACHYON_WORKLOAD_IDENTITY_SUBJECT_TOKEN_ENV", "MY_OIDC")
        .env("MY_OIDC", "env-oidc-jwt")
        .args(["auth", "workload-identity", "list", "--json"])
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let exchange = rx.recv().unwrap();
    let body: serde_json::Value = serde_json::from_str(&exchange.body).unwrap();
    assert!(body.get("action").is_none(), "{body}");
    let list = rx.recv().unwrap();
    assert!(list
        .head
        .contains(&format!("authorization: Bearer {EXCHANGED}")));
}

#[test]
fn token_without_action_hints_at_old_servers() {
    let tmp = TempDir::new().unwrap();
    let (url, _rx, handle) = start_server(vec![(
        "422 Unprocessable Entity",
        r#"{"message":"missing field `action`"}"#.to_string(),
    )]);

    let output = isolated_command(tmp.path(), &url)
        .env("MY_OIDC", "env-oidc-jwt")
        .args([
            "auth",
            "workload-identity",
            "token",
            "--app-id",
            APP,
            "--environment",
            "production",
            "--subject-token-env",
            "MY_OIDC",
        ])
        .output()
        .unwrap();
    handle.join().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("status=422"), "{stderr}");
    assert!(stderr.contains("v0.92.921"), "{stderr}");
    assert!(
        stderr.contains("TACHYON_WORKLOAD_IDENTITY_ACTION"),
        "{stderr}"
    );
}

#[test]
fn ensure_without_allowed_actions_sends_wildcard() {
    let tmp = TempDir::new().unwrap();
    let sa_list =
        r#"{"serviceAccounts":[{"id":"sa_01jz0aaaaaaaaaaaaaaaaaaaaa","name":"gha.deploy"}]}"#;
    let binding = serde_json::json!({
        "id": "wi_01",
        "appId": APP,
        "operatorId": TENANT,
        "environment": "production",
        "serviceAccountId": "sa_01jz0aaaaaaaaaaaaaaaaaaaaa",
        "allowedActions": ["*"],
        "enabled": true,
    })
    .to_string();
    let (url, rx, handle) =
        start_server(vec![("200 OK", sa_list.to_string()), ("200 OK", binding)]);

    let output = isolated_command(tmp.path(), &url)
        .env("TACHYON_API_KEY", "test-api-key")
        .args([
            "auth",
            "workload-identity",
            "ensure",
            "--app-id",
            APP,
            "--environment",
            "production",
            "--service-account",
            "gha.deploy",
            "--json",
        ])
        .output()
        .unwrap();
    assert_success(&output);
    handle.join().unwrap();

    let _lookup = rx.recv().unwrap();
    let put = rx.recv().unwrap();
    assert!(put.head.starts_with("PUT /v1/auth/workload-identities "));
    let body: serde_json::Value = serde_json::from_str(&put.body).unwrap();
    assert_eq!(body["allowedActions"], serde_json::json!(["*"]));
}
