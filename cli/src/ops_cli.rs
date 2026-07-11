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
    /// Run coding-agent jobs that push a branch and open a PR
    CodingJobs {
        #[command(subcommand)]
        command: CodingJobsCommand,
    },
    /// Send chat notifications to configured Slack/Discord destinations
    #[command(visible_alias = "slack")]
    Notify {
        #[command(subcommand)]
        command: NotifyCommand,
    },
    /// Inspect and manage Sentry issues
    Sentry {
        #[command(subcommand)]
        command: SentryCommand,
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
pub enum CodingJobsCommand {
    /// Run a coding job and optionally wait for the PR URL.
    Run {
        /// Git repository URL to clone.
        #[arg(long = "repo")]
        repository_url: String,
        /// Base branch to clone and target for the PR.
        #[arg(long, default_value = "main")]
        base: String,
        /// Branch name the agent will push.
        #[arg(long)]
        branch: String,
        /// Prompt to send. If omitted, stdin is read.
        #[arg(long)]
        prompt: Option<String>,
        /// PR title. Defaults to the pushed branch.
        #[arg(long = "title")]
        pull_request_title: Option<String>,
        /// PR body.
        #[arg(long = "body")]
        pull_request_body: Option<String>,
        /// Git commit message. Defaults to the PR title or branch.
        #[arg(long = "commit-message")]
        git_commit_message: Option<String>,
        /// Secret selector for the GitHub token, formatted as NAME:KEY.
        #[arg(long = "github-token-secret")]
        github_token_secret: String,
        /// Optional Codex access token secret selector, formatted as NAME:KEY.
        #[arg(long = "codex-access-token-secret")]
        codex_access_token_secret: Option<String>,
        /// Optional OpenAI API key secret selector, formatted as NAME:KEY.
        #[arg(long = "openai-api-key-secret")]
        openai_api_key_secret: Option<String>,
        /// Codex model/output profile.
        #[arg(long)]
        model: Option<String>,
        /// Container image to run.
        #[arg(long)]
        image: Option<String>,
        /// Wait until the underlying tool job reaches a terminal status.
        #[arg(long)]
        wait: bool,
        /// Maximum wait time in seconds.
        #[arg(long, default_value_t = 1800)]
        wait_timeout_seconds: u64,
        /// Poll interval in seconds while waiting.
        #[arg(long, default_value_t = 5)]
        wait_interval_seconds: u64,
        /// Print the API response as JSON.
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
        /// Slack mention, raw Slack token, or friendly alias. Can be specified multiple times.
        #[arg(long = "mention")]
        mentions: Vec<String>,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
    /// List Slack users through the tenant's saved Slack connection
    Users {
        /// Deprecated and ignored; the server uses the tenant connection
        #[arg(long = "bot-token")]
        bot_token: Option<String>,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum SentryCommand {
    /// Inspect and manage Sentry issues
    #[command(visible_alias = "issue")]
    Issues {
        #[command(subcommand)]
        command: SentryIssuesCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum SentryIssuesCommand {
    /// List Sentry issues
    List {
        /// Sentry project slug or id
        #[arg(long)]
        project: Option<String>,
        /// Sentry search query
        #[arg(long)]
        query: Option<String>,
        /// Maximum number of issues to return
        #[arg(long)]
        limit: Option<u32>,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get Sentry issue details
    View {
        issue_id: String,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
    /// Mark a Sentry issue as resolved
    Resolve {
        issue_id: String,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
    /// Assign a Sentry issue to a user
    Assign {
        issue_id: String,
        /// Sentry user id, username, or email
        user: String,
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

#[derive(Debug, Deserialize, Serialize)]
struct CloudCodingJobCreatedResponse {
    tool_job_id: String,
    job_run_id: String,
    worker_id: String,
    provider: String,
    execution_backend: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct CloudCodingJobCreateRequest {
    prompt: String,
    repository_url: String,
    branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    github_token_secret: SecretKeySelector,
    #[serde(skip_serializing_if = "Option::is_none")]
    codex_access_token_secret: Option<SecretKeySelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    openai_api_key_secret: Option<SecretKeySelector>,
    git_push_branch: String,
    git_commit_message: String,
    metadata: Value,
}

#[derive(Debug, Clone, Serialize)]
struct SecretKeySelector {
    name: String,
    key: String,
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
struct ListSlackUsersResponse {
    users: Vec<SlackUser>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SlackUser {
    id: String,
    name: Option<String>,
    email: Option<String>,
    display_name: Option<String>,
    real_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum SentryIssueListResponse {
    Wrapped {
        #[serde(default)]
        issues: Vec<SentryIssueResponse>,
    },
    Bare(Vec<SentryIssueResponse>),
}

#[derive(Debug, Deserialize, Serialize)]
struct SentryIssueResponse {
    id: String,
    #[serde(default, alias = "shortId")]
    short_id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    culprit: Option<String>,
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    count: Option<Value>,
    #[serde(default, alias = "userCount")]
    user_count: Option<Value>,
    #[serde(default, alias = "firstSeen")]
    first_seen: Option<String>,
    #[serde(default, alias = "lastSeen")]
    last_seen: Option<String>,
    #[serde(default)]
    permalink: Option<String>,
    #[serde(default, alias = "assignedTo")]
    assigned_to: Option<SentryAssignedTo>,
    #[serde(default, alias = "latestEvent")]
    latest_event: Option<SentryEventResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SentryAssignedTo {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    username: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SentryEventResponse {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    event_id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
}

#[derive(Debug, Serialize)]
struct SentryAssignRequest {
    user: String,
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

fn parse_secret_selector(value: &str) -> Result<SecretKeySelector> {
    let (name, key) = value
        .split_once(':')
        .ok_or_else(|| anyhow!("secret selector must be formatted as NAME:KEY"))?;
    if name.trim().is_empty() || key.trim().is_empty() {
        bail!("secret selector must include non-empty NAME and KEY");
    }
    Ok(SecretKeySelector {
        name: name.to_string(),
        key: key.to_string(),
    })
}

fn extract_pull_request_url(output: &Value) -> Option<String> {
    for pointer in [
        "/pull_request/url",
        "/pullRequest/url",
        "/pr_url",
        "/prUrl",
        "/html_url",
        "/body/pull_request/url",
        "/body/pr_url",
    ] {
        if let Some(value) = output.pointer(pointer).and_then(Value::as_str) {
            if is_github_pull_request_url(value) {
                return Some(value.to_string());
            }
        }
    }
    output
        .pointer("/body/text")
        .and_then(Value::as_str)
        .and_then(find_github_pull_request_url)
}

fn find_github_pull_request_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|part| {
            part.trim_matches(|c: char| matches!(c, '"' | '\'' | ')' | ']' | '}' | ',' | '.'))
        })
        .find(|part| is_github_pull_request_url(part))
        .map(ToString::to_string)
}

fn is_github_pull_request_url(value: &str) -> bool {
    value.starts_with("https://github.com/") && value.contains("/pull/")
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
async fn run_coding_jobs_run(
    api: &ApiClient,
    repository_url: &str,
    base: &str,
    branch: &str,
    prompt: &Option<String>,
    pull_request_title: Option<&str>,
    pull_request_body: Option<&str>,
    git_commit_message: Option<&str>,
    github_token_secret: &str,
    codex_access_token_secret: Option<&str>,
    openai_api_key_secret: Option<&str>,
    model: Option<&str>,
    image: Option<&str>,
    wait: bool,
    wait_timeout_seconds: u64,
    wait_interval_seconds: u64,
    json: bool,
) -> Result<()> {
    let prompt = read_prompt(prompt)?;
    let title = pull_request_title.unwrap_or(branch);
    let request = CloudCodingJobCreateRequest {
        prompt,
        repository_url: repository_url.to_string(),
        branch: base.to_string(),
        output_profile: model.map(ToString::to_string),
        model: model.map(ToString::to_string),
        image: image.map(ToString::to_string),
        github_token_secret: parse_secret_selector(github_token_secret)?,
        codex_access_token_secret: codex_access_token_secret
            .map(parse_secret_selector)
            .transpose()?,
        openai_api_key_secret: openai_api_key_secret
            .map(parse_secret_selector)
            .transpose()?,
        git_push_branch: branch.to_string(),
        git_commit_message: git_commit_message.unwrap_or(title).to_string(),
        metadata: json!({
            "source": "tachyon-cli",
            "cloud_coding": {
                "repository_url": repository_url,
                "base": base,
                "branch": branch,
                "pull_request": {
                    "base": base,
                    "head": branch,
                    "title": title,
                    "body": pull_request_body,
                },
            },
        }),
    };

    let created: CloudCodingJobCreatedResponse =
        api.post("/v1/agent/cloud-coding-jobs", &request).await?;
    if !wait {
        if json {
            return print_json(&created);
        }
        println!("Coding job created.");
        println!("Tool job ID: {}", created.tool_job_id);
        println!("JobRun ID:   {}", created.job_run_id);
        println!("Status:      {}", created.status);
        return Ok(());
    }

    let final_job = wait_tool_job(
        api,
        &created.tool_job_id,
        wait_timeout_seconds,
        wait_interval_seconds,
    )
    .await?;
    if json {
        return print_json(&json!({
            "cloud_coding_job": created,
            "tool_job": final_job,
        }));
    }
    println!("Coding job created.");
    println!("Tool job ID: {}", created.tool_job_id);
    println!("JobRun ID:   {}", created.job_run_id);
    print_tool_job_final(&final_job)?;
    if let Some(output) = final_job.normalized_output.as_ref() {
        if let Some(url) = extract_pull_request_url(output) {
            println!("Pull request: {url}");
        }
    }
    Ok(())
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
    json: bool,
) -> Result<()> {
    let mentions = normalize_mentions(mentions)?;
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

async fn run_notify_users(api: &ApiClient, json: bool) -> Result<()> {
    let response: ListSlackUsersResponse = api.get("/v1/chat/users").await?;
    if json {
        return print_json(&response);
    }

    if response.users.is_empty() {
        println!("No Slack users found.");
        return Ok(());
    }

    println!("{:<14}  {:<36}  DISPLAY NAME", "USER ID", "EMAIL");
    println!("{:-<14}  {:-<36}  {:-<24}", "", "", "");
    for user in &response.users {
        println!(
            "{:<14}  {:<36}  {}",
            user.id,
            truncate(user.email.as_deref().unwrap_or("-"), 36),
            slack_user_display_name(user),
        );
    }
    Ok(())
}

async fn run_sentry_issues_list(
    api: &ApiClient,
    project: Option<&str>,
    query: Option<&str>,
    limit: Option<u32>,
    json: bool,
) -> Result<()> {
    let mut params = Vec::new();
    if let Some(project) = project.filter(|value| !value.trim().is_empty()) {
        params.push(("project", project));
    }
    if let Some(query) = query.filter(|value| !value.trim().is_empty()) {
        params.push(("query", query));
    }
    let limit_value;
    if let Some(limit) = limit {
        limit_value = limit.to_string();
        params.push(("limit", limit_value.as_str()));
    }

    let response: SentryIssueListResponse = api.get_query("/v1/ops/sentry/issues", &params).await?;
    if json {
        return print_json(&response);
    }
    let issues = sentry_issue_list_items(response);
    if issues.is_empty() {
        println!("No Sentry issues found.");
        return Ok(());
    }

    println!(
        "{:<14}  {:<10}  {:<10}  {:<8}  {:<18}  TITLE",
        "ISSUE", "PROJECT", "STATUS", "COUNT", "LAST SEEN"
    );
    println!(
        "{:-<14}  {:-<10}  {:-<10}  {:-<8}  {:-<18}  {:-<60}",
        "", "", "", "", "", ""
    );
    for issue in &issues {
        println!(
            "{:<14}  {:<10}  {:<10}  {:<8}  {:<18}  {}",
            sentry_issue_label(issue),
            truncate(issue.project.as_deref().unwrap_or("-"), 10),
            issue.status.as_deref().unwrap_or("-"),
            sentry_value_label(issue.count.as_ref()),
            issue.last_seen.as_deref().unwrap_or("-"),
            truncate(issue.title.as_deref().unwrap_or("-"), 60),
        );
    }
    Ok(())
}

async fn run_sentry_issue_view(api: &ApiClient, issue_id: &str, json: bool) -> Result<()> {
    let issue: SentryIssueResponse = api
        .get(&format!("/v1/ops/sentry/issues/{issue_id}"))
        .await?;
    if json {
        return print_json(&issue);
    }
    print_sentry_issue(&issue);
    Ok(())
}

async fn run_sentry_issue_resolve(api: &ApiClient, issue_id: &str, json: bool) -> Result<()> {
    let issue: SentryIssueResponse = api
        .post(
            &format!("/v1/ops/sentry/issues/{issue_id}/resolve"),
            &json!({}),
        )
        .await?;
    if json {
        return print_json(&issue);
    }
    println!("Sentry issue {} resolved.", sentry_issue_label(&issue));
    Ok(())
}

async fn run_sentry_issue_assign(
    api: &ApiClient,
    issue_id: &str,
    user: &str,
    json: bool,
) -> Result<()> {
    let request = SentryAssignRequest {
        user: user.to_string(),
    };
    let issue: SentryIssueResponse = api
        .post(
            &format!("/v1/ops/sentry/issues/{issue_id}/assign"),
            &request,
        )
        .await?;
    if json {
        return print_json(&issue);
    }
    println!(
        "Sentry issue {} assigned to {user}.",
        sentry_issue_label(&issue)
    );
    Ok(())
}

fn sentry_issue_label(issue: &SentryIssueResponse) -> &str {
    issue.short_id.as_deref().unwrap_or(&issue.id)
}

fn sentry_issue_list_items(response: SentryIssueListResponse) -> Vec<SentryIssueResponse> {
    match response {
        SentryIssueListResponse::Wrapped { issues } => issues,
        SentryIssueListResponse::Bare(issues) => issues,
    }
}

fn sentry_value_label(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        Some(value) if value.is_null() => "-".to_string(),
        Some(value) => value.to_string(),
        None => "-".to_string(),
    }
}

fn sentry_assignee_label(assigned_to: Option<&SentryAssignedTo>) -> &str {
    assigned_to
        .and_then(|assignee| {
            assignee
                .name
                .as_deref()
                .filter(|value| !value.is_empty())
                .or_else(|| assignee.email.as_deref().filter(|value| !value.is_empty()))
                .or_else(|| {
                    assignee
                        .username
                        .as_deref()
                        .filter(|value| !value.is_empty())
                })
                .or_else(|| assignee.id.as_deref().filter(|value| !value.is_empty()))
        })
        .unwrap_or("-")
}

fn sentry_latest_event_label(latest_event: Option<&SentryEventResponse>) -> String {
    let Some(event) = latest_event else {
        return "-".to_string();
    };
    let id = event
        .event_id
        .as_deref()
        .or(event.id.as_deref())
        .unwrap_or("-");
    let title = event
        .title
        .as_deref()
        .or(event.message.as_deref())
        .unwrap_or("-");
    let timestamp = event.timestamp.as_deref().unwrap_or("-");
    format!("{id}  {timestamp}  {title}")
}

fn print_sentry_issue(issue: &SentryIssueResponse) {
    println!("Issue:        {}", sentry_issue_label(issue));
    println!("ID:           {}", issue.id);
    println!("Title:        {}", issue.title.as_deref().unwrap_or("-"));
    println!("Culprit:      {}", issue.culprit.as_deref().unwrap_or("-"));
    println!("Project:      {}", issue.project.as_deref().unwrap_or("-"));
    println!("Status:       {}", issue.status.as_deref().unwrap_or("-"));
    println!("Count:        {}", sentry_value_label(issue.count.as_ref()));
    println!(
        "Users:        {}",
        sentry_value_label(issue.user_count.as_ref())
    );
    println!(
        "Assigned To:  {}",
        sentry_assignee_label(issue.assigned_to.as_ref())
    );
    println!(
        "First Seen:   {}",
        issue.first_seen.as_deref().unwrap_or("-")
    );
    println!(
        "Last Seen:    {}",
        issue.last_seen.as_deref().unwrap_or("-")
    );
    println!(
        "Latest Event: {}",
        sentry_latest_event_label(issue.latest_event.as_ref())
    );
    println!(
        "Permalink:    {}",
        issue.permalink.as_deref().unwrap_or("-")
    );
}

fn normalize_mentions(mentions: &[String]) -> Result<Vec<String>> {
    let mut resolved = Vec::with_capacity(mentions.len());
    let mut seen = std::collections::HashSet::new();

    for mention in mentions {
        let mention = normalize_mention(mention)?;
        if seen.insert(mention.clone()) {
            resolved.push(mention);
        }
    }

    Ok(resolved)
}

fn normalize_mention(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        bail!("Slack mention cannot be empty");
    }
    if trimmed.chars().any(char::is_control) {
        bail!("Slack mention contains control characters");
    }

    if let Some(token) = normalize_broadcast_mention(trimmed)? {
        return Ok(token);
    }
    if let Some(token) = normalize_user_mention(trimmed)? {
        return Ok(token);
    }
    if let Some(token) = normalize_user_group_mention(trimmed)? {
        return Ok(token);
    }
    if trimmed.starts_with('<') || trimmed.ends_with('>') {
        bail!("Malformed Slack mention: {trimmed}");
    }

    Ok(trimmed.to_string())
}

fn normalize_broadcast_mention(input: &str) -> Result<Option<String>> {
    match input.to_ascii_lowercase().as_str() {
        "here" | "@here" | "<!here>" => Ok(Some("<!here>".to_string())),
        "channel" | "@channel" | "<!channel>" => Ok(Some("<!channel>".to_string())),
        _ => Ok(None),
    }
}

fn normalize_user_mention(input: &str) -> Result<Option<String>> {
    if let Some(value) = input
        .strip_prefix("<@")
        .and_then(|value| value.strip_suffix('>'))
    {
        let user_id = value
            .split_once('|')
            .map(|(id, _)| id)
            .unwrap_or(value)
            .trim();
        if !(user_id.starts_with('U') || user_id.starts_with('W'))
            || !user_id.chars().all(|c| c.is_ascii_alphanumeric())
        {
            bail!("Slack user mention must use a Slack user ID");
        }
        return Ok(Some(format!("<@{user_id}>")));
    }

    if input.chars().all(|c| c.is_ascii_alphanumeric())
        && (input.starts_with('U') || input.starts_with('W'))
    {
        return Ok(Some(format!("<@{input}>")));
    }

    Ok(None)
}

fn normalize_user_group_mention(input: &str) -> Result<Option<String>> {
    if let Some(value) = input
        .strip_prefix("<!subteam^")
        .and_then(|value| value.strip_suffix('>'))
    {
        let group_id = value
            .split_once('|')
            .map(|(id, _)| id)
            .unwrap_or(value)
            .trim();
        if !group_id.starts_with('S') || !group_id.chars().all(|c| c.is_ascii_alphanumeric()) {
            bail!("Slack user group mention must use a Slack user group ID");
        }
        return Ok(Some(format!("<!subteam^{group_id}>")));
    }

    if input.chars().all(|c| c.is_ascii_alphanumeric()) && input.starts_with('S') {
        return Ok(Some(format!("<!subteam^{input}>")));
    }

    Ok(None)
}

fn slack_user_display_name(user: &SlackUser) -> &str {
    user.display_name
        .as_deref()
        .filter(|value| !value.is_empty())
        .or_else(|| user.real_name.as_deref().filter(|value| !value.is_empty()))
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
        OpsCommand::CodingJobs { command } => match command {
            CodingJobsCommand::Run {
                repository_url,
                base,
                branch,
                prompt,
                pull_request_title,
                pull_request_body,
                git_commit_message,
                github_token_secret,
                codex_access_token_secret,
                openai_api_key_secret,
                model,
                image,
                wait,
                wait_timeout_seconds,
                wait_interval_seconds,
                json,
            } => {
                run_coding_jobs_run(
                    &api,
                    repository_url,
                    base,
                    branch,
                    prompt,
                    pull_request_title.as_deref(),
                    pull_request_body.as_deref(),
                    git_commit_message.as_deref(),
                    github_token_secret,
                    codex_access_token_secret.as_deref(),
                    openai_api_key_secret.as_deref(),
                    model.as_deref(),
                    image.as_deref(),
                    *wait,
                    *wait_timeout_seconds,
                    *wait_interval_seconds,
                    *json,
                )
                .await
            }
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
                json,
            } => run_notify_send(&api, text, mentions, *json).await,
            NotifyCommand::Users { bot_token: _, json } => run_notify_users(&api, *json).await,
        },
        OpsCommand::Sentry { command } => match command {
            SentryCommand::Issues { command } => match command {
                SentryIssuesCommand::List {
                    project,
                    query,
                    limit,
                    json,
                } => {
                    run_sentry_issues_list(
                        &api,
                        project.as_deref(),
                        query.as_deref(),
                        *limit,
                        *json,
                    )
                    .await
                }
                SentryIssuesCommand::View { issue_id, json } => {
                    run_sentry_issue_view(&api, issue_id, *json).await
                }
                SentryIssuesCommand::Resolve { issue_id, json } => {
                    run_sentry_issue_resolve(&api, issue_id, *json).await
                }
                SentryIssuesCommand::Assign {
                    issue_id,
                    user,
                    json,
                } => run_sentry_issue_assign(&api, issue_id, user, *json).await,
            },
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
    fn normalizes_slack_broadcast_mentions_without_bot_token() {
        let mentions = normalize_mentions(&["here".to_string(), "channel".to_string()]).unwrap();

        assert_eq!(mentions, vec!["<!here>", "<!channel>"]);
    }

    #[test]
    fn normalizes_slack_mentions_and_dedupes() {
        let mentions = normalize_mentions(&[
            "CEO".to_string(),
            "<@U123|taka>".to_string(),
            "U123".to_string(),
            "here".to_string(),
        ])
        .unwrap();

        assert_eq!(mentions, vec!["CEO", "<@U123>", "<!here>"]);
    }

    #[test]
    fn rejects_malformed_slack_mentions() {
        let err = normalize_mentions(&["<@CEO>".to_string()]).unwrap_err();

        assert!(err
            .to_string()
            .contains("Slack user mention must use a Slack user ID"));
    }

    #[test]
    fn display_name_prefers_display_then_real_then_name() {
        let user = SlackUser {
            id: "U123".to_string(),
            name: Some("fallback".to_string()),
            email: Some("user@example.com".to_string()),
            display_name: Some("display".to_string()),
            real_name: Some("real".to_string()),
        };

        assert_eq!(slack_user_display_name(&user), "display");
    }

    #[test]
    fn deserializes_tenant_slack_users_response() {
        let response: ListSlackUsersResponse = serde_json::from_value(json!({
            "users": [{
                "id": "U123",
                "name": "taka",
                "email": "taka@example.com",
                "display_name": "Taka",
                "real_name": "Takanori Fukuyama"
            }]
        }))
        .unwrap();

        assert_eq!(response.users.len(), 1);
        assert_eq!(response.users[0].id, "U123");
        assert_eq!(response.users[0].email.as_deref(), Some("taka@example.com"));
    }

    #[test]
    fn deserializes_sentry_issues_from_wrapped_and_bare_responses() {
        let wrapped: SentryIssueListResponse = serde_json::from_value(json!({
            "issues": [{
                "id": "12345",
                "shortId": "FIELDADMIN-1",
                "title": "TypeError",
                "count": 42,
                "userCount": "3",
                "firstSeen": "2026-07-01T00:00:00Z",
                "lastSeen": "2026-07-02T00:00:00Z",
                "assignedTo": {"email": "user@example.com"},
                "latestEvent": {"event_id": "evt_1", "timestamp": "2026-07-02T00:00:00Z"}
            }]
        }))
        .unwrap();
        let wrapped = sentry_issue_list_items(wrapped);
        assert_eq!(wrapped[0].short_id.as_deref(), Some("FIELDADMIN-1"));
        assert_eq!(sentry_value_label(wrapped[0].count.as_ref()), "42");
        assert_eq!(sentry_value_label(wrapped[0].user_count.as_ref()), "3");
        assert_eq!(
            sentry_assignee_label(wrapped[0].assigned_to.as_ref()),
            "user@example.com"
        );

        let bare: SentryIssueListResponse =
            serde_json::from_value(json!([{ "id": "12345", "title": "TypeError" }])).unwrap();
        assert_eq!(sentry_issue_list_items(bare).len(), 1);
    }

    #[test]
    fn serializes_sentry_assign_request() {
        let request = SentryAssignRequest {
            user: "user@example.com".to_string(),
        };

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({ "user": "user@example.com" })
        );
    }
}
