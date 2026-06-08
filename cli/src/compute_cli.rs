use anyhow::{anyhow, Result};
use chrono::{DateTime, TimeZone, Utc};
use clap::{Args, Subcommand, ValueEnum};
use dialoguer::{theme::ColorfulTheme, Password};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use tachyon_sdk::apis::configuration::Configuration;
use tokio::time::{sleep, Duration};

use crate::build_reproduce;
use crate::client::{print_json, truncate, ApiClient};
use crate::config::loader::ProjectConfig;
use crate::resolve;

#[derive(Debug, Clone, Args)]
pub struct ComputeArgs {
    #[command(subcommand)]
    pub command: ComputeCommand,
}

#[derive(Debug, Clone, Args)]
pub struct EnvArgs {
    #[command(subcommand)]
    pub command: EnvCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ComputeCommand {
    /// List compute apps
    Apps {
        #[command(subcommand)]
        command: AppsCommand,
    },
    /// Manage builds
    Builds {
        #[command(subcommand)]
        command: BuildsCommand,
    },
    /// Manage deployments
    Deployments {
        #[command(subcommand)]
        command: DeploymentsCommand,
    },
    /// Manage environment variables
    Env {
        #[command(subcommand)]
        command: EnvCommand,
    },
    /// Manage custom domains
    Domains {
        #[command(subcommand)]
        command: DomainsCommand,
    },
    /// Manage scaling configuration
    Scaling {
        #[command(subcommand)]
        command: ScalingCommand,
    },
    /// Build a Cloudflare Pages app locally (emulates CodeBuild pipeline)
    Build {
        /// App to build (tachyon, cms, docs)
        app: PagesApp,
        /// Also deploy to Cloudflare Pages preview environment
        #[arg(long)]
        deploy: bool,
        /// Project root directory (defaults to current directory)
        #[arg(long)]
        project_dir: Option<PathBuf>,
    },
    /// Build and start local preview server (wrangler pages dev)
    Dev {
        /// App to preview (tachyon, cms, docs)
        app: PagesApp,
        /// Project root directory (defaults to current directory)
        #[arg(long)]
        project_dir: Option<PathBuf>,
        /// Port for the preview server
        #[arg(long, default_value_t = 8788)]
        port: u16,
    },
    /// Show build status for a compute app (shortcut for builds list)
    Status {
        /// App ID or name
        app_id: Option<String>,
        /// Maximum number of builds to display
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Stream or fetch build logs (shortcut for builds logs)
    Logs {
        /// App ID or name (optional when --build-id is specified)
        app_id: Option<String>,
        /// Tail runtime logs for a Cloudflare-backed app
        #[arg(long, value_name = "APP_ID")]
        tail: Option<String>,
        /// Build ID (defaults to the latest build for the given app)
        #[arg(long)]
        build_id: Option<String>,
        /// Keep polling until the build is complete;
        /// exits with code 1 if the build fails
        #[arg(long)]
        follow: bool,
        /// Emit compact JSON Lines for coding agents
        #[arg(long)]
        agent: bool,
        /// Emit raw runtime log JSON Lines when used with --tail
        #[arg(long)]
        json: bool,
    },
}

// --- Pages app target ---

#[derive(Debug, Clone, ValueEnum)]
pub enum PagesApp {
    /// Main Tachyon app (@cloudflare/next-on-pages)
    Tachyon,
    /// CMS app (@opennextjs/cloudflare)
    Cms,
    /// Documentation site (@cloudflare/next-on-pages)
    Docs,
}

impl PagesApp {
    fn name(&self) -> &str {
        match self {
            PagesApp::Tachyon => "tachyon",
            PagesApp::Cms => "cms",
            PagesApp::Docs => "docs",
        }
    }

    fn cf_project_name(&self) -> &str {
        match self {
            PagesApp::Tachyon => "tachyon-app",
            PagesApp::Cms => "tachyon-apps-cms-app",
            PagesApp::Docs => "tachyon-docs",
        }
    }

    /// Returns the pages build command and output directory.
    fn pages_build_info(&self) -> (&str, &str) {
        match self {
            PagesApp::Cms => ("npx opennextjs-cloudflare build", ".open-next/assets"),
            PagesApp::Tachyon | PagesApp::Docs => {
                ("npx @cloudflare/next-on-pages", ".vercel/output/static")
            }
        }
    }

    fn preview_command(&self, port: u16) -> String {
        match self {
            PagesApp::Cms => {
                format!("npx opennextjs-cloudflare preview --port {port}")
            }
            PagesApp::Tachyon | PagesApp::Docs => {
                format!("npx wrangler pages dev .vercel/output/static --port {port}")
            }
        }
    }
}

// --- Local build pipeline ---

fn run_shell(description: &str, cmd: &str, cwd: &std::path::Path) -> Result<()> {
    println!("  > {cmd}");
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(cwd)
        .status()?;
    if !status.success() {
        return Err(anyhow!(
            "{description} failed with exit code: {:?}",
            status.code()
        ));
    }
    Ok(())
}

fn resolve_project_dir(project_dir: Option<&PathBuf>) -> Result<PathBuf> {
    match project_dir {
        Some(p) => {
            let abs = std::fs::canonicalize(p)?;
            Ok(abs)
        }
        None => Ok(std::env::current_dir()?),
    }
}

fn run_local_build(app: &PagesApp, project_dir: Option<&PathBuf>, deploy: bool) -> Result<()> {
    let root = resolve_project_dir(project_dir)?;
    let app_dir = root.join("apps").join(app.name());
    if !app_dir.exists() {
        return Err(anyhow!(
            "App directory not found: {}. \
             Make sure you're in the tachyon-apps repository root \
             or specify --project-dir.",
            app_dir.display()
        ));
    }
    let (pages_build_cmd, output_dir) = app.pages_build_info();

    println!("=== Cloudflare Pages Build Pipeline ===");
    println!("  App:     {}", app.name());
    println!("  Root:    {}", root.display());
    println!("  Deploy:  {deploy}");
    println!();

    // Step 1: Install dependencies
    println!("[1/4] Installing dependencies...");
    run_shell("yarn install", "yarn install", &root)?;
    println!();

    // Step 2: Next.js build via turbo
    println!("[2/4] Building {} (turbo)...", app.name());
    run_shell(
        "turbo build",
        &format!("npx turbo run build --filter={}", app.name()),
        &root,
    )?;
    println!();

    // Step 3: Pages build
    println!("[3/4] Building for Cloudflare Pages...");
    run_shell("pages build", pages_build_cmd, &app_dir)?;
    println!();

    // Step 4: Deploy or finish
    if deploy {
        println!("[4/4] Deploying to Cloudflare Pages...");
        let deploy_cmd = match app {
            PagesApp::Cms => "npx opennextjs-cloudflare deploy".to_string(),
            _ => format!(
                "npx wrangler pages deploy {output_dir} \
                 --project-name {}",
                app.cf_project_name()
            ),
        };
        run_shell("pages deploy", &deploy_cmd, &app_dir)?;
    } else {
        println!("[4/4] Build complete.");
        println!("  Output: {}/{output_dir}", app_dir.display());
        println!();
        println!("  To preview: tachyon compute dev {}", app.name());
        println!(
            "  To deploy:  tachyon compute build {} --deploy",
            app.name()
        );
    }

    println!();
    println!("=== Done ===");
    Ok(())
}

fn run_local_dev(app: &PagesApp, project_dir: Option<&PathBuf>, port: u16) -> Result<()> {
    let root = resolve_project_dir(project_dir)?;
    let app_dir = root.join("apps").join(app.name());
    if !app_dir.exists() {
        return Err(anyhow!(
            "App directory not found: {}. \
             Make sure you're in the tachyon-apps repository root \
             or specify --project-dir.",
            app_dir.display()
        ));
    }
    let (pages_build_cmd, _) = app.pages_build_info();

    println!("=== Cloudflare Pages Local Preview ===");
    println!("  App:  {}", app.name());
    println!("  Port: {port}");
    println!();

    // Step 1: Install
    println!("[1/3] Installing dependencies...");
    run_shell("yarn install", "yarn install", &root)?;
    println!();

    // Step 2: Build
    println!("[2/3] Building {} ...", app.name());
    run_shell(
        "turbo build",
        &format!("npx turbo run build --filter={}", app.name()),
        &root,
    )?;
    run_shell("pages build", pages_build_cmd, &app_dir)?;
    println!();

    // Step 3: Preview server
    println!("[3/3] Starting preview server on port {port}...");
    let preview_cmd = app.preview_command(port);
    run_shell("preview server", &preview_cmd, &app_dir)?;

    Ok(())
}

// --- Apps subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum AppsCommand {
    /// List all compute apps
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get details of a compute app
    Get {
        /// App ID or name
        app_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Delete a compute app
    Delete {
        /// App ID or name
        app_id: Option<String>,
    },
    /// Create or update compute apps from tachyon.yml
    Apply {
        /// Manifest file path
        #[arg(short = 'f', long, default_value = "tachyon.yml")]
        file: PathBuf,
        /// App name to select from a multi-app CloudApps manifest
        #[arg(long)]
        app: Option<String>,
        /// Target environment label for this apply operation
        #[arg(long, default_value = "sandbox")]
        environment: String,
        /// Preview changes without mutating Cloud Apps
        #[arg(long)]
        dry_run: bool,
    },
    /// Generate a user feedback report for a compute app
    Feedback(FeedbackArgs),
}

#[derive(Debug, Clone, Args)]
pub struct FeedbackArgs {
    /// App ID or name the feedback is about.
    pub app_id: String,
    /// Feedback body from the user.
    #[arg(long, short = 'm')]
    pub message: String,
    /// Feedback kind.
    #[arg(long, value_enum, default_value_t = FeedbackKind::Other)]
    pub kind: FeedbackKind,
    /// Feedback severity.
    #[arg(long, value_enum, default_value_t = FeedbackSeverity::Medium)]
    pub severity: FeedbackSeverity,
    /// URL where the user observed the issue or request.
    #[arg(long)]
    pub url: Option<String>,
    /// Build ID related to the feedback.
    #[arg(long)]
    pub build_id: Option<String>,
    /// Deployment ID related to the feedback.
    #[arg(long)]
    pub deployment_id: Option<String>,
    /// Optional contact information for follow-up.
    #[arg(long)]
    pub contact: Option<String>,
    /// Additional KEY=VALUE metadata. Secret-like keys are rejected.
    #[arg(long = "metadata")]
    pub metadata: Vec<String>,
    /// Emit a JSON payload instead of Markdown.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackKind {
    Bug,
    Feature,
    Question,
    Other,
}

impl std::fmt::Display for FeedbackKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Bug => "bug",
            Self::Feature => "feature",
            Self::Question => "question",
            Self::Other => "other",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for FeedbackSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct FeedbackPayload {
    app_id: String,
    operator_id: String,
    kind: FeedbackKind,
    severity: FeedbackSeverity,
    message: String,
    url: Option<String>,
    build_id: Option<String>,
    deployment_id: Option<String>,
    contact: Option<String>,
    metadata: BTreeMap<String, String>,
    created_at: String,
}

// --- Builds subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum BuildsCommand {
    /// List builds for an app
    List {
        /// App ID or name
        app_id: Option<String>,
        /// Maximum number of builds to display
        #[arg(long, default_value_t = 10)]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get details of a specific build
    Get {
        /// Build ID
        build_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Trigger a new build
    Trigger {
        /// App ID or name
        app_id: Option<String>,
        /// Branch to build (optional)
        #[arg(long)]
        branch: Option<String>,
    },
    /// Cancel a running build
    Cancel {
        /// Build ID
        build_id: String,
    },
    /// Fetch build logs
    Logs {
        /// App ID (used to resolve latest build if --build-id is not given)
        app_id: Option<String>,
        /// Build ID (defaults to the latest build)
        #[arg(long)]
        build_id: Option<String>,
        /// Keep polling until the build is complete
        #[arg(long)]
        follow: bool,
        /// Emit compact JSON Lines for coding agents
        #[arg(long)]
        agent: bool,
    },
    /// Watch build logs and final status until completion
    Watch {
        /// App ID or name (optional when --build-id is specified)
        app_id: Option<String>,
        /// Build ID (defaults to the latest build for the given app)
        #[arg(long)]
        build_id: Option<String>,
        /// Poll interval in seconds
        #[arg(long, default_value_t = 5)]
        interval_secs: u64,
        /// Maximum wait time in seconds
        #[arg(long)]
        timeout_secs: Option<u64>,
        /// Do not print build logs, only status/result
        #[arg(long)]
        no_logs: bool,
        /// Emit compact JSON Lines for coding agents
        #[arg(long)]
        agent: bool,
    },
    /// Reproduce a cloud build locally in Docker.
    ///
    /// Phase 1: requires `--mock <path>` pointing at a local build-config
    /// fixture (PLT-914). Phase 2 (PLT-913) will fetch the buildspec and
    /// environment from tachyon-api given the build id.
    Reproduce {
        /// Build ID to reproduce (informational in --mock mode).
        build_id: String,
        /// Path to a local mock build-config (json or yaml). Required until
        /// PLT-913 endpoint is available.
        #[arg(long)]
        mock: Option<PathBuf>,
        /// Source tree to mount into the container (defaults to cwd).
        #[arg(long)]
        source_dir: Option<PathBuf>,
        /// Override the CodeBuild image (e.g.
        /// public.ecr.aws/codebuild/amazonlinux-x86_64-standard:5.0).
        #[arg(long)]
        image: Option<String>,
        /// Print the docker invocation instead of running it.
        #[arg(long)]
        dry_run: bool,
    },
    /// Run a Cloud App build workload from a JobRun spec.
    #[command(hide = true)]
    RunJob {
        /// Environment variable that contains the BuildWorkloadSpec JSON.
        #[arg(long, default_value = "TACHYON_BUILD_WORKLOAD_SPEC_JSON")]
        spec_env: String,
    },
}

// --- Deployments subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum DeploymentsCommand {
    /// List deployments for an app
    List {
        /// App ID or name
        app_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get details of a specific deployment
    Get {
        /// Deployment ID
        deployment_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Rollback an app to a previous deployment
    Rollback {
        /// App ID or name
        app_id: Option<String>,
        /// Deployment ID to roll back to
        #[arg(long)]
        deployment_id: String,
    },
}

// --- Env subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum EnvCommand {
    /// List environment variables for an app
    List {
        /// App ID or name
        app_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Set environment variables for an app
    Set {
        /// App ID or name
        app_id: Option<String>,
        /// App ID or name (alternative to positional app_id)
        #[arg(long)]
        app: Option<String>,
        /// Register this key as a Cloudflare Pages secret
        #[arg(long)]
        secret: Option<String>,
        /// Target environment
        #[arg(long, default_value = "all")]
        target: String,
        /// Git branch to scope plain variables to
        #[arg(long)]
        branch: Option<String>,
        /// Variables in KEY=VALUE format
        #[arg(num_args = 0..)]
        vars: Vec<String>,
    },
    /// Delete environment variables by key
    Unset {
        /// App ID or name (alternative to positional app_id)
        #[arg(long)]
        app: Option<String>,
        /// Target environment to delete
        #[arg(long)]
        target: Option<String>,
        /// KEY, or APP KEY when no project config is available
        #[arg(num_args = 1..=2)]
        args: Vec<String>,
    },
    /// Delete an environment variable
    Delete {
        /// App ID or name
        app_id: String,
        /// Env var ID to delete
        env_id: String,
    },
}

// --- Domains subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum DomainsCommand {
    /// List custom domains for an app
    List {
        /// App ID or name
        app_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a custom domain
    Add {
        /// App ID or name
        app_id: Option<String>,
        /// Domain name
        domain: String,
    },
    /// Verify a custom domain
    Verify {
        /// Domain ID
        domain_id: String,
    },
    /// Remove a custom domain
    Remove {
        /// Domain ID
        domain_id: String,
    },
}

// --- Scaling subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum ScalingCommand {
    /// Show current scaling configuration
    Get {
        /// App ID or name
        app_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Update scaling configuration
    Update {
        /// App ID or name
        app_id: Option<String>,
        /// Minimum number of instances
        #[arg(long)]
        min_instances: Option<i32>,
        /// Maximum number of instances
        #[arg(long)]
        max_instances: Option<i32>,
    },
}

// ---- Response types ----

#[derive(Debug, Deserialize, Serialize)]
struct ListAppsResponse {
    apps: Vec<AppResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AppResponse {
    id: String,
    name: String,
    #[serde(default)]
    framework: Option<String>,
    #[serde(default)]
    repository_url: Option<String>,
    #[serde(default)]
    repository_owner: Option<String>,
    #[serde(default)]
    repository_name: Option<String>,
    #[serde(default)]
    default_branch: Option<String>,
    #[serde(default)]
    deployment_target: Option<String>,
    #[serde(default)]
    connection_id: Option<String>,
    #[serde(default)]
    root_directory: Option<String>,
    #[serde(default)]
    docker_context: Option<String>,
    #[serde(default)]
    build_command: Option<String>,
    #[serde(default)]
    install_command: Option<String>,
    #[serde(default)]
    output_directory: Option<String>,
    #[serde(default)]
    node_version: Option<String>,
    #[serde(default)]
    buildspec_strategy: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ListBuildsResponse {
    builds: Vec<BuildResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BuildResponse {
    id: String,
    app_id: String,
    #[serde(default)]
    trigger: Option<String>,
    #[serde(default)]
    source_branch: Option<String>,
    #[serde(default)]
    commit_sha: Option<String>,
    #[serde(default)]
    commit_message: Option<String>,
    #[serde(default)]
    pr_number: Option<i32>,
    status: String,
    #[serde(default)]
    duration_secs: Option<i32>,
    #[serde(default)]
    error_message: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BuildLogsResponse {
    lines: Vec<BuildLogLineResponse>,
    next_token: Option<String>,
    is_complete: bool,
}

#[derive(Debug, Deserialize)]
struct BuildLogLineResponse {
    timestamp: i64,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AgentBuildEvent<'a> {
    Build {
        build_id: &'a str,
        status: &'a str,
    },
    Log {
        build_id: &'a str,
        timestamp: i64,
        message: String,
    },
    Result {
        build_id: &'a str,
        status: &'a str,
        exit_code: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_message: Option<&'a str>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct ListDeploymentsResponse {
    deployments: Vec<DeploymentResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DeploymentResponse {
    id: String,
    #[serde(default)]
    app_id: Option<String>,
    #[serde(default)]
    build_id: Option<String>,
    #[serde(default)]
    pr_number: Option<i32>,
    status: String,
    #[serde(default)]
    source_branch: Option<String>,
    #[serde(default)]
    public_url: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
}

impl DeploymentResponse {
    fn display_url(&self) -> String {
        self.display_url_with_pr(self.pr_number)
    }

    fn display_url_with_pr(&self, pr_number: Option<i32>) -> String {
        if let Some(public_url) = self.public_url.as_deref() {
            return public_url.to_string();
        }

        let Some(url) = self.url.as_deref() else {
            return "-".to_string();
        };

        public_preview_url_from_pages_url(url, pr_number).unwrap_or_else(|| url.to_string())
    }
}

fn public_preview_url_from_pages_url(url: &str, pr_number: Option<i32>) -> Option<String> {
    let pr_number = pr_number?;
    let host = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or(url);
    let pages_project = host.strip_suffix(".pages.dev")?;
    let app_name = pages_project.rsplit_once('.')?.1;

    Some(format!("https://pr{pr_number}--{app_name}.txcloud.app"))
}

#[derive(Debug, Deserialize, Serialize)]
struct ListEnvVarsResponse {
    env_vars: Vec<EnvVarResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EnvVarResponse {
    id: String,
    key: String,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    is_secret: Option<bool>,
}

#[derive(Debug, Serialize)]
struct SetEnvVarsRequest {
    env_vars: Vec<SetEnvVarEntry>,
}

#[derive(Serialize)]
struct SetAppSecretRequest {
    key: String,
    value: String,
    target: String,
}

#[derive(Deserialize)]
struct SetAppSecretResponse {
    key: String,
    target: String,
}

#[derive(Debug, Serialize, Clone)]
struct SetEnvVarEntry {
    key: String,
    value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_secret: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ListDomainsResponse {
    domains: Vec<CustomDomainResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CustomDomainResponse {
    id: String,
    domain: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    verified: Option<bool>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct AddDomainRequest {
    domain: String,
}

#[derive(Debug, Serialize)]
struct RollbackRequest {
    deployment_id: String,
}

#[derive(Debug, Serialize)]
struct TriggerBuildRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScalingConfigResponse {
    #[serde(default)]
    min_instances: Option<i32>,
    #[serde(default)]
    max_instances: Option<i32>,
}

#[derive(Debug, Serialize)]
struct UpdateScalingRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    min_instances: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_instances: Option<i32>,
}

// ---- Formatting helpers ----

fn format_timestamp_ms(millis: i64) -> String {
    match Utc.timestamp_millis_opt(millis) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => format!("{millis}"),
    }
}

fn format_created_at(created_at: &str) -> String {
    if let Ok(dt) = created_at.parse::<DateTime<Utc>>() {
        return dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }
    created_at.to_string()
}

fn truncate_sha(sha: &str) -> &str {
    if sha.len() > 8 {
        &sha[..8]
    } else {
        sha
    }
}

fn is_terminal_build_status(status: &str) -> bool {
    matches!(
        status.to_ascii_lowercase().as_str(),
        "succeeded" | "failed" | "cancelled" | "canceled" | "timed_out" | "timeout"
    )
}

fn is_success_build_status(status: &str) -> bool {
    matches!(status.to_ascii_lowercase().as_str(), "succeeded")
}

fn print_agent_event(event: &AgentBuildEvent<'_>) -> Result<()> {
    println!("{}", serde_json::to_string(event)?);
    Ok(())
}

fn compact_agent_message(message: &str) -> String {
    const MAX_LEN: usize = 500;
    if message.chars().count() <= MAX_LEN {
        return message.to_string();
    }
    let mut compacted: String = message.chars().take(MAX_LEN - 3).collect();
    compacted.push_str("...");
    compacted
}

// ---- Command handlers ----

async fn run_apps_list(api: &ApiClient, json: bool) -> Result<()> {
    let resp: ListAppsResponse = api.get("/v1/compute/apps").await?;
    if json {
        return print_json(&resp.apps);
    }
    if resp.apps.is_empty() {
        println!("No apps found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<24}  {:<12}  {:<10}  CREATED AT",
        "ID", "NAME", "FRAMEWORK", "STATUS"
    );
    println!(
        "{:-<28}  {:-<24}  {:-<12}  {:-<10}  {:-<19}",
        "", "", "", "", ""
    );
    for app in &resp.apps {
        println!(
            "{:<28}  {:<24}  {:<12}  {:<10}  {}",
            app.id,
            truncate(&app.name, 24),
            app.framework.as_deref().unwrap_or("-"),
            app.status.as_deref().unwrap_or("-"),
            app.created_at
                .as_deref()
                .map(format_created_at)
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    Ok(())
}

async fn run_apps_get(api: &ApiClient, app_id: &str, json: bool) -> Result<()> {
    let app: AppResponse = api.get(&format!("/v1/compute/apps/{app_id}")).await?;
    if json {
        return print_json(&app);
    }
    println!("ID:         {}", app.id);
    println!("Name:       {}", app.name);
    println!("Framework:  {}", app.framework.as_deref().unwrap_or("-"));
    println!(
        "Repository: {}",
        app.repository_url.as_deref().unwrap_or("-")
    );
    println!("Status:     {}", app.status.as_deref().unwrap_or("-"));
    println!(
        "Created:    {}",
        app.created_at
            .as_deref()
            .map(format_created_at)
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "Updated:    {}",
        app.updated_at
            .as_deref()
            .map(format_created_at)
            .unwrap_or_else(|| "-".to_string())
    );
    Ok(())
}

async fn run_apps_delete(api: &ApiClient, app_id: &str) -> Result<()> {
    api.delete(&format!("/v1/compute/apps/{app_id}")).await?;
    println!("App {app_id} deleted.");
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum ApplyAction {
    Create,
    Update,
    NoChange,
}

#[derive(Debug, Default)]
struct EnvPlan {
    plain: Vec<SetEnvVarEntry>,
    secret_refs: Vec<SecretEnvRef>,
}

#[derive(Debug)]
struct SecretEnvRef {
    key: String,
    target: String,
}

async fn run_apps_apply(
    api: &ApiClient,
    file: &Path,
    selected_app: Option<&str>,
    environment: &str,
    dry_run: bool,
) -> Result<()> {
    let manifest = load_cloud_apps_manifest(file)?;
    let entries = select_app_entries(&manifest, selected_app)?;
    let live: ListAppsResponse = api.get("/v1/compute/apps").await?;

    println!("Manifest:    {}", file.display());
    println!("Environment: {environment}");
    println!("Mode:        {}", if dry_run { "dry-run" } else { "apply" });
    println!();

    let mut created = 0;
    let mut updated = 0;
    let mut unchanged = 0;
    for entry in entries {
        let name = entry
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("app entry is missing name"))?;
        let body = app_entry_to_api_body(&entry)?;
        let env_plan = plan_env_vars(&entry, environment)?;
        let existing = live.apps.iter().find(|app| app.name == name);
        let (action, changed_fields) = classify_app_action(existing, &body);
        let app_id = match (existing, &action, dry_run) {
            (Some(app), ApplyAction::Update, false) => {
                let updated: AppResponse = api
                    .patch(&format!("/v1/compute/apps/{}", app.id), &body)
                    .await?;
                updated.id
            }
            (Some(app), _, _) => app.id.clone(),
            (None, ApplyAction::Create, false) => {
                let created: AppResponse = api.post("/v1/compute/apps", &body).await?;
                created.id
            }
            (None, _, true) => "<new app>".to_string(),
            (None, _, false) => unreachable!(),
        };

        let (env_changed, missing_secrets) =
            apply_env_plan(api, &app_id, &env_plan, dry_run).await?;
        match action {
            ApplyAction::Create => created += 1,
            ApplyAction::Update => updated += 1,
            ApplyAction::NoChange => unchanged += 1,
        }
        let label = match action {
            ApplyAction::Create => "CREATED",
            ApplyAction::Update => "UPDATED",
            ApplyAction::NoChange => "UNCHANGED",
        };
        println!("{label} {name} ({app_id})");
        println!("  environment: {environment}");
        println!("  manifest:    {}", file.display());
        if changed_fields.is_empty() {
            println!("  changed:     <none>");
        } else {
            println!("  changed:     {}", changed_fields.join(", "));
        }
        if !env_changed.is_empty() {
            println!("  env:         {}", env_changed.join(", "));
        }
        if !missing_secrets.is_empty() {
            println!("  missing secrets: {}", missing_secrets.join(", "));
            println!("  next:        tachyon compute env set {app_id} KEY=<value>");
        }
    }

    println!();
    println!("Summary: {created} created, {updated} updated, {unchanged} unchanged");
    Ok(())
}

fn load_cloud_apps_manifest(path: &Path) -> Result<Value> {
    let content = std::fs::read_to_string(path)?;
    let value: Value = serde_yaml::from_str(&content)?;
    match value.get("kind").and_then(Value::as_str) {
        Some("CloudApps") => Ok(value),
        Some("CloudApp") => {
            let name = value
                .get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("CloudApp manifest is missing metadata.name"))?;
            let mut entry = value.get("spec").cloned().unwrap_or_else(|| json!({}));
            entry
                .as_object_mut()
                .ok_or_else(|| anyhow!("CloudApp spec must be an object"))?
                .insert("name".to_string(), Value::String(name.to_string()));
            Ok(json!({
                "apiVersion": "apps.tachy.one/v1alpha",
                "kind": "CloudApps",
                "metadata": { "name": name },
                "spec": { "apps": [entry] }
            }))
        }
        Some(kind) => Err(anyhow!("unsupported manifest kind: {kind}")),
        None => Err(anyhow!("manifest is missing kind")),
    }
}

fn select_app_entries(manifest: &Value, app: Option<&str>) -> Result<Vec<Value>> {
    let apps = manifest
        .get("spec")
        .and_then(|s| s.get("apps"))
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("CloudApps manifest must contain spec.apps[]"))?;
    let entries = apps
        .iter()
        .filter(|entry| {
            app.is_none_or(|name| entry.get("name").and_then(Value::as_str) == Some(name))
        })
        .cloned()
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return Err(anyhow!(
            "no app entry matched {}",
            app.unwrap_or("<all apps>")
        ));
    }
    Ok(entries)
}

fn app_entry_to_api_body(entry: &Value) -> Result<Value> {
    let name = entry
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("app entry is missing name"))?;
    let repo = entry
        .get("repository")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("app entry {name} is missing repository"))?;
    let repo_str = |key: &str| -> Result<String> {
        repo.get(key)
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| anyhow!("app entry {name} repository.{key} is required"))
    };
    let framework = entry
        .get("framework")
        .and_then(Value::as_str)
        .unwrap_or("next_js");
    let deployment_target = entry
        .get("deploymentTarget")
        .and_then(Value::as_str)
        .unwrap_or("cloud_run");
    let build = entry.get("build").and_then(Value::as_object);
    let build_command = build.and_then(|b| {
        b.get("command")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .or_else(|| cargo_lambda_build_command(framework, b))
    });
    let mut body = json!({
        "name": name,
        "repository_url": repo_str("url")?,
        "repository_owner": repo_str("owner")?,
        "repository_name": repo_str("name")?,
        "default_branch": repo.get("defaultBranch").and_then(Value::as_str).unwrap_or("main"),
        "framework": framework,
        "deployment_target": deployment_target,
    });
    let obj = body.as_object_mut().unwrap();
    copy_string_field(entry, obj, "rootDirectory", "root_directory");
    copy_string_field(entry, obj, "dockerContext", "docker_context");
    copy_string_field(entry, obj, "buildspecStrategy", "buildspec_strategy");
    if let Some(command) = build_command {
        obj.insert("build_command".to_string(), Value::String(command));
    }
    if let Some(build) = build {
        copy_string_field_from_map(build, obj, "installCommand", "install_command");
        copy_string_field_from_map(build, obj, "outputDirectory", "output_directory");
        copy_string_field_from_map(build, obj, "nodeVersion", "node_version");
    }
    Ok(body)
}

fn copy_string_field(
    source: &Value,
    target: &mut serde_json::Map<String, Value>,
    from: &str,
    to: &str,
) {
    if let Some(value) = source.get(from).and_then(Value::as_str) {
        target.insert(to.to_string(), Value::String(value.to_string()));
    }
}

fn copy_string_field_from_map(
    source: &serde_json::Map<String, Value>,
    target: &mut serde_json::Map<String, Value>,
    from: &str,
    to: &str,
) {
    if let Some(value) = source.get(from).and_then(Value::as_str) {
        target.insert(to.to_string(), Value::String(value.to_string()));
    }
}

fn cargo_lambda_build_command(
    framework: &str,
    build: &serde_json::Map<String, Value>,
) -> Option<String> {
    if framework != "cargo_lambda" {
        return None;
    }
    let package = build.get("package").and_then(Value::as_str)?;
    let binary = build.get("binary").and_then(Value::as_str);
    let release = build
        .get("release")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let arch = build.get("arch").and_then(Value::as_str).unwrap_or("arm64");
    let mut command = format!("cargo lambda build --package {package}");
    if let Some(binary) = binary {
        command.push_str(&format!(" --bin {binary}"));
    }
    if release {
        command.push_str(" --release");
    }
    if arch == "arm64" {
        command.push_str(" --arm64");
    }
    Some(command)
}

fn classify_app_action(existing: Option<&AppResponse>, body: &Value) -> (ApplyAction, Vec<String>) {
    match existing {
        None => (ApplyAction::Create, manifest_body_fields(body)),
        Some(app) => {
            let fields = manifest_body_fields(body)
                .into_iter()
                .filter(|field| app_field_value(app, field) != body[field])
                .collect::<Vec<_>>();
            let action = if fields.is_empty() {
                ApplyAction::NoChange
            } else {
                ApplyAction::Update
            };
            (action, fields)
        }
    }
}

fn manifest_body_fields(body: &Value) -> Vec<String> {
    body.as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

fn app_field_value(app: &AppResponse, field: &str) -> Value {
    match field {
        "name" => json!(app.name),
        "repository_url" => opt_string_value(app.repository_url.as_deref()),
        "repository_owner" => opt_string_value(app.repository_owner.as_deref()),
        "repository_name" => opt_string_value(app.repository_name.as_deref()),
        "default_branch" => opt_string_value(app.default_branch.as_deref()),
        "framework" => opt_string_value(app.framework.as_deref()),
        "deployment_target" => opt_string_value(app.deployment_target.as_deref()),
        "connection_id" => opt_string_value(app.connection_id.as_deref()),
        "root_directory" => opt_string_value(app.root_directory.as_deref()),
        "docker_context" => opt_string_value(app.docker_context.as_deref()),
        "build_command" => opt_string_value(app.build_command.as_deref()),
        "install_command" => opt_string_value(app.install_command.as_deref()),
        "output_directory" => opt_string_value(app.output_directory.as_deref()),
        "node_version" => opt_string_value(app.node_version.as_deref()),
        "buildspec_strategy" => {
            opt_string_value(app.buildspec_strategy.as_deref().or(Some("inline")))
        }
        _ => Value::Null,
    }
}

fn opt_string_value(value: Option<&str>) -> Value {
    match value.filter(|v| !v.is_empty()) {
        Some(value) => Value::String(value.to_string()),
        None => Value::Null,
    }
}

fn plan_env_vars(entry: &Value, environment: &str) -> Result<EnvPlan> {
    let Some(env_vars) = entry.get("envVars") else {
        return Ok(EnvPlan::default());
    };
    let env_vars = env_vars
        .as_array()
        .ok_or_else(|| anyhow!("envVars must be an array"))?;
    let mut plan = EnvPlan::default();
    for env in env_vars {
        let key = env
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("env var entry is missing name"))?;
        let target = env
            .get("target")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .unwrap_or_else(|| default_env_target(environment).to_string());
        let env_type = env.get("type").and_then(Value::as_str).unwrap_or("plain");
        let value = env.get("value").and_then(Value::as_str);
        let value_from = env.get("valueFrom");
        if env_type == "credential" || value_from.is_some() {
            if value.is_some() {
                return Err(anyhow!("credential env var {key} must use valueFrom.secret; literal values are not allowed"));
            }
            let secret = extract_secret_ref(
                key,
                value_from
                    .ok_or_else(|| anyhow!("credential env var {key} is missing valueFrom"))?,
            )?;
            plan.secret_refs.push(SecretEnvRef {
                key: secret,
                target,
            });
        } else {
            plan.plain.push(SetEnvVarEntry {
                key: key.to_string(),
                value: value
                    .ok_or_else(|| anyhow!("plain env var {key} must define value"))?
                    .to_string(),
                target: Some(target),
                branch: None,
                is_secret: Some(false),
            });
        }
    }
    Ok(plan)
}

fn default_env_target(environment: &str) -> &'static str {
    match environment {
        "production" | "prod" => "production",
        "preview" => "preview",
        _ => "all",
    }
}

fn extract_secret_ref(key: &str, value_from: &Value) -> Result<String> {
    let secret = value_from
        .get("secret")
        .ok_or_else(|| anyhow!("env var {key} only supports valueFrom.secret"))?;
    let path = if let Some(path) = secret.as_str() {
        path
    } else if let Some(path) = secret.get("path").and_then(Value::as_str) {
        if secret.get("field").is_some() {
            return Err(anyhow!(
                "env var {key} valueFrom.secret.field is not supported"
            ));
        }
        path
    } else {
        return Err(anyhow!(
            "env var {key} valueFrom.secret must be a key string or object with path"
        ));
    };
    if path.is_empty() {
        return Err(anyhow!(
            "env var {key} valueFrom.secret must reference a single env key"
        ));
    }
    Ok(path.to_string())
}

async fn apply_env_plan(
    api: &ApiClient,
    app_id: &str,
    plan: &EnvPlan,
    dry_run: bool,
) -> Result<(Vec<String>, Vec<String>)> {
    let changed = plan
        .plain
        .iter()
        .map(|entry| {
            format!(
                "{}({})",
                entry.key,
                entry.target.as_deref().unwrap_or("all")
            )
        })
        .collect::<Vec<_>>();
    if !plan.plain.is_empty() && !dry_run && app_id != "<new app>" {
        let req = SetEnvVarsRequest {
            env_vars: plan.plain.clone(),
        };
        let _: ListEnvVarsResponse = api
            .put(&format!("/v1/compute/apps/{app_id}/env"), &req)
            .await?;
    }

    let mut missing = Vec::new();
    if !plan.secret_refs.is_empty() && !dry_run && app_id != "<new app>" {
        let resp: ListEnvVarsResponse = api
            .get(&format!("/v1/compute/apps/{app_id}/env"))
            .await
            .unwrap_or(ListEnvVarsResponse { env_vars: vec![] });
        let current = resp
            .env_vars
            .into_iter()
            .filter(|var| var.is_secret.unwrap_or(false))
            .map(|var| (var.key, var.target.unwrap_or_else(|| "all".to_string())))
            .collect::<BTreeSet<_>>();
        for secret in &plan.secret_refs {
            if !current.contains(&(secret.key.clone(), secret.target.clone()))
                && !current.contains(&(secret.key.clone(), "all".to_string()))
            {
                missing.push(format!("{}({})", secret.key, secret.target));
            }
        }
    } else {
        missing.extend(
            plan.secret_refs
                .iter()
                .map(|secret| format!("{}({})", secret.key, secret.target)),
        );
    }
    Ok((changed, missing))
}

fn run_apps_feedback(tenant_id: &str, app_id: &str, args: &FeedbackArgs) -> Result<()> {
    let payload = build_feedback_payload(tenant_id, app_id, args)?;
    if args.json {
        print_json(&payload)?;
    } else {
        println!("{}", render_feedback_markdown(&payload));
    }
    Ok(())
}

fn build_feedback_payload(
    tenant_id: &str,
    app_id: &str,
    args: &FeedbackArgs,
) -> Result<FeedbackPayload> {
    let metadata = parse_feedback_metadata(&args.metadata)?;
    Ok(FeedbackPayload {
        app_id: app_id.to_string(),
        operator_id: tenant_id.to_string(),
        kind: args.kind,
        severity: args.severity,
        message: args.message.clone(),
        url: args.url.clone(),
        build_id: args.build_id.clone(),
        deployment_id: args.deployment_id.clone(),
        contact: args.contact.clone(),
        metadata,
        created_at: Utc::now().to_rfc3339(),
    })
}

fn parse_feedback_metadata(entries: &[String]) -> Result<BTreeMap<String, String>> {
    let mut metadata = BTreeMap::new();
    for entry in entries {
        let (key, value) = entry
            .split_once('=')
            .ok_or_else(|| anyhow!("metadata must be KEY=VALUE, got `{entry}`"))?;
        let key = key.trim();
        if key.is_empty() {
            return Err(anyhow!("metadata key must not be empty"));
        }
        if is_secret_like_key(key) {
            return Err(anyhow!(
                "metadata key `{key}` looks secret-like; do not pass secret values to feedback"
            ));
        }
        metadata.insert(key.to_string(), value.trim().to_string());
    }
    Ok(metadata)
}

fn is_secret_like_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '.'], "_");
    [
        "secret",
        "token",
        "password",
        "passwd",
        "api_key",
        "apikey",
        "private_key",
        "credential",
        "authorization",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn render_feedback_markdown(payload: &FeedbackPayload) -> String {
    let mut lines = vec![
        "# Cloud App Feedback".to_string(),
        String::new(),
        format!("- App ID: {}", payload.app_id),
        format!("- Operator ID: {}", payload.operator_id),
        format!("- Kind: {}", payload.kind),
        format!("- Severity: {}", payload.severity),
        format!("- Created At: {}", payload.created_at),
    ];

    if let Some(url) = &payload.url {
        lines.push(format!("- URL: {url}"));
    }
    if let Some(build_id) = &payload.build_id {
        lines.push(format!("- Build ID: {build_id}"));
    }
    if let Some(deployment_id) = &payload.deployment_id {
        lines.push(format!("- Deployment ID: {deployment_id}"));
    }
    if let Some(contact) = &payload.contact {
        lines.push(format!("- Contact: {contact}"));
    }
    if !payload.metadata.is_empty() {
        lines.push("- Metadata:".to_string());
        for (key, value) in &payload.metadata {
            lines.push(format!("  - {key}: {value}"));
        }
    }

    lines.extend([
        String::new(),
        "## Message".to_string(),
        String::new(),
        payload.message.clone(),
    ]);

    lines.join("\n")
}

async fn run_builds_list(api: &ApiClient, app_id: &str, limit: usize, json: bool) -> Result<()> {
    let resp: ListBuildsResponse = api
        .get(&format!("/v1/compute/apps/{app_id}/builds"))
        .await?;
    if json {
        let builds = &resp.builds[..resp.builds.len().min(limit)];
        return print_json(&builds);
    }
    if resp.builds.is_empty() {
        println!("No builds found for app {app_id}");
        return Ok(());
    }
    let builds = &resp.builds[..resp.builds.len().min(limit)];
    println!(
        "{:<26}  {:<11}  {:<20}  {:<8}  {:<19}",
        "BUILD ID", "STATUS", "BRANCH", "COMMIT", "CREATED AT"
    );
    println!(
        "{:-<26}  {:-<11}  {:-<20}  {:-<8}  {:-<19}",
        "", "", "", "", ""
    );
    for build in builds {
        println!(
            "{:<26}  {:<11}  {:<20}  {:<8}  {:<19}",
            build.id,
            build.status,
            truncate(build.source_branch.as_deref().unwrap_or("-"), 20),
            truncate_sha(build.commit_sha.as_deref().unwrap_or("-")),
            build
                .created_at
                .as_deref()
                .map(format_created_at)
                .unwrap_or_else(|| "-".to_string()),
        );
    }

    // Show active preview URLs below the build list.
    let build_pr_numbers: HashMap<&str, i32> = builds
        .iter()
        .filter_map(|build| build.pr_number.map(|pr| (build.id.as_str(), pr)))
        .collect();
    let preview_url = format!("/v1/compute/apps/{app_id}/deployments?environment=preview");
    if let Ok(dep_resp) = api.get::<ListDeploymentsResponse>(&preview_url).await {
        let active: Vec<_> = dep_resp
            .deployments
            .iter()
            .filter(|d| d.status == "active" || d.status == "deploying")
            .collect();
        if !active.is_empty() {
            println!();
            println!("Preview URLs:");
            for dep in &active {
                let branch = dep.source_branch.as_deref().unwrap_or("-");
                let pr_number = dep.pr_number.or_else(|| {
                    dep.build_id
                        .as_deref()
                        .and_then(|build_id| build_pr_numbers.get(build_id).copied())
                });
                println!("  [{branch}] {}", dep.display_url_with_pr(pr_number));
            }
        }
    }
    Ok(())
}

async fn run_builds_get(api: &ApiClient, build_id: &str, json: bool) -> Result<()> {
    let build: BuildResponse = api.get(&format!("/v1/compute/builds/{build_id}")).await?;
    if json {
        return print_json(&build);
    }
    println!("ID:       {}", build.id);
    println!("App ID:   {}", build.app_id);
    println!("Status:   {}", build.status);
    println!(
        "Branch:   {}",
        build.source_branch.as_deref().unwrap_or("-")
    );
    println!("Commit:   {}", build.commit_sha.as_deref().unwrap_or("-"));
    println!(
        "Message:  {}",
        build.commit_message.as_deref().unwrap_or("-")
    );
    println!("Trigger:  {}", build.trigger.as_deref().unwrap_or("-"));
    if let Some(dur) = build.duration_secs {
        println!("Duration: {dur}s");
    }
    if let Some(err) = &build.error_message {
        println!("Error:    {err}");
    }
    println!(
        "Created:  {}",
        build
            .created_at
            .as_deref()
            .map(format_created_at)
            .unwrap_or_else(|| "-".to_string())
    );

    // Fetch the associated deployment to show the preview URL.
    if build.status == "succeeded" {
        let url: String = format!("/v1/compute/apps/{}/deployments", build.app_id);
        if let Ok(resp) = api.get::<ListDeploymentsResponse>(&url).await {
            if let Some(dep) = resp
                .deployments
                .iter()
                .find(|d| d.build_id.as_deref() == Some(&build.id))
            {
                let pr_number = dep.pr_number.or(build.pr_number);
                println!("Preview:  {}", dep.display_url_with_pr(pr_number));
            }
        }
    }
    Ok(())
}

async fn run_builds_trigger(api: &ApiClient, app_id: &str, branch: Option<&str>) -> Result<()> {
    let req = TriggerBuildRequest {
        branch: branch.map(String::from),
    };
    let build: BuildResponse = api
        .post(&format!("/v1/compute/apps/{app_id}/builds"), &req)
        .await?;
    println!("Build triggered: {}", build.id);
    println!("Status: {}", build.status);
    Ok(())
}

async fn run_builds_cancel(api: &ApiClient, build_id: &str) -> Result<()> {
    api.post_no_body(&format!("/v1/compute/builds/{build_id}/cancel"))
        .await?;
    println!("Build {build_id} cancelled.");
    Ok(())
}

async fn run_builds_logs(
    api: &ApiClient,
    app_id: Option<&str>,
    build_id: Option<&str>,
    follow: bool,
    agent: bool,
) -> Result<()> {
    let resolved_build_id = match build_id {
        Some(id) => id.to_string(),
        None => {
            let app_id = app_id
                .ok_or_else(|| anyhow!("app_id required when --build-id is not specified"))?;
            let resp: ListBuildsResponse = api
                .get(&format!("/v1/compute/apps/{app_id}/builds"))
                .await?;
            resp.builds
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("no builds found for app {app_id}"))?
                .id
        }
    };

    let mut next_token: Option<String> = None;
    let mut is_complete = false;
    loop {
        let path = format!("/v1/compute/builds/{resolved_build_id}/logs");
        let logs: BuildLogsResponse = if let Some(token) = &next_token {
            api.get_query(&path, &[("next_token", token.as_str())])
                .await?
        } else {
            api.get(&path).await?
        };

        for line in &logs.lines {
            if agent {
                print_agent_event(&AgentBuildEvent::Log {
                    build_id: &resolved_build_id,
                    timestamp: line.timestamp,
                    message: compact_agent_message(&line.message),
                })?;
            } else {
                println!("[{}] {}", format_timestamp_ms(line.timestamp), line.message);
            }
        }

        if logs.is_complete {
            is_complete = true;
            break;
        }
        next_token = logs.next_token;
        if follow {
            sleep(Duration::from_secs(2)).await;
        } else {
            break;
        }
    }

    if follow && is_complete {
        let build: BuildResponse = api
            .get(&format!("/v1/compute/builds/{resolved_build_id}"))
            .await?;
        if agent {
            let exit_code = if is_success_build_status(&build.status) {
                0
            } else {
                1
            };
            print_agent_event(&AgentBuildEvent::Result {
                build_id: &resolved_build_id,
                status: &build.status,
                exit_code,
                error_message: build.error_message.as_deref(),
            })?;
        }
        if !is_success_build_status(&build.status) {
            return Err(anyhow!("build {} failed", resolved_build_id));
        }
    }

    Ok(())
}

async fn run_builds_watch(
    api: &ApiClient,
    app_id: Option<&str>,
    build_id: Option<&str>,
    interval_secs: u64,
    timeout_secs: Option<u64>,
    no_logs: bool,
    agent: bool,
) -> Result<()> {
    let resolved_build_id = match build_id {
        Some(id) => id.to_string(),
        None => {
            let app_id = app_id
                .ok_or_else(|| anyhow!("app_id required when --build-id is not specified"))?;
            let resp: ListBuildsResponse = api
                .get(&format!("/v1/compute/apps/{app_id}/builds"))
                .await?;
            resp.builds
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("no builds found for app {app_id}"))?
                .id
        }
    };

    let interval = Duration::from_secs(interval_secs.max(1));
    let started = Instant::now();
    let timeout = timeout_secs.map(Duration::from_secs);
    let mut next_token: Option<String> = None;
    let mut last_status: Option<String> = None;

    loop {
        if let Some(timeout) = timeout {
            if started.elapsed() >= timeout {
                if agent {
                    print_agent_event(&AgentBuildEvent::Result {
                        build_id: &resolved_build_id,
                        status: "timeout",
                        exit_code: 124,
                        error_message: Some("watch timed out"),
                    })?;
                }
                return Err(anyhow!("build {} watch timed out", resolved_build_id));
            }
        }

        let build: BuildResponse = api
            .get(&format!("/v1/compute/builds/{resolved_build_id}"))
            .await?;
        if last_status.as_deref() != Some(build.status.as_str()) {
            if agent {
                print_agent_event(&AgentBuildEvent::Build {
                    build_id: &resolved_build_id,
                    status: &build.status,
                })?;
            } else {
                println!("Build {}: {}", resolved_build_id, build.status);
            }
            last_status = Some(build.status.clone());
        }

        if !no_logs {
            let path = format!("/v1/compute/builds/{resolved_build_id}/logs");
            let logs: BuildLogsResponse = if let Some(token) = &next_token {
                api.get_query(&path, &[("next_token", token.as_str())])
                    .await?
            } else {
                api.get(&path).await?
            };
            for line in &logs.lines {
                if agent {
                    print_agent_event(&AgentBuildEvent::Log {
                        build_id: &resolved_build_id,
                        timestamp: line.timestamp,
                        message: compact_agent_message(&line.message),
                    })?;
                } else {
                    println!("[{}] {}", format_timestamp_ms(line.timestamp), line.message);
                }
            }
            next_token = logs.next_token;
        }

        if is_terminal_build_status(&build.status) {
            let exit_code = if is_success_build_status(&build.status) {
                0
            } else {
                1
            };
            if agent {
                print_agent_event(&AgentBuildEvent::Result {
                    build_id: &resolved_build_id,
                    status: &build.status,
                    exit_code,
                    error_message: build.error_message.as_deref(),
                })?;
            } else if exit_code == 0 {
                println!("Build {} completed successfully.", resolved_build_id);
            }
            if exit_code != 0 {
                return Err(anyhow!(
                    "build {} finished with status {}",
                    resolved_build_id,
                    build.status
                ));
            }
            return Ok(());
        }

        sleep(interval).await;
    }
}

#[derive(Debug, Clone, Copy)]
struct RuntimeLogTailOptions {
    raw_json: bool,
}

#[derive(Debug, Default)]
struct SseEvent {
    event: Option<String>,
    data: Vec<String>,
}

async fn run_runtime_log_tail(
    api: &ApiClient,
    app_id: &str,
    options: RuntimeLogTailOptions,
) -> Result<()> {
    let url = format!("{}/v1/compute/apps/{app_id}/logs/tail", api.base_url);
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);

    loop {
        let result = tokio::select! {
            biased;
            signal = tokio::signal::ctrl_c() => {
                signal?;
                eprintln!("runtime log tail stopped.");
                return Ok(());
            }
            result = run_runtime_log_tail_once(api, &url, options) => result,
        };

        if let Err(error) = result {
            eprintln!("runtime log tail error: {error}");
        } else {
            eprintln!("runtime log tail disconnected.");
        }

        eprintln!("reconnecting in {}s...", backoff.as_secs());
        tokio::select! {
            biased;
            signal = tokio::signal::ctrl_c() => {
                signal?;
                eprintln!("runtime log tail stopped.");
                return Ok(());
            }
            _ = sleep(backoff) => {}
        }
        backoff = std::cmp::min(backoff * 2, max_backoff);
    }
}

async fn run_runtime_log_tail_once(
    api: &ApiClient,
    url: &str,
    options: RuntimeLogTailOptions,
) -> Result<()> {
    let mut resp = api
        .client
        .get(url)
        .header(reqwest::header::ACCEPT, "text/event-stream")
        .send()
        .await
        .map_err(|e| anyhow!("GET {url} failed: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "runtime log tail failed: status={status}, body={body}"
        ));
    }

    let mut pending = String::new();
    while let Some(chunk) = resp.chunk().await? {
        pending.push_str(&String::from_utf8_lossy(&chunk));
        for event in drain_sse_events(&mut pending) {
            if print_runtime_log_tail_event(&event, options.raw_json)? {
                return Err(anyhow!("runtime log tail stream error"));
            }
        }
        std::io::stdout().flush().ok();
    }
    if let Some(event) = parse_sse_event(&pending) {
        if print_runtime_log_tail_event(&event, options.raw_json)? {
            return Err(anyhow!("runtime log tail stream error"));
        }
        std::io::stdout().flush().ok();
    }

    Ok(())
}

fn drain_sse_events(pending: &mut String) -> Vec<SseEvent> {
    let mut events = Vec::new();
    while let Some((pos, delimiter_len)) = find_sse_event_boundary(pending) {
        let raw = pending[..pos].to_string();
        pending.drain(..pos + delimiter_len);
        if let Some(event) = parse_sse_event(&raw) {
            events.push(event);
        }
    }
    events
}

fn find_sse_event_boundary(input: &str) -> Option<(usize, usize)> {
    match (input.find("\r\n\r\n"), input.find("\n\n")) {
        (Some(crlf), Some(lf)) if crlf <= lf => Some((crlf, 4)),
        (Some(_), Some(lf)) => Some((lf, 2)),
        (Some(crlf), None) => Some((crlf, 4)),
        (None, Some(lf)) => Some((lf, 2)),
        (None, None) => None,
    }
}

fn parse_sse_event(raw: &str) -> Option<SseEvent> {
    let mut event = SseEvent::default();
    for line in raw.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        if let Some(value) = line.strip_prefix("event:") {
            event.event = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            event.data.push(value.trim_start().to_string());
        }
    }
    (!event.data.is_empty()).then_some(event)
}

fn print_runtime_log_tail_event(event: &SseEvent, raw_json: bool) -> Result<bool> {
    let data = event.data.join("\n");
    if event.event.as_deref() == Some("error") {
        eprintln!(
            "runtime log tail stream error: {}",
            format_runtime_log_line(&data)
        );
        return Ok(true);
    }

    if raw_json {
        println!("{data}");
    } else {
        for line in format_runtime_log_lines(&data) {
            println!("{line}");
        }
    }
    Ok(false)
}

fn format_runtime_log_lines(data: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(data) else {
        return vec![data.to_string()];
    };
    let Some(object) = value.as_object() else {
        return vec![format_runtime_log_value(&value)];
    };

    let timestamp = runtime_log_timestamp(object);
    let mut lines = Vec::new();

    if let Some(summary) = runtime_log_request_summary(object) {
        lines.push(format!("{timestamp} {summary}"));
    }

    if let Some(exceptions) = object.get("exceptions").and_then(Value::as_array) {
        for exception in exceptions {
            lines.push(format!(
                "{timestamp} ERROR {}",
                format_runtime_log_exception(exception)
            ));
        }
    }

    if let Some(logs) = object.get("logs").and_then(Value::as_array) {
        for log in logs {
            lines.push(format!("{timestamp} {}", format_runtime_log_console(log)));
        }
    }

    if lines.is_empty() {
        lines.push(format!(
            "{timestamp} {}",
            runtime_log_generic_summary(object, &value)
        ));
    }

    lines
}

fn format_runtime_log_line(data: &str) -> String {
    format_runtime_log_lines(data).join(" | ")
}

fn runtime_log_timestamp(object: &serde_json::Map<String, Value>) -> String {
    let millis = object
        .get("eventTimestamp")
        .or_else(|| object.get("timestamp"))
        .and_then(value_to_i64);
    if let Some(millis) = millis {
        return match Utc.timestamp_millis_opt(millis) {
            chrono::LocalResult::Single(dt) => format!("[{}]", dt.format("%H:%M:%S")),
            _ => format!("[{millis}]"),
        };
    }

    let timestamp = object
        .get("timestamp")
        .or_else(|| object.get("eventTimestamp"))
        .and_then(Value::as_str);
    if let Some(timestamp) = timestamp {
        if let Ok(parsed) = DateTime::parse_from_rfc3339(timestamp) {
            return format!("[{}]", parsed.with_timezone(&Utc).format("%H:%M:%S"));
        }
    }

    format!("[{}]", Utc::now().format("%H:%M:%S"))
}

fn runtime_log_request_summary(object: &serde_json::Map<String, Value>) -> Option<String> {
    let request = object.get("request")?.as_object()?;
    let method = request
        .get("method")
        .or_else(|| request.get("requestMethod"))
        .and_then(Value::as_str)
        .unwrap_or("REQUEST");
    let url = request
        .get("url")
        .or_else(|| request.get("path"))
        .and_then(Value::as_str)
        .unwrap_or("-");
    let path = runtime_log_url_path(url);
    let status = object
        .get("response")
        .and_then(Value::as_object)
        .and_then(|response| {
            response
                .get("status")
                .or_else(|| response.get("statusCode"))
                .and_then(value_to_i64)
        })
        .map(|status| status.to_string())
        .unwrap_or_else(|| "-".to_string());
    let cpu = object
        .get("cpuTime")
        .or_else(|| object.get("cpu_time"))
        .and_then(value_to_i64);
    let source = object
        .get("scriptName")
        .or_else(|| object.get("source"))
        .and_then(Value::as_str);

    let mut line = format!("{method} {path} {status}");
    if let Some(cpu) = cpu {
        line.push_str(&format!("  (cpu: {cpu}ms)"));
    }
    if let Some(source) = source {
        line.push_str(&format!("  source={source}"));
    }
    Some(line)
}

fn runtime_log_generic_summary(object: &serde_json::Map<String, Value>, value: &Value) -> String {
    let level = object
        .get("level")
        .or_else(|| object.get("severity"))
        .and_then(Value::as_str)
        .map(|level| level.to_ascii_uppercase());
    let source = object
        .get("source")
        .or_else(|| object.get("scriptName"))
        .and_then(Value::as_str);
    let message = runtime_log_message(object).unwrap_or_else(|| format_runtime_log_value(value));

    let mut parts = Vec::new();
    if let Some(level) = level {
        parts.push(level);
    }
    parts.push(message);
    if let Some(source) = source {
        parts.push(format!("source={source}"));
    }
    parts.join(" ")
}

fn runtime_log_message(object: &serde_json::Map<String, Value>) -> Option<String> {
    for key in ["message", "outcome", "event"] {
        if let Some(value) = object.get(key) {
            let text = format_runtime_log_value(value);
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    None
}

fn runtime_log_url_path(url: &str) -> String {
    if let Ok(parsed) = reqwest::Url::parse(url) {
        let mut path = parsed.path().to_string();
        if let Some(query) = parsed.query() {
            path.push('?');
            path.push_str(query);
        }
        if path.is_empty() {
            "/".to_string()
        } else {
            path
        }
    } else {
        url.to_string()
    }
}

fn format_runtime_log_console(value: &Value) -> String {
    if let Some(object) = value.as_object() {
        let level = object
            .get("level")
            .or_else(|| object.get("severity"))
            .and_then(Value::as_str)
            .map(|level| level.to_ascii_uppercase());
        let message = object
            .get("message")
            .or_else(|| object.get("text"))
            .or_else(|| object.get("args"))
            .map(format_runtime_log_value)
            .unwrap_or_else(|| format_runtime_log_value(value));
        if let Some(level) = level {
            return format!("{level} {message}");
        }
        return message;
    }
    format_runtime_log_value(value)
}

fn format_runtime_log_exception(value: &Value) -> String {
    if let Some(object) = value.as_object() {
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Exception");
        let message = object
            .get("message")
            .or_else(|| object.get("text"))
            .map(format_runtime_log_value)
            .unwrap_or_else(|| format_runtime_log_value(value));
        if message.starts_with(name) {
            message
        } else {
            format!("{name}: {message}")
        }
    } else {
        format_runtime_log_value(value)
    }
}

fn format_runtime_log_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Array(items) => items
            .iter()
            .map(format_runtime_log_value)
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        _ => serde_json::to_string(value).unwrap_or_else(|_| value.to_string()),
    }
}

fn value_to_i64(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .or_else(|| value.as_f64().map(|value| value as i64))
}

/// Reproduce a Cloud Apps build locally in Docker. See `build_reproduce` for
/// the parsing/invocation logic; this function wires CLI args and exit codes.
fn run_builds_reproduce(
    build_id: &str,
    mock_path: Option<&std::path::Path>,
    source_dir: Option<&std::path::Path>,
    image_override: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let mock_path = mock_path.ok_or_else(|| {
        anyhow!(
            "Phase 1: --mock <path> is required. \
             The build-config endpoint (PLT-913) is not yet available; \
             pass a local YAML/JSON fixture matching BuildConfig."
        )
    })?;

    let config = build_reproduce::load_mock_config(mock_path)?;
    let spec = build_reproduce::BuildSpec::parse(&config.buildspec)?;

    let owned_cwd;
    let source_dir: &std::path::Path = match source_dir {
        Some(p) => p,
        None => {
            owned_cwd = std::env::current_dir()?;
            owned_cwd.as_path()
        }
    };

    let invocation = build_reproduce::build_invocation(&config, &spec, source_dir, image_override);

    eprintln!("=== reproduce build {} ===", build_id);
    eprintln!("  config build_id: {}", config.build_id);
    eprintln!("  image:           {}", invocation.image);
    eprintln!("  source:          {}", invocation.source_dir.display());
    if !config.environment.secret_names.is_empty() {
        eprintln!(
            "  secrets (names): {}",
            config.environment.secret_names.join(", ")
        );
        eprintln!("  (secret values are not provided in mock mode)");
    }
    eprintln!();

    if dry_run {
        println!("{}", invocation.to_display_string());
        return Ok(());
    }

    let exit_code = invocation.execute()?;
    if exit_code != 0 {
        return Err(anyhow!("reproduce build exited with code {exit_code}"));
    }
    Ok(())
}

async fn run_deployments_list(api: &ApiClient, app_id: &str, json: bool) -> Result<()> {
    let resp: ListDeploymentsResponse = api
        .get(&format!("/v1/compute/apps/{app_id}/deployments"))
        .await?;
    if json {
        return print_json(&resp.deployments);
    }
    if resp.deployments.is_empty() {
        println!("No deployments found for app {app_id}");
        return Ok(());
    }
    println!(
        "{:<28}  {:<12}  {:<28}  {:<40}  CREATED AT",
        "DEPLOYMENT ID", "STATUS", "BUILD ID", "URL"
    );
    println!(
        "{:-<28}  {:-<12}  {:-<28}  {:-<40}  {:-<19}",
        "", "", "", "", ""
    );
    for dep in &resp.deployments {
        println!(
            "{:<28}  {:<12}  {:<28}  {:<40}  {}",
            dep.id,
            dep.status,
            dep.build_id.as_deref().unwrap_or("-"),
            truncate(&dep.display_url(), 40),
            dep.created_at
                .as_deref()
                .map(format_created_at)
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    Ok(())
}

async fn run_deployments_get(api: &ApiClient, deployment_id: &str, json: bool) -> Result<()> {
    let dep: DeploymentResponse = api
        .get(&format!("/v1/compute/deployments/{deployment_id}"))
        .await?;
    if json {
        return print_json(&dep);
    }
    println!("ID:       {}", dep.id);
    println!("Status:   {}", dep.status);
    println!("App ID:   {}", dep.app_id.as_deref().unwrap_or("-"));
    println!("Build ID: {}", dep.build_id.as_deref().unwrap_or("-"));
    println!("URL:      {}", dep.display_url());
    println!(
        "Created:  {}",
        dep.created_at
            .as_deref()
            .map(format_created_at)
            .unwrap_or_else(|| "-".to_string())
    );
    Ok(())
}

async fn run_deployments_rollback(
    api: &ApiClient,
    app_id: &str,
    deployment_id: &str,
) -> Result<()> {
    let req = RollbackRequest {
        deployment_id: deployment_id.to_string(),
    };
    let dep: DeploymentResponse = api
        .post(&format!("/v1/compute/apps/{app_id}/rollback"), &req)
        .await?;
    println!("Rollback initiated. New deployment: {}", dep.id);
    println!("Status: {}", dep.status);
    Ok(())
}

async fn run_env_list(api: &ApiClient, app_id: &str, json: bool) -> Result<()> {
    let resp: ListEnvVarsResponse = api.get(&format!("/v1/apps/{app_id}/env")).await?;
    if json {
        return print_json(&resp.env_vars);
    }
    if resp.env_vars.is_empty() {
        println!("No environment variables set for app {app_id}");
        return Ok(());
    }
    println!(
        "{:<28}  {:<24}  {:<11}  {:<16}  {:<8}  VALUE",
        "ID", "KEY", "TARGET", "BRANCH", "SECRET"
    );
    println!(
        "{:-<28}  {:-<24}  {:-<11}  {:-<16}  {:-<8}  {:-<40}",
        "", "", "", "", "", ""
    );
    for var in &resp.env_vars {
        let is_secret = var.is_secret.unwrap_or(false);
        let value = if is_secret {
            "****".to_string()
        } else {
            var.value.as_deref().unwrap_or("-").to_string()
        };
        println!(
            "{:<28}  {:<24}  {:<11}  {:<16}  {:<8}  {}",
            var.id,
            var.key,
            var.target.as_deref().unwrap_or("all"),
            var.branch.as_deref().unwrap_or("-"),
            if is_secret { "yes" } else { "no" },
            value,
        );
    }
    Ok(())
}

async fn run_env_set(
    api: &ApiClient,
    app_id: &str,
    vars: &[String],
    target: &str,
    branch: Option<&str>,
) -> Result<()> {
    if vars.is_empty() {
        return Err(anyhow!(
            "at least one KEY=VALUE pair is required unless --secret is used"
        ));
    }
    validate_secret_target(target)?;

    let entries: Vec<SetEnvVarEntry> = vars
        .iter()
        .map(|v| {
            let (key, value) = v
                .split_once('=')
                .ok_or_else(|| anyhow!("invalid env var format: '{v}' (expected KEY=VALUE)"))?;
            Ok(SetEnvVarEntry {
                key: key.to_string(),
                value: value.to_string(),
                target: Some(target.to_string()),
                branch: branch.filter(|b| !b.is_empty()).map(ToString::to_string),
                is_secret: None,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let req = SetEnvVarsRequest { env_vars: entries };
    let resp: ListEnvVarsResponse = api.post(&format!("/v1/apps/{app_id}/env"), &req).await?;
    println!("Set {} environment variable(s).", resp.env_vars.len());
    Ok(())
}

async fn run_env_unset_key(
    api: &ApiClient,
    app_id: &str,
    key: &str,
    target: Option<&str>,
) -> Result<()> {
    if let Some(target) = target {
        validate_secret_target(target)?;
    }
    let suffix = target
        .map(|target| format!("?target={target}"))
        .unwrap_or_default();
    api.delete(&format!("/v1/apps/{app_id}/env/{key}{suffix}"))
        .await?;
    println!("Environment variable {key} deleted.");
    Ok(())
}

async fn run_env_set_secret(
    api: &ApiClient,
    app_id: &str,
    key: &str,
    target: &str,
    config_flag: Option<&Path>,
) -> Result<()> {
    validate_secret_key(key)?;
    validate_secret_target(target)?;
    let value = read_secret_value(key)?;
    let req = SetAppSecretRequest {
        key: key.to_string(),
        value,
        target: target.to_string(),
    };
    let resp: SetAppSecretResponse = api
        .post(&format!("/v1/apps/{app_id}/secrets"), &req)
        .await?;

    update_manifest_secret_ref(config_flag, &resp.key, &resp.target)?;
    println!("Set secret {} for target {}.", resp.key, resp.target);
    println!("Updated tachyon.yml with valueFrom.secret: {}", resp.key);
    Ok(())
}

fn validate_secret_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(anyhow!("secret key must not be empty"));
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(anyhow!(
            "secret key must contain only uppercase ASCII letters, digits, and underscores"
        ));
    }
    Ok(())
}

fn validate_secret_target(target: &str) -> Result<()> {
    match target {
        "production" | "preview" | "all" => Ok(()),
        _ => Err(anyhow!(
            "invalid target '{target}' (expected production, preview, or all)"
        )),
    }
}

fn read_secret_value(key: &str) -> Result<String> {
    if let Ok(value) = std::env::var("TACHYON_SECRET_VALUE") {
        if value.is_empty() {
            return Err(anyhow!("TACHYON_SECRET_VALUE must not be empty"));
        }
        return Ok(value);
    }

    let value = Password::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Enter value for {key}"))
        .allow_empty_password(false)
        .interact()?;
    Ok(value)
}

fn update_manifest_secret_ref(config_flag: Option<&Path>, key: &str, target: &str) -> Result<()> {
    let loaded = crate::config::loader::load_with_path(config_flag)?
        .ok_or_else(|| anyhow!("tachyon.yml not found. Run `tachyon init` first."))?;
    upsert_manifest_secret_ref(&loaded.path, key, target)
}

fn upsert_manifest_secret_ref(path: &Path, key: &str, target: &str) -> Result<()> {
    let raw = std::fs::read_to_string(path)?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&raw)?;
    let kind = doc
        .get("kind")
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or("CloudApp");

    match kind {
        "CloudApp" => {
            let spec = ensure_mapping_child(&mut doc, "spec")?;
            upsert_env_var_ref(spec, key, target)?;
        }
        "CloudApps" => {
            let app_name = doc
                .get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(serde_yaml::Value::as_str)
                .map(ToString::to_string);
            let apps = doc
                .get_mut("spec")
                .and_then(|s| s.get_mut("apps"))
                .and_then(serde_yaml::Value::as_sequence_mut)
                .ok_or_else(|| anyhow!("CloudApps manifest is missing spec.apps"))?;
            let entry_index = app_name
                .as_deref()
                .and_then(|name| {
                    apps.iter().position(|app| {
                        app.get("name").and_then(serde_yaml::Value::as_str) == Some(name)
                    })
                })
                .unwrap_or(0);
            let entry = apps
                .get_mut(entry_index)
                .ok_or_else(|| anyhow!("CloudApps manifest has no apps"))?;
            let mapping = entry
                .as_mapping_mut()
                .ok_or_else(|| anyhow!("CloudApps spec.apps entry must be an object"))?;
            upsert_env_var_ref(mapping, key, target)?;
        }
        other => return Err(anyhow!("unsupported manifest kind: {other}")),
    }

    let next = serde_yaml::to_string(&doc)?;
    std::fs::write(path, next)?;
    Ok(())
}

fn ensure_mapping_child<'a>(
    value: &'a mut serde_yaml::Value,
    key: &str,
) -> Result<&'a mut serde_yaml::Mapping> {
    let mapping = value
        .as_mapping_mut()
        .ok_or_else(|| anyhow!("manifest root must be an object"))?;
    let key_value = serde_yaml::Value::String(key.to_string());
    if !mapping.contains_key(&key_value) {
        mapping.insert(
            key_value.clone(),
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
    }
    mapping
        .get_mut(&key_value)
        .and_then(serde_yaml::Value::as_mapping_mut)
        .ok_or_else(|| anyhow!("{key} must be an object"))
}

fn upsert_env_var_ref(spec: &mut serde_yaml::Mapping, key: &str, target: &str) -> Result<()> {
    let env_key = serde_yaml::Value::String("envVars".to_string());
    if !spec.contains_key(&env_key) {
        spec.insert(env_key.clone(), serde_yaml::Value::Sequence(Vec::new()));
    }
    let env_vars = spec
        .get_mut(&env_key)
        .and_then(serde_yaml::Value::as_sequence_mut)
        .ok_or_else(|| anyhow!("spec.envVars must be an array"))?;

    if let Some(existing) = env_vars
        .iter_mut()
        .find(|env| env.get("name").and_then(serde_yaml::Value::as_str) == Some(key))
    {
        let mapping = existing
            .as_mapping_mut()
            .ok_or_else(|| anyhow!("spec.envVars entries must be objects"))?;
        set_secret_env_mapping(mapping, key, target);
        return Ok(());
    }

    let mut mapping = serde_yaml::Mapping::new();
    set_secret_env_mapping(&mut mapping, key, target);
    env_vars.push(serde_yaml::Value::Mapping(mapping));
    Ok(())
}

fn set_secret_env_mapping(mapping: &mut serde_yaml::Mapping, key: &str, target: &str) {
    mapping.insert(yaml_key("name"), serde_yaml::Value::String(key.to_string()));
    mapping.insert(
        yaml_key("type"),
        serde_yaml::Value::String("credential".to_string()),
    );
    mapping.remove(yaml_key("value"));
    if target == "all" {
        mapping.remove(yaml_key("target"));
    } else {
        mapping.insert(
            yaml_key("target"),
            serde_yaml::Value::String(target.to_string()),
        );
    }

    let mut value_from = serde_yaml::Mapping::new();
    value_from.insert(
        yaml_key("secret"),
        serde_yaml::Value::String(key.to_string()),
    );
    mapping.insert(
        yaml_key("valueFrom"),
        serde_yaml::Value::Mapping(value_from),
    );
}

fn yaml_key(key: &str) -> serde_yaml::Value {
    serde_yaml::Value::String(key.to_string())
}

async fn run_env_delete(api: &ApiClient, app_id: &str, env_id: &str) -> Result<()> {
    api.delete(&format!("/v1/apps/{app_id}/env/{env_id}"))
        .await?;
    println!("Environment variable {env_id} deleted.");
    Ok(())
}

async fn run_domains_list(api: &ApiClient, app_id: &str, json: bool) -> Result<()> {
    let resp: ListDomainsResponse = api
        .get(&format!("/v1/compute/apps/{app_id}/domains"))
        .await?;
    if json {
        return print_json(&resp.domains);
    }
    if resp.domains.is_empty() {
        println!("No custom domains for app {app_id}");
        return Ok(());
    }
    println!(
        "{:<28}  {:<40}  {:<10}  {:<8}  CREATED AT",
        "ID", "DOMAIN", "STATUS", "VERIFIED"
    );
    println!(
        "{:-<28}  {:-<40}  {:-<10}  {:-<8}  {:-<19}",
        "", "", "", "", ""
    );
    for d in &resp.domains {
        println!(
            "{:<28}  {:<40}  {:<10}  {:<8}  {}",
            d.id,
            d.domain,
            d.status.as_deref().unwrap_or("-"),
            d.verified
                .map(|v| if v { "yes" } else { "no" })
                .unwrap_or("-"),
            d.created_at
                .as_deref()
                .map(format_created_at)
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    Ok(())
}

async fn run_domains_add(api: &ApiClient, app_id: &str, domain: &str) -> Result<()> {
    let req = AddDomainRequest {
        domain: domain.to_string(),
    };
    let resp: CustomDomainResponse = api
        .post(&format!("/v1/compute/apps/{app_id}/domains"), &req)
        .await?;
    println!("Domain added: {} (ID: {})", resp.domain, resp.id);
    Ok(())
}

async fn run_domains_verify(api: &ApiClient, domain_id: &str) -> Result<()> {
    api.post_no_body(&format!("/v1/compute/domains/{domain_id}/verify"))
        .await?;
    println!("Domain {domain_id} verification initiated.");
    Ok(())
}

async fn run_domains_remove(api: &ApiClient, domain_id: &str) -> Result<()> {
    api.delete(&format!("/v1/compute/domains/{domain_id}"))
        .await?;
    println!("Domain {domain_id} removed.");
    Ok(())
}

async fn run_scaling_get(api: &ApiClient, app_id: &str, json: bool) -> Result<()> {
    // Scaling info is part of app details; fetch app and display scaling-relevant fields
    let app: serde_json::Value = api.get(&format!("/v1/compute/apps/{app_id}")).await?;
    if json {
        return print_json(&app);
    }
    println!("App ID: {app_id}");
    if let Some(scaling) = app.get("scaling") {
        println!(
            "Min instances: {}",
            scaling
                .get("min_instances")
                .and_then(|v| v.as_i64())
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string())
        );
        println!(
            "Max instances: {}",
            scaling
                .get("max_instances")
                .and_then(|v| v.as_i64())
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string())
        );
    } else {
        println!("No scaling configuration found.");
    }
    Ok(())
}

async fn run_scaling_update(
    api: &ApiClient,
    app_id: &str,
    min_instances: Option<i32>,
    max_instances: Option<i32>,
) -> Result<()> {
    if min_instances.is_none() && max_instances.is_none() {
        return Err(anyhow!(
            "at least one of --min-instances or --max-instances is required"
        ));
    }
    let req = UpdateScalingRequest {
        min_instances,
        max_instances,
    };
    let resp: ScalingConfigResponse = api
        .patch(&format!("/v1/compute/apps/{app_id}/scaling"), &req)
        .await?;
    println!("Scaling updated.");
    if let Some(min) = resp.min_instances {
        println!("Min instances: {min}");
    }
    if let Some(max) = resp.max_instances {
        println!("Max instances: {max}");
    }
    Ok(())
}

// ---- Entry point ----

fn app_id_or_default<'a>(
    app_id: &'a Option<String>,
    project_config: Option<&'a ProjectConfig>,
) -> Result<&'a str> {
    app_id_or_default_value(app_id.as_deref(), project_config)
}

fn app_id_or_default_value<'a>(
    app_id: Option<&'a str>,
    project_config: Option<&'a ProjectConfig>,
) -> Result<&'a str> {
    app_id
        .or_else(|| {
            project_config
                .and_then(|config| config.metadata.name.as_deref())
                .filter(|name| !name.is_empty())
        })
        .ok_or_else(|| anyhow!("app_id is required (or set metadata.name in tachyon.yml)"))
}

