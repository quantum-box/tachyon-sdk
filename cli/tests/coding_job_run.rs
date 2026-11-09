use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_tachyon")
}

fn config_roots(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".config").join("tachyon"),
        home.join("Library")
            .join("Application Support")
            .join("tachyon"),
    ]
}

fn write_profile(home: &Path, name: &str, token: &str, operator_id: &str) {
    for root in config_roots(home) {
        let dir = root.join("profiles");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{name}.json")),
            format!(
                r#"{{
  "access_token": "{token}",
  "refresh_token": null,
  "id_token": null,
  "expires_at": null,
  "token_type": "Bearer",
  "operator_id": "{operator_id}"
}}"#
            ),
        )
        .unwrap();
    }
}

fn write_fake_docker(bin_dir: &Path, capture_path: &Path) {
    fs::create_dir_all(bin_dir).unwrap();
    let docker = bin_dir.join("docker");
    fs::write(
        &docker,
        format!(
            r#"#!/bin/sh
set -eu
echo "fake docker stdout"
echo "fake docker stderr" >&2
if [ "${{1:-}}" = "ps" ]; then
  exit 0
fi
if [ "${{1:-}}" = "stop" ]; then
  exit 0
fi
{{
  printf 'args='
  printf '%s|' "$@"
  printf '\n'
  printf 'TACHYON_API_URL=%s\n' "${{TACHYON_API_URL:-}}"
  printf 'TACHYON_API_KEY=%s\n' "${{TACHYON_API_KEY:-}}"
  printf 'TACHYON_OPERATOR_ID=%s\n' "${{TACHYON_OPERATOR_ID:-}}"
  printf 'TACHYON_QUIC_GATEWAY_URL=%s\n' "${{TACHYON_QUIC_GATEWAY_URL:-}}"
  printf 'TACHYON_AUTH_TOKEN=%s\n' "${{TACHYON_AUTH_TOKEN:-}}"
  printf 'CODING_JOB_OPERATOR_ID=%s\n' "${{CODING_JOB_OPERATOR_ID:-}}"
  printf 'QUIC_GATEWAY_ADDR=%s\n' "${{QUIC_GATEWAY_ADDR:-}}"
  printf 'RUST_LOG=%s\n' "${{RUST_LOG:-}}"
  printf 'MANAGE_OPENCODE_SERVER=%s\n' "${{MANAGE_OPENCODE_SERVER:-}}"
  printf 'TACHYOND_ENABLE_OPENCODE=%s\n' "${{TACHYOND_ENABLE_OPENCODE:-}}"
  printf 'OPENCODE_SERVER_PORT=%s\n' "${{OPENCODE_SERVER_PORT:-}}"
  printf 'OPENCODE_RESTART_LIMIT=%s\n' "${{OPENCODE_RESTART_LIMIT:-}}"
  printf 'OPENCODE_HEALTH_INTERVAL_SECS=%s\n' "${{OPENCODE_HEALTH_INTERVAL_SECS:-}}"
}} > '{}'
"#,
            capture_path.display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&docker, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

#[test]
fn coding_job_run_uses_active_profile_config_and_streams_docker_output() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path(), "work", "profile-token", "op_profile");
    for root in config_roots(tmp.path()) {
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("active_profile"), "work\n").unwrap();
    }

    let bin_dir = tmp.path().join("bin");
    let capture_path = tmp.path().join("docker-capture.txt");
    write_fake_docker(&bin_dir, &capture_path);

    let output = Command::new(bin())
        .env("HOME", tmp.path())
        .env("XDG_CONFIG_HOME", tmp.path().join(".config"))
        .env("PATH", bin_dir)
        .env_remove("TACHYON_API_KEY")
        .env_remove("TACHYON_PROFILE")
        .env_remove("TACHYON_TENANT_ID")
        .env_remove("RUST_LOG")
        .env_remove("TACHYOND_RUST_LOG")
        .args([
            "--api-url",
            "https://api.test.example",
            "agent",
            "coding-job",
            "run",
            "--quic-gateway-url",
            "quic.test.example:4433",
            "--image",
            "tachyond:test",
        ])
        .output()
        .expect("run tachyon agent coding-job run");

    assert!(
        output.status.success(),
        "coding-job run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Starting tachyond coding job worker..."));
    assert!(stdout.contains("Stop with: tachyon agent coding-job stop"));
    assert!(stdout.contains("Logs with: docker logs -f tachyond-coding-job-worker"));
    assert!(stdout.contains("fake docker stdout"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("fake docker stderr"));

    let capture = fs::read_to_string(capture_path).unwrap();
    assert!(capture.contains("args=run|--pull|always|--rm|"));
    assert!(capture.contains("--label|com.tachyon.role=coding-job-worker|"));
    assert!(capture.contains("-e|TACHYON_API_URL=https://api.test.example|"));
    assert!(capture.contains("-e|TACHYON_API_KEY=profile-token|"));
    assert!(capture.contains("-e|TACHYON_OPERATOR_ID=op_profile|"));
    assert!(capture.contains("-e|TACHYON_QUIC_GATEWAY_URL=quic.test.example:4433|"));
    assert!(capture.contains("-e|TACHYON_AUTH_TOKEN=profile-token|"));
    assert!(capture.contains("-e|CODING_JOB_OPERATOR_ID=op_profile|"));
    assert!(capture.contains("-e|QUIC_GATEWAY_ADDR=quic.test.example:4433|"));
    assert!(capture.contains("-e|RUST_LOG=warn,streaming=info,llms=info,tachyon_code=info|"));
    assert!(capture.contains("-e|TACHYOND_ENABLE_OPENCODE=false|"));
    assert!(capture.contains("-e|MANAGE_OPENCODE_SERVER=false|"));
    assert!(capture.contains("-e|OPENCODE_SERVER_PORT=0|"));
    assert!(capture.contains("-e|OPENCODE_RESTART_LIMIT=5|"));
    assert!(capture.contains("-e|OPENCODE_HEALTH_INTERVAL_SECS=30|"));
    assert!(capture.contains("tachyond:test|"));
}
