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
const BUILDKIT_DAEMONLESS: &str = "/usr/local/bin/buildctl-daemonless.sh";
const BUILDKIT_DOCKER_CONFIG_DIR: &str = "/workspace/.docker";

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

    // Workers detection wins over Pages: an opennextjs build
    // (output under `.open-next`) produces a Worker module even
    // when the app is registered as a Pages target. Mirrors the
    // CodeBuild buildspec branching.
    let is_workers = is_cloudflare_workers(workload);
    let is_pages = !is_workers && is_cloudflare_pages(workload, &env);
    if is_pages {
        validate_pages_binding_keys(&env)?;
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
            run_buildkit(workload, &checkout_dir, &app_dir, &env).await?,
        );
    }

    if is_workers {
        append_result(
            &mut combined,
            run_cloudflare_workers_deploy(workload, &app_dir, &env).await?,
        );
    } else if is_pages {
        append_result(
            &mut combined,
            run_cloudflare_pages_deploy(workload, &checkout_dir, &app_dir, &env).await?,
        );
    }

    if is_lambda(workload) {
        append_result(
            &mut combined,
            run_lambda_package_and_upload(workload, &app_dir, &env).await?,
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

fn is_cloudflare_pages(workload: &BuildWorkloadSpec, env: &BTreeMap<String, String>) -> bool {
    workload
        .deployment_target
        .as_deref()
        .is_some_and(|target| target == "cloudflare_pages")
        || env_value(env, "PAGES_PROJECT_NAME").is_some()
}

fn is_cloudflare_workers(workload: &BuildWorkloadSpec) -> bool {
    workload
        .deployment_target
        .as_deref()
        .is_some_and(|target| target == "cloudflare_workers")
        || workload
            .artifact
            .output_directory
            .as_deref()
            .is_some_and(|dir| dir.trim().starts_with(".open-next"))
}

fn is_lambda(workload: &BuildWorkloadSpec) -> bool {
    workload
        .deployment_target
        .as_deref()
        .is_some_and(|target| target == "lambda")
}

/// Pinned wrangler version for Pages deploys.
///
/// `npx wrangler` resolves the latest release when the app does
/// not depend on wrangler itself, so an upstream release can
/// change CLI behavior mid-pipeline. Bump deliberately.
const WRANGLER_NPM_SPEC: &str = "wrangler@4.100.0";

async fn run_cloudflare_pages_deploy(
    workload: &BuildWorkloadSpec,
    checkout_dir: &Path,
    app_dir: &Path,
    env: &BTreeMap<String, String>,
) -> Result<CommandResult> {
    let account_id = required_env(env, "CLOUDFLARE_ACCOUNT_ID")?;
    required_env(env, "CLOUDFLARE_API_TOKEN")?;
    let project_name = required_env(env, "PAGES_PROJECT_NAME")?;
    let output_dir = pages_output_dir(workload, checkout_dir, app_dir, env)?;
    let commit_message = ascii_commit_message(env);

    // `wrangler pages deploy` has no `--binding` flag: runtime
    // bindings live in the Pages project settings (dashboard or
    // wrangler.toml). Older wrangler releases silently ignored
    // unknown arguments; 4.9x+ rejects them, so the keys are
    // validated before build and surfaced for observability here.
    let binding_keys = pages_binding_keys(env);
    if !binding_keys.is_empty() {
        tracing::warn!(
            keys = %binding_keys.join(","),
            "CF_PAGES_BINDING_KEYS are not applied by `wrangler pages deploy`; \
             configure runtime bindings on the Pages project instead"
        );
    }

    tracing::info!(
        project_name = %project_name,
        account_id = %account_id,
        branch = %workload.source.branch,
        output_dir = %output_dir.display(),
        "deploying Cloudflare Pages output"
    );

    let mut command = Command::new("npx");
    command
        .arg("--yes")
        .arg(WRANGLER_NPM_SPEC)
        .arg("pages")
        .arg("deploy")
        .arg(&output_dir)
        .arg("--project-name")
        .arg(project_name)
        .arg("--branch")
        .arg(&workload.source.branch)
        .arg("--commit-message")
        .arg(commit_message)
        .envs(env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    run_command("wrangler pages deploy", command).await
}

/// Deploy a Worker module with `wrangler deploy`.
///
/// Unlike Pages, `wrangler deploy` reads `wrangler.toml` /
/// `wrangler.json` for the entry module and bindings, so it must
/// run from the app directory. `WORKERS_SCRIPT_NAME` (set by the
/// control plane to the Cloud App name) overrides the script
/// name from the wrangler config when present.
async fn run_cloudflare_workers_deploy(
    workload: &BuildWorkloadSpec,
    app_dir: &Path,
    env: &BTreeMap<String, String>,
) -> Result<CommandResult> {
    required_env(env, "CLOUDFLARE_ACCOUNT_ID")?;
    required_env(env, "CLOUDFLARE_API_TOKEN")?;

    let has_wrangler_config = ["wrangler.toml", "wrangler.json", "wrangler.jsonc"]
        .iter()
        .any(|name| app_dir.join(name).exists());
    if !has_wrangler_config {
        return Err(anyhow!(
            "wrangler configuration file not found: Workers deployments \
             require wrangler.toml or wrangler.json"
        ));
    }

    tracing::info!(
        branch = %workload.source.branch,
        app_dir = %app_dir.display(),
        script_name = env_value(env, "WORKERS_SCRIPT_NAME").unwrap_or("<from wrangler config>"),
        "deploying Cloudflare Workers script"
    );

    let mut command = Command::new("npx");
    command.arg("--yes").arg(WRANGLER_NPM_SPEC).arg("deploy");
    if let Some(name) = env_value(env, "WORKERS_SCRIPT_NAME") {
        command.arg("--name").arg(name);
    }
    command
        .current_dir(app_dir)
        .envs(env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    run_command("wrangler deploy", command).await
}

/// Presigned S3 PUT URL for the Lambda artifact. Injected by the
/// control plane so the builder needs no AWS credentials.
const LAMBDA_UPLOAD_URL_ENV: &str = "TACHYON_LAMBDA_ARTIFACT_UPLOAD_URL";
/// `s3://bucket/key` path reported back in the completion
/// callback so `CreateDeployment` can locate the artifact.
const LAMBDA_ARTIFACT_PATH_ENV: &str = "TACHYON_LAMBDA_ARTIFACT_PATH";

fn is_rust_lambda(workload: &BuildWorkloadSpec) -> bool {
    let framework = workload.framework.as_deref();
    matches!(framework, Some("rust_server") | Some("cargo_lambda"))
        || workload
            .commands
            .build
            .as_deref()
            .is_some_and(|command| command.contains("cargo"))
        || workload
            .commands
            .install
            .as_deref()
            .is_some_and(|command| command.contains("cargo"))
}

/// Package the Lambda artifact and upload it through the
/// presigned URL.
///
/// Node Lambda only: the builder image carries no Rust
/// toolchain, so cargo / cargo-lambda / rust_server apps must
/// keep using the CodeBuild backend until the image gains one.
async fn run_lambda_package_and_upload(
    workload: &BuildWorkloadSpec,
    app_dir: &Path,
    env: &BTreeMap<String, String>,
) -> Result<CommandResult> {
    if is_rust_lambda(workload) {
        return Err(anyhow!(
            "Rust Lambda builds are not supported by the JobRun builder \
             yet; use the CodeBuild backend for cargo / cargo-lambda / \
             rust_server apps"
        ));
    }

    let upload_url = env_value(env, LAMBDA_UPLOAD_URL_ENV)
        .ok_or_else(|| anyhow!("{LAMBDA_UPLOAD_URL_ENV} is required for Lambda artifact upload"))?;

    let (archive, source) = package_lambda_archive(app_dir)?;
    let size = archive.len();
    tracing::info!(%source, size_bytes = size, "uploading Lambda artifact");

    reqwest::Client::new()
        .put(upload_url)
        .body(archive)
        .send()
        .await
        .context("failed to upload Lambda artifact")?
        .error_for_status()
        .context("Lambda artifact upload returned error")?;

    Ok(CommandResult {
        stdout: format!("uploaded lambda.zip ({size} bytes) from {source}\n"),
        stderr: String::new(),
    })
}

fn package_lambda_archive(app_dir: &Path) -> Result<(Vec<u8>, String)> {
    let prebuilt = app_dir.join("lambda.zip");
    if prebuilt.is_file() {
        let bytes = std::fs::read(&prebuilt).context("failed to read pre-built lambda.zip")?;
        return Ok((bytes, "pre-built lambda.zip".to_string()));
    }
    let dist = app_dir.join("dist");
    if dist.is_dir() {
        return Ok((zip_directory(&dist)?, "dist/".to_string()));
    }
    Err(anyhow!(
        "no lambda.zip or dist/ found after build; Lambda apps must \
         produce either a pre-built lambda.zip or a dist/ directory"
    ))
}

/// Zip a directory recursively with the entries rooted at the
/// directory itself (so `dist/index.js` becomes `index.js`).
fn zip_directory(dir: &Path) -> Result<Vec<u8>> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    add_zip_entries(&mut writer, dir, dir)?;
    let cursor = writer.finish().context("failed to finalize zip archive")?;
    Ok(cursor.into_inner())
}

fn add_zip_entries(
    writer: &mut zip::ZipWriter<std::io::Cursor<Vec<u8>>>,
    root: &Path,
    dir: &Path,
) -> Result<()> {
    use std::io::Write;

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read directory: {}", dir.display()))?
        .collect::<std::io::Result<_>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .context("zip entry escapes archive root")?
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            writer.add_directory(format!("{relative}/"), options)?;
            add_zip_entries(writer, root, &path)?;
        } else {
            writer.start_file(relative, options)?;
            let bytes = std::fs::read(&path)
                .with_context(|| format!("failed to read file: {}", path.display()))?;
            writer.write_all(&bytes)?;
        }
    }
    Ok(())
}

fn env_value<'a>(env: &'a BTreeMap<String, String>, key: &str) -> Option<&'a str> {
    env.get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
}

