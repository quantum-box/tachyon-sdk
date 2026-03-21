use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Deserialize;
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{api_url, build_client, get_json, get_json_with_query};

#[derive(Debug, Clone, Args)]
pub struct OpsArgs {
    #[command(subcommand)]
    pub command: OpsCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum OpsCommand {
    /// Manage deployments
    Deployments(DeploymentsArgs),
    /// Manage tool jobs
    #[command(name = "tool-jobs")]
    ToolJobs(ToolJobsArgs),
}

// ── Deployments ──

#[derive(Debug, Clone, Args)]
pub struct DeploymentsArgs {
    #[command(subcommand)]
    pub command: DeploymentsCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum DeploymentsCommand {
    /// List deployments
    List {
        /// Maximum number of deployments
        #[arg(long, default_value_t = 20)]
        limit: i64,
        /// Offset for pagination
        #[arg(long, default_value_t = 0)]
        offset: i64,
    },
}

#[derive(Debug, Deserialize)]
struct ListDeploymentsResponse {
    deployments: Vec<DeploymentSummary>,
    #[allow(dead_code)]
    total: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct DeploymentSummary {
    id: String,
    service_name: Option<String>,
    version: Option<String>,
    environment: Option<String>,
    status: String,
    ci_branch: Option<String>,
    #[allow(dead_code)]
    started_at: Option<String>,
    #[allow(dead_code)]
    completed_at: Option<String>,
}

// ── Tool Jobs ──

#[derive(Debug, Clone, Args)]
pub struct ToolJobsArgs {
    #[command(subcommand)]
    pub command: ToolJobsCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ToolJobsCommand {
    /// List tool jobs
    List,
}

#[derive(Debug, Deserialize)]
struct ToolJobListResponse {
    jobs: Vec<ToolJobItem>,
}

#[derive(Debug, Deserialize)]
struct ToolJobItem {
    id: String,
    provider: String,
    status: String,
    prompt: String,
    created_at: String,
    #[allow(dead_code)]
    updated_at: String,
}

// ── Runners ──

async fn run_deployments_list(
    config: &Configuration,
    tenant_id: &str,
    limit: i64,
    offset: i64,
) -> Result<()> {
    let client = build_client(config, tenant_id)?;
    let url = api_url(config, "/v1/ops/deployments");

    let resp: ListDeploymentsResponse = get_json_with_query(
        &client,
        &url,
        &[("limit", limit.to_string()), ("offset", offset.to_string())],
    )
    .await?;

    if resp.deployments.is_empty() {
        println!("No deployments found.");
        return Ok(());
    }

    let id_w = 30;
    let svc_w = 20;
    let ver_w = 10;
    let env_w = 12;
    let status_w = 12;
    let branch_w = 20;

    println!(
        "{:<id_w$}  {:<svc_w$}  {:<ver_w$}  {:<env_w$}  {:<status_w$}  {:<branch_w$}",
        "ID", "SERVICE", "VERSION", "ENV", "STATUS", "BRANCH",
    );
    println!(
        "{:-<id_w$}  {:-<svc_w$}  {:-<ver_w$}  {:-<env_w$}  {:-<status_w$}  {:-<branch_w$}",
        "", "", "", "", "", "",
    );

    for d in &resp.deployments {
        println!(
            "{:<id_w$}  {:<svc_w$}  {:<ver_w$}  {:<env_w$}  {:<status_w$}  {:<branch_w$}",
            d.id,
            d.service_name.as_deref().unwrap_or("-"),
            d.version.as_deref().unwrap_or("-"),
            d.environment.as_deref().unwrap_or("-"),
            d.status,
            d.ci_branch.as_deref().unwrap_or("-"),
        );
    }

    if let Some(total) = resp.total {
        println!("\nTotal: {total}");
    }

    Ok(())
}

async fn run_tool_jobs_list(config: &Configuration, tenant_id: &str) -> Result<()> {
    let client = build_client(config, tenant_id)?;
    let url = api_url(config, "/v1/agent/tool-jobs");

    let resp: ToolJobListResponse = get_json(&client, &url).await?;

    if resp.jobs.is_empty() {
        println!("No tool jobs found.");
        return Ok(());
    }

    let id_w = 30;
    let provider_w = 12;
    let status_w = 12;
    let prompt_w = 40;
    let created_w = 19;

    println!(
        "{:<id_w$}  {:<provider_w$}  {:<status_w$}  {:<prompt_w$}  {:<created_w$}",
        "ID", "PROVIDER", "STATUS", "PROMPT", "CREATED AT",
    );
    println!(
        "{:-<id_w$}  {:-<provider_w$}  {:-<status_w$}  {:-<prompt_w$}  {:-<created_w$}",
        "", "", "", "", "",
    );

    for j in &resp.jobs {
        let prompt_display = if j.prompt.chars().count() > prompt_w {
            let truncated: String = j.prompt.chars().take(prompt_w - 3).collect();
            format!("{truncated}...")
        } else {
            j.prompt.clone()
        };
        println!(
            "{:<id_w$}  {:<provider_w$}  {:<status_w$}  {:<prompt_w$}  {:<created_w$}",
            j.id,
            j.provider,
            j.status,
            prompt_display,
            &j.created_at[..19.min(j.created_at.len())],
        );
    }

    Ok(())
}

pub async fn run(args: &OpsArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    match &args.command {
        OpsCommand::Deployments(d) => match &d.command {
            DeploymentsCommand::List { limit, offset } => {
                run_deployments_list(config, tenant_id, *limit, *offset).await
            }
        },
        OpsCommand::ToolJobs(tj) => match &tj.command {
            ToolJobsCommand::List => run_tool_jobs_list(config, tenant_id).await,
        },
    }
}
