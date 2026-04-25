//! Reproduce a Cloud Apps (Compute) build locally in Docker.
//!
//! Phase 1 (this module): mock fixture flow. The build-config endpoint that
//! returns the buildspec + environment for a given build_id is tracked under
//! PLT-913 and is not yet available; the `--mock <path>` flag lets developers
//! point at a local YAML fixture so the rest of the pipeline can be exercised.
//!
//! Phase 2 will replace `MockSource::Path` with a real `ApiClient::get` against
//! `/v1/compute/builds/{build_id}/build-config` (or whatever PLT-913 settles on).

#![allow(dead_code)] // Phase 1 carries forward-compat fields (source, version, etc.)
                     // for the buildspec/build-config schemas; consumers land in Phase 2.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Default CodeBuild standard image. Verified 2026-04 — AL2023-based standard
/// runtime, see https://gallery.ecr.aws/codebuild/amazonlinux-x86_64-standard.
pub const DEFAULT_IMAGE: &str = "public.ecr.aws/codebuild/amazonlinux-x86_64-standard:5.0";

/// Where the source tree is mounted inside the container — matches the path
/// CodeBuild itself uses so buildspec paths resolve identically.
pub const CONTAINER_SOURCE_DIR: &str = "/codebuild/output/src/src";

// --- Build config (what tachyon-api will return in Phase 2) ---

