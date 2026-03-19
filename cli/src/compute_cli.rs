use anyhow::{anyhow, Result};
use chrono::{DateTime, TimeZone, Utc};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;
use tokio::time::{sleep, Duration};

use crate::client::{print_json, truncate, ApiClient};
use crate::resolve;

#[derive(Debug, Clone, Args)]
pub struct ComputeArgs {
    #[command(subcommand)]
    pub command: ComputeCommand,
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
    /// Show build status for a compute app (shortcut for builds list)
    Status {
        /// App ID or name
        app_id: String,
        /// Maximum number of builds to display
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Stream or fetch build logs (shortcut for builds logs)
    Logs {
        /// App ID or name
        app_id: String,
        /// Build ID (defaults to the latest build)
        #[arg(long)]
        build_id: Option<String>,
        /// Keep polling until the build is complete
        #[arg(long)]
        follow: bool,
    },
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
        app_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Delete a compute app
    Delete {
        /// App ID or name
        app_id: String,
    },
}

// --- Builds subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum BuildsCommand {
    /// List builds for an app
    List {
        /// App ID or name
        app_id: String,
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
        app_id: String,
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
        app_id: String,
        /// Build ID (defaults to the latest build)
        #[arg(long)]
        build_id: Option<String>,
        /// Keep polling until the build is complete
        #[arg(long)]
        follow: bool,
    },
}

