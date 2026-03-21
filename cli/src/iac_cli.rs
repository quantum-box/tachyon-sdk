use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Deserialize;
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{api_url, build_client, get_json, get_json_with_query};

#[derive(Debug, Clone, Args)]
pub struct IacArgs {
    #[command(subcommand)]
    pub command: IacCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum IacCommand {
    /// Manage integrations
    Integrations(IntegrationsArgs),
}

// ── Integrations ──

#[derive(Debug, Clone, Args)]
pub struct IntegrationsArgs {
    #[command(subcommand)]
    pub command: IntegrationsCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum IntegrationsCommand {
    /// List integrations
    List {
        /// Filter by category (e.g. "vcs", "ci")
        #[arg(long)]
        category: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct ListIntegrationsResponse {
    integrations: Vec<IntegrationResponse>,
}

#[derive(Debug, Deserialize)]
struct IntegrationResponse {
    id: String,
    name: String,
    #[allow(dead_code)]
    description: Option<String>,
    category: Option<String>,
    provider: Option<String>,
    is_enabled: Option<bool>,
}

// ── Runners ──

async fn run_integrations_list(
    config: &Configuration,
    tenant_id: &str,
    category: Option<&str>,
) -> Result<()> {
    let client = build_client(config, tenant_id)?;
    let url = api_url(config, "/v1/integrations");

    let resp: ListIntegrationsResponse = if let Some(cat) = category {
        get_json_with_query(&client, &url, &[("category", cat)]).await?
    } else {
        get_json(&client, &url).await?
    };

    if resp.integrations.is_empty() {
        println!("No integrations found.");
        return Ok(());
    }

    let id_w = 30;
    let name_w = 20;
    let cat_w = 12;
    let provider_w = 12;

    println!(
        "{:<id_w$}  {:<name_w$}  {:<cat_w$}  {:<provider_w$}  ENABLED",
        "ID", "NAME", "CATEGORY", "PROVIDER",
    );
    println!(
        "{:-<id_w$}  {:-<name_w$}  {:-<cat_w$}  {:-<provider_w$}  -------",
        "", "", "", "",
    );

    for i in &resp.integrations {
        println!(
            "{:<id_w$}  {:<name_w$}  {:<cat_w$}  {:<provider_w$}  {}",
            i.id,
            i.name,
            i.category.as_deref().unwrap_or("-"),
            i.provider.as_deref().unwrap_or("-"),
            i.is_enabled
                .map(|v| if v { "yes" } else { "no" })
                .unwrap_or("-"),
        );
    }

    Ok(())
}

pub async fn run(
    args: &IacArgs,
    config: &Configuration,
    tenant_id: &str,
) -> Result<()> {
    match &args.command {
        IacCommand::Integrations(i) => match &i.command {
            IntegrationsCommand::List { category } => {
                run_integrations_list(
                    config,
                    tenant_id,
                    category.as_deref(),
                )
                .await
            }
        },
    }
}
