use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

#[test]
fn cli_enable_disable_use_explicit_boolean_and_tenant() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        for enabled in [true, false] {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(10)))
                .unwrap();
            let mut reader = BufReader::new(&mut stream);
            let mut first = String::new();
            reader.read_line(&mut first).unwrap();
            assert!(first.starts_with("POST /v1/graphql "));
            let mut length = 0;
            let mut tenant = false;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse::<usize>().unwrap();
                }
                tenant |= line
                    .trim()
                    .eq_ignore_ascii_case("x-operator-id: tn_test1234567890");
            }
            assert!(tenant);
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            let body: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(body["variables"]["enabled"], enabled);
            assert_eq!(body["variables"]["key"], "feature.example");
            let response = json!({"data": {"setFeatureFlagEnabled": {"key": "feature.example", "outcome": "UPDATED"}}}).to_string();
            write!(stream, "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}", response.len(), response).unwrap();
        }
    });
    let directory = tempfile::tempdir().unwrap();
    for command in ["enable", "disable"] {
        let output = Command::new(env!("CARGO_BIN_EXE_tachyon"))
            .current_dir(directory.path())
            .env_remove("TACHYON_CONFIG")
            .args([
                "--api-url",
                &url,
                "--api-key",
                "test-only",
                "--tenant-id",
                "tn_test1234567890",
                "feature-flag",
                command,
                "feature.example",
                "--json",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], "UPDATED");
    }
    server.join().unwrap();
}
