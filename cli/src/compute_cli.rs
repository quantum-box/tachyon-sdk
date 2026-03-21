use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use clap::{Args, Subcommand};
use reqwest::Client;
use serde::Deserialize;
use tachyon_sdk::apis::configuration::Configuration;
use tokio::time::{sleep, Duration};

use crate::client::{api_url, build_client, get_json};

#[derive(Debug, Clone, Args)]
pub struct ComputeArgs {
    #[command(subcommand)]
    pub command: ComputeCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ComputeCommand {
    /// Show build status for a compute app
    Status {
        /// App ID to show builds for
        app_id: String,
        /// Maximum number of builds to display
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Stream or fetch build logs for a compute app
    Logs {
        /// App ID to fetch logs for
        app_id: String,
        /// Build ID (defaults to the latest build)
        #[arg(long)]
        build_id: Option<String>,
        /// Keep polling until the build is complete
        #[arg(long)]
        follow: bool,
    },
}

#[derive(Debug, Deserialize)]
struct ListBuildsResponse {
    builds: Vec<BuildResponse>,
}

#[derive(Debug, Deserialize)]
struct BuildResponse {
    id: String,
    #[allow(dead_code)]
    app_id: String,
    #[allow(dead_code)]
    trigger: String,
    source_branch: String,
    commit_sha: String,
    #[allow(dead_code)]
    commit_message: Option<String>,
    status: String,
    #[allow(dead_code)]
    duration_secs: Option<i32>,
    #[allow(dead_code)]
    error_message: Option<String>,
    created_at: String,
    #[allow(dead_code)]
    updated_at: String,
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

async fn fetch_builds(
    client: &Client,
    config: &Configuration,
    app_id: &str,
) -> Result<Vec<BuildResponse>> {
    let url = format!("{}/v1/compute/apps/{}/builds", api_url(config, ""), app_id);
    let list: ListBuildsResponse = get_json(client, &url).await?;
    Ok(list.builds)
}

async fn fetch_build_logs(
    client: &Client,
    config: &Configuration,
    build_id: &str,
    next_token: Option<&str>,
) -> Result<BuildLogsResponse> {
    let url = format!(
        "{}/v1/compute/builds/{}/logs",
        api_url(config, ""),
        build_id
    );
    if let Some(token) = next_token {
        crate::client::get_json_with_query(client, &url, &[("next_token", token)]).await
    } else {
        get_json(client, &url).await
    }
}

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

async fn run_status(
    config: &Configuration,
    tenant_id: &str,
    app_id: &str,
    limit: usize,
) -> Result<()> {
    let client = build_client(config, tenant_id)?;
    let builds = fetch_builds(&client, config, app_id)
        .await
        .with_context(|| format!("failed to fetch builds for app {app_id}"))?;

    if builds.is_empty() {
        println!("No builds found for app {app_id}");
        return Ok(());
    }

    let builds_to_show = &builds[..builds.len().min(limit)];

    let id_width = 26;
    let status_width = 11;
    let branch_width = 20;
    let commit_width = 8;
    let created_width = 19;

    println!(
        "{:<id_width$}  {:<status_width$}  {:<branch_width$}  {:<commit_width$}  {:<created_width$}",
        "BUILD ID",
        "STATUS",
        "BRANCH",
        "COMMIT",
        "CREATED AT",
        id_width = id_width,
        status_width = status_width,
        branch_width = branch_width,
        commit_width = commit_width,
        created_width = created_width,
    );
    println!(
        "{:-<id_width$}  {:-<status_width$}  {:-<branch_width$}  {:-<commit_width$}  {:-<created_width$}",
        "",
        "",
        "",
        "",
        "",
        id_width = id_width,
        status_width = status_width,
        branch_width = branch_width,
        commit_width = commit_width,
        created_width = created_width,
    );

    for build in builds_to_show {
        let branch = if build.source_branch.chars().count() > branch_width {
            let truncated: String = build.source_branch.chars().take(branch_width - 3).collect();
            format!("{truncated}...")
        } else {
            build.source_branch.clone()
        };
        println!(
            "{:<id_width$}  {:<status_width$}  {:<branch_width$}  {:<commit_width$}  {:<created_width$}",
            build.id,
            build.status,
            branch,
            truncate_sha(&build.commit_sha),
            format_created_at(&build.created_at),
            id_width = id_width,
            status_width = status_width,
            branch_width = branch_width,
            commit_width = commit_width,
            created_width = created_width,
        );
    }

    Ok(())
}

async fn run_logs(
    config: &Configuration,
    tenant_id: &str,
    app_id: &str,
    build_id: Option<&str>,
    follow: bool,
) -> Result<()> {
    let client = build_client(config, tenant_id)?;

    let resolved_build_id = match build_id {
        Some(id) => id.to_string(),
        None => {
            let builds = fetch_builds(&client, config, app_id)
                .await
                .with_context(|| format!("failed to fetch builds for app {app_id}"))?;
            let latest = builds
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("no builds found for app {app_id}"))?;
            latest.id
        }
    };

    let mut next_token: Option<String> = None;

    loop {
        let logs = fetch_build_logs(&client, config, &resolved_build_id, next_token.as_deref())
            .await
            .with_context(|| format!("failed to fetch logs for build {resolved_build_id}"))?;

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

pub async fn run(args: &ComputeArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    match &args.command {
        ComputeCommand::Status { app_id, limit } => {
            run_status(config, tenant_id, app_id, *limit).await
        }
        ComputeCommand::Logs {
            app_id,
            build_id,
            follow,
        } => run_logs(config, tenant_id, app_id, build_id.as_deref(), *follow).await,
    }
}
