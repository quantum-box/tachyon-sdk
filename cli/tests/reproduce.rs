//! Integration test for `tachyon compute builds reproduce --dry-run`.
//!
//! Exercises the full CLI path against the mock fixture without invoking
//! Docker — `--dry-run` prints the would-be `docker run …` invocation, and we
//! assert key tokens appear (image, mount point, env vars, phase commands).

use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn reproduce_dry_run_emits_docker_invocation() {
    let bin = env!("CARGO_BIN_EXE_tachyon");
    let fixture = fixture_path("mock-build-config.yaml");

    let output = Command::new(bin)
        .args(["compute", "builds", "reproduce", "bld-mock-0001", "--mock"])
        .arg(&fixture)
        .args(["--source-dir", "/tmp", "--dry-run"])
        .output()
        .expect("run tachyon binary");

    assert!(
        output.status.success(),
        "non-zero exit\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");

    // Image from fixture flowed through.
    assert!(
        stdout.contains("public.ecr.aws/codebuild/amazonlinux-x86_64-standard:5.0"),
        "expected default image in dry-run output:\n{stdout}",
    );
    // Source mount.
    assert!(
        stdout.contains("/tmp:/codebuild/output/src/src"),
        "expected source mount in dry-run output:\n{stdout}",
    );
    // Env from build-config fixture.
    assert!(
        stdout.contains("TACHYON_APP_ID=app-mock-0001"),
        "expected build-config env vars:\n{stdout}",
    );
    // Phase commands embedded in script.
    assert!(
        stdout.contains("[phase install]") && stdout.contains("[phase build]"),
        "expected phase markers in script:\n{stdout}",
    );
    assert!(
        stdout.contains("building app"),
        "expected build phase commands in script:\n{stdout}",
    );
}

#[test]
fn reproduce_requires_mock_in_phase1() {
    let bin = env!("CARGO_BIN_EXE_tachyon");
    let output = Command::new(bin)
        .args(["compute", "builds", "reproduce", "bld-x"])
        .output()
        .expect("run tachyon binary");

    assert!(!output.status.success(), "should fail without --mock");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--mock") && stderr.contains("PLT-913"),
        "error should explain Phase 1 mock requirement:\n{stderr}",
    );
}
