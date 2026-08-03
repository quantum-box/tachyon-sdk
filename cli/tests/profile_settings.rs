use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;

use serde_json::{json, Value};
use tempfile::TempDir;

const TENANT_ID: &str = "tn_01fakeprofilesettings";

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_tachyon")
}

fn isolated_command(home: &Path) -> Command {
    let mut command = Command::new(bin());
    command
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_TENANT_ID", TENANT_ID)
        .env("TACHYON_API_KEY", "fake-api-key")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE")
        .env_remove("TACHYON_PM_NO_DELEGATE")
        .env_remove("TACHYON_PM_DEFAULT_TEAM");
    command
}

fn run(home: &Path, args: &[&str]) -> Output {
    isolated_command(home)
        .args(args)
        .output()
        .expect("run tachyon binary")
}

fn assert_ok(output: &Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn config_root(home: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        return home
            .join("Library")
            .join("Application Support")
            .join("tachyon");
    }
    #[cfg(not(target_os = "macos"))]
    {
        home.join(".config").join("tachyon")
    }
}

fn set(home: &Path, profile: &str, key: &str, value: &str) {
    let output = run(home, &["config", "set", key, value, "--profile", profile]);
    assert_ok(&output, "config set");
}

fn read_request(stream: &mut std::net::TcpStream) -> String {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 4_096];
    loop {
        let size = stream.read(&mut buffer).unwrap();
        if size == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..size]);

        let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                line.to_ascii_lowercase()
                    .strip_prefix("content-length: ")
                    .and_then(|value| value.parse::<usize>().ok())
            })
            .unwrap_or(0);
        if request.len() >= header_end + 4 + content_length {
            break;
        }
    }
    String::from_utf8_lossy(&request).to_string()
}