// --- Deployments subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum DeploymentsCommand {
    /// List deployments for an app
    List {
        /// App ID or name
        app_id: String,
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
        app_id: String,
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
        app_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Set environment variables for an app
    Set {
        /// App ID or name
        app_id: String,
        /// Variables in KEY=VALUE format
        #[arg(required = true, num_args = 1..)]
        vars: Vec<String>,
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
        app_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a custom domain
    Add {
        /// App ID or name
        app_id: String,
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
        app_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Update scaling configuration
    Update {
        /// App ID or name
        app_id: String,
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
    status: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
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
    is_secret: Option<bool>,
}

#[derive(Debug, Serialize)]
struct SetEnvVarsRequest {
    env_vars: Vec<SetEnvVarEntry>,
}

#[derive(Debug, Serialize)]
struct SetEnvVarEntry {
    key: String,
    value: String,
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
    app_id: &str,
    build_id: Option<&str>,
    follow: bool,
) -> Result<()> {
    let resolved_build_id = match build_id {
        Some(id) => id.to_string(),
        None => {
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
    loop {
        let path = format!("/v1/compute/builds/{resolved_build_id}/logs");
        let logs: BuildLogsResponse = if let Some(token) = &next_token {
            api.get_query(&path, &[("next_token", token.as_str())])
                .await?
        } else {
            api.get(&path).await?
        };

        for line in &logs.lines {
            println!("[{}] {}", format_timestamp_ms(line.timestamp), line.message);
        }

        if logs.is_complete {
            break;
        }
        if logs.next_token.is_some() {
            next_token = logs.next_token;
        }
        if follow {
            sleep(Duration::from_secs(2)).await;
        } else {
            break;
        }
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
            truncate(dep.url.as_deref().unwrap_or("-"), 40),
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
    println!("URL:      {}", dep.url.as_deref().unwrap_or("-"));
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
    let resp: ListEnvVarsResponse = api.get(&format!("/v1/compute/apps/{app_id}/env")).await?;
    if json {
        return print_json(&resp.env_vars);
    }
    if resp.env_vars.is_empty() {
        println!("No environment variables set for app {app_id}");
        return Ok(());
    }
    println!("{:<28}  {:<24}  {:<8}  VALUE", "ID", "KEY", "SECRET");
    println!("{:-<28}  {:-<24}  {:-<8}  {:-<40}", "", "", "", "");
    for var in &resp.env_vars {
        let is_secret = var.is_secret.unwrap_or(false);
        let value = if is_secret {
            "********".to_string()
        } else {
            var.value.as_deref().unwrap_or("-").to_string()
        };
        println!(
            "{:<28}  {:<24}  {:<8}  {}",
            var.id,
            var.key,
            if is_secret { "yes" } else { "no" },
            value,
        );
    }
    Ok(())
}

async fn run_env_set(api: &ApiClient, app_id: &str, vars: &[String]) -> Result<()> {
    let entries: Vec<SetEnvVarEntry> = vars
        .iter()
        .map(|v| {
            let (key, value) = v
                .split_once('=')
                .ok_or_else(|| anyhow!("invalid env var format: '{v}' (expected KEY=VALUE)"))?;
            Ok(SetEnvVarEntry {
                key: key.to_string(),
                value: value.to_string(),
                is_secret: None,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let req = SetEnvVarsRequest { env_vars: entries };
    let resp: ListEnvVarsResponse = api
        .put(&format!("/v1/compute/apps/{app_id}/env"), &req)
        .await?;
    println!("Set {} environment variable(s).", resp.env_vars.len());
    Ok(())
}

async fn run_env_delete(api: &ApiClient, app_id: &str, env_id: &str) -> Result<()> {
    api.delete(&format!("/v1/compute/apps/{app_id}/env/{env_id}"))
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

pub async fn run(args: &ComputeArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    match &args.command {
        ComputeCommand::Apps { command } => match command {
            AppsCommand::List { json } => run_apps_list(&api, *json).await,
            AppsCommand::Get { app_id, json } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_apps_get(&api, &id, *json).await
            }
            AppsCommand::Delete { app_id } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_apps_delete(&api, &id).await
            }
        },
        ComputeCommand::Builds { command } => match command {
            BuildsCommand::List {
                app_id,
                limit,
                json,
            } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_builds_list(&api, &id, *limit, *json).await
            }
            BuildsCommand::Get { build_id, json } => run_builds_get(&api, build_id, *json).await,
            BuildsCommand::Trigger { app_id, branch } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_builds_trigger(&api, &id, branch.as_deref()).await
            }
            BuildsCommand::Cancel { build_id } => run_builds_cancel(&api, build_id).await,
            BuildsCommand::Logs {
                app_id,
                build_id,
                follow,
            } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_builds_logs(&api, &id, build_id.as_deref(), *follow).await
            }
        },
        ComputeCommand::Deployments { command } => match command {
            DeploymentsCommand::List { app_id, json } => {
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
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_deployments_rollback(&api, &id, deployment_id).await
            }
        },
        ComputeCommand::Env { command } => match command {
            EnvCommand::List { app_id, json } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_env_list(&api, &id, *json).await
            }
            EnvCommand::Set { app_id, vars } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_env_set(&api, &id, vars).await
            }
            EnvCommand::Delete { app_id, env_id } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_env_delete(&api, &id, env_id).await
            }
        },
        ComputeCommand::Domains { command } => match command {
            DomainsCommand::List { app_id, json } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_domains_list(&api, &id, *json).await
            }
            DomainsCommand::Add { app_id, domain } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_domains_add(&api, &id, domain).await
            }
            DomainsCommand::Verify { domain_id } => run_domains_verify(&api, domain_id).await,
            DomainsCommand::Remove { domain_id } => run_domains_remove(&api, domain_id).await,
        },
        ComputeCommand::Scaling { command } => match command {
            ScalingCommand::Get { app_id, json } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_scaling_get(&api, &id, *json).await
            }
            ScalingCommand::Update {
                app_id,
                min_instances,
                max_instances,
            } => {
                let id = resolve::resolve_app_id(&api, app_id).await?;
                run_scaling_update(&api, &id, *min_instances, *max_instances).await
            }
        },
        // Legacy shortcuts
        ComputeCommand::Status { app_id, limit } => {
            let id = resolve::resolve_app_id(&api, app_id).await?;
            run_builds_list(&api, &id, *limit, false).await
        }
        ComputeCommand::Logs {
            app_id,
            build_id,
            follow,
        } => {
            let id = resolve::resolve_app_id(&api, app_id).await?;
            run_builds_logs(&api, &id, build_id.as_deref(), *follow).await
        }
    }
}
