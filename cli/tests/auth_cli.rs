use std::fs;
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
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_TENANT_ID")
        .env_remove("TACHYON_PROFILE")
        .env_remove("TACHYON_ENV");
    cmd
}

fn write_project_config(dir: &Path) {
    fs::write(
        dir.join("tachyon.yml"),
        "apiVersion: tachyon/v1\nkind: CloudApp\nmetadata:\n  name: test-app\n  tenant_id: tn_test\nspec:\n  framework: vite\n",
    )
    .unwrap();
}

fn start_auth_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for _ in 0..1 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            tx.send(req).unwrap();

            let body = r#"{"client_id":"dummy-client","client_secret":"dummy-secret","secret_arn":"arn:aws:secretsmanager:ap-northeast-1:123:secret:test","expires_at":"2026-06-01T00:00:00Z"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (url, rx, handle)
}

#[test]
fn auth_init_non_interactive_updates_tachyon_yml() {
    let tmp = TempDir::new().unwrap();
    write_project_config(tmp.path());

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .args([
            "auth",
            "init",
            "cognito-default",
            "--type",
            "oauth2_client_credentials",
            "--audience",
            "https://api.tachyon.cloud",
            "--non-interactive",
        ])
        .output()
        .expect("run tachyon auth init");

    assert!(
        output.status.success(),
        "auth init failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let yaml = fs::read_to_string(tmp.path().join("tachyon.yml")).unwrap();
    assert!(yaml.contains("auth:"));
    assert!(yaml.contains("name: cognito-default"));
    assert!(yaml.contains("type: oauth2_client_credentials"));
    assert!(yaml.contains("audience: https://api.tachyon.cloud"));
    assert!(yaml.contains("expiry_days: 90"));
}

#[test]
fn auth_init_requires_tachyon_yml() {
    let tmp = TempDir::new().unwrap();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .args(["auth", "init", "cognito-default", "--non-interactive"])
        .output()
        .expect("run tachyon auth init");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Run `tachyon init` first"));
}

#[test]
fn auth_issue_dev_writes_local_credentials() {
    let tmp = TempDir::new().unwrap();
    write_project_config(tmp.path());
    let init = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .args(["auth", "init", "cognito-default", "--non-interactive"])
        .output()
        .expect("run tachyon auth init");
    assert!(init.status.success());
    let (api_url, rx, handle) = start_auth_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_ENV", "dev")
        .args(["auth", "issue", "cognito-default"])
        .output()
        .expect("run tachyon auth issue");

    assert!(
        output.status.success(),
        "auth issue failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(req.starts_with("POST /v1/cloud-apps/test-app/auth/credentials "));
    assert!(req.contains("authorization: Bearer test-token"));
    assert!(tmp.path().join(".tachyon/credentials.json").is_file());
    assert!(fs::read_to_string(tmp.path().join(".gitignore"))
        .unwrap()
        .contains(".tachyon/credentials.json"));
    let yaml = fs::read_to_string(tmp.path().join("tachyon.yml")).unwrap();
    assert!(yaml.contains("secret_ref: .tachyon/credentials.json#cognito-default"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("dummy-secret"));
}

#[test]
fn auth_issue_staging_updates_secret_ref_without_local_file() {
    let tmp = TempDir::new().unwrap();
    write_project_config(tmp.path());
    let init = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .args(["auth", "init", "cognito-default", "--non-interactive"])
        .output()
        .expect("run tachyon auth init");
    assert!(init.status.success());
    let (api_url, rx, handle) = start_auth_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_ENV", "staging")
        .args(["auth", "issue", "cognito-default"])
        .output()
        .expect("run tachyon auth issue");

    assert!(
        output.status.success(),
        "auth issue failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(!tmp.path().join(".tachyon/credentials.json").exists());
    let yaml = fs::read_to_string(tmp.path().join("tachyon.yml")).unwrap();
    assert!(yaml.contains("secret_ref: arn:aws:secretsmanager"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("dummy-secret"));
}
