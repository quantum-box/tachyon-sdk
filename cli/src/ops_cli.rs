use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, truncate, ApiClient};

#[derive(Debug, Clone, Args)]
pub struct OpsArgs {
    #[command(subcommand)]
    pub command: OpsCommand,
}

#[derive(Debug, Clone, Subcommand)]
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
pub enum ToolJobsCommand {
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
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    job_id: Option<String>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ToolJobProviderResponse {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    tools: Option<Vec<String>>,
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
        "{:<28}  {:<20}  {:<12}  {:<12}  {:<16}  {}",
        "ID", "SERVICE", "ENVIRONMENT", "STATUS", "VERSION", "CREATED AT"
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
        "{:<28}  {:<10}  {:<8}  {:<8}  {:<8}  {}",
        "RUN ID", "STATUS", "TOTAL", "PASSED", "FAILED", "CREATED AT"
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

async fn run_tool_jobs_list(api: &ApiClient, json: bool) -> Result<()> {
    let jobs: Vec<ToolJobResponse> = api.get("/v1/agent/tool-jobs").await?;
    if json {
        return print_json(&jobs);
    }
    if jobs.is_empty() {
        println!("No tool jobs found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<16}  {:<12}  {:<20}  {}",
        "JOB ID", "PROVIDER", "STATUS", "TOOL", "CREATED AT"
    );
    println!(
        "{:-<28}  {:-<16}  {:-<12}  {:-<20}  {:-<19}",
        "", "", "", "", ""
    );
    for j in &jobs {
        let id = j.job_id.as_deref().or(j.id.as_deref()).unwrap_or("-");
        println!(
            "{:<28}  {:<16}  {:<12}  {:<20}  {}",
            id,
            j.provider.as_deref().unwrap_or("-"),
            j.status.as_deref().unwrap_or("-"),
            truncate(j.tool_name.as_deref().unwrap_or("-"), 20),
            j.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_tool_jobs_get(api: &ApiClient, job_id: &str, json: bool) -> Result<()> {
    let j: serde_json::Value = api.get(&format!("/v1/agent/tool-jobs/{job_id}")).await?;
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
    let providers: Vec<ToolJobProviderResponse> = api.get("/v1/agent/tool-jobs/providers").await?;
    if json {
        return print_json(&providers);
    }
    if providers.is_empty() {
        println!("No providers found.");
        return Ok(());
    }
    for p in &providers {
        println!("Provider: {}", p.name.as_deref().unwrap_or("-"));
        println!("  Description: {}", p.description.as_deref().unwrap_or("-"));
        if let Some(tools) = &p.tools {
            println!("  Tools: {}", tools.join(", "));
        }
        println!();
    }
    Ok(())
}

// ---- Entry point ----

pub async fn run(args: &OpsArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
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
            ToolJobsCommand::List { json } => run_tool_jobs_list(&api, *json).await,
            ToolJobsCommand::Get { job_id, json } => run_tool_jobs_get(&api, job_id, *json).await,
            ToolJobsCommand::Cancel { job_id } => run_tool_jobs_cancel(&api, job_id).await,
            ToolJobsCommand::Providers { json } => run_tool_jobs_providers(&api, *json).await,
        },
    }
}
