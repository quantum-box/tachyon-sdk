use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tempfile::TempDir;

const TENANT_ID: &str = "tn_01fakelinearreconcile";
const REQUEST_PATH: &str = "/v1beta/tn_01fakelinearreconcile/pm/issues";

enum Response {
    Json { status: &'static str, body: Value },
    Disconnect,
}

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_tachyon")
}

fn isolated_command(home: &Path) -> Command {
    let mut command = Command::new(bin());
    command
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_TENANT_ID", TENANT_ID)
        .env("TACHYON_API_KEY", "fake-api-key")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE")
        .env_remove("TACHYON_PM_NO_DELEGATE")
        .env_remove("TACHYON_PM_DEFAULT_TEAM");
    command
}

fn run_create(api_url: &str, home: &Path) -> Output {
    isolated_command(home)
        .env("TACHYON_API_URL", api_url)
        .args([
            "linear",
            "issue",
            "create",
            "--team",
            "PLT",
            "--title",
            "Fake reconcile test issue",
            "--json",
        ])
        .output()
        .expect("run tachyon linear issue create")
}

fn run_unassigned_create(api_url: &str, home: &Path) -> Output {
    isolated_command(home)
        .env("TACHYON_API_URL", api_url)
        .args([
            "issue",
            "create",
            "--provider",
            "linear",
            "--team",
            "PLT",
            "--title",
            "Fake reconcile test issue",
            "--delegate-id",
            "",
            "--json",
        ])
        .output()
        .expect("run tachyon issue create with an empty delegate id")
}

fn read_request(stream: &mut std::net::TcpStream) -> String {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 4_096];

    loop {
        let size = stream.read(&mut buffer).unwrap();
        if size == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..size]);

        let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                line.to_ascii_lowercase()
                    .strip_prefix("content-length: ")
                    .and_then(|value| value.parse::<usize>().ok())
            })
            .unwrap_or(0);
        if request.len() >= header_end + 4 + content_length {
            break;
        }
    }

    String::from_utf8_lossy(&request).to_string()
}

fn start_server(
    responses: Vec<Response>,
) -> (String, mpsc::Receiver<Vec<String>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        let mut response_index = 0;
        let mut idle_since = None;
        let started = Instant::now();

        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    requests.push(read_request(&mut stream));

                    match responses.get(response_index) {
                        Some(Response::Json { status, body }) => {
                            let body = serde_json::to_string(body).unwrap();
                            let response = format!(
                                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\n\
                                 content-length: {}\r\nconnection: close\r\n\r\n{body}",
                                body.len()
                            );
                            stream.write_all(response.as_bytes()).unwrap();
                        }
                        Some(Response::Disconnect) => {}
                        None => {
                            let body = r#"{"error":"unexpected retry"}"#;
                            let response = format!(
                                "HTTP/1.1 500 Internal Server Error\r\n\
                                 content-type: application/json\r\ncontent-length: {}\r\n\
                                 connection: close\r\n\r\n{body}",
                                body.len()
                            );
                            stream.write_all(response.as_bytes()).unwrap();
                        }
                    }

                    response_index += 1;
                    if response_index >= responses.len() {
                        idle_since = Some(Instant::now());
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if idle_since.is_some_and(|idle| idle.elapsed() >= Duration::from_millis(300)) {
                        break;
                    }
                    if started.elapsed() >= Duration::from_secs(5) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept request: {error}"),
            }
        }

        tx.send(requests).unwrap();
    });

    (url, rx, handle)
}

fn reconcile_pending(retry_after_seconds: u64, reconcile_interval_seconds: u64) -> Value {
    json!({
        "error": "Linear OAuth access token expired; background reconcile pending",
        "provider": "linear",
        "operation": "linear_oauth_client_resolve",
        "reason": "token_expired",
        "state": "expired",
        "retryable": true,
        "token_refresh_required": false,
        "recovery": {
            "type": "reconcile_pending",
            "retry_after_seconds": retry_after_seconds,
            "reconcile_interval_seconds": reconcile_interval_seconds
        }
    })
}

fn reconnect_required() -> Value {
    json!({
        "error": "Linear OAuth connection is inactive; reconnect Linear",
        "provider": "linear",
        "operation": "linear_oauth_client_resolve",
        "reason": "connection_paused",
        "state": "inactive",
        "retryable": false,
        "token_refresh_required": true,
        "recovery": {
            "type": "reconnect_required"
        }
    })
}

fn created_issue() -> Value {
    json!({
        "issue": {
            "provider": "linear",
            "id": "issue_fake_2909",
            "key": "PLT-2909",
            "title": "Fake reconcile test issue",
            "url": "https://linear.example.test/PLT-2909",
            "status": "Todo",
            "priority": "high"
        },
        "created": true
    })
}

fn finish_requests(rx: mpsc::Receiver<Vec<String>>, handle: thread::JoinHandle<()>) -> Vec<String> {
    handle.join().unwrap();
    rx.recv().unwrap()
}

fn request_body(request: &str) -> Value {
    let (_, body) = request
        .split_once("\r\n\r\n")
        .expect("request contains a body");
    serde_json::from_str(body).expect("request body is JSON")
}

