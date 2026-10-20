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
        // Serve until the credentials POST: the CLI first resolves the
        // app name via GET /v1/compute/apps, then issues credentials.
        loop {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let is_issue = req.starts_with("POST ");
            tx.send(req.clone()).unwrap();

            let body = if req.starts_with("GET /v1/compute/apps") {
                r#"{"apps":[{"id":"app_01km2dr0f6hvgj0qvcteyydfbe","name":"test-app"}]}"#
            } else {
                r#"{"client_id":"dummy-client","client_secret":"dummy-secret","user_pool_id":"ap-northeast-1_8Ga4bK5M4","secret_arn":"arn:aws:secretsmanager:ap-northeast-1:123:secret:test","expires_at":"2026-06-01T00:00:00Z"}"#
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
            if is_issue {
                break;
            }
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
            "--user-pool",
            "shared",
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
    assert!(yaml.contains("user_pool: shared"));
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
        .args(["auth", "issue", "cognito-default", "--shared-pool"])
        .output()
        .expect("run tachyon auth issue");

    assert!(
        output.status.success(),
        "auth issue failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let resolve_req = rx.recv().unwrap();
    assert!(resolve_req.starts_with("GET /v1/compute/apps "));
    let req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(req.starts_with("POST /v1/cloud-apps/app_01km2dr0f6hvgj0qvcteyydfbe/auth/credentials "));
    assert!(req.contains("authorization: Bearer test-token"));
    assert!(req.contains(r#""user_pool":"shared""#));
    assert!(tmp.path().join(".tachyon/credentials.json").is_file());
    assert!(fs::read_to_string(tmp.path().join(".gitignore"))
        .unwrap()
        .contains(".tachyon/credentials.json"));
    let yaml = fs::read_to_string(tmp.path().join("tachyon.yml")).unwrap();
    assert!(yaml.contains("secret_ref: .tachyon/credentials.json#cognito-default"));
    assert!(
        fs::read_to_string(tmp.path().join(".tachyon/credentials.json"))
            .unwrap()
            .contains("ap-northeast-1_8Ga4bK5M4")
    );
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
    let _resolve_req = rx.recv().unwrap();
    let _req = rx.recv().unwrap();
    handle.join().unwrap();
    assert!(!tmp.path().join(".tachyon/credentials.json").exists());
    let yaml = fs::read_to_string(tmp.path().join("tachyon.yml")).unwrap();
    assert!(yaml.contains("secret_ref: arn:aws:secretsmanager"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("dummy-secret"));
}

const ACCESS: &str = "stored-access-token-value";
const REFRESH: &str = "stored-refresh-token-value";

/// Write a profile file where `dirs::config_dir()` looks on this OS when HOME
/// and XDG_CONFIG_HOME point into `home`.
fn write_profile(home: &Path, name: &str, expires_at: i64) {
    let creds = format!(
        r#"{{"access_token":"{ACCESS}","refresh_token":"{REFRESH}","id_token":null,"expires_at":{expires_at},"token_type":"Bearer"}}"#
    );
    for base in [
        home.join("Library/Application Support"),
        home.join(".config"),
    ] {
        let dir = base.join("tachyon/profiles");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{name}.json")), &creds).unwrap();
    }
}

fn read_profile(home: &Path, name: &str) -> String {
    let base = if cfg!(target_os = "macos") {
        home.join("Library/Application Support")
    } else {
        home.join(".config")
    };
    fs::read_to_string(base.join(format!("tachyon/profiles/{name}.json"))).unwrap()
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// One-shot OAuth token endpoint that answers with `status` and `body`.
fn start_token_server(
    status: &'static str,
    body: &'static str,
) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 8192];
        let n = stream.read(&mut buf).unwrap();
        let req = String::from_utf8_lossy(&buf[..n]).to_string();
        let response = format!(
            "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
        req
    });
    (url, handle)
}

fn assert_no_secret(text: &str) {
    for secret in [ACCESS, REFRESH, "refreshed-access-token-value"] {
        assert!(!text.contains(secret), "secret leaked: {text}");
    }
}

#[test]
fn auth_token_prints_only_the_token_to_stdout() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path(), "work", now() + 3_600);

    let output = isolated_command(tmp.path())
        .env_remove("TACHYON_API_KEY")
        .args(["auth", "token", "--profile", "work"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("{ACCESS}\n")
    );
    assert_no_secret(&String::from_utf8_lossy(&output.stderr));
}

#[test]
fn auth_token_refreshes_a_token_that_is_about_to_expire() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path(), "work", now() + 60);
    let (url, server) = start_token_server(
        "200 OK",
        r#"{"access_token":"refreshed-access-token-value","expires_in":3600,"token_type":"Bearer"}"#,
    );

    let output = isolated_command(tmp.path())
        .env_remove("TACHYON_API_KEY")
        .args([
            "--cognito-domain",
            &url,
            "auth",
            "token",
            "--profile",
            "work",
        ])
        .output()
        .unwrap();

    let request = server.join().unwrap();
    assert!(request.starts_with("POST /oauth2/token"), "{request}");
    assert!(request.contains("grant_type=refresh_token"), "{request}");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "refreshed-access-token-value\n"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Token refreshed (profile: work)"),
        "{stderr}"
    );
    assert_no_secret(&stderr);
    // The refreshed token is saved, and the refresh token is kept.
    let saved = read_profile(tmp.path(), "work");
    assert!(saved.contains("refreshed-access-token-value"), "{saved}");
    assert!(saved.contains(REFRESH), "{saved}");
}

#[test]
fn auth_token_failed_refresh_prints_nothing_secret() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path(), "work", now() - 10);
    let (url, server) = start_token_server("400 Bad Request", r#"{"error":"invalid_grant"}"#);

    let output = isolated_command(tmp.path())
        .env_remove("TACHYON_API_KEY")
        .args([
            "--cognito-domain",
            &url,
            "auth",
            "token",
            "--profile",
            "work",
        ])
        .output()
        .unwrap();

    server.join().unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tachyon auth login --profile work"),
        "{stderr}"
    );
    assert_no_secret(&stderr);
}

#[test]
fn auth_token_refuses_when_an_api_key_is_set() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path(), "work", now() + 3_600);

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_KEY", "api-key-secret-value")
        .args(["auth", "token", "--profile", "work"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("TACHYON_API_KEY"), "{stderr}");
    assert!(!stderr.contains("api-key-secret-value"), "{stderr}");
    assert_no_secret(&stderr);
}
