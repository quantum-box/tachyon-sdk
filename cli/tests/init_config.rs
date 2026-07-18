use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

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
        .env_remove("TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN")
        .env_remove("TACHYON_SECRET_VALUE");
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

fn build_runner_backend_availability_body() -> &'static str {
    r#"{"default_backend":null,"backends":[{"backend":"codebuild","available":true},{"backend":"kubernetes_kata","available":true}]}"#
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

fn start_apply_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for _ in 0..8 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            tx.send(req.clone()).unwrap();

            let body = if req.starts_with("GET /v1/compute/apps ") {
                r#"{"apps":[]}"#
            } else if req.starts_with("GET /v1/build-runner-backends ") {
                build_runner_backend_availability_body()
            } else if req.starts_with("POST /v1/compute/apps ") {
                r#"{"id":"app_created","name":"bakuure-api","repository_url":"https://github.com/quantum-box/erp","repository_owner":"quantum-box","repository_name":"erp","default_branch":"main","framework":"cargo_lambda","deployment_target":"lambda","root_directory":"apps/bakuure-api","docker_context":".","build_command":"cargo lambda build --package bakuure-api --bin lambda-bakuure-api --release --arm64","buildspec_strategy":"repo:.codebuild/bakuure_api_lambda_buildspec.yml"}"#
            } else if req.starts_with("PUT /v1/compute/apps/app_created/env ") {
                r#"{"env_vars":[{"id":"env_01testtesttesttesttesttest","key":"ENVIRONMENT","value":"sandbox","target":"all","is_secret":false}]}"#
            } else if req.starts_with("GET /v1/compute/apps/app_created/env ") {
                r#"{"env_vars":[{"id":"env_secret","key":"DATABASE_URL","value":"********","target":"all","is_secret":true}]}"#
            } else if req.starts_with("POST /v1/graphql ") {
                r#"{"data":{"saveManifest":{"kind":"CloudApp"},"applyManifest":{"success":true}}}"#
            } else {
                r#"{"error":"unexpected request"}"#
            };
            let status = if body.contains("unexpected request") {
                "HTTP/1.1 500 Internal Server Error"
            } else {
                "HTTP/1.1 200 OK"
            };
            let response = format!(
                "{status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (url, rx, handle)
}

fn start_collecting_apply_server() -> (String, mpsc::Receiver<Vec<String>>, thread::JoinHandle<()>)
{
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut requests = Vec::new();
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_nonblocking(false).unwrap();
                    let mut buf = [0_u8; 8192];
                    let n = stream.read(&mut buf).unwrap();
                    let req = String::from_utf8_lossy(&buf[..n]).to_string();
                    requests.push(req.clone());

                    let body = if req.starts_with("GET /v1/compute/apps ") {
                        r#"{"apps":[]}"#
                    } else if req.starts_with("GET /v1/build-runner-backends ") {
                        build_runner_backend_availability_body()
                    } else if req.starts_with("POST /v1/internal-service-refs/preflight ") {
                        r#"{"checked":1}"#
                    } else if req.starts_with("POST /v1/compute/apps ") {
                        r#"{"id":"app_created","name":"valid-app","repository_url":"https://github.com/quantum-box/erp","repository_owner":"quantum-box","repository_name":"erp","default_branch":"main","framework":"next_js","deployment_target":"cloud_run"}"#
                    } else if req.starts_with("POST /v1/graphql ") {
                        r#"{"data":{"saveManifest":{"kind":"CloudApp"},"applyManifest":{"success":true}}}"#
                    } else {
                        r#"{"error":"unexpected request"}"#
                    };
                    let status = if body.contains("unexpected request") {
                        "HTTP/1.1 500 Internal Server Error"
                    } else {
                        "HTTP/1.1 200 OK"
                    };
                    let response = format!(
                        "{status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream.write_all(response.as_bytes()).unwrap();
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        tx.send(requests).unwrap();
                        return;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("accept apply request: {err}"),
            }
        }
    });
    (url, rx, handle)
}