pub async fn run(
    args: &ComputeArgs,
    config: &Configuration,
    tenant_id: &str,
    project_config: Option<&ProjectConfig>,
    config_flag: Option<&Path>,
) -> Result<()> {
    // Local-only commands (no API call needed)
    match &args.command {
        ComputeCommand::Build {
            app,
            deploy,
            project_dir,
        } => return run_local_build(app, project_dir.as_ref(), *deploy),
        ComputeCommand::Dev {
            app,
            project_dir,
            port,
        } => return run_local_dev(app, project_dir.as_ref(), *port),
        _ => {}
    }

    let api = ApiClient::new(config, tenant_id)?;

    match &args.command {
        ComputeCommand::Build { .. } | ComputeCommand::Dev { .. } => {
            unreachable!()
        }
        ComputeCommand::Apps { command } => match command {
            AppsCommand::List { json } => run_apps_list(&api, *json).await,
            AppsCommand::Get { app_id, json } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_apps_get(&api, &id, *json).await
            }
            AppsCommand::Delete { app_id } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_apps_delete(&api, &id).await
            }
            AppsCommand::Apply {
                file,
                app,
                environment,
                dry_run,
            } => run_apps_apply(&api, file, app.as_deref(), environment, *dry_run).await,
            AppsCommand::Feedback(feedback_args) => {
                let id = resolve::resolve_app_id(&api, &feedback_args.app_id).await?;
                run_apps_feedback(tenant_id, &id, feedback_args)
            }
        },
        ComputeCommand::Builds { command } => match command {
            BuildsCommand::List {
                app_id,
                limit,
                json,
            } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_builds_list(&api, &id, *limit, *json).await
            }
            BuildsCommand::Get { build_id, json } => run_builds_get(&api, build_id, *json).await,
            BuildsCommand::Trigger { app_id, branch } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_builds_trigger(&api, &id, branch.as_deref()).await
            }
            BuildsCommand::Cancel { build_id } => run_builds_cancel(&api, build_id).await,
            BuildsCommand::Logs {
                app_id,
                build_id,
                follow,
                agent,
            } => {
                let resolved_app_id = match app_id {
                    Some(id) => Some(resolve::resolve_app_id(&api, id).await?),
                    None if build_id.is_none() => {
                        let app_id = app_id_or_default(app_id, project_config)?;
                        Some(resolve::resolve_app_id(&api, app_id).await?)
                    }
                    None => None,
                };
                run_builds_logs(
                    &api,
                    resolved_app_id.as_deref(),
                    build_id.as_deref(),
                    *follow,
                    *agent,
                )
                .await
            }
            BuildsCommand::Watch {
                app_id,
                build_id,
                interval_secs,
                timeout_secs,
                no_logs,
                agent,
            } => {
                let resolved_app_id = match app_id {
                    Some(id) => Some(resolve::resolve_app_id(&api, id).await?),
                    None if build_id.is_none() => {
                        let app_id = app_id_or_default(app_id, project_config)?;
                        Some(resolve::resolve_app_id(&api, app_id).await?)
                    }
                    None => None,
                };
                run_builds_watch(
                    &api,
                    resolved_app_id.as_deref(),
                    build_id.as_deref(),
                    *interval_secs,
                    *timeout_secs,
                    *no_logs,
                    *agent,
                )
                .await
            }
            BuildsCommand::Reproduce {
                build_id,
                mock,
                source_dir,
                image,
                dry_run,
            } => run_builds_reproduce(
                build_id,
                mock.as_deref(),
                source_dir.as_deref(),
                image.as_deref(),
                *dry_run,
            ),
            BuildsCommand::RunJob { spec_env } => {
                crate::cloud_app_build_job::run_from_env(spec_env).await
            }
        },
        ComputeCommand::Deployments { command } => match command {
            DeploymentsCommand::List { app_id, json } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_deployments_list(&api, &id, *json).await
            }
            DeploymentsCommand::Get {
                deployment_id,
                json,
            } => run_deployments_get(&api, deployment_id, *json).await,
            DeploymentsCommand::Rollback {
                app_id,
                deployment_id,
            } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_deployments_rollback(&api, &id, deployment_id).await
            }
        },
        ComputeCommand::Env { command } => match command {
            EnvCommand::List { app_id, json } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_env_list(&api, &id, *json).await
            }
            EnvCommand::Set {
                app_id,
                app,
                secret,
                target,
                branch,
                vars,
            } => {
                let mut vars = vars.clone();
                let positional_app = app_id.as_ref().and_then(|value| {
                    if value.contains('=') {
                        vars.insert(0, value.clone());
                        None
                    } else {
                        Some(value)
                    }
                });
                let selected_app = app.as_ref().or(positional_app);
                let app_id =
                    app_id_or_default_value(selected_app.map(String::as_str), project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                if let Some(key) = secret {
                    if !vars.is_empty() {
                        return Err(anyhow!("KEY=VALUE arguments cannot be used with --secret"));
                    }
                    run_env_set_secret(&api, &id, key, target, config_flag).await
                } else {
                    run_env_set(&api, &id, &vars, target, branch.as_deref()).await
                }
            }
            EnvCommand::Unset { app, target, args } => {
                let (positional_app, key) = match args.as_slice() {
                    [key] => (None, key),
                    [app_id, key] => (Some(app_id), key),
                    _ => unreachable!("clap enforces one or two unset args"),
                };
                let selected_app = app.as_ref().or(positional_app);
                let app_id =
                    app_id_or_default_value(selected_app.map(String::as_str), project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_env_unset_key(&api, &id, key, target.as_deref()).await
            }
            EnvCommand::Delete { app_id, env_id } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_env_delete(&api, &id, env_id).await
            }
        },
        ComputeCommand::Domains { command } => match command {
            DomainsCommand::List { app_id, json } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_domains_list(&api, &id, *json).await
            }
            DomainsCommand::Add { app_id, domain } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_domains_add(&api, &id, domain).await
            }
            DomainsCommand::Verify { domain_id } => run_domains_verify(&api, domain_id).await,
            DomainsCommand::Remove { domain_id } => run_domains_remove(&api, domain_id).await,
        },
        ComputeCommand::Scaling { command } => match command {
            ScalingCommand::Get { app_id, json } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_scaling_get(&api, &id, *json).await
            }
            ScalingCommand::Update {
                app_id,
                min_instances,
                max_instances,
            } => {
                let app_id = app_id_or_default(app_id, project_config)?;
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_scaling_update(&api, &id, *min_instances, *max_instances).await
            }
        },
        // Legacy shortcuts
        ComputeCommand::Status { app_id, limit } => {
            let app_id = app_id_or_default(app_id, project_config)?;
            let id = resolve::resolve_app_id(&api, app_id).await?;
            run_builds_list(&api, &id, *limit, false).await
        }
        ComputeCommand::Logs {
            app_id,
            tail,
            build_id,
            follow,
            agent,
            json,
        } => {
            if let Some(app_id) = tail {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                return run_runtime_log_tail(&api, &id, RuntimeLogTailOptions { raw_json: *json })
                    .await;
            }
            if *json {
                return Err(anyhow!("--json is only supported with compute logs --tail"));
            }
            let resolved_app_id = match app_id {
                Some(id) => Some(resolve::resolve_app_id(&api, id).await?),
                None => None,
            };
            if resolved_app_id.is_none() && build_id.is_none() {
                return Err(anyhow!("either app_id or --build-id must be provided"));
            }
            run_builds_logs(
                &api,
                resolved_app_id.as_deref(),
                build_id.as_deref(),
                *follow,
                *agent,
            )
            .await
        }
    }
}