#[test]
fn issue_create_help_documents_empty_delegate_id_for_all_aliases() {
    let temporary_home = TempDir::new().unwrap();
    let aliases: &[&[&str]] = &[
        &["issue", "create", "--help"],
        &["pm", "issue", "create", "--help"],
        &["linear", "issue", "create", "--help"],
    ];

    for args in aliases {
        let output = isolated_command(temporary_home.path())
            .args(*args)
            .output()
            .expect("render issue create help");
        assert!(
            output.status.success(),
            "{} help failed:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );

        let help = String::from_utf8_lossy(&output.stdout);
        assert!(
            help.contains("pass an empty string (`--delegate-id \"\"`)"),
            "{} help does not document the empty value:\n{help}",
            args.join(" ")
        );
        assert!(
            help.contains("disable automatic delegation and create the issue unassigned"),
            "{} help does not explain the effect:\n{help}",
            args.join(" ")
        );
    }
}

#[test]
fn empty_delegate_id_disables_linear_auto_delegate() {
    let temporary_home = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![Response::Json {
        status: "201 Created",
        body: created_issue(),
    }]);

    let output = run_unassigned_create(&api_url, temporary_home.path());
    let requests = finish_requests(rx, handle);

    assert!(
        output.status.success(),
        "issue create failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(requests.len(), 1);
    let body = request_body(&requests[0]);
    assert_eq!(body["delegate_id"], "");
    assert_eq!(
        body.get("auto_delegate_to_linear_agent"),
        None,
        "false is omitted from the request; the field must only be sent when auto-delegation is enabled"
    );
}

#[test]
fn waits_for_reconcile_then_creates_issue_and_preserves_json_stdout() {
    let temporary_home = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![
        Response::Json {
            status: "424 Failed Dependency",
            body: reconcile_pending(0, 600),
        },
        Response::Json {
            status: "201 Created",
            body: created_issue(),
        },
    ]);

    let output = run_create(&api_url, temporary_home.path());
    let requests = finish_requests(rx, handle);

    assert!(
        output.status.success(),
        "issue create failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(requests.len(), 2);
    assert!(requests
        .iter()
        .all(|request| request.starts_with(&format!("POST {REQUEST_PATH} "))));

    let stdout: Value = serde_json::from_slice(&output.stdout).expect("stdout remains JSON");
    assert_eq!(stdout, created_issue());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("reconcile is pending"));
    assert!(stderr.contains("retrying issue creation in 0 seconds"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("reconcile is pending"));
}

#[test]
fn reconcile_wait_timeout_requests_operational_checks_without_reconnect_claim() {
    let temporary_home = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![Response::Json {
        status: "424 Failed Dependency",
        body: reconcile_pending(1_801, 600),
    }]);

    let output = run_create(&api_url, temporary_home.path());
    let requests = finish_requests(rx, handle);

    assert!(!output.status.success());
    assert_eq!(requests.len(), 1);
    let stdout: Value = serde_json::from_slice(&output.stdout).expect("timeout stdout is JSON");
    assert_eq!(stdout["error"], "linear_oauth_reconcile_timeout");
    assert_eq!(stdout["provider"], "linear");
    assert_eq!(stdout["operation"], "linear_oauth_client_resolve");
    assert_eq!(stdout["waited_seconds"], 0);
    assert_eq!(stdout["wait_timeout_seconds"], 1_800);
    assert!(stdout["message"]
        .as_str()
        .unwrap()
        .contains("reconcile is incomplete"));
    assert!(!stdout
        .to_string()
        .to_ascii_lowercase()
        .contains("reconnect"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("reconcile is incomplete"));
    assert!(stderr.contains("1800-second wait deadline"));
    assert!(stderr.contains("reconcile worker operation"));
    assert!(stderr.contains("connection state"));
    assert!(!stderr.to_ascii_lowercase().contains("reconnect"));
}

#[test]
fn reconnect_required_is_not_retried() {
    let temporary_home = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![Response::Json {
        status: "424 Failed Dependency",
        body: reconnect_required(),
    }]);

    let output = run_create(&api_url, temporary_home.path());
    let requests = finish_requests(rx, handle);

    assert!(!output.status.success());
    assert_eq!(requests.len(), 1);
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("reconnect_required"));
}

#[test]
fn unrelated_http_error_is_not_retried() {
    let temporary_home = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![Response::Json {
        status: "503 Service Unavailable",
        body: json!({"error": "provider unavailable"}),
    }]);

    let output = run_create(&api_url, temporary_home.path());
    let requests = finish_requests(rx, handle);

    assert!(!output.status.success());
    assert_eq!(requests.len(), 1);
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("provider unavailable"));
}

#[test]
fn transport_failure_after_request_arrival_is_not_retried() {
    let temporary_home = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![Response::Disconnect]);

    let output = run_create(&api_url, temporary_home.path());
    let requests = finish_requests(rx, handle);

    assert!(!output.status.success());
    assert_eq!(requests.len(), 1);
    assert!(requests[0].starts_with(&format!("POST {REQUEST_PATH} ")));
    assert!(requests[0].contains(r#""title":"Fake reconcile test issue""#));
    assert!(output.stdout.is_empty());
}

#[test]
fn unauthorized_response_is_not_retried_for_issue_creation() {
    let temporary_home = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_server(vec![Response::Json {
        status: "401 Unauthorized",
        body: json!({"error": "authentication rejected"}),
    }]);

    let output = run_create(&api_url, temporary_home.path());
    let requests = finish_requests(rx, handle);

    assert!(!output.status.success());
    assert_eq!(requests.len(), 1);
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("authentication rejected"));
}
