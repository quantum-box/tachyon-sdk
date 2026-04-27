//! Integration tests for PLT-724 multi-profile auth.
//!
//! Each test redirects `dirs::config_dir()` at a fresh tempdir by overriding
//! both `HOME` and `XDG_CONFIG_HOME` (covers Linux + macOS lookup paths).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_tachyon")
}

/// Run `tachyon <args...>` against an isolated config home rooted at `home`.
fn run(home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        // Strip any inherited TACHYON_PROFILE so tests stay deterministic.
        .env_remove("TACHYON_PROFILE")
        .args(args)
        .output()
        .expect("run tachyon binary")
}

fn config_root(home: &Path) -> PathBuf {
    home.join(".config").join("tachyon")
}

fn write_profile(home: &Path, name: &str, body: &str) {
    let dir = config_root(home).join("profiles");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{name}.json")), body).unwrap();
}

fn fixture_creds(operator: &str) -> String {
    format!(
        r#"{{
  "access_token": "fake-token-{operator}",
  "refresh_token": null,
  "id_token": null,
  "expires_at": null,
  "token_type": "Bearer",
  "operator_id": "{operator}"
}}"#
    )
}

fn assert_ok(output: &std::process::Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn auth_list_empty_reports_no_profiles() {
    let tmp = TempDir::new().unwrap();
    let out = run(tmp.path(), &["auth", "list"]);
    assert_ok(&out, "auth list (empty)");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No profiles registered"),
        "expected empty-state message, got:\n{stdout}"
    );
}

#[test]
fn auth_list_marks_active_profile() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path(), "work", &fixture_creds("op_work"));
    write_profile(tmp.path(), "personal", &fixture_creds("op_personal"));
    fs::write(config_root(tmp.path()).join("active_profile"), "personal\n").unwrap();

    let out = run(tmp.path(), &["auth", "list"]);
    assert_ok(&out, "auth list (two profiles)");
    let stdout = String::from_utf8_lossy(&out.stdout);

    // Active marker on personal, blank on work. We check both lines exist
    // and that the marker only appears on the personal row.
    let personal_line = stdout
        .lines()
        .find(|l| l.contains("personal"))
        .expect("personal row missing");
    let work_line = stdout
        .lines()
        .find(|l| l.contains("work"))
        .expect("work row missing");
    assert!(
        personal_line.trim_start().starts_with('*'),
        "personal should be marked active: {personal_line:?}"
    );
    assert!(
        !work_line.trim_start().starts_with('*'),
        "work should not be marked active: {work_line:?}"
    );
    // Tenant column shows operator_id.
    assert!(personal_line.contains("op_personal"));
    assert!(work_line.contains("op_work"));
}

#[test]
fn auth_use_switches_active_profile() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path(), "work", &fixture_creds("op_work"));
    write_profile(tmp.path(), "personal", &fixture_creds("op_personal"));
    fs::write(config_root(tmp.path()).join("active_profile"), "work\n").unwrap();

    let out = run(tmp.path(), &["auth", "use", "personal"]);
    assert_ok(&out, "auth use personal");

    let active = fs::read_to_string(config_root(tmp.path()).join("active_profile")).unwrap();
    assert_eq!(active.trim(), "personal");
}

#[test]
fn auth_use_rejects_unknown_profile() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path(), "work", &fixture_creds("op_work"));

    let out = run(tmp.path(), &["auth", "use", "ghost"]);
    assert!(
        !out.status.success(),
        "auth use should fail for missing profile"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("does not exist"),
        "expected explanatory error, got:\n{stderr}"
    );

    // Active pointer should not have been overwritten.
    let active_path = config_root(tmp.path()).join("active_profile");
    if active_path.exists() {
        let active = fs::read_to_string(&active_path).unwrap();
        assert_ne!(active.trim(), "ghost");
    }
}

#[test]
fn auth_logout_removes_profile_and_repoints_active() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path(), "work", &fixture_creds("op_work"));
    write_profile(tmp.path(), "personal", &fixture_creds("op_personal"));
    fs::write(config_root(tmp.path()).join("active_profile"), "personal\n").unwrap();

    let out = run(tmp.path(), &["auth", "logout", "--profile", "personal"]);
    assert_ok(&out, "auth logout --profile personal");

    let profiles_dir = config_root(tmp.path()).join("profiles");
    assert!(!profiles_dir.join("personal.json").exists());
    assert!(profiles_dir.join("work.json").exists());

    // Active pointer now points at the surviving profile.
    let active = fs::read_to_string(config_root(tmp.path()).join("active_profile")).unwrap();
    assert_eq!(active.trim(), "work");
}

#[test]
fn auth_logout_last_profile_clears_pointer() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path(), "default", &fixture_creds("op_default"));
    fs::write(config_root(tmp.path()).join("active_profile"), "default\n").unwrap();

    let out = run(tmp.path(), &["auth", "logout", "--profile", "default"]);
    assert_ok(&out, "auth logout --profile default");

    let active_path = config_root(tmp.path()).join("active_profile");
    assert!(
        !active_path.exists(),
        "active_profile should be cleared when no profiles remain"
    );
}

#[test]
fn legacy_credentials_auto_migrate_to_default_profile() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(config_root(tmp.path())).unwrap();
    fs::write(
        config_root(tmp.path()).join("credentials.json"),
        fixture_creds("op_legacy"),
    )
    .unwrap();

    // First profile-aware command triggers migration.
    let out = run(tmp.path(), &["auth", "list"]);
    assert_ok(&out, "auth list (legacy migration)");

    let migrated = config_root(tmp.path())
        .join("profiles")
        .join("default.json");
    assert!(
        migrated.exists(),
        "default.json should exist after migration"
    );
    let content = fs::read_to_string(&migrated).unwrap();
    assert!(content.contains("op_legacy"));

    // Legacy file is preserved on disk for downgrade safety.
    assert!(config_root(tmp.path()).join("credentials.json").exists());

    // The list command reports the migrated profile as active by default
    // (no active_profile pointer = "default").
    let stdout = String::from_utf8_lossy(&out.stdout);
    let default_line = stdout
        .lines()
        .find(|l| l.contains("default"))
        .expect("default row missing");
    assert!(
        default_line.trim_start().starts_with('*'),
        "default should be active: {default_line:?}"
    );
}

#[test]
fn global_profile_flag_does_not_mutate_active_pointer() {
    let tmp = TempDir::new().unwrap();
    write_profile(tmp.path(), "work", &fixture_creds("op_work"));
    write_profile(tmp.path(), "personal", &fixture_creds("op_personal"));
    fs::write(config_root(tmp.path()).join("active_profile"), "work\n").unwrap();

    // `auth list` doesn't read the active flag for selection logic, but
    // `--profile` should leave the pointer untouched regardless.
    let out = run(tmp.path(), &["--profile", "personal", "auth", "list"]);
    assert_ok(&out, "tachyon --profile personal auth list");

    let active = fs::read_to_string(config_root(tmp.path()).join("active_profile")).unwrap();
    assert_eq!(active.trim(), "work");
}

#[test]
fn invalid_profile_name_is_rejected() {
    let tmp = TempDir::new().unwrap();
    let out = run(tmp.path(), &["auth", "use", ".."]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("reserved") || stderr.contains("invalid"),
        "expected validation error, got:\n{stderr}"
    );
}