pub async fn run_env(
    args: &EnvArgs,
    config: &Configuration,
    tenant_id: &str,
    project_config: Option<&ProjectConfig>,
    config_flag: Option<&Path>,
) -> Result<()> {
    let compute_args = ComputeArgs {
        command: ComputeCommand::Env {
            command: args.command.clone(),
        },
    };
    run(
        &compute_args,
        config,
        tenant_id,
        project_config,
        config_flag,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_tail_formatter_summarizes_cloudflare_request_logs_and_exceptions() {
        let raw = r#"{
            "eventTimestamp": 1767273332000,
            "scriptName": "tachyon-console",
            "cpuTime": 86,
            "request": {
                "method": "GET",
                "url": "https://example.com/api/reservations?limit=1"
            },
            "response": { "status": 200 },
            "logs": [
                { "level": "log", "message": ["reservation", 42] }
            ],
            "exceptions": [
                { "name": "TypeError", "message": "Cannot read property 'id' of undefined" }
            ]
        }"#;

        let lines = format_runtime_log_lines(raw);

        assert_eq!(
            lines,
            vec![
                "[13:15:32] GET /api/reservations?limit=1 200  (cpu: 86ms)  source=tachyon-console",
                "[13:15:32] ERROR TypeError: Cannot read property 'id' of undefined",
                "[13:15:32] LOG reservation 42",
            ]
        );
    }

    #[test]
    fn runtime_tail_formatter_falls_back_for_unknown_payloads() {
        let json_lines = format_runtime_log_lines(r#"{"unexpected":{"nested":true}}"#);
        assert_eq!(json_lines.len(), 1);
        assert!(json_lines[0].starts_with('['));
        assert!(json_lines[0].contains(r#""unexpected":{"nested":true}"#));

        assert_eq!(
            format_runtime_log_lines("not json"),
            vec!["not json".to_string()]
        );
    }

    #[test]
    fn runtime_tail_formatter_includes_top_level_level_message_and_source() {
        let lines = format_runtime_log_lines(
            r#"{"timestamp":"2026-01-01T20:15:40Z","level":"error","message":"boom","source":"worker"}"#,
        );

        assert_eq!(lines, vec!["[20:15:40] ERROR boom source=worker"]);
    }

    #[test]
    fn runtime_tail_sse_parser_handles_multiline_data_and_crlf() {
        let mut pending =
            "event: log\r\ndata: {\"message\":\"a\"}\r\ndata: {\"message\":\"b\"}\r\n\r\n"
                .to_string();

        let events = drain_sse_events(&mut pending);

        assert!(pending.is_empty());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("log"));
        assert_eq!(
            events[0].data,
            vec![
                "{\"message\":\"a\"}".to_string(),
                "{\"message\":\"b\"}".to_string(),
            ]
        );
    }

    #[test]
    fn feedback_payload_includes_cloud_app_context() {
        let args = FeedbackArgs {
            app_id: "app_01test".to_string(),
            message: "The production page returns 500.".to_string(),
            kind: FeedbackKind::Bug,
            severity: FeedbackSeverity::High,
            url: Some("https://example.txcloud.app".to_string()),
            build_id: Some("bld_01test".to_string()),
            deployment_id: Some("dep_01test".to_string()),
            contact: Some("user@example.com".to_string()),
            metadata: vec!["browser=Chrome".to_string()],
            json: false,
        };

        let payload = build_feedback_payload("tn_01test", "app_01resolved", &args).unwrap();

        assert_eq!(payload.app_id, "app_01resolved");
        assert_eq!(payload.operator_id, "tn_01test");
        assert_eq!(payload.kind, FeedbackKind::Bug);
        assert_eq!(payload.severity, FeedbackSeverity::High);
        assert_eq!(
            payload.metadata.get("browser").map(String::as_str),
            Some("Chrome")
        );
    }

    #[test]
    fn feedback_metadata_rejects_secret_like_keys() {
        let err = parse_feedback_metadata(&["api_key=secret".to_string()]).unwrap_err();

        assert!(err.to_string().contains("secret-like"), "{err}");
    }

    #[test]
    fn feedback_markdown_formats_context() {
        let payload = FeedbackPayload {
            app_id: "app_01test".to_string(),
            operator_id: "tn_01test".to_string(),
            kind: FeedbackKind::Feature,
            severity: FeedbackSeverity::Medium,
            message: "Please add CSV export.".to_string(),
            url: None,
            build_id: None,
            deployment_id: None,
            contact: None,
            metadata: BTreeMap::from([("browser".to_string(), "Safari".to_string())]),
            created_at: "2026-05-29T00:00:00+00:00".to_string(),
        };

        let markdown = render_feedback_markdown(&payload);

        assert!(markdown.contains("# Cloud App Feedback"));
        assert!(markdown.contains("- App ID: app_01test"));
        assert!(markdown.contains("- Kind: feature"));
        assert!(markdown.contains("Please add CSV export."));
        assert!(markdown.contains("  - browser: Safari"));
    }
}
