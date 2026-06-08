//! Execute a Cloud App build workload inside a runner container.
//!
//! This is the non-interactive entrypoint used by Tachyon-controlled build
//! runners. It intentionally talks back through the existing Cloud Build
//! completion webhook so the control plane can reuse the same build state
//! transitions as CodeBuild.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Stdio,
    time::Instant,
};
use tokio::process::Command;

const OUTPUT_CAPTURE_LIMIT: usize = 128 * 1024;
const DOCKER_CONFIG_ENV: &str = "TACHYON_DOCKER_CONFIG_JSON";
const KANIKO_EXECUTOR: &str = "/kaniko/executor";

#[derive(Debug, Deserialize)]
pub struct BuildWorkloadSpec {
    project_id: String,
    source: BuildWorkloadSource,
    workspace: BuildWorkloadWorkspace,
    commands: BuildWorkloadCommands,
    #[serde(default)]
    env: Vec<BuildWorkloadEnvVar>,
    artifact: BuildWorkloadArtifact,
    callback: Option<BuildWorkloadCallback>,
    deployment_target: Option<String>,
    framework: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BuildWorkloadSource {
    repository_url: String,
    branch: String,
    commit_sha: String,
}

#[derive(Debug, Deserialize)]
struct BuildWorkloadWorkspace {
    root_directory: Option<String>,
    docker_context: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BuildWorkloadCommands {
    install: Option<String>,
    build: Option<String>,
    node_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BuildWorkloadEnvVar {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct BuildWorkloadArtifact {
    image_name: String,
    image_tag: String,
    output_directory: Option<String>,
    container_registry: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BuildWorkloadCallback {
    url: String,
    build_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct CloudBuildWebhookPayload {
    build_id: String,
    status: String,
    image_uri: Option<String>,
    artifact_path: Option<String>,
    duration_secs: Option<i32>,
    codebuild_instance_type: Option<String>,
    error_message: Option<String>,
    iac_plan_output: Option<String>,
    iac_success: Option<bool>,
    d1_migrations: Option<Vec<serde_json::Value>>,
    cache_hit: Option<bool>,
    cache_key: Option<String>,
    runner_stdout: Option<String>,
    runner_stderr: Option<String>,
    runner_failure_reason: Option<String>,
}

#[derive(Debug)]
struct CommandResult {
    stdout: String,
    stderr: String,
}

pub async fn run_from_env(spec_env: &str) -> Result<()> {
    let started = Instant::now();
    let workload = parse_workload_from_env(spec_env)?;
    tracing::info!(
        project_id = %workload.project_id,
        branch = %workload.source.branch,
        commit_sha = %workload.source.commit_sha,
        deployment_target = ?workload.deployment_target,
        framework = ?workload.framework,
        node_version = ?workload.commands.node_version,
        docker_context = ?workload.workspace.docker_context,
        image_name = %workload.artifact.image_name,
        image_tag = %workload.artifact.image_tag,
        container_registry = ?workload.artifact.container_registry,
        "starting Tachyon Cloud App build job"
    );

    let result = run_build(&workload).await;
    let duration_secs = i32::try_from(started.elapsed().as_secs()).unwrap_or(i32::MAX);

    match &result {
        Ok(output) => {
            send_callback(&workload, "SUCCESS", duration_secs, Some(output), None).await?;
            tracing::info!("Tachyon Cloud App build job succeeded");
            Ok(())
        }
        Err(error) => {
            send_callback(
                &workload,
                "FAILURE",
                duration_secs,
                None,
                Some(error.to_string()),
            )
            .await?;
            Err(anyhow!("{error}"))
        }
    }
}

fn parse_workload_from_env(spec_env: &str) -> Result<BuildWorkloadSpec> {
    let json = std::env::var(spec_env).with_context(|| format!("{spec_env} is required"))?;
    serde_json::from_str(&json).context("failed to parse build workload spec")
}

async fn run_build(workload: &BuildWorkloadSpec) -> Result<CommandResult> {
    let checkout_dir = PathBuf::from("/workspace/source");
    if checkout_dir.exists() {
        tokio::fs::remove_dir_all(&checkout_dir)
            .await
            .context("failed to clean checkout directory")?;
    }
    tokio::fs::create_dir_all(&checkout_dir)
        .await
        .context("failed to create checkout directory")?;

    let env = workload_env(workload);
    clone_repository(workload, &checkout_dir, &env).await?;

    let app_dir = workspace_dir(workload, &checkout_dir);
    if !app_dir.exists() {
        return Err(anyhow!(
            "workspace directory does not exist: {}",
            app_dir.display()
        ));
    }

    let mut combined = CommandResult {
        stdout: String::new(),
        stderr: String::new(),
    };

    if let Some(command) = install_command(workload, &app_dir) {
        append_result(
            &mut combined,
            run_shell("install", &command, &app_dir, &env).await?,
        );
    }

    let build_command = build_command(workload, &app_dir)
        .ok_or_else(|| anyhow!("build command is not configured"))?;
    append_result(
        &mut combined,
        run_shell("build", &build_command, &app_dir, &env).await?,
    );

    if is_cloud_run(workload) {
        prepare_docker_config(&env).await?;
        append_result(
            &mut combined,
            run_kaniko(workload, &checkout_dir, &app_dir, &env).await?,
        );
    }

    Ok(combined)
}

async fn clone_repository(
    workload: &BuildWorkloadSpec,
    checkout_dir: &Path,
    env: &BTreeMap<String, String>,
) -> Result<()> {
    if workload.source.repository_url.trim().is_empty() {
        return Err(anyhow!("repository_url is required"));
    }

    let mut clone = Command::new("git");
    clone.arg("clone").arg("--depth").arg("1");
    if !workload.source.branch.trim().is_empty() {
        clone.arg("--branch").arg(&workload.source.branch);
    }
    clone
        .arg(&workload.source.repository_url)
        .arg(checkout_dir)
        .current_dir("/workspace")
        .envs(env);
    run_command("git clone", clone).await?;

    let commit = workload.source.commit_sha.trim();
    if !commit.is_empty() && commit != "HEAD" {
        let mut fetch = Command::new("git");
        fetch
            .arg("fetch")
            .arg("--depth")
            .arg("1")
            .arg("origin")
            .arg(commit)
            .current_dir(checkout_dir)
            .envs(env);
        if let Err(error) = run_command("git fetch", fetch).await {
            tracing::warn!(%error, "git fetch for requested commit failed; trying checkout anyway");
        }

        let mut checkout = Command::new("git");
        checkout
            .arg("checkout")
            .arg(commit)
            .current_dir(checkout_dir)
            .envs(env);
        run_command("git checkout", checkout).await?;
    }

    Ok(())
}

fn workspace_dir(workload: &BuildWorkloadSpec, checkout_dir: &Path) -> PathBuf {
    workload
        .workspace
        .root_directory
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| checkout_dir.join(value))
        .unwrap_or_else(|| checkout_dir.to_path_buf())
}

fn workload_env(workload: &BuildWorkloadSpec) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for item in &workload.env {
        env.insert(item.name.clone(), item.value.clone());
    }
    env.insert(
        "TACHYON_PROJECT_ID".to_string(),
        workload.project_id.clone(),
    );
    env.insert(
        "TACHYON_BUILD_IMAGE_NAME".to_string(),
        workload.artifact.image_name.clone(),
    );
    env.insert(
        "TACHYON_BUILD_IMAGE_TAG".to_string(),
        workload.artifact.image_tag.clone(),
    );
    if let Some(registry) = workload
        .artifact
        .container_registry
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        env.insert(
            "TACHYON_BUILD_CONTAINER_REGISTRY".to_string(),
            registry.clone(),
        );
    }
    env
}

fn is_cloud_run(workload: &BuildWorkloadSpec) -> bool {
    workload
        .deployment_target
        .as_deref()
        .is_some_and(|target| target == "cloud_run")
}

fn image_uri(workload: &BuildWorkloadSpec) -> String {
    match workload
        .artifact
        .container_registry
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some(registry) => format!(
            "{}/{}:{}",
            registry.trim_end_matches('/'),
            workload.artifact.image_name,
            workload.artifact.image_tag
        ),
        None => format!(
            "{}:{}",
            workload.artifact.image_name, workload.artifact.image_tag
        ),
    }
}

async fn prepare_docker_config(env: &BTreeMap<String, String>) -> Result<()> {
    let Some(config_json) = env.get(DOCKER_CONFIG_ENV) else {
        return Ok(());
    };
    let docker_dir = Path::new("/workspace/.docker");
    tokio::fs::create_dir_all(docker_dir)
        .await
        .context("failed to create Docker config directory")?;
    tokio::fs::write(docker_dir.join("config.json"), config_json)
        .await
        .context("failed to write Docker registry config")?;
    Ok(())
}

async fn run_kaniko(
    workload: &BuildWorkloadSpec,
    checkout_dir: &Path,
    app_dir: &Path,
    env: &BTreeMap<String, String>,
) -> Result<CommandResult> {
    let dockerfile = dockerfile_path(workload, app_dir);
    if !dockerfile.exists() {
        return Err(anyhow!(
            "Dockerfile is required for Cloud Run JobRun builds: {}",
            dockerfile.display()
        ));
    }

    let context = docker_context_path(workload, checkout_dir, app_dir);
    if !context.exists() {
        return Err(anyhow!(
            "Docker build context does not exist: {}",
            context.display()
        ));
    }

    let destination = image_uri(workload);
    tracing::info!(
        dockerfile = %dockerfile.display(),
        context = %context.display(),
        destination = %destination,
        "building and pushing Cloud Run image with kaniko"
    );

    let mut command = Command::new(KANIKO_EXECUTOR);
    command
        .arg("--dockerfile")
        .arg(&dockerfile)
        .arg("--context")
        .arg(format!("dir://{}", context.display()))
        .arg("--destination")
        .arg(&destination)
        .arg("--cache=true")
        .arg("--cache-copy-layers")
        .arg("--single-snapshot")
        .arg("--verbosity=info")
        .current_dir(app_dir)
        .envs(env)
        .env("DOCKER_CONFIG", "/workspace/.docker");
    run_command("kaniko", command).await
}

fn dockerfile_path(workload: &BuildWorkloadSpec, app_dir: &Path) -> PathBuf {
    workload
        .workspace
        .root_directory
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .map(|_| app_dir.join("Dockerfile"))
        .unwrap_or_else(|| app_dir.join("Dockerfile"))
}

fn docker_context_path(
    workload: &BuildWorkloadSpec,
    checkout_dir: &Path,
    app_dir: &Path,
) -> PathBuf {
    workload
        .workspace
        .docker_context
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            if value == "." {
                checkout_dir.to_path_buf()
            } else {
                checkout_dir.join(value)
            }
        })
        .unwrap_or_else(|| app_dir.to_path_buf())
}

