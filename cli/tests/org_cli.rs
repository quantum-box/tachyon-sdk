use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;

use tempfile::TempDir;

const TENANT_ID: &str = "tn_test1234567890";

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_tachyon")
}

fn isolated_command(home: &Path) -> Command {
    let mut cmd = Command::new(bin());
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_TENANT_ID", TENANT_ID)
        .env_remove("TACHYON_API_KEY")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE");
    cmd
}

struct MockResponse {
    status: &'static str,
    body: &'static str,
}

fn start_server(
    responses: Vec<MockResponse>,
) -> (String, mpsc::Receiver<Vec<String>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut requests = Vec::with_capacity(responses.len());
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            requests.push(String::from_utf8_lossy(&buf[..n]).to_string());

            let raw_response = if response.body.is_empty() {
                format!(
                    "HTTP/1.1 {}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
                    response.status
                )
            } else {
                format!(
                    "HTTP/1.1 {}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    response.status,
                    response.body.len(),
                    response.body
                )
            };
            stream.write_all(raw_response.as_bytes()).unwrap();
        }
        tx.send(requests).unwrap();
    });
    (url, rx, handle)
}

fn run_org(home: &Path, api_url: String, args: &[&str]) -> Output {
    isolated_command(home)
        .env("TACHYON_API_URL", api_url)
        .args(args)
        .output()
        .expect("run tachyon org command")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn finish_requests(rx: mpsc::Receiver<Vec<String>>, handle: thread::JoinHandle<()>) -> Vec<String> {
    let requests = rx.recv().unwrap();
    handle.join().unwrap();
    requests
}

fn assert_tenant_request(request: &str, request_line: &str) {
    assert!(request.starts_with(request_line), "request was:\n{request}");
    assert!(
        request.contains(&format!("x-operator-id: {TENANT_ID}")),
        "request was:\n{request}"
    );
}

#[test]
fn service_accounts_list_sends_tenant_query_and_decodes_openapi_envelope() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"serviceAccounts":[{"id":"sa_123456789012","name":"inventory","createdAt":"2026-08-01T00:00:00Z"}]}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["org", "service-accounts", "list", "--json"],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts?operator_id=tn_test1234567890 ",
    );
    let accounts: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).expect("service accounts json");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0]["id"], "sa_123456789012");
}

#[test]
fn service_accounts_list_keeps_legacy_array_compatibility() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"[{"id":"sa_123456789012","name":"legacy"}]"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["org", "service-accounts", "list", "--json"],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts?operator_id=tn_test1234567890 ",
    );
    let accounts: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).expect("service accounts json");
    assert_eq!(accounts.len(), 1);
}

#[test]
fn service_accounts_get_sends_tenant_query_and_header() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"id":"sa_123456789012","name":"inventory"}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "service-accounts",
            "get",
            "sa_123456789012",
            "--json",
        ],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts/sa_123456789012?operator_id=tn_test1234567890 ",
    );
}

#[test]
fn service_account_api_keys_sends_tenant_query_and_decodes_openapi_envelope() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"apiKeys":[{"id":"key_123456789012","name":"audit","prefix":"masked","createdAt":"2026-08-01T00:00:00Z"}]}"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "service-accounts",
            "api-keys",
            "sa_123456789012",
            "--json",
        ],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts/sa_123456789012/api-keys?operator_id=tn_test1234567890 ",
    );
    let keys: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).expect("api keys json");
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0]["id"], "key_123456789012");
}

#[test]
fn service_account_api_keys_keeps_legacy_array_compatibility() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"[{"id":"key_123456789012","name":"legacy"}]"#,
    }]);

    let output = run_org(
        tmp.path(),
        api_url,
        &[
            "org",
            "service-accounts",
            "api-keys",
            "sa_123456789012",
            "--json",
        ],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts/sa_123456789012/api-keys?operator_id=tn_test1234567890 ",
    );
    let keys: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).expect("api keys json");
    assert_eq!(keys.len(), 1);
}

#[test]
fn api_key_name_resolution_uses_tenant_query() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        MockResponse {
            status: "200 OK",
            body: r#"{"serviceAccounts":[{"id":"sa_123456789012","name":"inventory"}]}"#,
        },
        MockResponse {
            status: "200 OK",
            body: r#"{"apiKeys":[]}"#,
        },
    ]);

    let output = run_org(
        tmp.path(),
        api_url,
        &["api-key", "list", "inventory", "--json"],
    );
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 2);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/service-accounts?operator_id=tn_test1234567890 ",
    );
    assert_tenant_request(
        &requests[1],
        "GET /v1/auth/service-accounts/sa_123456789012/api-keys?operator_id=tn_test1234567890 ",
    );
}

#[test]
fn users_list_sends_tenant_query_and_decodes_openapi_envelope() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"users":[{"id":"us_123456789012","username":"member","email":"member@example.invalid","role":"member","createdAt":"2026-08-01T00:00:00Z"}]}"#,
    }]);

    let output = run_org(tmp.path(), api_url, &["org", "users", "list", "--json"]);
    assert_success(&output);

    let requests = finish_requests(rx, handle);
    assert_tenant_request(
        &requests[0],
        "GET /v1/auth/users?operator_id=tn_test1234567890 ",
    );
    let users: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).expect("users json");
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["id"], "us_123456789012");
}
