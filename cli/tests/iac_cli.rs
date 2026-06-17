use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
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

fn start_graphql_server_once(
    body: &'static str,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
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
                    return;
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        tx.send(String::new()).unwrap();
                        return;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("accept graphql request: {err}"),
            }
        }
    });
    (url, rx, handle)
}

#[test]
fn iac_apply_reconciles_no_change_manifest() {
    let tmp = TempDir::new().unwrap();
    let manifest = json!({
        "apiVersion": "apps.tachy.one/v1alpha",
        "kind": "CloudApp",
        "metadata": {
            "tenantId": "tn_test1234567890",
            "name": "fieldadmin"
        },
        "spec": {
            "envVars": []
        }
    });
    let manifest_path = tmp.path().join("tachyon.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let state_path = tmp.path().join("tachyon.tfstate");
    fs::write(
        &state_path,
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "serial": 1,
            "lineage": "ln_test",
            "resources": [{
                "kind": "CloudApp",
                "name": "fieldadmin",
                "content_hash": "already-applied",
                "manifest": manifest,
                "applied_at": "2026-06-17T00:00:00Z"
            }]
        }))
        .unwrap(),
    )
    .unwrap();

    let (api_url, rx, handle) = start_graphql_server_once(
        r#"{"data":{"applyManifest":{"success":true,"serviceAccountsCreated":[],"serviceAccountsModified":[],"providersApplied":[],"seedDataTables":[]}}}"#,
    );

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "iac",
            "apply",
            "--file",
            manifest_path.to_str().unwrap(),
            "--state",
            state_path.to_str().unwrap(),
        ])
        .output()
        .expect("run tachyon iac apply");

    assert!(
        output.status.success(),
        "iac apply failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let req = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    handle.join().unwrap();
    assert!(
        req.starts_with("POST /v1/graphql "),
        "unexpected request: {req}"
    );
    assert!(req.contains("applyManifest"));
    assert!(!req.contains("saveManifest"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Reconciled: CloudApp / fieldadmin (no manifest changes)"));
}