fn install_command(workload: &BuildWorkloadSpec, app_dir: &Path) -> Option<String> {
    workload
        .commands
        .install
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .or_else(|| infer_install_command(app_dir))
}

fn build_command(workload: &BuildWorkloadSpec, app_dir: &Path) -> Option<String> {
    workload
        .commands
        .build
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .or_else(|| infer_build_command(app_dir))
}

fn infer_install_command(app_dir: &Path) -> Option<String> {
    let root = package_manager_root(app_dir)?;
    if root.join("yarn.lock").exists() {
        Some("corepack enable && yarn install --immutable".to_string())
    } else if root.join("pnpm-lock.yaml").exists() {
        Some("corepack enable && pnpm install --frozen-lockfile".to_string())
    } else if root.join("package-lock.json").exists() {
        Some("npm ci".to_string())
    } else {
        None
    }
}

fn infer_build_command(app_dir: &Path) -> Option<String> {
    if !app_dir.join("package.json").exists() {
        return None;
    }

    let root = package_manager_root(app_dir).unwrap_or_else(|| app_dir.to_path_buf());
    if root.join("yarn.lock").exists() {
        Some("corepack enable && yarn build".to_string())
    } else if root.join("pnpm-lock.yaml").exists() {
        Some("corepack enable && pnpm build".to_string())
    } else {
        Some("npm run build".to_string())
    }
}

