use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};

#[derive(Debug, Clone, Args)]
pub struct IacArgs {
    #[command(subcommand)]
    pub command: IacCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum IacCommand {
    /// Show OAuth provider configurations (GitHub, Linear, etc.)
    OauthProviders {
        /// Tenant ID to query (uses default if not specified)
        #[arg(long)]
        tenant_id: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// List integrations (connected external services)
    Integrations {
        #[command(subcommand)]
        command: IntegrationsCommand,
    },
    /// List integration connections
    Connections {
        #[command(subcommand)]
        command: ConnectionsCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum IntegrationsCommand {
    /// List available integrations
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get integration details
    Get {
        id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConnectionsCommand {
    /// List integration connections
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get connection details
    Get {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Disconnect an integration
    Disconnect { id: String },
}

// ---- Response types ----

#[derive(Debug, Deserialize, Serialize)]
struct OAuthProvidersResponse {
    #[serde(default)]
    github: Option<OAuthProviderConfig>,
    #[serde(default)]
    linear: Option<OAuthProviderConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OAuthProviderConfig {
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IntegrationResponse {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConnectionResponse {
    id: String,
    #[serde(default)]
    integration_id: Option<String>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    account_name: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

// ---- Handlers ----

async fn run_oauth_providers(
    api: &ApiClient,
    tenant_id: Option<&str>,
    default_tenant_id: &str,
    json: bool,
) -> Result<()> {
    let tid = tenant_id.unwrap_or(default_tenant_id);
    let resp: OAuthProvidersResponse = api
        .get_query("/v1/iac/oauth-providers", &[("tenant_id", tid)])
        .await?;
    if json {
        return print_json(&resp);
    }
    println!("OAuth Provider Configurations:");
    println!();
    if let Some(gh) = &resp.github {
        println!("  GitHub:");
        println!(
            "    Enabled:   {}",
            gh.enabled
                .map(|v| if v { "yes" } else { "no" })
                .unwrap_or("-")
        );
        println!("    Client ID: {}", gh.client_id.as_deref().unwrap_or("-"));
        if let Some(scopes) = &gh.scopes {
            println!("    Scopes:    {}", scopes.join(", "));
        }
    } else {
        println!("  GitHub: not configured");
    }
    println!();
    if let Some(lr) = &resp.linear {
        println!("  Linear:");
        println!(
            "    Enabled:   {}",
            lr.enabled
                .map(|v| if v { "yes" } else { "no" })
                .unwrap_or("-")
        );
        println!("    Client ID: {}", lr.client_id.as_deref().unwrap_or("-"));
        if let Some(scopes) = &lr.scopes {
            println!("    Scopes:    {}", scopes.join(", "));
        }
    } else {
        println!("  Linear: not configured");
    }
    Ok(())
}

async fn run_integrations_list(api: &ApiClient, json: bool) -> Result<()> {
    let integrations: Vec<IntegrationResponse> = api.get("/v1/integrations").await?;
    if json {
        return print_json(&integrations);
    }
    if integrations.is_empty() {
        println!("No integrations found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<20}  {:<16}  {:<12}  CREATED AT",
        "ID", "NAME", "PROVIDER", "STATUS"
    );
    println!(
        "{:-<28}  {:-<20}  {:-<16}  {:-<12}  {:-<19}",
        "", "", "", "", ""
    );
    for i in &integrations {
        println!(
            "{:<28}  {:<20}  {:<16}  {:<12}  {}",
            i.id,
            i.name.as_deref().unwrap_or("-"),
            i.provider.as_deref().unwrap_or("-"),
            i.status.as_deref().unwrap_or("-"),
            i.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_integrations_get(api: &ApiClient, id: &str, json: bool) -> Result<()> {
    let i: IntegrationResponse = api.get(&format!("/v1/integrations/{id}")).await?;
    if json {
        return print_json(&i);
    }
    println!("ID:       {}", i.id);
    println!("Name:     {}", i.name.as_deref().unwrap_or("-"));
    println!("Provider: {}", i.provider.as_deref().unwrap_or("-"));
    println!("Status:   {}", i.status.as_deref().unwrap_or("-"));
    println!("Created:  {}", i.created_at.as_deref().unwrap_or("-"));
    Ok(())
}

async fn run_connections_list(api: &ApiClient, json: bool) -> Result<()> {
    let conns: Vec<ConnectionResponse> = api.get("/v1/integrations/connections").await?;
    if json {
        return print_json(&conns);
    }
    if conns.is_empty() {
        println!("No connections found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<16}  {:<12}  {:<20}  CREATED AT",
        "ID", "PROVIDER", "STATUS", "ACCOUNT"
    );
    println!(
        "{:-<28}  {:-<16}  {:-<12}  {:-<20}  {:-<19}",
        "", "", "", "", ""
    );
    for c in &conns {
        println!(
            "{:<28}  {:<16}  {:<12}  {:<20}  {}",
            c.id,
            c.provider.as_deref().unwrap_or("-"),
            c.status.as_deref().unwrap_or("-"),
            c.account_name.as_deref().unwrap_or("-"),
            c.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_connections_get(api: &ApiClient, id: &str, json: bool) -> Result<()> {
    let c: ConnectionResponse = api
        .get(&format!("/v1/integrations/connections/{id}"))
        .await?;
    if json {
        return print_json(&c);
    }
    println!("ID:       {}", c.id);
    println!("Provider: {}", c.provider.as_deref().unwrap_or("-"));
    println!("Status:   {}", c.status.as_deref().unwrap_or("-"));
    println!("Account:  {}", c.account_name.as_deref().unwrap_or("-"));
    println!("Created:  {}", c.created_at.as_deref().unwrap_or("-"));
    Ok(())
}

async fn run_connections_disconnect(api: &ApiClient, id: &str) -> Result<()> {
    api.delete(&format!("/v1/integrations/connections/{id}"))
        .await?;
    println!("Connection {id} disconnected.");
    Ok(())
}

// ---- Entry point ----

pub async fn run(args: &IacArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    match &args.command {
        IacCommand::OauthProviders {
            tenant_id: tid,
            json,
        } => run_oauth_providers(&api, tid.as_deref(), tenant_id, *json).await,
        IacCommand::Integrations { command } => match command {
            IntegrationsCommand::List { json } => run_integrations_list(&api, *json).await,
            IntegrationsCommand::Get { id, json } => run_integrations_get(&api, id, *json).await,
        },
        IacCommand::Connections { command } => match command {
            ConnectionsCommand::List { json } => run_connections_list(&api, *json).await,
            ConnectionsCommand::Get { id, json } => run_connections_get(&api, id, *json).await,
            ConnectionsCommand::Disconnect { id } => run_connections_disconnect(&api, id).await,
        },
    }
}
