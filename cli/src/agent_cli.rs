use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Deserialize;
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{api_url, build_client, get_json, get_json_with_query};

#[derive(Debug, Clone, Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AgentCommand {
    /// List agent sessions
    Sessions(SessionsArgs),
    /// List available LLM models
    Models {
        /// Filter by supported feature
        #[arg(long)]
        feature: Option<String>,
    },
}

// ── Sessions ──

#[derive(Debug, Clone, Args)]
pub struct SessionsArgs {
    #[command(subcommand)]
    pub command: SessionsCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum SessionsCommand {
    /// List agent sessions
    List {
        /// Maximum number of sessions
        #[arg(long, default_value_t = 20)]
        limit: i64,
        /// Offset for pagination
        #[arg(long, default_value_t = 0)]
        offset: i64,
    },
}

#[derive(Debug, Deserialize)]
struct SessionListResponse {
    sessions: Vec<SessionEntry>,
}

#[derive(Debug, Deserialize)]
struct SessionEntry {
    id: String,
    name: Option<String>,
    created_at: String,
    updated_at: String,
}

// ── Models ──

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    models: Vec<ModelInfo>,
    #[allow(dead_code)]
    total_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
    #[allow(dead_code)]
    name: String,
    provider: String,
    #[allow(dead_code)]
    description: Option<String>,
    context_window: Option<i32>,
    max_output_tokens: Option<i32>,
    supported_features: Option<Vec<String>>,
}

// ── Runners ──

async fn run_sessions_list(
    config: &Configuration,
    tenant_id: &str,
    limit: i64,
    offset: i64,
) -> Result<()> {
    let client = build_client(config, tenant_id)?;
    let url = api_url(config, "/v1/llms/sessions");

    let resp: SessionListResponse = get_json_with_query(
        &client,
        &url,
        &[("limit", limit.to_string()), ("offset", offset.to_string())],
    )
    .await?;

    if resp.sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }

    let id_w = 30;
    let name_w = 30;
    let created_w = 19;
    let updated_w = 19;

    println!(
        "{:<id_w$}  {:<name_w$}  {:<created_w$}  {:<updated_w$}",
        "ID", "NAME", "CREATED AT", "UPDATED AT",
    );
    println!(
        "{:-<id_w$}  {:-<name_w$}  {:-<created_w$}  {:-<updated_w$}",
        "", "", "", "",
    );

    for s in &resp.sessions {
        println!(
            "{:<id_w$}  {:<name_w$}  {:<created_w$}  {:<updated_w$}",
            s.id,
            s.name.as_deref().unwrap_or("-"),
            &s.created_at[..19.min(s.created_at.len())],
            &s.updated_at[..19.min(s.updated_at.len())],
        );
    }

    Ok(())
}

async fn run_models(config: &Configuration, tenant_id: &str, feature: Option<&str>) -> Result<()> {
    let client = build_client(config, tenant_id)?;
    let url = api_url(config, "/v1/llms/models");

    let resp: ModelsResponse = if let Some(feat) = feature {
        get_json_with_query(&client, &url, &[("supported_feature", feat)]).await?
    } else {
        get_json(&client, &url).await?
    };

    if resp.models.is_empty() {
        println!("No models found.");
        return Ok(());
    }

    let id_w = 40;
    let provider_w = 12;
    let ctx_w = 10;
    let max_w = 8;

    println!(
        "{:<id_w$}  {:<provider_w$}  {:<ctx_w$}  {:<max_w$}  FEATURES",
        "MODEL ID", "PROVIDER", "CONTEXT", "MAX OUT",
    );
    println!(
        "{:-<id_w$}  {:-<provider_w$}  {:-<ctx_w$}  {:-<max_w$}  --------",
        "", "", "", "",
    );

    for m in &resp.models {
        let features = m
            .supported_features
            .as_deref()
            .map(|f| f.join(", "))
            .unwrap_or_default();
        println!(
            "{:<id_w$}  {:<provider_w$}  {:<ctx_w$}  {:<max_w$}  {}",
            m.id,
            m.provider,
            m.context_window
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            m.max_output_tokens
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            features,
        );
    }

    if let Some(total) = resp.total_count {
        println!("\nTotal: {total}");
    }

    Ok(())
}

pub async fn run(args: &AgentArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    match &args.command {
        AgentCommand::Sessions(s) => match &s.command {
            SessionsCommand::List { limit, offset } => {
                run_sessions_list(config, tenant_id, *limit, *offset).await
            }
        },
        AgentCommand::Models { feature } => run_models(config, tenant_id, feature.as_deref()).await,
    }
}
