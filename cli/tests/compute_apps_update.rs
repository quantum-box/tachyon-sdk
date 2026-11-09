use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;

use tempfile::TempDir;

const TENANT_ID: &str = "tn_test1234567890";
const APP_ID: &str = "app_test1234567890";

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_tachyon")
}

fn isolated_command(home: &Path) -> Command {
    let mut cmd = Command::new(bin());
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_TENANT_ID", TENANT_ID)
        .env("TACHYON_API_KEY", "test-token")
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

            let raw_response = format!(
                "HTTP/1.1 {}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                response.status,
                response.body.len(),
                response.body
            );
            stream.write_all(raw_response.as_bytes()).unwrap();
        }
        tx.send(requests).unwrap();
    });
    (url, rx, handle)
}

fn run_update(home: &Path, api_url: String, connection_id: &str, yes: bool) -> Output {
    let mut command = isolated_command(home);
    command.env("TACHYON_API_URL", api_url).args([
        "compute",
        "apps",
        "update",
        APP_ID,
        "--connection-id",
        connection_id,
    ]);
    if yes {
        command.arg("--yes");
    }
    command.output().expect("run tachyon compute apps update")
}

fn finish_requests(rx: mpsc::Receiver<Vec<String>>, handle: thread::JoinHandle<()>) -> Vec<String> {
    let requests = rx.recv().unwrap();
    handle.join().unwrap();
    requests
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn request_body(request: &str) -> serde_json::Value {
    let body = request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("request body separator");
    serde_json::from_str(body).expect("request JSON body")
}

#[test]
fn update_requires_explicit_app_and_connection_ids() {
    let tmp = TempDir::new().unwrap();

    let missing_app = isolated_command(tmp.path())
        .args(["compute", "apps", "update", "--connection-id", "conn_new"])
        .output()
        .unwrap();
    assert!(!missing_app.status.success());

    let missing_connection = isolated_command(tmp.path())
        .args(["compute", "apps", "update", APP_ID])
        .output()
        .unwrap();
    assert!(!missing_connection.status.success());
    assert!(String::from_utf8_lossy(&missing_connection.stderr).contains("--connection-id"));
}

#[test]
fn update_previews_target_without_patch_until_yes_is_passed() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"id":"app_test1234567890","name":"courseboard","connection_id":"conn_old"}"#,
    }]);

    let output = run_update(tmp.path(), api_url, "conn_new", false);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("courseboard (app_test1234567890)"));
    assert!(stdout.contains("Current connection:   conn_old"));
    assert!(stdout.contains("Requested connection: conn_new"));
    assert!(stdout.contains("No changes made. Re-run with --yes"));
    assert!(!stdout.contains("test-token"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("test-token"));

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0].starts_with("GET /v1/compute/apps/app_test1234567890 "),
        "request was:\n{}",
        requests[0]
    );
}

#[test]
fn update_patches_only_connection_id_after_explicit_confirmation() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        MockResponse {
            status: "200 OK",
            body: r#"{"id":"app_test1234567890","name":"courseboard","connection_id":"conn_old"}"#,
        },
        MockResponse {
            status: "200 OK",
            body: r#"{"id":"app_test1234567890","name":"courseboard","connection_id":"conn_new"}"#,
        },
    ]);

    let output = run_update(tmp.path(), api_url, "conn_new", true);
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Updated connection for courseboard"));
    assert!(stdout.contains("conn_new"));

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 2);
    assert!(
        requests[1].starts_with("PATCH /v1/apps/app_test1234567890 "),
        "request was:\n{}",
        requests[1]
    );
    assert_eq!(
        request_body(&requests[1]),
        serde_json::json!({"connection_id": "conn_new"})
    );
}

#[test]
fn update_preserves_empty_string_clear_semantics() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        MockResponse {
            status: "200 OK",
            body: r#"{"id":"app_test1234567890","name":"courseboard","connection_id":"conn_old"}"#,
        },
        MockResponse {
            status: "200 OK",
            body: r#"{"id":"app_test1234567890","name":"courseboard","connection_id":null}"#,
        },
    ]);

    let output = run_update(tmp.path(), api_url, "", true);
    assert_success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("Requested connection: <clear>"));

    let requests = finish_requests(rx, handle);
    assert_eq!(
        request_body(&requests[1]),
        serde_json::json!({"connection_id": ""})
    );
}
