use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

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
        .env_remove("TACHYON_PROFILE");
    cmd
}

fn start_models_server(
    body: &'static str,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 4096];
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

#[test]
fn agent_models_json_decodes_models_envelope() {
    let tmp = TempDir::new().unwrap();
    let (api_url, rx, handle) = start_models_server(
        r#"{"models":[{"id":"openai/gpt-5.5","name":"GPT-5.5","provider":"openai"}]}"#,
    );

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args(["agent", "models", "--json"])
        .output()
        .expect("run tachyon agent models");

    assert!(
        output.status.success(),
        "agent models failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(req.starts_with("GET /v1/llms/models "));
    assert!(req.contains("authorization: Bearer test-token"));
    assert!(req.contains("x-operator-id: tn_test1234567890"));

    let models: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).expect("models json array");
    assert_eq!(models.len(), 1);
    assert_eq!(models[0]["id"], "openai/gpt-5.5");
    assert_eq!(models[0]["name"], "GPT-5.5");
    assert_eq!(models[0]["provider"], "openai");
}