fn package_manager_root(app_dir: &Path) -> Option<PathBuf> {
    let mut cursor = Some(app_dir);
    while let Some(dir) = cursor {
        if dir.join("yarn.lock").exists()
            || dir.join("pnpm-lock.yaml").exists()
            || dir.join("package-lock.json").exists()
        {
            return Some(dir.to_path_buf());
        }
        if dir == Path::new("/workspace/source") {
            break;
        }
        cursor = dir.parent();
    }
    None
}

async fn run_shell(
    label: &str,
    command: &str,
    cwd: &Path,
    env: &BTreeMap<String, String>,
) -> Result<CommandResult> {
    tracing::info!(%label, %command, cwd = %cwd.display(), "running build command");
    let mut cmd = Command::new("bash");
    cmd.arg("-lc")
        .arg(command)
        .current_dir(cwd)
        .envs(env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    run_command(label, cmd).await
}

async fn run_command(label: &str, mut command: Command) -> Result<CommandResult> {
    let output = command
        .output()
        .await
        .with_context(|| format!("failed to spawn {label} command"))?;
    let stdout = truncate_output(&String::from_utf8_lossy(&output.stdout));
    let stderr = truncate_output(&String::from_utf8_lossy(&output.stderr));

    if !stdout.is_empty() {
        print!("{stdout}");
    }
    if !stderr.is_empty() {
        eprint!("{stderr}");
    }

    if !output.status.success() {
        return Err(anyhow!(
            "{label} command failed with exit code {:?}",
            output.status.code()
        ));
    }

    Ok(CommandResult { stdout, stderr })
}

fn append_result(target: &mut CommandResult, next: CommandResult) {
    target.stdout.push_str(&next.stdout);
    target.stderr.push_str(&next.stderr);
    target.stdout = truncate_output(&target.stdout);
    target.stderr = truncate_output(&target.stderr);
}

fn truncate_output(value: &str) -> String {
    if value.len() <= OUTPUT_CAPTURE_LIMIT {
        return value.to_string();
    }

    let start = value.len().saturating_sub(OUTPUT_CAPTURE_LIMIT);
    format!("... truncated {} bytes ...\n{}", start, &value[start..])
}

async fn send_callback(
    workload: &BuildWorkloadSpec,
    status: &str,
    duration_secs: i32,
    output: Option<&CommandResult>,
    error_message: Option<String>,
) -> Result<()> {
    let Some(callback) = workload.callback.as_ref() else {
        tracing::warn!("build callback URL is not configured");
        return Ok(());
    };
    let Some(build_id) = callback.build_id.as_ref() else {
        tracing::warn!("build callback ID is not configured");
        return Ok(());
    };

    let payload = CloudBuildWebhookPayload {
        build_id: build_id.clone(),
        status: status.to_string(),
        image_uri: Some(image_uri(workload)),
        artifact_path: workload.artifact.output_directory.clone(),
        duration_secs: Some(duration_secs),
        codebuild_instance_type: Some("hetzner-k3s-tachyon-cli".to_string()),
        error_message,
        iac_plan_output: None,
        iac_success: None,
        d1_migrations: None,
        cache_hit: None,
        cache_key: cache_key(Path::new("/workspace/source")),
        runner_stdout: output.map(|value| value.stdout.clone()),
        runner_stderr: output.map(|value| value.stderr.clone()),
        runner_failure_reason: None,
    };

    tracing::info!(
        build_id = %build_id,
        callback_url = %callback.url,
        status = %payload.status,
        "sending build completion callback"
    );

    reqwest::Client::new()
        .post(&callback.url)
        .json(&payload)
        .send()
        .await
        .context("failed to send build completion callback")?
        .error_for_status()
        .context("build completion callback returned error")?;

    Ok(())
}

fn cache_key(source_dir: &Path) -> Option<String> {
    for name in ["yarn.lock", "pnpm-lock.yaml", "package-lock.json"] {
        let path = source_dir.join(name);
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let digest = Sha256::digest(bytes);
        return Some(format!("{name}:sha256:{digest:x}"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_manager_root_walks_to_checkout_root() {
        let temp = tempfile::tempdir().unwrap();
        let checkout = temp.path().join("source");
        let app = checkout.join("apps").join("demo");
        std::fs::create_dir_all(&app).unwrap();
        std::fs::write(checkout.join("yarn.lock"), "").unwrap();

        assert_eq!(package_manager_root(&app), Some(checkout));
    }

    #[test]
    fn infer_build_command_uses_package_manager_lockfile() {
        let temp = tempfile::tempdir().unwrap();
        let checkout = temp.path().join("source");
        let app = checkout.join("apps").join("demo");
        std::fs::create_dir_all(&app).unwrap();
        std::fs::write(app.join("package.json"), "{}").unwrap();
        std::fs::write(checkout.join("pnpm-lock.yaml"), "").unwrap();

        assert_eq!(
            infer_build_command(&app).as_deref(),
            Some("corepack enable && pnpm build")
        );
    }

    #[test]
    fn image_uri_uses_container_registry_when_present() {
        let workload = BuildWorkloadSpec {
            project_id: "app_123".to_string(),
            source: BuildWorkloadSource {
                repository_url: "https://github.com/example/app".to_string(),
                branch: "main".to_string(),
                commit_sha: "abc".to_string(),
            },
            workspace: BuildWorkloadWorkspace {
                root_directory: None,
                docker_context: None,
            },
            commands: BuildWorkloadCommands {
                install: None,
                build: Some("true".to_string()),
                node_version: None,
            },
            env: vec![],
            artifact: BuildWorkloadArtifact {
                image_name: "demo".to_string(),
                image_tag: "bld_123".to_string(),
                output_directory: None,
                container_registry: Some(
                    "418272779906.dkr.ecr.ap-northeast-1.amazonaws.com/compute-apps".to_string(),
                ),
            },
            callback: None,
            deployment_target: Some("cloud_run".to_string()),
            framework: Some("docker".to_string()),
        };

        assert_eq!(
            image_uri(&workload),
            "418272779906.dkr.ecr.ap-northeast-1.amazonaws.com/compute-apps/demo:bld_123"
        );
    }

    #[test]
    fn docker_context_defaults_to_app_dir() {
        let temp = tempfile::tempdir().unwrap();
        let checkout = temp.path().join("source");
        let app = checkout.join("apps").join("demo");
        let workload = BuildWorkloadSpec {
            project_id: "app_123".to_string(),
            source: BuildWorkloadSource {
                repository_url: "https://github.com/example/app".to_string(),
                branch: "main".to_string(),
                commit_sha: "abc".to_string(),
            },
            workspace: BuildWorkloadWorkspace {
                root_directory: Some("apps/demo".to_string()),
                docker_context: None,
            },
            commands: BuildWorkloadCommands {
                install: None,
                build: Some("true".to_string()),
                node_version: None,
            },
            env: vec![],
            artifact: BuildWorkloadArtifact {
                image_name: "demo".to_string(),
                image_tag: "bld_123".to_string(),
                output_directory: None,
                container_registry: None,
            },
            callback: None,
            deployment_target: Some("cloud_run".to_string()),
            framework: Some("docker".to_string()),
        };

        assert_eq!(docker_context_path(&workload, &checkout, &app), app);
    }

    #[test]
    fn explicit_dot_docker_context_uses_checkout_root() {
        let temp = tempfile::tempdir().unwrap();
        let checkout = temp.path().join("source");
        let app = checkout.join("apps").join("demo");
        let workload = BuildWorkloadSpec {
            project_id: "app_123".to_string(),
            source: BuildWorkloadSource {
                repository_url: "https://github.com/example/app".to_string(),
                branch: "main".to_string(),
                commit_sha: "abc".to_string(),
            },
            workspace: BuildWorkloadWorkspace {
                root_directory: Some("apps/demo".to_string()),
                docker_context: Some(".".to_string()),
            },
            commands: BuildWorkloadCommands {
                install: None,
                build: Some("true".to_string()),
                node_version: None,
            },
            env: vec![],
            artifact: BuildWorkloadArtifact {
                image_name: "demo".to_string(),
                image_tag: "bld_123".to_string(),
                output_directory: None,
                container_registry: None,
            },
            callback: None,
            deployment_target: Some("cloud_run".to_string()),
            framework: Some("docker".to_string()),
        };

        assert_eq!(docker_context_path(&workload, &checkout, &app), checkout);
    }
}