fn start_internal_service_preflight_error_server(
) -> (String, mpsc::Receiver<Vec<String>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut requests = Vec::new();
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_nonblocking(false).unwrap();
                    let mut buf = [0_u8; 8192];
                    let n = stream.read(&mut buf).unwrap();
                    let req = String::from_utf8_lossy(&buf[..n]).to_string();
                    requests.push(req.clone());

                    let (status, body) = if req.starts_with("GET /v1/build-runner-backends ") {
                        ("HTTP/1.1 200 OK", build_runner_backend_availability_body())
                    } else if req.starts_with("POST /v1/internal-service-refs/preflight ") {
                        (
                            "HTTP/1.1 400 Bad Request",
                            r#"{"error":"internalService preflight failed: env var fieldadmin:TACHYON_FIELD_API_URL references missing internalService app tachyon-field-api in production"}"#,
                        )
                    } else {
                        (
                            "HTTP/1.1 500 Internal Server Error",
                            r#"{"error":"unexpected request"}"#,
                        )
                    };
                    let response = format!(
                        "{status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream.write_all(response.as_bytes()).unwrap();
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        tx.send(requests).unwrap();
                        return;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("accept apply request: {err}"),
            }
        }
    });
    (url, rx, handle)
}

fn start_apply_graphql_error_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for _ in 0..6 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            tx.send(req.clone()).unwrap();

            let mut should_stop = false;
            let (status, content_type, body) = if req.starts_with("GET /v1/compute/apps ") {
                ("HTTP/1.1 200 OK", "application/json", r#"{"apps":[]}"#)
            } else if req.starts_with("GET /v1/build-runner-backends ") {
                (
                    "HTTP/1.1 200 OK",
                    "application/json",
                    build_runner_backend_availability_body(),
                )
            } else if req.starts_with("POST /v1/compute/apps ") {
                (
                    "HTTP/1.1 200 OK",
                    "application/json",
                    r#"{"id":"app_created","name":"bakuure-api","repository_url":"https://github.com/quantum-box/erp","repository_owner":"quantum-box","repository_name":"erp","default_branch":"main","framework":"cargo_lambda","deployment_target":"lambda","root_directory":"apps/bakuure-api","docker_context":".","build_command":"cargo lambda build --package bakuure-api --bin lambda-bakuure-api --release --arm64"}"#,
                )
            } else if req.starts_with("PUT /v1/compute/apps/app_created/env ") {
                (
                    "HTTP/1.1 200 OK",
                    "application/json",
                    r#"{"env_vars":[{"id":"env_01testtesttesttesttesttest","key":"DATABASE_URL","value":"********","target":"all","is_secret":true}]}"#,
                )
            } else if req.starts_with("GET /v1/compute/apps/app_created/env ") {
                (
                    "HTTP/1.1 200 OK",
                    "application/json",
                    r#"{"env_vars":[{"id":"env_secret","key":"DATABASE_URL","value":"********","target":"all","is_secret":true}]}"#,
                )
            } else if req.starts_with("POST /v1/graphql ") {
                should_stop = true;
                (
                    "HTTP/1.1 502 Bad Gateway",
                    "text/html",
                    "<html>upstream unavailable</html>",
                )
            } else {
                (
                    "HTTP/1.1 500 Internal Server Error",
                    "application/json",
                    r#"{"error":"unexpected request"}"#,
                )
            };
            let response = format!(
                "{status}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
            if should_stop {
                break;
            }
        }
    });
    (url, rx, handle)
}

fn start_secret_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for idx in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            tx.send(req.clone()).unwrap();

            let body = if idx == 0 {
                r#"{"apps":[{"id":"app_configured","name":"configured-app"}]}"#
            } else if req.starts_with("POST /v1/apps/app_configured/secrets ") {
                r#"{"key":"RESEND_API_KEY","target":"production"}"#
            } else {
                r#"{"error":"unexpected request"}"#
            };
            let status = if body.contains("unexpected request") {
                "HTTP/1.1 500 Internal Server Error"
            } else {
                "HTTP/1.1 200 OK"
            };
            let response = format!(
                "{status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (url, rx, handle)
}

