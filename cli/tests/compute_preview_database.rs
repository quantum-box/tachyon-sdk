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

#[test]
fn delete_without_yes_explains_and_sends_nothing() {
    let tmp = TempDir::new().unwrap();

    // Port 1 is unreachable: if the command sent a request it would fail
    // rather than print the dry-run explanation.
    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", "http://127.0.0.1:1")
        .args([
            "compute", "preview", "database", "delete", "--app", APP_ID, "--pr", "263",
        ])
        .output()
        .unwrap();
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No changes made"), "stdout was:\n{stdout}");
    assert!(stdout.contains("--yes"), "stdout was:\n{stdout}");
}

#[test]
fn delete_rejects_a_non_positive_pull_request_number() {
    let tmp = TempDir::new().unwrap();

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", "http://127.0.0.1:1")
        .args([
            "compute", "preview", "database", "delete", "--app", APP_ID, "--pr", "0", "--yes",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("positive"), "stderr was:\n{stderr}");
}

#[test]
fn delete_with_yes_calls_the_endpoint_and_reports_recreation() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"deleted":true}"#,
    }]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute", "preview", "database", "delete", "--app", APP_ID, "--pr", "263", "--yes",
        ])
        .output()
        .unwrap();
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Deleted the preview database"),
        "stdout was:\n{stdout}"
    );
    // The database is recreated by the next build, not by this command;
    // saying so is what stops a caller from expecting a usable database.
    assert!(stdout.contains("next build"), "stdout was:\n{stdout}");

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0].starts_with(&format!("DELETE /v1/apps/{APP_ID}/previews/263/database ")),
        "request was:\n{}",
        requests[0]
    );
}

#[test]
fn delete_reports_an_absent_database_as_success() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![MockResponse {
        status: "200 OK",
        body: r#"{"deleted":false}"#,
    }]);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "compute", "preview", "database", "delete", "--app", APP_ID, "--pr", "263", "--yes",
        ])
        .output()
        .unwrap();
    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No preview database was provisioned"),
        "stdout was:\n{stdout}"
    );

    finish_requests(rx, handle);
}
