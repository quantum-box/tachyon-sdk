use std::{
    collections::HashMap,
    io::{self, Read},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, truncate, ApiClient};
use crate::config::loader::{self, LoadedProjectConfig, ProjectConfig, RepositoryConfig};
use crate::resolve;

#[derive(Debug, Clone, Args)]
pub struct OpsArgs {
    #[command(subcommand)]
    pub command: OpsCommand,
}

#[derive(Debug, Clone, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum OpsCommand {
    /// Manage deployment events
    Deployments {
        #[command(subcommand)]
        command: OpsDeploymentsCommand,
    },
    /// Manage scenario test reports
    Reports {
        #[command(subcommand)]
        command: ReportsCommand,
    },
    /// Manage tool jobs (async agent tasks)
    ToolJobs {
        #[command(subcommand)]
        command: ToolJobsCommand,
    },
    /// Send chat notifications to configured Slack/Discord destinations
    #[command(visible_alias = "slack")]
    Notify {
        #[command(subcommand)]
        command: NotifyCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum OpsDeploymentsCommand {
    /// List deployment events
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get deployment event details
    Get {
        id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ReportsCommand {
    /// List scenario test reports
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get a scenario test report
    Get {
        run_id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ToolJobsCommand {
    /// Create a tool job
    Create {
        /// Tool job provider (codex, claude_code, cursor_agent, opencode)
        #[arg(long)]
        provider: String,
        /// Prompt to send. If omitted, stdin is read.
        #[arg(long)]
        prompt: Option<String>,
        /// Context path to send. Can be specified multiple times.
        #[arg(long = "context-path")]
        context_paths: Vec<String>,
        /// Working directory recorded in metadata.cwd
        #[arg(long)]
        cwd: Option<String>,
        /// Repository name whose manifest repository.localPath is used as metadata.cwd
        #[arg(long)]
        repo: Option<String>,
        /// CloudApp manifest app name used to find its repository cwd
        #[arg(long = "cloud-app")]
        cloud_app: Option<String>,
        /// Environment variable to pass, formatted as KEY=VALUE.
        #[arg(long = "env")]
        environment: Vec<String>,
        /// Output profile name
        #[arg(long)]
        output_profile: Option<String>,
        /// Existing session ID to resume or group with this job.
        #[arg(long, visible_alias = "session-id")]
        resume_session_id: Option<String>,
        /// Target worker ID or name.
        #[arg(long)]
        worker_id: Option<String>,
        /// Use a git worktree for isolated editing.
        #[arg(long)]
        use_worktree: bool,
        /// Auto-merge the worktree branch on success.
        #[arg(long)]
        auto_merge: bool,
        /// Codex execution mode stored in metadata.codex_mode.
        #[arg(long)]
        codex_mode: Option<String>,
        /// Wait until the job reaches a terminal status.
        #[arg(long)]
        wait: bool,
        /// Maximum wait time in seconds.
        #[arg(long, default_value_t = 300)]
        wait_timeout_seconds: u64,
        /// Poll interval in seconds while waiting.
        #[arg(long, default_value_t = 2)]
        wait_interval_seconds: u64,
        /// Print the API response as JSON.
        #[arg(long)]
        json: bool,
    },
    /// List tool jobs
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get tool job details
    Get {
        job_id: String,
        #[arg(long)]
        json: bool,
    },
    /// Cancel a tool job
    Cancel { job_id: String },
    /// List available tool job providers
    Providers {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum NotifyCommand {
    /// Send a plain-text notification
    Send {
        /// Notification text to send
        #[arg(long)]
        text: String,
        /// Slack user ID or email to mention. Can be specified multiple times.
        #[arg(long = "mention")]
        mentions: Vec<String>,
        /// Slack Bot token used for email lookups
        #[arg(long = "bot-token", env = "TACHYON_SLACK_BOT_TOKEN")]
        bot_token: Option<String>,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
    /// List Slack users visible to the configured Slack Bot token
    Users {
        /// Slack Bot token used for users.list
        #[arg(long = "bot-token", env = "TACHYON_SLACK_BOT_TOKEN")]
        bot_token: Option<String>,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
}

// ---- Response types ----

#[derive(Debug, Deserialize, Serialize)]
struct OpsDeploymentResponse {
    id: String,
    #[serde(default)]
    service: Option<String>,
    #[serde(default)]
    environment: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScenarioReportResponse {
    #[serde(default)]
    run_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    total: Option<i32>,
    #[serde(default)]
    passed: Option<i32>,
    #[serde(default)]
    failed: Option<i32>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ToolJobResponse {
    id: String,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    context_paths: Vec<String>,
    #[serde(default)]
    normalized_output: Option<Value>,
    #[serde(default)]
    error_message: Option<String>,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    resume_session_id: Option<String>,
    #[serde(default)]
    assigned_worker_id: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ToolJobListResponse {
    jobs: Vec<ToolJobResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ToolJobCreatedResponse {
    job: ToolJobResponse,
}

#[derive(Debug, Deserialize, Serialize)]
struct ToolJobProvidersResponse {
    providers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ToolJobCreateRequest {
    provider: String,
    prompt: String,
    context_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_profile: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    environment: HashMap<String, String>,
    metadata: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    resume_session_id: Option<String>,
    use_worktree: bool,
    auto_merge: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    worker_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct SendNotificationRequest {
    text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    mentions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SendNotificationResponse {
    accepted: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct SlackUsersListResponse {
    ok: bool,
    #[serde(default)]
    members: Vec<SlackUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    response_metadata: Option<SlackResponseMetadata>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SlackLookupByEmailResponse {
    ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user: Option<SlackUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SlackUser {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    deleted: bool,
    #[serde(default)]
    is_bot: bool,
    #[serde(default)]
    profile: SlackUserProfile,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct SlackUserProfile {
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    real_name: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct SlackResponseMetadata {
    #[serde(default)]
    next_cursor: Option<String>,
}

// ---- Handlers ----

async fn run_deployments_list(api: &ApiClient, json: bool) -> Result<()> {
    let deps: Vec<OpsDeploymentResponse> = api.get("/v1/ops/deployments").await?;
    if json {
        return print_json(&deps);
    }
    if deps.is_empty() {
        println!("No deployment events found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<20}  {:<12}  {:<12}  {:<16}  CREATED AT",
        "ID", "SERVICE", "ENVIRONMENT", "STATUS", "VERSION"
    );
    println!(
        "{:-<28}  {:-<20}  {:-<12}  {:-<12}  {:-<16}  {:-<19}",
        "", "", "", "", "", ""
    );
    for d in &deps {
        println!(
            "{:<28}  {:<20}  {:<12}  {:<12}  {:<16}  {}",
            d.id,
            truncate(d.service.as_deref().unwrap_or("-"), 20),
            d.environment.as_deref().unwrap_or("-"),
            d.status.as_deref().unwrap_or("-"),
            d.version.as_deref().unwrap_or("-"),
            d.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_deployments_get(api: &ApiClient, id: &str, json: bool) -> Result<()> {
    let d: OpsDeploymentResponse = api.get(&format!("/v1/ops/deployments/{id}")).await?;
    if json {
        return print_json(&d);
    }
    println!("ID:          {}", d.id);
    println!("Service:     {}", d.service.as_deref().unwrap_or("-"));
    println!("Environment: {}", d.environment.as_deref().unwrap_or("-"));
    println!("Status:      {}", d.status.as_deref().unwrap_or("-"));
    println!("Version:     {}", d.version.as_deref().unwrap_or("-"));
    println!("Created:     {}", d.created_at.as_deref().unwrap_or("-"));
    Ok(())
}

async fn run_reports_list(api: &ApiClient, json: bool) -> Result<()> {
    let reports: Vec<ScenarioReportResponse> = api.get("/v1/ops/scenario-reports").await?;
    if json {
        return print_json(&reports);
    }
    if reports.is_empty() {
        println!("No scenario reports found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<10}  {:<8}  {:<8}  {:<8}  CREATED AT",
        "RUN ID", "STATUS", "TOTAL", "PASSED", "FAILED"
    );
    println!(
        "{:-<28}  {:-<10}  {:-<8}  {:-<8}  {:-<8}  {:-<19}",
        "", "", "", "", "", ""
    );
    for r in &reports {
        println!(
            "{:<28}  {:<10}  {:<8}  {:<8}  {:<8}  {}",
            r.run_id.as_deref().unwrap_or("-"),
            r.status.as_deref().unwrap_or("-"),
            r.total
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            r.passed
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            r.failed
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            r.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_reports_get(api: &ApiClient, run_id: &str, json: bool) -> Result<()> {
    let r: serde_json::Value = api
        .get(&format!("/v1/ops/scenario-reports/{run_id}"))
        .await?;
    if json {
        return print_json(&r);
    }
    println!("{}", serde_json::to_string_pretty(&r)?);
    Ok(())
}

fn read_prompt(prompt: &Option<String>) -> Result<String> {
    if let Some(prompt) = prompt {
        if prompt.trim().is_empty() {
            return Err(anyhow!("--prompt must not be empty"));
        }
        return Ok(prompt.clone());
    }

    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .context("read prompt from stdin")?;
    if buf.trim().is_empty() {
        return Err(anyhow!(
            "prompt is required. Pass --prompt or pipe prompt text to stdin"
        ));
    }
    Ok(buf)
}

fn parse_environment(values: &[String]) -> Result<HashMap<String, String>> {
    let mut environment = HashMap::new();
    for value in values {
        let Some((key, val)) = value.split_once('=') else {
            return Err(anyhow!(
                "--env must be formatted as KEY=VALUE, got '{value}'"
            ));
        };
        if key.is_empty() {
            return Err(anyhow!("--env key must not be empty"));
        }
        environment.insert(key.to_string(), val.to_string());
    }
    Ok(environment)
}

fn build_tool_job_metadata(
    provider: &str,
    cwd: Option<&str>,
    codex_mode: Option<&str>,
) -> Result<Value> {
    let cwd = match cwd {
        Some(cwd) => cwd.to_string(),
        None => std::env::current_dir()
            .context("resolve current directory")?
            .display()
            .to_string(),
    };
    let mut metadata = json!({
        "cwd": cwd,
        "source": "tachyon-cli",
    });

    if provider == "codex" || codex_mode.is_some() {
        metadata["codex_mode"] = json!(codex_mode.unwrap_or("app_server_ws"));
    }

    Ok(metadata)
}

fn resolve_manifest_path(manifest_path: &Path, value: &Path) -> PathBuf {
    if value.is_absolute() {
        value.to_path_buf()
    } else {
        manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(value)
    }
}

#[derive(Debug)]
struct ManifestRepository<'a> {
    app_name: &'a str,
    repo_name: &'a str,
    local_path: &'a Path,
}

#[derive(Debug)]
struct ResolvedToolJobCwd {
    cwd: String,
}

fn repository_entry<'a>(
    manifest_path: &Path,
    app_name: &'a str,
    repository: Option<&'a RepositoryConfig>,
) -> Result<ManifestRepository<'a>> {
    let repository = repository.ok_or_else(|| {
        anyhow!(
            "cloud app '{app_name}' in {} is missing repository",
            manifest_path.display()
        )
    })?;
    let repo_name = repository.name.as_deref().ok_or_else(|| {
        anyhow!(
            "cloud app '{app_name}' in {} is missing repository.name",
            manifest_path.display()
        )
    })?;
    let local_path = repository.local_path.as_deref().ok_or_else(|| {
        anyhow!(
            "repository '{repo_name}' for cloud app '{app_name}' in {} is missing repository.localPath",
            manifest_path.display()
        )
    })?;
    Ok(ManifestRepository {
        app_name,
        repo_name,
        local_path,
    })
}

fn collect_manifest_repositories<'a>(
    loaded: &'a LoadedProjectConfig,
) -> Result<Vec<ManifestRepository<'a>>> {
    let ProjectConfig {
        kind,
        metadata,
        spec,
        ..
    } = &loaded.config;
    match kind.as_deref().unwrap_or("CloudApp") {
        "CloudApp" => {
            let manifest_app_name = metadata
                .name
                .as_deref()
                .ok_or_else(|| anyhow!("CloudApp manifest is missing metadata.name"))?;
            Ok(vec![repository_entry(
                &loaded.path,
                manifest_app_name,
                spec.repository.as_ref(),
            )?])
        }
        "CloudApps" => spec
            .apps
            .iter()
            .map(|app| {
                let app_name = app.name.as_deref().ok_or_else(|| {
                    anyhow!(
                        "CloudApps entry in {} is missing name",
                        loaded.path.display()
                    )
                })?;
                repository_entry(&loaded.path, app_name, app.repository.as_ref())
            })
            .collect(),
        other => Err(anyhow!(
            "unsupported manifest kind '{other}' in {}",
            loaded.path.display()
        )),
    }
}

fn available_repos(repositories: &[ManifestRepository<'_>]) -> String {
    let mut names = repositories
        .iter()
        .map(|repository| repository.repo_name)
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    if names.is_empty() {
        "<none>".to_string()
    } else {
        names.join(", ")
    }
}

fn available_cloud_apps(repositories: &[ManifestRepository<'_>]) -> String {
    let mut names = repositories
        .iter()
        .map(|repository| repository.app_name)
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    if names.is_empty() {
        "<none>".to_string()
    } else {
        names.join(", ")
    }
}

fn resolve_repo_cwd_from_loaded(
    loaded: &LoadedProjectConfig,
    repo_name: &str,
) -> Result<ResolvedToolJobCwd> {
    let repositories = collect_manifest_repositories(loaded)?;
    let matches = repositories
        .iter()
        .filter(|repository| repository.repo_name == repo_name)
        .collect::<Vec<_>>();
    match matches.len() {
        0 => Err(anyhow!(
            "unknown repo '{repo_name}' in {}. Available repos: {}",
            loaded.path.display(),
            available_repos(&repositories)
        )),
        1 => Ok(ResolvedToolJobCwd {
            cwd: resolve_manifest_path(&loaded.path, matches[0].local_path)
                .display()
                .to_string(),
        }),
        _ => {
            let mut paths = matches
                .iter()
                .map(|repository| {
                    resolve_manifest_path(&loaded.path, repository.local_path)
                        .display()
                        .to_string()
                })
                .collect::<Vec<_>>();
            paths.sort();
            paths.dedup();
            if paths.len() == 1 {
                Ok(ResolvedToolJobCwd {
                    cwd: paths[0].clone(),
                })
            } else {
                Err(anyhow!(
                    "multiple repositories named '{repo_name}' in {} have different local paths: {}",
                    loaded.path.display(),
                    paths.join(", ")
                ))
            }
        }
    }
}

fn resolve_cloud_app_cwd_from_loaded(
    loaded: &LoadedProjectConfig,
    app_name: &str,
) -> Result<ResolvedToolJobCwd> {
    let repositories = collect_manifest_repositories(loaded)?;
    let matches = repositories
        .iter()
        .filter(|repository| repository.app_name == app_name)
        .collect::<Vec<_>>();
    match matches.len() {
        0 => Err(anyhow!(
            "unknown cloud app '{app_name}' in {}. Available cloud apps: {}",
            loaded.path.display(),
            available_cloud_apps(&repositories)
        )),
        1 => resolve_repo_cwd_from_loaded(loaded, matches[0].repo_name),
        _ => Err(anyhow!(
            "multiple cloud apps named '{app_name}' in {}",
            loaded.path.display()
        )),
    }
}

fn resolve_tool_job_cwd(
    config_flag: Option<&Path>,
    cwd: Option<&str>,
    repo: Option<&str>,
    cloud_app: Option<&str>,
) -> Result<Option<String>> {
    let specified = [cwd.is_some(), repo.is_some(), cloud_app.is_some()]
        .into_iter()
        .filter(|specified| *specified)
        .count();
    if specified > 1 {
        return Err(anyhow!(
            "--cwd, --repo, and --cloud-app are mutually exclusive"
        ));
    }

    if let Some(cwd) = cwd {
        return Ok(Some(cwd.to_string()));
    }

    match (repo, cloud_app) {
        (None, None) => Ok(None),
        (Some(repo), None) => {
            let loaded = loader::load_with_path(config_flag)?
                .ok_or_else(|| anyhow!("tachyon.yml not found. Run `tachyon init` first."))?;
            Ok(Some(resolve_repo_cwd_from_loaded(&loaded, repo)?.cwd))
        }
        (None, Some(cloud_app)) => {
            let loaded = loader::load_with_path(config_flag)?
                .ok_or_else(|| anyhow!("tachyon.yml not found. Run `tachyon init` first."))?;
            Ok(Some(
                resolve_cloud_app_cwd_from_loaded(&loaded, cloud_app)?.cwd,
            ))
        }
        (Some(_), Some(_)) => unreachable!("mutual exclusion checked above"),
    }
}

fn is_terminal_tool_job_status(status: Option<&str>) -> bool {
    matches!(status, Some("succeeded" | "failed" | "cancelled"))
}

fn print_tool_job_created(job: &ToolJobResponse) {
    println!("Tool job created.");
    println!("Job ID:   {}", job.id);
    println!("Provider: {}", job.provider.as_deref().unwrap_or("-"));
    println!("Status:   {}", job.status.as_deref().unwrap_or("-"));
    if let Some(worker_id) = &job.assigned_worker_id {
        println!("Worker:   {worker_id}");
    }
    if let Some(session_id) = &job.session_id {
        println!("Session:  {session_id}");
    } else if let Some(session_id) = &job.resume_session_id {
        println!("Session:  {session_id}");
    }
}

fn print_tool_job_final(job: &ToolJobResponse) -> Result<()> {
    println!("Final status: {}", job.status.as_deref().unwrap_or("-"));
    if let Some(output) = &job.normalized_output {
        if let Some(text) = output.pointer("/body/text").and_then(Value::as_str) {
            println!("{text}");
        } else {
            println!("{}", serde_json::to_string_pretty(output)?);
        }
    }
    if let Some(error) = &job.error_message {
        println!("Error: {error}");
    }
    Ok(())
}

async fn wait_tool_job(
    api: &ApiClient,
    job_id: &str,
    timeout_seconds: u64,
    interval_seconds: u64,
) -> Result<ToolJobResponse> {
    let started_at = Instant::now();
    let timeout = Duration::from_secs(timeout_seconds);
    let interval = Duration::from_secs(interval_seconds.max(1));

    loop {
        let response: ToolJobCreatedResponse =
            api.get(&format!("/v1/agent/tool-jobs/{job_id}")).await?;
        let job = response.job;
        if is_terminal_tool_job_status(job.status.as_deref()) {
            return Ok(job);
        }
        if started_at.elapsed() >= timeout {
            return Err(anyhow!(
                "timed out waiting for tool job {job_id} after {timeout_seconds}s"
            ));
        }
        tokio::time::sleep(interval).await;
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_tool_jobs_create(
    api: &ApiClient,
    config_flag: Option<&Path>,
    provider: &str,
    prompt: &Option<String>,
    context_paths: &[String],
    cwd: Option<&str>,
    repo: Option<&str>,
    cloud_app: Option<&str>,
    environment: &[String],
    output_profile: Option<&str>,
    resume_session_id: Option<&str>,
    worker_id: Option<&str>,
    use_worktree: bool,
    auto_merge: bool,
    codex_mode: Option<&str>,
    wait: bool,
    wait_timeout_seconds: u64,
    wait_interval_seconds: u64,
    json: bool,
) -> Result<()> {
    let prompt = read_prompt(prompt)?;
    let cwd = resolve_tool_job_cwd(config_flag, cwd, repo, cloud_app)?;
    let context_paths = if context_paths.is_empty() {
        vec![".".to_string()]
    } else {
        context_paths.to_vec()
    };
    let worker_id = match worker_id {
        Some(worker_id) => Some(resolve::resolve_worker_id(api, worker_id).await?),
        None => None,
    };
    let request = ToolJobCreateRequest {
        provider: provider.to_string(),
        prompt,
        context_paths,
        output_profile: output_profile.map(ToString::to_string),
        environment: parse_environment(environment)?,
        metadata: build_tool_job_metadata(provider, cwd.as_deref(), codex_mode)?,
        resume_session_id: resume_session_id.map(ToString::to_string),
        use_worktree,
        auto_merge,
        worker_id,
    };

    let created: ToolJobCreatedResponse = api.post("/v1/agent/tool-jobs", &request).await?;
    if !wait {
        if json {
            return print_json(&created);
        }
        print_tool_job_created(&created.job);
        return Ok(());
    }

    let final_job = wait_tool_job(
        api,
        &created.job.id,
        wait_timeout_seconds,
        wait_interval_seconds,
    )
    .await?;
    let response = ToolJobCreatedResponse { job: final_job };
    if json {
        return print_json(&response);
    }
    print_tool_job_created(&created.job);
    print_tool_job_final(&response.job)
}

async fn run_tool_jobs_list(api: &ApiClient, json: bool) -> Result<()> {
    let response: ToolJobListResponse = api.get("/v1/agent/tool-jobs").await?;
    if json {
        return print_json(&response);
    }
    let jobs = response.jobs;
    if jobs.is_empty() {
        println!("No tool jobs found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<16}  {:<12}  {:<20}  CREATED AT",
        "JOB ID", "PROVIDER", "STATUS", "TOOL"
    );
    println!(
        "{:-<28}  {:-<16}  {:-<12}  {:-<20}  {:-<19}",
        "", "", "", "", ""
    );
    for j in &jobs {
        println!(
            "{:<28}  {:<16}  {:<12}  {:<20}  {}",
            j.id,
            j.provider.as_deref().unwrap_or("-"),
            j.status.as_deref().unwrap_or("-"),
            truncate(j.tool_name.as_deref().unwrap_or("-"), 20),
            j.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_tool_jobs_get(api: &ApiClient, job_id: &str, json: bool) -> Result<()> {
    let j: ToolJobCreatedResponse = api.get(&format!("/v1/agent/tool-jobs/{job_id}")).await?;
    if json {
        return print_json(&j);
    }
    println!("{}", serde_json::to_string_pretty(&j)?);
    Ok(())
}

async fn run_tool_jobs_cancel(api: &ApiClient, job_id: &str) -> Result<()> {
    api.post_no_body(&format!("/v1/agent/tool-jobs/{job_id}/cancel"))
        .await?;
    println!("Tool job {job_id} cancelled.");
    Ok(())
}

async fn run_tool_jobs_providers(api: &ApiClient, json: bool) -> Result<()> {
    let response: ToolJobProvidersResponse = api.get("/v1/agent/tool-jobs/providers").await?;
    if json {
        return print_json(&response);
    }
    let providers = response.providers;
    if providers.is_empty() {
        println!("No providers found.");
        return Ok(());
    }
    for provider in &providers {
        println!("{provider}");
    }
    Ok(())
}

async fn run_notify_send(
    api: &ApiClient,
    text: &str,
    mentions: &[String],
    bot_token: Option<&str>,
    json: bool,
) -> Result<()> {
    let mentions = resolve_mentions(mentions, bot_token).await?;
    let response: SendNotificationResponse = api
        .post(
            "/v1/chat/send",
            &SendNotificationRequest {
                text: text.to_string(),
                mentions,
            },
        )
        .await?;
    if json {
        return print_json(&response);
    }
    if response.accepted {
        println!("Notification accepted.");
    } else {
        println!("Notification was not accepted.");
    }
    Ok(())
}

async fn run_notify_users(bot_token: Option<&str>, json: bool) -> Result<()> {
    let token = resolve_slack_bot_token(bot_token)?;
    let response = slack_users_list(&token).await?;
    if json {
        return print_json(&response);
    }

    let users = response
        .members
        .iter()
        .filter(|user| !user.deleted && !user.is_bot)
        .collect::<Vec<_>>();
    if users.is_empty() {
        println!("No Slack users found.");
        return Ok(());
    }

    println!("{:<14}  {:<36}  DISPLAY NAME", "USER ID", "EMAIL");
    println!("{:-<14}  {:-<36}  {:-<24}", "", "", "");
    for user in users {
        println!(
            "{:<14}  {:<36}  {}",
            user.id,
            truncate(user.profile.email.as_deref().unwrap_or("-"), 36),
            slack_user_display_name(user),
        );
    }
    Ok(())
}

async fn resolve_mentions(mentions: &[String], bot_token: Option<&str>) -> Result<Vec<String>> {
    let mut resolved = Vec::with_capacity(mentions.len());
    for mention in mentions {
        let mention = mention.trim();
        if mention.is_empty() {
            continue;
        }
        if mention.contains('@') && !mention.starts_with("<@") {
            let token = resolve_slack_bot_token(bot_token)?;
            resolved.push(slack_lookup_user_id_by_email(&token, mention).await?);
        } else {
            resolved.push(normalize_slack_user_id(mention).to_string());
        }
    }
    Ok(resolved)
}

fn resolve_slack_bot_token(explicit: Option<&str>) -> Result<String> {
    if let Some(token) = explicit.map(str::trim).filter(|token| !token.is_empty()) {
        return Ok(token.to_string());
    }
    ["TACHYON_SLACK_BOT_TOKEN", "SLACK_BOT_TOKEN"]
        .iter()
        .find_map(|name| {
            std::env::var(name)
                .ok()
                .map(|token| token.trim().to_string())
                .filter(|token| !token.is_empty())
        })
        .ok_or_else(|| {
            anyhow!(
                "Slack Bot token is required. Pass --bot-token or set TACHYON_SLACK_BOT_TOKEN or SLACK_BOT_TOKEN."
            )
        })
}

async fn slack_users_list(bot_token: &str) -> Result<SlackUsersListResponse> {
    let client = reqwest::Client::new();
    let mut members = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let mut request = client
            .get("https://slack.com/api/users.list")
            .bearer_auth(bot_token)
            .query(&[("limit", "200")]);
        if let Some(cursor) = cursor.as_deref().filter(|value| !value.is_empty()) {
            request = request.query(&[("cursor", cursor)]);
        }

        let response = request.send().await.context("GET Slack users.list")?;
        let status = response.status();
        if !status.is_success() {
            bail!("Slack users.list returned HTTP {status}");
        }
        let mut body = response
            .json::<SlackUsersListResponse>()
            .await
            .context("parse Slack users.list response")?;
        if !body.ok {
            bail!(
                "Slack users.list failed: {}",
                body.error
                    .clone()
                    .unwrap_or_else(|| "unknown_error".to_string())
            );
        }

        members.append(&mut body.members);
        cursor = body
            .response_metadata
            .and_then(|metadata| metadata.next_cursor)
            .filter(|value| !value.is_empty());
        if cursor.is_none() {
            break;
        }
    }

    Ok(SlackUsersListResponse {
        ok: true,
        members,
        error: None,
        response_metadata: None,
    })
}

async fn slack_lookup_user_id_by_email(bot_token: &str, email: &str) -> Result<String> {
    let response = reqwest::Client::new()
        .get("https://slack.com/api/users.lookupByEmail")
        .bearer_auth(bot_token)
        .query(&[("email", email)])
        .send()
        .await
        .with_context(|| format!("GET Slack users.lookupByEmail for {email}"))?;
    let status = response.status();
    if !status.is_success() {
        bail!("Slack users.lookupByEmail returned HTTP {status}");
    }
    let body = response
        .json::<SlackLookupByEmailResponse>()
        .await
        .context("parse Slack users.lookupByEmail response")?;
    if !body.ok {
        bail!(
            "Slack users.lookupByEmail failed for {email}: {}",
            body.error.unwrap_or_else(|| "unknown_error".to_string())
        );
    }
    body.user
        .map(|user| user.id)
        .ok_or_else(|| anyhow!("Slack users.lookupByEmail omitted user for {email}"))
}

fn normalize_slack_user_id(input: &str) -> &str {
    let trimmed = input.trim();
    if let Some(value) = trimmed
        .strip_prefix("<@")
        .and_then(|value| value.strip_suffix('>'))
    {
        return value
            .split_once('|')
            .map(|(id, _)| id)
            .unwrap_or(value)
            .trim();
    }
    trimmed
}

fn slack_user_display_name(user: &SlackUser) -> &str {
    user.profile
        .display_name
        .as_deref()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            user.profile
                .real_name
                .as_deref()
                .filter(|value| !value.is_empty())
        })
        .or(user.name.as_deref())
        .unwrap_or("-")
}

// ---- Entry point ----

pub async fn run(
    args: &OpsArgs,
    config: &Configuration,
    tenant_id: &str,
    config_flag: Option<&Path>,
) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    match &args.command {
        OpsCommand::Deployments { command } => match command {
            OpsDeploymentsCommand::List { json } => run_deployments_list(&api, *json).await,
            OpsDeploymentsCommand::Get { id, json } => run_deployments_get(&api, id, *json).await,
        },
        OpsCommand::Reports { command } => match command {
            ReportsCommand::List { json } => run_reports_list(&api, *json).await,
            ReportsCommand::Get { run_id, json } => run_reports_get(&api, run_id, *json).await,
        },
        OpsCommand::ToolJobs { command } => match command {
            ToolJobsCommand::Create {
                provider,
                prompt,
                context_paths,
                cwd,
                repo,
                cloud_app,
                environment,
                output_profile,
                resume_session_id,
                worker_id,
                use_worktree,
                auto_merge,
                codex_mode,
                wait,
                wait_timeout_seconds,
                wait_interval_seconds,
                json,
            } => {
                run_tool_jobs_create(
                    &api,
                    config_flag,
                    provider,
                    prompt,
                    context_paths,
                    cwd.as_deref(),
                    repo.as_deref(),
                    cloud_app.as_deref(),
                    environment,
                    output_profile.as_deref(),
                    resume_session_id.as_deref(),
                    worker_id.as_deref(),
                    *use_worktree,
                    *auto_merge,
                    codex_mode.as_deref(),
                    *wait,
                    *wait_timeout_seconds,
                    *wait_interval_seconds,
                    *json,
                )
                .await
            }
            ToolJobsCommand::List { json } => run_tool_jobs_list(&api, *json).await,
            ToolJobsCommand::Get { job_id, json } => run_tool_jobs_get(&api, job_id, *json).await,
            ToolJobsCommand::Cancel { job_id } => run_tool_jobs_cancel(&api, job_id).await,
            ToolJobsCommand::Providers { json } => run_tool_jobs_providers(&api, *json).await,
        },
        OpsCommand::Notify { command } => match command {
            NotifyCommand::Send {
                text,
                mentions,
                bot_token,
                json,
            } => run_notify_send(&api, text, mentions, bot_token.as_deref(), *json).await,
            NotifyCommand::Users { bot_token, json } => {
                run_notify_users(bot_token.as_deref(), *json).await
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_notification_mentions() {
        let request = SendNotificationRequest {
            text: "deploy complete".to_string(),
            mentions: vec!["U123".to_string(), "U456".to_string()],
        };

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({
                "text": "deploy complete",
                "mentions": ["U123", "U456"],
            })
        );
    }

    #[test]
    fn omits_empty_notification_mentions() {
        let request = SendNotificationRequest {
            text: "deploy complete".to_string(),
            mentions: Vec::new(),
        };

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({ "text": "deploy complete" })
        );
    }

    #[test]
    fn normalizes_slack_user_mentions() {
        assert_eq!(normalize_slack_user_id(" U123 "), "U123");
        assert_eq!(normalize_slack_user_id("<@U123>"), "U123");
        assert_eq!(normalize_slack_user_id("<@U123|taka>"), "U123");
    }

    #[test]
    fn display_name_prefers_display_then_real_then_name() {
        let user = SlackUser {
            id: "U123".to_string(),
            name: Some("fallback".to_string()),
            deleted: false,
            is_bot: false,
            profile: SlackUserProfile {
                email: Some("user@example.com".to_string()),
                display_name: Some("display".to_string()),
                real_name: Some("real".to_string()),
            },
        };

        assert_eq!(slack_user_display_name(&user), "display");
    }
}
