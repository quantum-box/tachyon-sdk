use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, truncate, ApiClient};
use crate::resolve;

#[derive(Debug, Clone, Args)]
pub struct ApiKeyArgs {
    #[command(subcommand)]
    pub command: ApiKeyCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ApiKeyCommand {
    /// Create an API key for a service account
    Create {
        /// Service account ID or name
        service_account: String,
        /// Display name for the API key
        #[arg(long)]
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// List API keys for a service account
    List {
        /// Service account ID or name
        service_account: String,
        #[arg(long)]
        json: bool,
    },
    /// Revoke an API key for a service account
    Revoke {
        /// Service account ID or name
        service_account: String,
        /// API key ID to revoke
        api_key_id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateApiKeyRequest {
    name: String,
    operator_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RevokeApiKeyRequest {
    operator_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResponse {
    id: String,
    service_account_id: String,
    name: String,
    value: String,
    created_at: String,
    expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiKeyListResponse {
    api_keys: Vec<ApiKeyResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RevokeApiKeyOutput {
    id: String,
    service_account_id: String,
    revoked: bool,
}

async fn run_create(
    api: &ApiClient,
    tenant_id: &str,
    service_account_id: &str,
    name: &str,
    json: bool,
) -> Result<()> {
    let request = CreateApiKeyRequest {
        name: name.to_string(),
        operator_id: tenant_id.to_string(),
    };
    let key: ApiKeyResponse = api
        .post(
            &format!("/v1/auth/service-accounts/{service_account_id}/api-keys"),
            &request,
        )
        .await?;
    if json {
        return print_json(&key);
    }

    println!("API key created.");
    println!("ID:                {}", key.id);
    println!("Service Account:   {}", key.service_account_id);
    println!("Name:              {}", key.name);
    println!("Created:           {}", key.created_at);
    println!("Value:             {}", key.value);
    println!("Store this value now. It may not be shown again.");
    Ok(())
}

async fn run_list(
    api: &ApiClient,
    tenant_id: &str,
    service_account_id: &str,
    json: bool,
) -> Result<()> {
    let response: ApiKeyListResponse = api
        .get_query(
            &format!("/v1/auth/service-accounts/{service_account_id}/api-keys"),
            &[("operator_id", tenant_id)],
        )
        .await?;
    let keys = response.api_keys;
    if json {
        return print_json(&keys);
    }
    if keys.is_empty() {
        println!("No API keys found for service account {service_account_id}");
        return Ok(());
    }

    println!(
        "{:<28}  {:<24}  {:<16}  {:<19}  EXPIRES AT",
        "ID", "NAME", "PREFIX", "CREATED AT"
    );
    println!(
        "{:-<28}  {:-<24}  {:-<16}  {:-<19}  {:-<19}",
        "", "", "", "", ""
    );
    for key in &keys {
        let prefix = api_key_prefix(&key.value);
        println!(
            "{:<28}  {:<24}  {:<16}  {:<19}  {}",
            key.id,
            truncate(&key.name, 24),
            prefix,
            key.created_at,
            key.expires_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_revoke(
    api: &ApiClient,
    tenant_id: &str,
    service_account_id: &str,
    api_key_id: &str,
    json: bool,
) -> Result<()> {
    let request = RevokeApiKeyRequest {
        operator_id: tenant_id.to_string(),
    };
    api.post_no_response(
        &format!("/v1/auth/service-accounts/{service_account_id}/api-keys/{api_key_id}/revoke"),
        &request,
    )
    .await?;

    if json {
        return print_json(&RevokeApiKeyOutput {
            id: api_key_id.to_string(),
            service_account_id: service_account_id.to_string(),
            revoked: true,
        });
    }
    println!("API key {api_key_id} revoked.");
    Ok(())
}

fn api_key_prefix(value: &str) -> &str {
    value.get(..16).unwrap_or(value)
}

pub async fn run(args: &ApiKeyArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    match &args.command {
        ApiKeyCommand::Create {
            service_account,
            name,
            json,
        } => {
            let id = resolve::resolve_service_account_id(&api, tenant_id, service_account).await?;
            run_create(&api, tenant_id, &id, name, *json).await
        }
        ApiKeyCommand::List {
            service_account,
            json,
        } => {
            let id = resolve::resolve_service_account_id(&api, tenant_id, service_account).await?;
            run_list(&api, tenant_id, &id, *json).await
        }
        ApiKeyCommand::Revoke {
            service_account,
            api_key_id,
            json,
        } => {
            let id = resolve::resolve_service_account_id(&api, tenant_id, service_account).await?;
            run_revoke(&api, tenant_id, &id, api_key_id, *json).await
        }
    }
}
