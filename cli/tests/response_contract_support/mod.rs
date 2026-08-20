use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;

use tempfile::TempDir;

const TENANT_ID: &str = "tn_test1234567890";

pub struct Case {
    pub label: &'static str,
    pub args: &'static [&'static str],
    pub path: &'static str,
    pub valid_body: &'static str,
    pub invalid_body: &'static str,
    pub missing_field: &'static str,
    pub forbidden_output: Option<&'static str>,
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

fn run(home: &Path, api_url: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tachyon"))
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_API_URL", api_url)
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE")
        .env_remove("TACHYON_TENANT_ID")
        .args(["--tenant-id", TENANT_ID])
        .args(args)
        .output()
        .expect("run tachyon response-contract command")
}

pub fn run_cases(cases: &[Case]) {
    for case in cases {
        for (body, succeeds) in [(case.valid_body, true), (case.invalid_body, false)] {
            let tmp = TempDir::new().unwrap();
            let (api_url, rx, handle) = start_server(body);
            let output = run(tmp.path(), &api_url, case.args);
            let request = rx.recv().unwrap();
            handle.join().unwrap();
            assert!(
                request.starts_with(&format!("GET {} ", case.path)),
                "{} request was:\n{}",
                case.label,
                request
            );

            if succeeds {
                assert!(
                    output.status.success(),
                    "{} valid response failed\nstdout:\n{}\nstderr:\n{}",
                    case.label,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                if let Some(forbidden) = case.forbidden_output {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    assert!(
                        !stdout.contains(forbidden),
                        "{} leaked forbidden output {forbidden:?}: {stdout}",
                        case.label
                    );
                }
            } else {
                assert!(
                    !output.status.success(),
                    "{} mismatched response unexpectedly decoded",
                    case.label
                );
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(
                    stderr.contains(case.missing_field),
                    "{} did not report missing {:?}; stderr was:\n{}",
                    case.label,
                    case.missing_field,
                    stderr
                );
            }
        }
    }
}