fn required_env<'a>(env: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str> {
    env_value(env, key).ok_or_else(|| anyhow!("{key} is required for Cloudflare deploy"))
}

fn pages_output_dir(
    workload: &BuildWorkloadSpec,
    checkout_dir: &Path,
    app_dir: &Path,
    env: &BTreeMap<String, String>,
) -> Result<PathBuf> {
    let raw = workload
        .artifact
        .output_directory
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| env_value(env, "OUTPUT_DIR"))
        .ok_or_else(|| {
            anyhow!("output_directory or OUTPUT_DIR is required for Cloudflare Pages deploy")
        })?;

    let raw_path = Path::new(raw);
    if raw_path.is_absolute() {
        return Ok(raw_path.to_path_buf());
    }

    let checkout_relative = checkout_dir.join(raw_path);
    if checkout_relative.exists() {
        return Ok(checkout_relative);
    }

    Ok(app_dir.join(raw_path))
}

fn ascii_commit_message(env: &BTreeMap<String, String>) -> String {
    let message = env_value(env, "COMMIT_MESSAGE")
        .or_else(|| env_value(env, "GIT_COMMIT_MESSAGE"))
        .unwrap_or("Tachyon Cloud App build");
    let ascii = message
        .chars()
        .map(|ch| {
            if ch.is_ascii() && !ch.is_control() {
                ch
            } else {
                ' '
            }
        })
        .collect::<String>();
    let normalized = ascii.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        "Tachyon Cloud App build".to_string()
    } else {
        normalized
    }
}

