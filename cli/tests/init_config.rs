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
        .env_remove("TACHYON_PROFILE");
    cmd
}

fn write_project_config(dir: &Path, name: &str, tenant_id: &str) {
    fs::write(
        dir.join("tachyon.yml"),
        format!(
            "apiVersion: tachyon/v1\nkind: CloudApp\nmetadata:\n  name: {name}\n  tenant_id: {tenant_id}\nspec:\n  framework: vite\n"
        ),
    )
    .unwrap();
}

fn start_compute_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for idx in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 4096];
            let n = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            tx.send(req).unwrap();

            let body = if idx == 0 {
                r#"{"apps":[{"id":"app_configured","name":"configured-app"},{"id":"app_explicit","name":"explicit-app"}]}"#
            } else {
                r#"{"builds":[]}"#
            };
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
fn init_non_interactive_creates_and_force_overwrites_tachyon_yml() {
    let tmp = TempDir::new().unwrap();
    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .args([
            "init",
            "--non-interactive",
            "--name",
            "plt1098-verify",
            "--framework",
            "vite",
            "--tenant-id",
            "test-tenant",
        ])
        .output()
        .expect("run tachyon init");
    assert!(
        output.status.success(),
        "init failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let yaml = fs::read_to_string(tmp.path().join("tachyon.yml")).unwrap();
    assert!(yaml.contains("name: plt1098-verify"));
    assert!(yaml.contains("tenant_id: test-tenant"));
    assert!(yaml.contains("framework: vite"));

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .args(["init", "--non-interactive", "--tenant-id", "test-tenant"])
        .output()
        .expect("run tachyon init existing");
    assert!(
        !output.status.success(),
        "init should fail for existing file"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"));

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .args([
            "init",
            "--non-interactive",
            "--name",
            "overwritten",
            "--framework",
            "static",
            "--tenant-id",
            "test-tenant",
            "--force",
        ])
        .output()
        .expect("run tachyon init force");
    assert!(output.status.success(), "force overwrite should succeed");
    let yaml = fs::read_to_string(tmp.path().join("tachyon.yml")).unwrap();
    assert!(yaml.contains("name: overwritten"));
    assert!(yaml.contains("framework: static"));
}

#[test]
fn compute_builds_list_uses_project_config_from_parent_dir() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".git")).unwrap();
    fs::create_dir_all(tmp.path().join("nested/work")).unwrap();
    write_project_config(tmp.path(), "configured-app", "op_123456789012");
    let (api_url, rx, handle) = start_compute_server();

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path().join("nested/work"))
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .args(["compute", "builds", "list", "--json"])
        .output()
        .expect("run compute builds list");
    assert!(
        output.status.success(),
        "compute failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let first = rx.recv().unwrap();
    let second = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(first.starts_with("GET /v1/compute/apps "));
    assert!(first.contains("x-operator-id: op_123456789012"));
    assert!(second.starts_with("GET /v1/compute/apps/app_configured/builds "));
    assert!(second.contains("x-operator-id: op_123456789012"));
}

#[test]
fn explicit_app_id_overrides_project_config_name() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".git")).unwrap();
    write_project_config(tmp.path(), "configured-app", "op_123456789012");
    let (api_url, rx, handle) = start_compute_server();

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .args(["compute", "builds", "list", "explicit-app", "--json"])
        .output()
        .expect("run compute builds list explicit");
    assert!(
        output.status.success(),
        "compute failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let _first = rx.recv().unwrap();
    let second = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(second.starts_with("GET /v1/compute/apps/app_explicit/builds "));
}
