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
        .env("TACHYON_API_KEY", "test-token")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE")
        .env_remove("TACHYON_TENANT_ID");
    cmd
}

fn start_server(
    bodies: Vec<&'static str>,
) -> (String, mpsc::Receiver<Vec<String>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut requests = Vec::with_capacity(bodies.len());
        for body in bodies {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            requests.push(String::from_utf8_lossy(&buf[..n]).to_string());

            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
        tx.send(requests).unwrap();
    });
    (url, rx, handle)
}

fn run(home: &Path, api_url: &str, args: &[&str]) -> Output {
    isolated_command(home)
        .env("TACHYON_API_URL", api_url)
        .args(["--tenant-id", TENANT_ID])
        .args(args)
        .output()
        .expect("run tachyon list command")
}

fn assert_success(output: &Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn finish_requests(rx: mpsc::Receiver<Vec<String>>, handle: thread::JoinHandle<()>) -> Vec<String> {
    let requests = rx.recv().unwrap();
    handle.join().unwrap();
    requests
}

fn assert_request(request: &str, expected_path: &str) {
    assert!(
        request.starts_with(&format!("GET {expected_path} ")),
        "request was:\n{request}"
    );
    assert!(
        request.contains("authorization: Bearer test-token"),
        "request was:\n{request}"
    );
    assert!(
        request.contains(&format!("x-operator-id: {TENANT_ID}")),
        "request was:\n{request}"
    );
}

#[test]
fn list_commands_decode_their_api_envelopes() {
    struct Case {
        label: &'static str,
        args: &'static [&'static str],
        path: &'static str,
        body: &'static str,
    }

    let cases = [
        Case {
            label: "org policies actions",
            args: &["org", "policies", "actions", "--json"],
            path: "/v1/auth/actions",
            body: r#"{"actions":[]}"#,
        },
        Case {
            label: "agent protocols list",
            args: &["agent", "protocols", "list", "--json"],
            path: "/v1/llms/agent-protocols",
            body: r#"{"items":[],"next_cursor":null}"#,
        },
        Case {
            label: "agent workers list",
            args: &["agent", "workers", "list", "--json"],
            path: "/v1/agent/workers",
            body: r#"{"workers":[]}"#,
        },
        Case {
            label: "agent worktrees list",
            args: &["agent", "worktrees", "list", "--json"],
            path: "/v1/agent/worktrees",
            body: r#"{"worktrees":[],"total":0}"#,
        },
        Case {
            label: "agent memory list",
            args: &["agent", "memory", "list", "--json"],
            path: "/v1/agent/memory",
            body: r#"{"memories":[]}"#,
        },
        Case {
            label: "iac integrations list",
            args: &["iac", "integrations", "list", "--json"],
            path: "/v1/integrations",
            body: r#"{"integrations":[]}"#,
        },
        Case {
            label: "ops reports list",
            args: &["ops", "reports", "list", "--json"],
            path: "/v1/ops/scenario-reports",
            body: r#"{"test_runs":[],"total":0}"#,
        },
    ];

    for case in cases {
        let tmp = TempDir::new().unwrap();
        let (api_url, rx, handle) = start_server(vec![case.body]);
        let output = run(tmp.path(), &api_url, case.args);
        assert_success(&output, case.label);

        let requests = finish_requests(rx, handle);
        assert_eq!(requests.len(), 1);
        assert_request(&requests[0], case.path);
        let value: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("list JSON output");
        assert_eq!(value, serde_json::json!([]), "{} output", case.label);
    }
}

#[test]
fn name_resolvers_decode_list_envelopes() {
    struct Case {
        label: &'static str,
        args: &'static [&'static str],
        list_path: &'static str,
        get_path: &'static str,
        list_body: &'static str,
        get_body: &'static str,
        expected_id: &'static str,
    }

    let cases = [
        Case {
            label: "worker resolver",
            args: &["agent", "workers", "get", "worker-one", "--json"],
            list_path: "/v1/agent/workers",
            get_path: "/v1/agent/workers/wrk_123456789012",
            list_body: r#"{"workers":[{"id":"wrk_123456789012","name":"worker-one"}]}"#,
            get_body: r#"{"id":"wrk_123456789012","name":"worker-one"}"#,
            expected_id: "wrk_123456789012",
        },
        Case {
            label: "protocol resolver",
            args: &["agent", "protocols", "get", "protocol-one", "--json"],
            list_path: "/v1/llms/agent-protocols",
            get_path: "/v1/llms/agent-protocols/ap_123456789012",
            list_body: r#"{"items":[{"id":"ap_123456789012","name":"protocol-one"}],"next_cursor":null}"#,
            get_body: r#"{"id":"ap_123456789012","name":"protocol-one"}"#,
            expected_id: "ap_123456789012",
        },
        Case {
            label: "integration resolver",
            args: &["iac", "integrations", "get", "integration-one", "--json"],
            list_path: "/v1/integrations",
            get_path: "/v1/integrations/int_123456789012",
            list_body: r#"{"integrations":[{"id":"int_123456789012","name":"integration-one"}]}"#,
            get_body: r#"{"id":"int_123456789012","name":"integration-one"}"#,
            expected_id: "int_123456789012",
        },
    ];

    for case in cases {
        let tmp = TempDir::new().unwrap();
        let (api_url, rx, handle) = start_server(vec![case.list_body, case.get_body]);
        let output = run(tmp.path(), &api_url, case.args);
        assert_success(&output, case.label);

        let requests = finish_requests(rx, handle);
        assert_eq!(requests.len(), 2);
        assert_request(&requests[0], case.list_path);
        assert_request(&requests[1], case.get_path);
        let value: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("resolved resource JSON output");
        assert_eq!(value["id"], case.expected_id, "{} output", case.label);
    }
}

#[test]
fn operators_list_keeps_decoding_the_api_bare_array() {
    let tmp = TempDir::new().unwrap();
    let body = r#"[{"id":"tn_operator1234567890","name":"Operator"}]"#;
    let (api_url, rx, handle) = start_server(vec![body]);
    let output = run(
        tmp.path(),
        &api_url,
        &["org", "operators", "list", "--json"],
    );
    assert_success(&output, "org operators list");

    let requests = finish_requests(rx, handle);
    assert_eq!(requests.len(), 1);
    assert_request(&requests[0], "/v1/auth/operators/by-user");
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("operators JSON output");
    assert_eq!(value.as_array().map(Vec::len), Some(1));
    assert_eq!(value[0]["id"], "tn_operator1234567890");
}