fn start_server(
    request_count: usize,
) -> (String, mpsc::Receiver<Vec<String>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        for _ in 0..request_count {
            let (mut stream, _) = listener.accept().unwrap();
            requests.push(read_request(&mut stream));
            let body = json!({
                "issue": {
                    "provider": "linear",
                    "id": "issue_profile_settings",
                    "key": "PLT-2985",
                    "title": "Profile settings test",
                    "url": "https://linear.example.test/PLT-2985",
                    "status": "Todo",
                    "priority": "medium"
                },
                "created": true
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 201 Created\r\ncontent-type: application/json\r\n\
                 content-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
        tx.send(requests).unwrap();
    });
    (url, rx, handle)
}

fn request_body(request: &str) -> Value {
    let (_, body) = request.split_once("\r\n\r\n").unwrap();
    serde_json::from_str(body).unwrap()
}

#[test]
fn config_commands_are_profile_scoped_and_separate_from_credentials() {
    let temporary_home = TempDir::new().unwrap();
    set(temporary_home.path(), "admin", "pm.no_delegate", "true");
    set(
        temporary_home.path(),
        "admin",
        "pm.default_team",
        "Platform Team",
    );
    set(
        temporary_home.path(),
        "agent_app",
        "pm.no_delegate",
        "false",
    );

    let root = config_root(temporary_home.path());
    assert!(root.join("settings.json").exists());
    assert!(!root.join("profiles/admin.json").exists());
    let stored: Value =
        serde_json::from_str(&fs::read_to_string(root.join("settings.json")).unwrap()).unwrap();
    assert_eq!(stored["profiles"]["admin"]["pm"]["no_delegate"], true);
    assert_eq!(
        stored["profiles"]["admin"]["pm"]["default_team"],
        "Platform Team"
    );
    assert_eq!(stored["profiles"]["agent_app"]["pm"]["no_delegate"], false);

    let get = run(
        temporary_home.path(),
        &["config", "get", "--profile", "admin"],
    );
    assert_ok(&get, "config get");
    let profile: Value = serde_json::from_slice(&get.stdout).unwrap();
    assert_eq!(profile["pm"]["no_delegate"], true);
    assert_eq!(profile["pm"]["default_team"], "Platform Team");

    let unset = run(
        temporary_home.path(),
        &["config", "unset", "pm.no_delegate", "--profile", "admin"],
    );
    assert_ok(&unset, "config unset");
    let stored: Value =
        serde_json::from_str(&fs::read_to_string(root.join("settings.json")).unwrap()).unwrap();
    assert!(stored["profiles"]["admin"]["pm"]
        .get("no_delegate")
        .is_none());
    assert_eq!(
        stored["profiles"]["admin"]["pm"]["default_team"],
        "Platform Team"
    );
    assert_eq!(stored["profiles"]["agent_app"]["pm"]["no_delegate"], false);
}

#[test]
fn profile_defaults_apply_to_all_issue_command_aliases() {
    let temporary_home = TempDir::new().unwrap();
    set(temporary_home.path(), "admin", "pm.no_delegate", "true");
    set(
        temporary_home.path(),
        "admin",
        "pm.default_team",
        "Profile Team",
    );
    let (api_url, rx, handle) = start_server(3);

    let commands: &[&[&str]] = &[
        &[
            "linear",
            "issue",
            "create",
            "--title",
            "Profile settings test",
            "--json",
        ],
        &[
            "issue",
            "create",
            "--provider",
            "linear",
            "--title",
            "Profile settings test",
            "--json",
        ],
        &[
            "pm",
            "issue",
            "create",
            "--provider",
            "linear",
            "--title",
            "Profile settings test",
            "--json",
        ],
    ];
    for args in commands {
        let output = isolated_command(temporary_home.path())
            .env("TACHYON_API_URL", &api_url)
            .arg("--profile")
            .arg("admin")
            .args(*args)
            .output()
            .unwrap();
        assert_ok(&output, &args.join(" "));
    }

    handle.join().unwrap();
    let requests = rx.recv().unwrap();
    assert_eq!(requests.len(), 3);
    for request in requests {
        let body = request_body(&request);
        assert_eq!(body["team"], "Profile Team");
        assert!(body.get("auto_delegate_to_linear_agent").is_none());
    }
}

#[test]
fn environment_and_explicit_flags_override_profile_defaults() {
    let temporary_home = TempDir::new().unwrap();
    set(temporary_home.path(), "admin", "pm.no_delegate", "true");
    set(
        temporary_home.path(),
        "admin",
        "pm.default_team",
        "Profile Team",
    );
    let (api_url, rx, handle) = start_server(3);

    let env_override = isolated_command(temporary_home.path())
        .env("TACHYON_API_URL", &api_url)
        .env("TACHYON_PM_NO_DELEGATE", "false")
        .env("TACHYON_PM_DEFAULT_TEAM", "Environment Team")
        .args([
            "--profile",
            "admin",
            "linear",
            "issue",
            "create",
            "--title",
            "Profile settings test",
            "--json",
        ])
        .output()
        .unwrap();
    assert_ok(&env_override, "environment override");

    let no_delegate_flag = isolated_command(temporary_home.path())
        .env("TACHYON_API_URL", &api_url)
        .env("TACHYON_PM_NO_DELEGATE", "false")
        .env("TACHYON_PM_DEFAULT_TEAM", "Environment Team")
        .args([
            "--profile",
            "admin",
            "linear",
            "issue",
            "create",
            "--team",
            "CLI Team",
            "--title",
            "Profile settings test",
            "--no-delegate",
            "--json",
        ])
        .output()
        .unwrap();
    assert_ok(&no_delegate_flag, "CLI no-delegate override");

    let explicit_delegate = isolated_command(temporary_home.path())
        .env("TACHYON_API_URL", &api_url)
        .env("TACHYON_PM_NO_DELEGATE", "true")
        .env("TACHYON_PM_DEFAULT_TEAM", "Environment Team")
        .args([
            "--profile",
            "admin",
            "linear",
            "issue",
            "create",
            "--team-id",
            "team_explicit",
            "--title",
            "Profile settings test",
            "--delegate-id",
            "agent_explicit",
            "--json",
        ])
        .output()
        .unwrap();
    assert_ok(&explicit_delegate, "explicit delegate override");

    handle.join().unwrap();
    let requests = rx.recv().unwrap();
    let env_body = request_body(&requests[0]);
    assert_eq!(env_body["team"], "Environment Team");
    assert_eq!(env_body["auto_delegate_to_linear_agent"], true);

    let cli_body = request_body(&requests[1]);
    assert_eq!(cli_body["team"], "CLI Team");
    assert!(cli_body.get("auto_delegate_to_linear_agent").is_none());

    let delegate_body = request_body(&requests[2]);
    assert!(delegate_body.get("team").is_none());
    assert_eq!(delegate_body["team_id"], "team_explicit");
    assert_eq!(delegate_body["delegate_id"], "agent_explicit");
    assert!(delegate_body.get("auto_delegate_to_linear_agent").is_none());
}

#[test]
fn malformed_settings_json_is_an_error() {
    let temporary_home = TempDir::new().unwrap();
    let root = config_root(temporary_home.path());
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("settings.json"), "{not-json").unwrap();

    let get = run(temporary_home.path(), &["config", "get"]);
    assert!(!get.status.success());
    let stderr = String::from_utf8_lossy(&get.stderr);
    assert!(stderr.contains("failed to parse"), "stderr:\n{stderr}");

    let issue = isolated_command(temporary_home.path())
        .env("TACHYON_API_URL", "http://127.0.0.1:1")
        .args([
            "linear",
            "issue",
            "create",
            "--title",
            "Profile settings test",
        ])
        .output()
        .unwrap();
    assert!(!issue.status.success());
    let stderr = String::from_utf8_lossy(&issue.stderr);
    assert!(stderr.contains("failed to parse"), "stderr:\n{stderr}");
}