#[derive(Debug, Clone, Deserialize)]
pub struct BuildConfig {
    pub build_id: String,
    /// Raw buildspec.yml content.
    pub buildspec: String,
    #[serde(default)]
    pub environment: BuildEnvironment,
    #[serde(default)]
    pub source: SourceInfo,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BuildEnvironment {
    /// CodeBuild image override; falls back to DEFAULT_IMAGE when None.
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
    /// Names only — secret values are never sent to the CLI.
    #[serde(default)]
    pub secret_names: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SourceInfo {
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
}

// --- Buildspec parsing ---

#[derive(Debug, Clone, Deserialize)]
pub struct BuildSpec {
    /// CodeBuild requires "0.2"; we accept any string for forward-compat.
    #[serde(default)]
    pub version: Option<serde_yaml::Value>,
    #[serde(default)]
    pub env: Option<BuildSpecEnv>,
    #[serde(default)]
    pub phases: BuildSpecPhases,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BuildSpecEnv {
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BuildSpecPhases {
    #[serde(default)]
    pub install: Option<BuildSpecPhase>,
    #[serde(default)]
    pub pre_build: Option<BuildSpecPhase>,
    #[serde(default)]
    pub build: Option<BuildSpecPhase>,
    #[serde(default)]
    pub post_build: Option<BuildSpecPhase>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BuildSpecPhase {
    #[serde(default)]
    pub commands: Vec<String>,
}

impl BuildSpec {
    pub fn parse(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("parse buildspec.yml")
    }

    /// Concatenate phase commands into a single bash script. Phases run in
    /// CodeBuild order (install → pre_build → build → post_build); a failing
    /// command in any phase aborts the build.
    pub fn to_phase_script(&self) -> String {
        let mut script = String::from("set -euxo pipefail\n");
        for (label, phase) in [
            ("install", self.phases.install.as_ref()),
            ("pre_build", self.phases.pre_build.as_ref()),
            ("build", self.phases.build.as_ref()),
            ("post_build", self.phases.post_build.as_ref()),
        ] {
            let Some(phase) = phase else { continue };
            if phase.commands.is_empty() {
                continue;
            }
            script.push_str(&format!("echo '[phase {label}]'\n"));
            for cmd in &phase.commands {
                script.push_str(cmd);
                script.push('\n');
            }
        }
        script
    }
}

// --- Docker invocation ---

#[derive(Debug, Clone)]
pub struct DockerInvocation {
    pub image: String,
    pub source_dir: PathBuf,
    /// CLI-merged env vars (BuildConfig.environment.variables ∪ buildspec.env.variables).
    pub env: BTreeMap<String, String>,
    pub script: String,
}

impl DockerInvocation {
    /// Render as the equivalent `docker run ...` command line. Used by
    /// `--dry-run` and as the basis for `Command::new("docker")`.
    pub fn to_argv(&self) -> Vec<String> {
        let mut argv = vec![
            "docker".into(),
            "run".into(),
            "--rm".into(),
            "-i".into(),
            "-v".into(),
            format!("{}:{}", self.source_dir.display(), CONTAINER_SOURCE_DIR),
            "-w".into(),
            CONTAINER_SOURCE_DIR.into(),
        ];
        // BTreeMap iteration is sorted → stable output for tests.
        for (k, v) in &self.env {
            argv.push("-e".into());
            argv.push(format!("{k}={v}"));
        }
        argv.push(self.image.clone());
        argv.push("/bin/bash".into());
        argv.push("-c".into());
        argv.push(self.script.clone());
        argv
    }

    /// Pretty-print the argv for `--dry-run`. Quotes values that contain
    /// whitespace; not a full shell-escape, just enough to be human-readable.
    pub fn to_display_string(&self) -> String {
        self.to_argv()
            .iter()
            .map(|a| {
                if a.chars().any(char::is_whitespace) {
                    format!("'{}'", a.replace('\'', "'\\''"))
                } else {
                    a.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Execute the docker invocation, streaming stdout/stderr to the parent
    /// process. Returns the exit code.
    pub fn execute(&self) -> Result<i32> {
        let argv = self.to_argv();
        let mut cmd = Command::new(&argv[0]);
        cmd.args(&argv[1..]);
        let status = cmd
            .status()
            .with_context(|| format!("spawn `{}`", argv[0]))?;
        Ok(status.code().unwrap_or(-1))
    }
}

/// Combine BuildConfig + parsed BuildSpec into a runnable docker invocation.
pub fn build_invocation(
    config: &BuildConfig,
    spec: &BuildSpec,
    source_dir: &Path,
    image_override: Option<&str>,
) -> DockerInvocation {
    let image = image_override
        .map(str::to_string)
        .or_else(|| config.environment.image.clone())
        .unwrap_or_else(|| DEFAULT_IMAGE.to_string());

    let mut env = config.environment.variables.clone();
    if let Some(spec_env) = &spec.env {
        for (k, v) in &spec_env.variables {
            env.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
    // CodeBuild itself sets $CODEBUILD_SRC_DIR inside the container; buildspecs
    // routinely reference it (e.g. `cd $CODEBUILD_SRC_DIR/subpath`). Mirror it.
    env.entry("CODEBUILD_SRC_DIR".into())
        .or_insert_with(|| CONTAINER_SOURCE_DIR.into());

    DockerInvocation {
        image,
        source_dir: source_dir.to_path_buf(),
        env,
        script: spec.to_phase_script(),
    }
}

// --- Mock fetch ---

/// Phase 1 stand-in for the PLT-913 endpoint. Reads a JSON file with the
/// `BuildConfig` shape so tests and manual smoke runs can exercise the
/// pipeline end-to-end.
pub fn load_mock_config(path: &Path) -> Result<BuildConfig> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read mock build-config: {}", path.display()))?;
    if path.extension().and_then(|e| e.to_str()) == Some("yaml")
        || path.extension().and_then(|e| e.to_str()) == Some("yml")
    {
        serde_yaml::from_str(&raw).context("parse mock build-config (yaml)")
    } else {
        serde_json::from_str(&raw).context("parse mock build-config (json)")
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_buildspec() {
        let yaml = r#"
version: 0.2
phases:
  install:
    commands:
      - echo install
  build:
    commands:
      - echo build
"#;
        let spec = BuildSpec::parse(yaml).unwrap();
        assert!(spec.phases.install.is_some());
        assert!(spec.phases.pre_build.is_none());
        assert_eq!(spec.phases.build.as_ref().unwrap().commands.len(), 1);
    }

    #[test]
    fn phase_script_orders_phases() {
        let yaml = r#"
version: 0.2
phases:
  install:
    commands: ["cmd_install"]
  pre_build:
    commands: ["cmd_prebuild"]
  build:
    commands: ["cmd_build"]
  post_build:
    commands: ["cmd_postbuild"]
"#;
        let spec = BuildSpec::parse(yaml).unwrap();
        let script = spec.to_phase_script();
        let p_install = script.find("cmd_install").unwrap();
        let p_pre = script.find("cmd_prebuild").unwrap();
        let p_build = script.find("cmd_build").unwrap();
        let p_post = script.find("cmd_postbuild").unwrap();
        assert!(p_install < p_pre && p_pre < p_build && p_build < p_post);
    }

    #[test]
    fn invocation_dry_run_is_stable() {
        let config = BuildConfig {
            build_id: "bld-test".into(),
            buildspec: "".into(),
            environment: BuildEnvironment {
                image: None,
                variables: BTreeMap::from([
                    ("FOO".into(), "bar".into()),
                    ("AAA".into(), "1".into()),
                ]),
                secret_names: vec![],
            },
            source: SourceInfo::default(),
        };
        let spec =
            BuildSpec::parse("version: 0.2\nphases:\n  build:\n    commands: [\"echo hi\"]\n")
                .unwrap();
        let inv = build_invocation(&config, &spec, Path::new("/tmp/src"), None);
        let argv = inv.to_argv();
        // Image is the default
        assert!(argv.iter().any(|a| a == DEFAULT_IMAGE));
        // Env vars sorted alphabetically (BTreeMap), AAA before FOO
        let aaa_pos = argv.iter().position(|a| a == "AAA=1").unwrap();
        let foo_pos = argv.iter().position(|a| a == "FOO=bar").unwrap();
        assert!(aaa_pos < foo_pos);
        // Mount point present
        assert!(argv
            .iter()
            .any(|a| a.contains(CONTAINER_SOURCE_DIR) && a.contains("/tmp/src")));
    }

    #[test]
    fn image_override_wins() {
        let config = BuildConfig {
            build_id: "x".into(),
            buildspec: "".into(),
            environment: BuildEnvironment {
                image: Some("from-config".into()),
                ..Default::default()
            },
            source: SourceInfo::default(),
        };
        let spec = BuildSpec::parse("version: 0.2\nphases: {}\n").unwrap();
        let inv = build_invocation(&config, &spec, Path::new("/tmp"), Some("from-cli"));
        assert_eq!(inv.image, "from-cli");
    }
}
