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
        .env("TACHYON_API_KEY", "test-token")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE");
    cmd
}

fn start_server(body: &'static str) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 8192];
        let n = stream.read(&mut buf).unwrap();
        tx.send(String::from_utf8_lossy(&buf[..n]).to_string())
            .unwrap();

        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, rx, handle)
}

fn run_list(home: &Path, api_url: String, json: bool) -> Output {
    let mut command = isolated_command(home);
    command
        .env("TACHYON_API_URL", api_url)
        .args(["ops", "deployments", "list"]);
    if json {
        command.arg("--json");
    }
    command.output().expect("run tachyon ops deployments list")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn finish_request(rx: mpsc::Receiver<String>, handle: thread::JoinHandle<()>) -> String {
    let request = rx.recv().unwrap();
    handle.join().unwrap();
    request
}

fn assert_list_request(request: &str) {
    assert!(
        request.starts_with("GET /v1/ops/deployments "),
        "request was:\n{request}"
    );
    assert!(
        request.contains(&format!("x-operator-id: {TENANT_ID}")),
        "request was:\n{request}"
    );
}

#[test]
fn deployments_list_decodes_api_envelope_and_prints_items() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(
        r#"{"deployments":[{"id":"dep_123456789012","service":"tachyon-api","environment":"production","status":"success","version":"0.92.134","created_at":"2026-08-09T00:00:00Z"}],"total":1}"#,
    );

    let output = run_list(tmp.path(), api_url, true);
    assert_success(&output);

    let request = finish_request(rx, handle);
    assert_list_request(&request);
    let deployments: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).expect("deployments json");
    assert_eq!(deployments.len(), 1);
    assert_eq!(deployments[0]["id"], "dep_123456789012");
    assert_eq!(deployments[0]["service"], "tachyon-api");
}

#[test]
fn deployments_list_handles_empty_api_envelope() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(r#"{"deployments":[],"total":0}"#);

    let output = run_list(tmp.path(), api_url, false);
    assert_success(&output);

    let request = finish_request(rx, handle);
    assert_list_request(&request);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "No deployment events found.\n"
    );
}