fn pages_binding_keys(env: &BTreeMap<String, String>) -> Vec<String> {
    env_value(env, "CF_PAGES_BINDING_KEYS")
        .map(|keys| {
            keys.split(',')
                .map(str::trim)
                .filter(|key| !key.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn validate_pages_binding_keys(env: &BTreeMap<String, String>) -> Result<Vec<String>> {
    let keys = pages_binding_keys(env);
    for key in &keys {
        if !is_valid_env_key(key) {
            return Err(anyhow!("invalid Cloudflare Pages binding key: {key}"));
        }
        if !env.contains_key(key) {
            return Err(anyhow!(
                "Cloudflare Pages binding env var is missing: {key}"
            ));
        }
    }
    Ok(keys)
}

fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn image_uri(workload: &BuildWorkloadSpec) -> String {
    match workload
        .artifact
        .container_registry
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some(registry) if is_ecr_repository_url(registry) => format!(
            "{}:{}-{}",
            registry.trim_end_matches('/'),
            docker_tag_component(&workload.artifact.image_name),
            workload.artifact.image_tag
        ),
        Some(registry) => {
            format!(
                "{}/{}:{}",
                registry.trim_end_matches('/'),
                workload.artifact.image_name,
                workload.artifact.image_tag
            )
        }
        None => format!(
            "{}:{}",
            workload.artifact.image_name, workload.artifact.image_tag
        ),
    }
}

fn is_ecr_repository_url(registry: &str) -> bool {
    registry
        .split('/')
        .next()
        .is_some_and(|host| host.contains(".dkr.ecr."))
}

fn docker_tag_component(value: &str) -> String {
    let tag = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();

    tag.trim_matches(['.', '-']).to_string()
}

async fn prepare_docker_config(env: &BTreeMap<String, String>) -> Result<()> {
    let Some(config_json) = env.get(DOCKER_CONFIG_ENV) else {
        return Ok(());
    };
    let docker_dir = Path::new(BUILDKIT_DOCKER_CONFIG_DIR);
    tokio::fs::create_dir_all(docker_dir)
        .await
        .with_context(|| {
            format!(
                "failed to create Docker config directory: {}",
                docker_dir.display()
            )
        })?;
    tokio::fs::write(docker_dir.join("config.json"), config_json)
        .await
        .with_context(|| {
            format!(
                "failed to write Docker registry config: {}",
                docker_dir.display()
            )
        })?;
    Ok(())
}

async fn run_buildkit(
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
        "building and pushing Cloud Run image with BuildKit"
    );

    let dockerfile_dir = dockerfile
        .parent()
        .ok_or_else(|| anyhow!("Dockerfile has no parent directory"))?;

    let mut command = Command::new(BUILDKIT_DAEMONLESS);
    command
        .arg("build")
        .arg("--progress=plain")
        .arg("--frontend")
        .arg("dockerfile.v0")
        .arg("--local")
        .arg(format!("context={}", context.display()))
        .arg("--local")
        .arg(format!("dockerfile={}", dockerfile_dir.display()))
        .arg("--opt")
        .arg(format!(
            "filename={}",
            dockerfile
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Dockerfile")
        ))
        .arg("--output")
        .arg(format!("type=image,name={destination},push=true"))
        .current_dir(app_dir)
        .envs(env)
        .env("HOME", "/workspace")
        .env("DOCKER_CONFIG", BUILDKIT_DOCKER_CONFIG_DIR);
    if let Some(cache_repo) = buildkit_cache_repository(workload) {
        command
            .arg("--import-cache")
            .arg(format!("type=registry,ref={cache_repo}"))
            .arg("--export-cache")
            .arg(format!("type=registry,ref={cache_repo},mode=max"));
    }
    run_command("buildkit", command).await
}

fn buildkit_cache_repository(workload: &BuildWorkloadSpec) -> Option<String> {
    workload
        .artifact
        .container_registry
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|registry| {
            format!(
                "{}/{}/cache:buildkit",
                registry.trim_end_matches('/'),
                workload.artifact.image_name
            )
        })
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

    let mut start = value.len().saturating_sub(OUTPUT_CAPTURE_LIMIT);
    // Slicing at an arbitrary byte offset panics when it lands inside a
    // multi-byte character (e.g. box-drawing lines in wrangler output).
    while !value.is_char_boundary(start) {
        start += 1;
    }
    format!("... truncated {} bytes ...\n{}", start, &value[start..])
}

/// Artifact path reported in the completion callback.
///
/// Lambda builds report the `s3://bucket/key` location the
/// artifact was uploaded to (consumed by `deploy_to_lambda` on
/// the control plane); other targets keep reporting the build
/// output directory.
fn callback_artifact_path(workload: &BuildWorkloadSpec) -> Option<String> {
    if is_lambda(workload) {
        let env = workload_env(workload);
        if let Some(path) = env_value(&env, LAMBDA_ARTIFACT_PATH_ENV) {
            return Some(path.to_string());
        }
    }
    workload.artifact.output_directory.clone()
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
        artifact_path: callback_artifact_path(workload),
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

    fn test_workload(deployment_target: Option<&str>) -> BuildWorkloadSpec {
        BuildWorkloadSpec {
            project_id: "app_123".to_string(),
            source: BuildWorkloadSource {
                repository_url: "https://github.com/example/app".to_string(),
                branch: "feature/demo".to_string(),
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
            deployment_target: deployment_target.map(str::to_string),
            framework: Some("next_js".to_string()),
        }
    }

    #[test]
    fn truncate_output_keeps_short_values_unchanged() {
        let value = "short output";
        assert_eq!(truncate_output(value), value);
    }

    #[test]
    fn truncate_output_does_not_split_multibyte_characters() {
        // Build a value whose truncation offset lands inside a
        // multi-byte character, like wrangler's box-drawing lines.
        // "─" is 3 bytes; the ascii tail length makes the raw
        // truncation offset land one byte inside the last "─".
        let mut value = "─".repeat(OUTPUT_CAPTURE_LIMIT / 3);
        value.push_str(&"x".repeat(OUTPUT_CAPTURE_LIMIT - 1));

        let truncated = truncate_output(&value);
        assert!(truncated.starts_with("... truncated "));
        assert!(truncated.len() <= OUTPUT_CAPTURE_LIMIT + 64);
        assert!(truncated.ends_with('x'));
    }

    #[test]
    fn workers_detection_uses_target_or_open_next_output() {
        assert!(is_cloudflare_workers(&test_workload(Some(
            "cloudflare_workers"
        ))));

        let mut open_next = test_workload(Some("cloudflare_pages"));
        open_next.artifact.output_directory = Some(".open-next/assets".to_string());
        assert!(is_cloudflare_workers(&open_next));

        assert!(!is_cloudflare_workers(&test_workload(Some(
            "cloudflare_pages"
        ))));
        assert!(!is_cloudflare_workers(&test_workload(None)));
    }

    #[test]
    fn lambda_rejects_rust_builds() {
        let mut workload = test_workload(Some("lambda"));
        workload.framework = Some("rust_server".to_string());
        assert!(is_rust_lambda(&workload));

        workload.framework = Some("cargo_lambda".to_string());
        assert!(is_rust_lambda(&workload));

        workload.framework = Some("hono".to_string());
        workload.commands.build = Some("cargo lambda build --release".to_string());
        assert!(is_rust_lambda(&workload));

        workload.commands.build = Some("yarn build".to_string());
        assert!(!is_rust_lambda(&workload));
    }

    #[test]
    fn package_lambda_archive_prefers_prebuilt_zip() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("lambda.zip"), b"prebuilt").unwrap();
        std::fs::create_dir_all(temp.path().join("dist")).unwrap();

        let (bytes, source) = package_lambda_archive(temp.path()).unwrap();
        assert_eq!(bytes, b"prebuilt");
        assert_eq!(source, "pre-built lambda.zip");
    }

    #[test]
    fn package_lambda_archive_zips_dist_directory() {
        let temp = tempfile::tempdir().unwrap();
        let dist = temp.path().join("dist");
        std::fs::create_dir_all(dist.join("lib")).unwrap();
        std::fs::write(dist.join("index.js"), "exports.handler = 1").unwrap();
        std::fs::write(dist.join("lib/util.js"), "module.exports = {}").unwrap();

        let (bytes, source) = package_lambda_archive(temp.path()).unwrap();
        assert_eq!(source, "dist/");

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(names.contains(&"index.js".to_string()));
        assert!(names.contains(&"lib/util.js".to_string()));
    }

    #[test]
    fn package_lambda_archive_requires_artifact() {
        let temp = tempfile::tempdir().unwrap();
        let error = package_lambda_archive(temp.path()).unwrap_err();
        assert!(error.to_string().contains("no lambda.zip or dist/"));
    }

    #[test]
    fn callback_artifact_path_uses_lambda_s3_path() {
        let mut workload = test_workload(Some("lambda"));
        workload.env.push(BuildWorkloadEnvVar {
            name: LAMBDA_ARTIFACT_PATH_ENV.to_string(),
            value: "s3://bucket/builds/app_123/abc/lambda.zip".to_string(),
        });
        assert_eq!(
            callback_artifact_path(&workload).as_deref(),
            Some("s3://bucket/builds/app_123/abc/lambda.zip")
        );

        let mut pages = test_workload(Some("cloudflare_pages"));
        pages.artifact.output_directory = Some("out".to_string());
        assert_eq!(callback_artifact_path(&pages).as_deref(), Some("out"));
    }

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
            "418272779906.dkr.ecr.ap-northeast-1.amazonaws.com/compute-apps:demo-bld_123"
        );
    }

    #[test]
    fn buildkit_cache_repository_uses_app_cache_ref() {
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
                    "418272779906.dkr.ecr.ap-northeast-1.amazonaws.com/compute-apps/".to_string(),
                ),
            },
            callback: None,
            deployment_target: Some("cloud_run".to_string()),
            framework: Some("docker".to_string()),
        };

        assert_eq!(
            buildkit_cache_repository(&workload).as_deref(),
            Some(
                "418272779906.dkr.ecr.ap-northeast-1.amazonaws.com/compute-apps/demo/cache:buildkit"
            )
        );
    }

    #[test]
    fn buildkit_docker_config_uses_workspace_path() {
        assert_eq!(BUILDKIT_DOCKER_CONFIG_DIR, "/workspace/.docker");
    }

    #[test]
    fn cloudflare_pages_runs_for_target_or_project_name() {
        let mut env = BTreeMap::new();
        assert!(is_cloudflare_pages(
            &test_workload(Some("cloudflare_pages")),
            &env
        ));
        assert!(!is_cloudflare_pages(
            &test_workload(Some("cloud_run")),
            &env
        ));

        env.insert("PAGES_PROJECT_NAME".to_string(), "field".to_string());
        assert!(is_cloudflare_pages(&test_workload(Some("cloud_run")), &env));
    }

    #[test]
    fn pages_output_dir_prefers_artifact_output_directory() {
        let temp = tempfile::tempdir().unwrap();
        let checkout = temp.path().join("source");
        let app = checkout.join("apps").join("demo");
        let output = checkout.join("apps").join("demo").join("dist");
        std::fs::create_dir_all(&output).unwrap();
        let mut workload = test_workload(Some("cloudflare_pages"));
        workload.artifact.output_directory = Some("apps/demo/dist".to_string());
        let mut env = BTreeMap::new();
        env.insert("OUTPUT_DIR".to_string(), "ignored".to_string());

        assert_eq!(
            pages_output_dir(&workload, &checkout, &app, &env).unwrap(),
            output
        );
    }

    #[test]
    fn pages_output_dir_uses_env_output_dir_when_artifact_missing() {
        let temp = tempfile::tempdir().unwrap();
        let checkout = temp.path().join("source");
        let app = checkout.join("apps").join("demo");
        let output = app.join(".vercel").join("output").join("static");
        std::fs::create_dir_all(&output).unwrap();
        let workload = test_workload(Some("cloudflare_pages"));
        let mut env = BTreeMap::new();
        env.insert(
            "OUTPUT_DIR".to_string(),
            ".vercel/output/static".to_string(),
        );

        assert_eq!(
            pages_output_dir(&workload, &checkout, &app, &env).unwrap(),
            output
        );
    }

    #[test]
    fn ascii_commit_message_strips_non_ascii_and_controls() {
        let mut env = BTreeMap::new();
        env.insert(
            "COMMIT_MESSAGE".to_string(),
            "fix: deploy 日本語\nmessage".to_string(),
        );

        assert_eq!(ascii_commit_message(&env), "fix: deploy message");
    }

    #[test]
    fn pages_binding_keys_parses_and_trims_keys() {
        let mut env = BTreeMap::new();
        env.insert(
            "CF_PAGES_BINDING_KEYS".to_string(),
            "API_URL, SECRET_TOKEN,,".to_string(),
        );

        assert_eq!(
            pages_binding_keys(&env),
            vec!["API_URL".to_string(), "SECRET_TOKEN".to_string()]
        );
    }

    #[test]
    fn validate_pages_binding_keys_requires_valid_present_env_keys() {
        let mut env = BTreeMap::new();
        env.insert(
            "CF_PAGES_BINDING_KEYS".to_string(),
            "API_URL, SECRET_TOKEN".to_string(),
        );
        env.insert("API_URL".to_string(), "https://example.com".to_string());
        env.insert("SECRET_TOKEN".to_string(), "secret-value".to_string());

        assert_eq!(
            validate_pages_binding_keys(&env).unwrap(),
            vec!["API_URL".to_string(), "SECRET_TOKEN".to_string()]
        );
    }

    #[test]
    fn validate_pages_binding_keys_rejects_invalid_keys() {
        let mut env = BTreeMap::new();
        env.insert("CF_PAGES_BINDING_KEYS".to_string(), "BAD-KEY".to_string());
        env.insert("BAD-KEY".to_string(), "value".to_string());

        let error = validate_pages_binding_keys(&env).unwrap_err().to_string();
        assert!(error.contains("invalid Cloudflare Pages binding key"));
    }

    #[test]
    fn validate_pages_binding_keys_rejects_missing_env_values() {
        let mut env = BTreeMap::new();
        env.insert(
            "CF_PAGES_BINDING_KEYS".to_string(),
            "SECRET_TOKEN".to_string(),
        );

        let error = validate_pages_binding_keys(&env).unwrap_err().to_string();
        assert!(error.contains("Cloudflare Pages binding env var is missing"));
    }

    #[test]
    fn pages_binding_keys_is_empty_without_env() {
        let env = BTreeMap::new();
        assert!(pages_binding_keys(&env).is_empty());
    }

    #[test]
    fn image_uri_keeps_nested_repository_for_non_ecr_registry() {
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
                container_registry: Some("ghcr.io/example/compute-apps".to_string()),
            },
            callback: None,
            deployment_target: Some("cloud_run".to_string()),
            framework: Some("docker".to_string()),
        };

        assert_eq!(
            image_uri(&workload),
            "ghcr.io/example/compute-apps/demo:bld_123"
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