fn start_env_mutation_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for idx in 0..4 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            tx.send(req.clone()).unwrap();

            let body = if idx == 0 || idx == 2 {
                r#"{"apps":[{"id":"app_configured","name":"configured-app"}]}"#
            } else if req.starts_with("POST /v1/apps/app_configured/env ") {
                r#"{"env_vars":[{"id":"env_01testtesttesttesttesttest","key":"PLT1510_PREVIEW","value":"plain","target":"preview","branch":"feature/plt-1510","is_secret":false}]}"#
            } else if req
                .starts_with("DELETE /v1/apps/app_configured/env/PLT1510_PREVIEW?target=preview ")
            {
                ""
            } else {
                r#"{"error":"unexpected request"}"#
            };
            let status = if body.contains("unexpected request") {
                "HTTP/1.1 500 Internal Server Error"
            } else if body.is_empty() {
                "HTTP/1.1 204 No Content"
            } else {
                "HTTP/1.1 200 OK"
            };
            let response = format!(
                "{status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (url, rx, handle)
}

fn start_sync_secrets_server() -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for idx in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            tx.send(req.clone()).unwrap();

            let body = if idx == 0 && req.starts_with("GET /v1/compute/apps ") {
                r#"{"apps":[{"id":"app_fieldadmin","name":"fieldadmin"}]}"#
            } else if idx == 1
                && req.starts_with("POST /v1/compute/apps/app_fieldadmin/secrets/sync ")
            {
                r#"{"synced":3,"skipped":[]}"#
            } else {
                r#"{"error":"unexpected request"}"#
            };
            let status = if body.contains("unexpected request") {
                "HTTP/1.1 500 Internal Server Error"
            } else {
                "HTTP/1.1 200 OK"
            };
            let response = format!(
                "{status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
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

#[test]
fn compute_apps_apply_creates_from_manifest_and_preserves_secret_refs() {
    let tmp = TempDir::new().unwrap();
    let manifest = tmp.path().join("bakuure.tachyon.yml");
    fs::write(
        &manifest,
        r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: bakuure-api
spec:
  apps:
    - name: bakuure-api
      repository:
        url: https://github.com/quantum-box/erp
        owner: quantum-box
        name: erp
        defaultBranch: main
      rootDirectory: apps/bakuure-api
      dockerContext: "."
      framework: cargo_lambda
      deploymentTarget: lambda
      buildspecStrategy: repo:.codebuild/bakuure_api_lambda_buildspec.yml
      build:
        package: bakuure-api
        binary: lambda-bakuure-api
        release: true
        arch: arm64
      envVars:
        - name: ENVIRONMENT
          value: sandbox
        - name: DATABASE_URL
          type: credential
          valueFrom:
            secret: DATABASE_URL
"#,
    )
    .unwrap();
    let (api_url, rx, handle) = start_apply_server();

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_TENANT_ID", "tn_01hjryxysgey07h5jz5wagqj0m")
        .args([
            "compute",
            "apps",
            "apply",
            "-f",
            manifest.to_str().unwrap(),
            "--environment",
            "sandbox",
        ])
        .output()
        .expect("run compute apps apply");
    assert!(
        output.status.success(),
        "apply failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let first = rx.recv().unwrap();
    let second = rx.recv().unwrap();
    let third = rx.recv().unwrap();
    let fourth = rx.recv().unwrap();
    let fifth = rx.recv().unwrap();
    let sixth = rx.recv().unwrap();
    let seventh = rx.recv().unwrap();
    let eighth = rx.recv().unwrap();
    handle.join().unwrap();

    assert!(first.starts_with("GET /v1/build-runner-backends "));
    assert!(second.starts_with("GET /v1/compute/apps "));
    assert!(third.starts_with("POST /v1/compute/apps "));
    assert!(third.contains("\"root_directory\":\"apps/bakuure-api\""));
    assert!(third.contains("\"docker_context\":\".\""));
    assert!(!third.contains("mysql://"));
    assert!(fourth.starts_with("PUT /v1/compute/apps/app_created/env "));
    assert!(fourth.contains("\"key\":\"ENVIRONMENT\""));
    assert!(fifth.starts_with("GET /v1/compute/apps/app_created/env "));
    assert!(sixth.starts_with("POST /v1/graphql "));
    assert!(sixth.contains("SaveManifest"));
    assert!(seventh.starts_with("POST /v1/graphql "));
    assert!(seventh.contains("ApplyManifest"));
    assert!(eighth.starts_with("GET /v1/compute/apps/app_created/env "));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("CREATED bakuure-api (app_created)"));
    assert!(stdout.contains("Environment: sandbox"));
    assert!(stdout.contains("iac:         applied server-managed env refs"));
    assert!(!stdout.contains("missing secrets:"));
    assert!(!stdout.contains("tachyon compute env set"));
}

#[test]
fn compute_apps_apply_rejects_production_without_change_control_token_before_api_write() {
    let tmp = TempDir::new().unwrap();
    let manifest = tmp.path().join("tachyon.yml");
    fs::write(
        &manifest,
        r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: fieldadmin
spec:
  apps:
    - name: fieldadmin
      repository:
        url: https://github.com/quantum-box/tachyonfield
        owner: quantum-box
        name: tachyonfield
      framework: next_js
      deploymentTarget: cloudflare_workers
      envVars:
        - name: NEXT_PUBLIC_API_URL
          value: https://fieldadmin.txcloud.app
"#,
    )
    .unwrap();
    let (api_url, rx, handle) = start_collecting_apply_server();

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_TENANT_ID", "tn_01hjryxysgey07h5jz5wagqj0m")
        .args([
            "compute",
            "apps",
            "apply",
            "-f",
            manifest.to_str().unwrap(),
            "--environment",
            "production",
        ])
        .output()
        .expect("run compute apps apply");

    assert!(
        !output.status.success(),
        "production apply unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let requests = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    handle.join().unwrap();
    assert!(
        requests.is_empty(),
        "approval gate must reject before any API request; requests: {requests:#?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("production Cloud App apply requires change-control approval"));
    assert!(stderr.contains("TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN"));
    assert!(!stderr.contains("test-token"));
}

#[test]
fn compute_apps_apply_allows_production_with_change_control_token() {
    let tmp = TempDir::new().unwrap();
    let manifest = tmp.path().join("tachyon.yml");
    fs::write(
        &manifest,
        r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: valid-app
spec:
  apps:
    - name: valid-app
      repository:
        url: https://github.com/quantum-box/tachyonfield
        owner: quantum-box
        name: tachyonfield
      framework: next_js
      deploymentTarget: cloudflare_workers
"#,
    )
    .unwrap();
    let (api_url, rx, handle) = start_collecting_apply_server();

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_TENANT_ID", "tn_01hjryxysgey07h5jz5wagqj0m")
        .args([
            "compute",
            "apps",
            "apply",
            "-f",
            manifest.to_str().unwrap(),
            "--environment",
            "production",
            "--change-control-token",
            "dummy-approved-token",
        ])
        .output()
        .expect("run compute apps apply");

    assert!(
        output.status.success(),
        "approved production apply failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let requests = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    handle.join().unwrap();
    assert!(
        requests
            .iter()
            .any(|request| request.starts_with("GET /v1/compute/apps ")),
        "approved apply must pass the local gate and read live apps; requests: {requests:#?}"
    );
    assert!(
        requests
            .iter()
            .any(|request| request.starts_with("POST /v1/compute/apps ")),
        "approved apply must pass the local gate and create the app; requests: {requests:#?}"
    );
    let joined_requests = requests.join("\n");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!joined_requests.contains("dummy-approved-token"));
    assert!(!stdout.contains("dummy-approved-token"));
    assert!(!stderr.contains("dummy-approved-token"));
    assert!(stdout.contains("Environment: production"));
    assert!(stdout.contains("CREATED valid-app (app_created)"));
}

#[test]
fn manifest_apply_delegates_cloud_apps_manifest() {
    let tmp = TempDir::new().unwrap();
    let manifest = tmp.path().join("bakuure.tachyon.yml");
    fs::write(
        &manifest,
        r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: bakuure-api
spec:
  apps:
    - name: bakuure-api
      repository:
        url: https://github.com/quantum-box/erp
        owner: quantum-box
        name: erp
        defaultBranch: main
      rootDirectory: apps/bakuure-api
      dockerContext: "."
      framework: cargo_lambda
      deploymentTarget: lambda
      build:
        package: bakuure-api
        binary: lambda-bakuure-api
      envVars:
        - name: ENVIRONMENT
          value: sandbox
        - name: DATABASE_URL
          type: credential
          valueFrom:
            secret: DATABASE_URL
"#,
    )
    .unwrap();
    let (api_url, rx, handle) = start_apply_server();

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_TENANT_ID", "tn_01hjryxysgey07h5jz5wagqj0m")
        .args([
            "manifest",
            "apply",
            "-f",
            manifest.to_str().unwrap(),
            "--environment",
            "sandbox",
        ])
        .output()
        .expect("run manifest apply");
    assert!(
        output.status.success(),
        "manifest apply failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let first = rx.recv().unwrap();
    let second = rx.recv().unwrap();
    let third = rx.recv().unwrap();
    let fourth = rx.recv().unwrap();
    let fifth = rx.recv().unwrap();
    let sixth = rx.recv().unwrap();
    let seventh = rx.recv().unwrap();
    let eighth = rx.recv().unwrap();
    handle.join().unwrap();

    assert!(first.starts_with("GET /v1/build-runner-backends "));
    assert!(second.starts_with("GET /v1/compute/apps "));
    assert!(third.starts_with("POST /v1/compute/apps "));
    assert!(fourth.starts_with("PUT /v1/compute/apps/app_created/env "));
    assert!(fifth.starts_with("GET /v1/compute/apps/app_created/env "));
    assert!(sixth.starts_with("POST /v1/graphql "));
    assert!(sixth.contains("SaveManifest"));
    assert!(seventh.starts_with("POST /v1/graphql "));
    assert!(seventh.contains("ApplyManifest"));
    assert!(eighth.starts_with("GET /v1/compute/apps/app_created/env "));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("=== Cloud Apps Manifest Apply ==="));
    assert!(stdout.contains("CREATED bakuure-api (app_created)"));
}

#[test]
fn compute_apps_apply_preflights_all_apps_before_mutation() {
    let tmp = TempDir::new().unwrap();
    let manifest = tmp.path().join("tachyon.yml");
    fs::write(
        &manifest,
        r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: atomic-apply
spec:
  apps:
    - name: valid-app
      repository:
        url: https://github.com/quantum-box/erp
        owner: quantum-box
        name: erp
    - name: invalid-app
      envVars:
        - name: BROKEN
          value: missing-repository-should-fail-before-valid-app-is-created
"#,
    )
    .unwrap();
    let (api_url, rx, handle) = start_collecting_apply_server();

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_TENANT_ID", "tn_01hjryxysgey07h5jz5wagqj0m")
        .args([
            "compute",
            "apps",
            "apply",
            "-f",
            manifest.to_str().unwrap(),
            "--environment",
            "sandbox",
        ])
        .output()
        .expect("run compute apps apply");

    assert!(
        !output.status.success(),
        "apply unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let requests = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    handle.join().unwrap();

    assert!(
        requests.iter().all(|request| !request.starts_with("POST ")),
        "preflight failure must not create the first app; requests: {requests:#?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("app entry invalid-app is missing repository"));
}

#[test]
fn compute_apps_apply_preflights_internal_service_refs_before_mutation() {
    let tmp = TempDir::new().unwrap();
    let manifest = tmp.path().join("tachyon.yml");
    fs::write(
        &manifest,
        r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: fieldadmin
spec:
  apps:
    - name: fieldadmin
      repository:
        url: https://github.com/quantum-box/tachyonfield
        owner: quantum-box
        name: tachyonfield
      envVars:
        - name: TACHYON_FIELD_API_URL
          valueFrom:
            internalService:
              appName: tachyon-field-api
"#,
    )
    .unwrap();
    let (api_url, rx, handle) = start_internal_service_preflight_error_server();

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_TENANT_ID", "tn_01hjryxysgey07h5jz5wagqj0m")
        .args([
            "compute",
            "apps",
            "apply",
            "-f",
            manifest.to_str().unwrap(),
            "--environment",
            "production",
            "--change-control-token",
            "dummy-approved-token",
        ])
        .output()
        .expect("run compute apps apply");

    assert!(
        !output.status.success(),
        "apply unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let requests = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    handle.join().unwrap();

    let runner_preflight = requests
        .iter()
        .position(|request| request.starts_with("GET /v1/build-runner-backends "));
    let internal_service_preflight = requests
        .iter()
        .position(|request| request.starts_with("POST /v1/internal-service-refs/preflight "));
    let create_app = requests
        .iter()
        .position(|request| request.starts_with("POST /v1/compute/apps "));

    assert!(runner_preflight.is_some(), "requests: {requests:#?}");
    let internal_service_preflight =
        internal_service_preflight.expect("missing internalService preflight");
    assert!(
        create_app.is_none(),
        "preflight failure must not mutate: {requests:#?}"
    );
    assert!(requests[internal_service_preflight].contains("TACHYON_FIELD_API_URL"));
    assert!(requests[internal_service_preflight].contains("tachyon-field-api"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("POST /v1/internal-service-refs/preflight failed"));
    assert!(stderr.contains("internalService preflight failed"));
    assert!(stderr.contains("tachyon-field-api"));
    assert!(!stderr.contains("dummy-approved-token"));
}

#[test]
fn compute_apps_apply_reports_raw_graphql_http_error_body() {
    let tmp = TempDir::new().unwrap();
    let manifest = tmp.path().join("tachyon.yml");
    fs::write(
        &manifest,
        r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: bakuure-api
spec:
  apps:
    - name: bakuure-api
      repository:
        url: https://github.com/quantum-box/erp
        owner: quantum-box
        name: erp
        defaultBranch: main
      rootDirectory: apps/bakuure-api
      dockerContext: "."
      framework: cargo_lambda
      deploymentTarget: lambda
      build:
        package: bakuure-api
        binary: lambda-bakuure-api
      envVars:
        - name: DATABASE_URL
          type: credential
          valueFrom:
            databaseRef:
              name: app-db
              field: connectionString
"#,
    )
    .unwrap();
    let (api_url, rx, handle) = start_apply_graphql_error_server();

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "test-token")
        .env("TACHYON_TENANT_ID", "tn_01hjryxysgey07h5jz5wagqj0m")
        .args([
            "compute",
            "apps",
            "apply",
            "-f",
            manifest.to_str().unwrap(),
            "--environment",
            "sandbox",
        ])
        .output()
        .expect("run compute apps apply");

    assert!(
        !output.status.success(),
        "apply unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let _list = rx.recv().unwrap();
    let _availability = rx.recv().unwrap();
    let _create = rx.recv().unwrap();
    let graphql = rx.recv().unwrap();
    handle.join().unwrap();

    assert!(graphql.starts_with("POST /v1/graphql "));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("graphql request failed: status=502 Bad Gateway"));
    assert!(stderr.contains("body=<html>upstream unavailable</html>"));
    assert!(!stderr.contains("error decoding response body"));
}

#[test]
fn compute_apps_sync_secrets_posts_manifest_refs_without_secret_values() {
    let tmp = TempDir::new().unwrap();
    let manifest = tmp.path().join("tachyon.yml");
    fs::write(
        &manifest,
        r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: fieldadmin
spec:
  apps:
    - name: fieldadmin
      repository:
        url: https://github.com/quantum-box/fieldadmin
        owner: quantum-box
        name: fieldadmin
      framework: next_js
      deploymentTarget: cloudflare_pages
      envVars:
        - name: COGNITO_CLIENT_ID
          type: credential
          target: production
          valueFrom:
            oauth2ClientRef:
              name: fieldadmin-login
              field: clientId
        - name: COGNITO_CLIENT_SECRET
          type: credential
          target: production
          valueFrom:
            oauth2ClientRef:
              name: fieldadmin-login
              field: clientSecret
        - name: RESEND_API_KEY
          type: credential
          target: production
          valueFrom:
            secret: prod/resend/api-key
"#,
    )
    .unwrap();
    let (api_url, rx, handle) = start_sync_secrets_server();

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "placeholder")
        .env("TACHYON_TENANT_ID", "tn_01hjryxysgey07h5jz5wagqj0m")
        .args([
            "compute",
            "apps",
            "sync-secrets",
            "-f",
            manifest.to_str().unwrap(),
            "--environment",
            "production",
        ])
        .output()
        .expect("run compute apps sync-secrets");
    assert!(
        output.status.success(),
        "sync-secrets failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let first = rx.recv().unwrap();
    let second = rx.recv().unwrap();
    handle.join().unwrap();

    assert!(first.starts_with("GET /v1/compute/apps "));
    assert!(second.starts_with("POST /v1/compute/apps/app_fieldadmin/secrets/sync "));
    assert!(second.contains(r#""app_name":"fieldadmin""#));
    assert!(second.contains(r#""environment":"production""#));
    assert!(second.contains(r#""key":"COGNITO_CLIENT_ID""#));
    assert!(second.contains(r#""source":"oauth2ClientRef""#));
    assert!(second.contains(r#""source":"secretRef""#));
    assert!(!second.contains("secret-value"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Secret refs:"));
    assert!(stdout.contains("COGNITO_CLIENT_ID(production; oauth2ClientRef)"));
    assert!(stdout.contains("Synced 3 secret reference(s)."));
    assert!(!stdout.contains("clientSecret"));
    assert!(!stdout.contains("prod/resend/api-key"));
}

#[test]
fn env_set_secret_posts_value_and_updates_manifest_reference_only() {
    let tmp = TempDir::new().unwrap();
    write_project_config(
        tmp.path(),
        "configured-app",
        "tn_01hjryxysgey07h5jz5wagqj0m",
    );
    let (api_url, rx, handle) = start_secret_server();

    let mut cmd = isolated_command(tmp.path());
    let output = cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "x")
        .args([
            "env",
            "set",
            "--secret",
            "RESEND_API_KEY",
            "--value",
            "from-flag",
            "--target",
            "production",
        ])
        .output()
        .expect("run env set secret");
    assert!(
        output.status.success(),
        "env set secret failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let _list_req = rx.recv().unwrap();
    let post_req = rx.recv().unwrap();
    handle.join().unwrap();

    assert!(post_req.starts_with("POST /v1/apps/app_configured/secrets "));
    assert!(post_req.contains(r#""key":"RESEND_API_KEY""#));
    assert!(post_req.contains(r#""value":"from-flag""#));
    assert!(post_req.contains(r#""target":"production""#));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Set secret RESEND_API_KEY"));

    let yaml = fs::read_to_string(tmp.path().join("tachyon.yml")).unwrap();
    assert!(yaml.contains("name: RESEND_API_KEY"));
    assert!(yaml.contains("type: credential"));
    assert!(yaml.contains("target: production"));
    assert!(yaml.contains("secret: RESEND_API_KEY"));
    assert!(!yaml.contains("\n  value:"));
}

#[test]
fn env_set_secret_reads_value_from_stdin_when_piped() {
    let tmp = TempDir::new().unwrap();
    write_project_config(
        tmp.path(),
        "configured-app",
        "tn_01hjryxysgey07h5jz5wagqj0m",
    );
    let (api_url, rx, handle) = start_secret_server();

    let mut child = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "x")
        .args([
            "env",
            "set",
            "--secret",
            "RESEND_API_KEY",
            "--target",
            "production",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn env set secret");

    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(b"from-stdin\n")
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait env set secret");
    assert!(
        output.status.success(),
        "env set secret (stdin) failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let _list_req = rx.recv().unwrap();
    let post_req = rx.recv().unwrap();
    handle.join().unwrap();

    assert!(post_req.contains(r#""value":"from-stdin""#));
}

#[test]
fn env_set_secret_accepts_app_then_key_after_secret_with_value_flag() {
    let tmp = TempDir::new().unwrap();
    write_project_config(
        tmp.path(),
        "configured-app",
        "tn_01hjryxysgey07h5jz5wagqj0m",
    );
    let (api_url, rx, handle) = start_secret_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "x")
        .args([
            "env",
            "set",
            "--secret",
            "configured-app",
            "RESEND_API_KEY",
            "--value",
            "from-flag",
            "--target",
            "production",
        ])
        .output()
        .expect("run env set secret");

    assert!(
        output.status.success(),
        "env set secret failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let _list_req = rx.recv().unwrap();
    let post_req = rx.recv().unwrap();
    handle.join().unwrap();

    assert!(post_req.starts_with("POST /v1/apps/app_configured/secrets "));
    assert!(post_req.contains(r#""key":"RESEND_API_KEY""#));
    assert!(post_req.contains(r#""value":"from-flag""#));
}

#[test]
fn env_set_secret_accepts_app_then_key_after_secret_with_piped_stdin() {
    let tmp = TempDir::new().unwrap();
    write_project_config(
        tmp.path(),
        "configured-app",
        "tn_01hjryxysgey07h5jz5wagqj0m",
    );
    let (api_url, rx, handle) = start_secret_server();

    let mut child = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "x")
        .args([
            "env",
            "set",
            "--secret",
            "configured-app",
            "RESEND_API_KEY",
            "--target",
            "production",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn env set secret");

    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(b"from-stdin\n")
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait env set secret");

    assert!(
        output.status.success(),
        "env set secret failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let _list_req = rx.recv().unwrap();
    let post_req = rx.recv().unwrap();
    handle.join().unwrap();

    assert!(post_req.starts_with("POST /v1/apps/app_configured/secrets "));
    assert!(post_req.contains(r#""key":"RESEND_API_KEY""#));
    assert!(post_req.contains(r#""value":"from-stdin""#));
}

#[test]
fn env_set_secret_honors_tachyon_secret_value_env_when_no_flag() {
    let tmp = TempDir::new().unwrap();
    write_project_config(
        tmp.path(),
        "configured-app",
        "tn_01hjryxysgey07h5jz5wagqj0m",
    );
    let (api_url, rx, handle) = start_secret_server();

    let output = isolated_command(tmp.path())
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "x")
        .env("TACHYON_SECRET_VALUE", "from-env")
        .args([
            "env",
            "set",
            "--secret",
            "RESEND_API_KEY",
            "--target",
            "production",
        ])
        .output()
        .expect("run env set secret");

    assert!(output.status.success());
    let _list_req = rx.recv().unwrap();
    let post_req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(post_req.contains(r#""value":"from-env""#));
}

#[test]
fn env_set_preview_plain_and_unset_key_use_targeted_paths() {
    let tmp = TempDir::new().unwrap();
    write_project_config(
        tmp.path(),
        "configured-app",
        "tn_01hjryxysgey07h5jz5wagqj0m",
    );
    let (api_url, rx, handle) = start_env_mutation_server();

    let mut set_cmd = isolated_command(tmp.path());
    let set_output = set_cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url.clone())
        .env("TACHYON_API_KEY", "x")
        .args([
            "env",
            "set",
            "--target",
            "preview",
            "--branch",
            "feature/plt-1510",
            "PLT1510_PREVIEW=plain",
        ])
        .output()
        .expect("run env set preview");
    assert!(
        set_output.status.success(),
        "env set failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&set_output.stdout),
        String::from_utf8_lossy(&set_output.stderr)
    );

    let mut unset_cmd = isolated_command(tmp.path());
    let unset_output = unset_cmd
        .current_dir(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .env("TACHYON_API_KEY", "x")
        .args(["env", "unset", "--target", "preview", "PLT1510_PREVIEW"])
        .output()
        .expect("run env unset preview");
    assert!(
        unset_output.status.success(),
        "env unset failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&unset_output.stdout),
        String::from_utf8_lossy(&unset_output.stderr)
    );

    let list_before_set = rx.recv().unwrap();
    let put_req = rx.recv().unwrap();
    let list_before_unset = rx.recv().unwrap();
    let delete_req = rx.recv().unwrap();
    handle.join().unwrap();

    assert!(list_before_set.starts_with("GET /v1/compute/apps "));
    assert!(put_req.starts_with("POST /v1/apps/app_configured/env "));
    assert!(put_req.contains(r#""key":"PLT1510_PREVIEW""#));
    assert!(put_req.contains(r#""target":"preview""#));
    assert!(put_req.contains(r#""branch":"feature/plt-1510""#));
    assert!(list_before_unset.starts_with("GET /v1/compute/apps "));
    assert!(
        delete_req
            .starts_with("DELETE /v1/apps/app_configured/env/PLT1510_PREVIEW?target=preview "),
        "delete request was {delete_req}"
    );
}
