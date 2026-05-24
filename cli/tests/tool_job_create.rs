use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

use serde_json::Value;
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

fn start_tool_jobs_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 16384];
        let n = stream.read(&mut buf).unwrap();
        let req = String::from_utf8_lossy(&buf[..n]).to_string();
        tx.send(req).unwrap();

        let body = r#"{"job":{"id":"job_01testtooljob","provider":"codex","status":"queued"}}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, rx, handle)
}

fn request_json_body(request: &str) -> Value {
    let body = request.split("\r\n\r\n").nth(1).unwrap();
    serde_json::from_str(body).unwrap()
}

fn write_cloud_apps_manifest(tmp: &TempDir) {
    fs::write(
        tmp.path().join("tachyon.yml"),
        r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: workspace
  tenantId: tn_test1234567890
spec:
  apps:
    - name: fieldadmin
      repository:
        url: https://github.com/quantum-box/tachyonfield
        owner: quantum-box
        name: tachyonfield
        defaultBranch: main
        localPath: repos/tachyonfield
    - name: fieldapi
      repository:
        url: https://github.com/quantum-box/tachyonfield
        owner: quantum-box
        name: tachyonfield
        defaultBranch: main
        localPath: repos/tachyonfield
    - name: otherapp
      repository:
        url: https://github.com/quantum-box/other
        owner: quantum-box
        name: other
        defaultBranch: main
        localPath: /opt/other
"#,
    )
    .unwrap();
}

#[test]
fn tool_jobs_create_uses_repo_local_path_as_cwd() {
    let tmp = TempDir::new().unwrap();
    write_cloud_apps_manifest(&tmp);
    let (api_url, rx, handle) = start_tool_jobs_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "ops",
            "tool-jobs",
            "create",
            "--provider",
            "codex",
            "--repo",
            "tachyonfield",
            "--prompt",
            "inspect",
            "--json",
        ])
        .output()
        .expect("run tachyon ops tool-jobs create");

    assert!(
        output.status.success(),
        "tool-jobs create failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    handle.join().unwrap();
    let req = rx.recv().unwrap();
    assert!(req.starts_with("POST /v1/agent/tool-jobs "));
    assert!(req.contains("authorization: Bearer test-token"));
    assert!(req.contains("x-operator-id: tn_test1234567890"));
    let body = request_json_body(&req);
    assert_eq!(body["provider"], "codex");
    assert_eq!(body["prompt"], "inspect");
    assert_eq!(
        body["metadata"]["cwd"],
        tmp.path().join("repos/tachyonfield").display().to_string()
    );
    assert_eq!(body["metadata"]["source"], "tachyon-cli");
    assert_eq!(body["metadata"]["codex_mode"], "app_server_ws");
}

#[test]
fn tool_jobs_create_uses_cloud_app_repo_local_path_as_cwd() {
    let tmp = TempDir::new().unwrap();
    write_cloud_apps_manifest(&tmp);
    let (api_url, rx, handle) = start_tool_jobs_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "ops",
            "tool-jobs",
            "create",
            "--provider",
            "codex",
            "--cloud-app",
            "fieldadmin",
            "--prompt",
            "inspect",
            "--json",
        ])
        .output()
        .expect("run tachyon ops tool-jobs create");

    assert!(
        output.status.success(),
        "tool-jobs create failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    handle.join().unwrap();
    let req = rx.recv().unwrap();
    let body = request_json_body(&req);
    assert_eq!(
        body["metadata"]["cwd"],
        tmp.path().join("repos/tachyonfield").display().to_string()
    );
}

#[test]
fn tool_jobs_create_rejects_multiple_cwd_selectors() {
    let tmp = TempDir::new().unwrap();
    write_cloud_apps_manifest(&tmp);

    for args in [
        vec!["--cwd", "/tmp/repo", "--repo", "tachyonfield"],
        vec!["--cwd", "/tmp/repo", "--cloud-app", "fieldadmin"],
        vec!["--repo", "tachyonfield", "--cloud-app", "fieldadmin"],
    ] {
        let mut command_args = vec!["ops", "tool-jobs", "create", "--provider", "codex"];
        command_args.extend(args);
        command_args.extend(["--prompt", "inspect"]);

        let output = isolated_command(tmp.path())
            .current_dir(tmp.path())
            .args(command_args)
            .output()
            .expect("run tachyon ops tool-jobs create");

        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains("--cwd, --repo, and --cloud-app are mutually exclusive"));
    }
}

#[test]
fn tool_jobs_create_reports_unknown_repo() {
    let tmp = TempDir::new().unwrap();
    write_cloud_apps_manifest(&tmp);

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .args([
            "ops",
            "tool-jobs",
            "create",
            "--provider",
            "codex",
            "--repo",
            "missing",
            "--prompt",
            "inspect",
        ])
        .output()
        .expect("run tachyon ops tool-jobs create");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown repo 'missing'"));
    assert!(stderr.contains("Available repos: other, tachyonfield"));
}

#[test]
fn tool_jobs_create_reports_unknown_cloud_app() {
    let tmp = TempDir::new().unwrap();
    write_cloud_apps_manifest(&tmp);

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .args([
            "ops",
            "tool-jobs",
            "create",
            "--provider",
            "codex",
            "--cloud-app",
            "missing",
            "--prompt",
            "inspect",
        ])
        .output()
        .expect("run tachyon ops tool-jobs create");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown cloud app 'missing'"));
    assert!(stderr.contains("Available cloud apps: fieldadmin, fieldapi, otherapp"));
}
