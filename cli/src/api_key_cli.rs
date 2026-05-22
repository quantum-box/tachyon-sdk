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
    #[serde(default)]
    service_account_id: Option<String>,
    name: String,
    #[serde(default)]
    value: Option<String>,
    #[serde(default, alias = "created_at")]
    created_at: Option<String>,
    #[serde(default, alias = "revoked_at")]
    revoked_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ApiKeyListResponse {
    Envelope {
        #[serde(rename = "apiKeys")]
        api_keys: Vec<ApiKeyResponse>,
    },
    Array(Vec<ApiKeyResponse>),
}

impl ApiKeyListResponse {
    fn into_vec(self) -> Vec<ApiKeyResponse> {
        match self {
            ApiKeyListResponse::Envelope { api_keys } => api_keys,
            ApiKeyListResponse::Array(api_keys) => api_keys,
        }
    }
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
    println!(
        "Service Account:   {}",
        key.service_account_id
            .as_deref()
            .unwrap_or(service_account_id)
    );
    println!("Name:              {}", key.name);
    println!(
        "Created:           {}",
        key.created_at.as_deref().unwrap_or("-")
    );
    if let Some(value) = key.value.as_deref() {
        println!("Value:             {value}");
        println!("Store this value now. It may not be shown again.");
    }
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
    let keys = response.into_vec();
    if json {
        return print_json(&keys);
    }
    if keys.is_empty() {
        println!("No API keys found for service account {service_account_id}");
        return Ok(());
    }

    println!(
        "{:<28}  {:<24}  {:<16}  {:<19}  REVOKED AT",
        "ID", "NAME", "PREFIX", "CREATED AT"
    );
    println!(
        "{:-<28}  {:-<24}  {:-<16}  {:-<19}  {:-<19}",
        "", "", "", "", ""
    );
    for key in &keys {
        let prefix = key.value.as_deref().map(api_key_prefix).unwrap_or("-");
        println!(
            "{:<28}  {:<24}  {:<16}  {:<19}  {}",
            key.id,
            truncate(&key.name, 24),
            prefix,
            key.created_at.as_deref().unwrap_or("-"),
            key.revoked_at.as_deref().unwrap_or("-"),
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
            let id = resolve::resolve_service_account_id(&api, service_account).await?;
            run_create(&api, tenant_id, &id, name, *json).await
        }
        ApiKeyCommand::List {
            service_account,
            json,
        } => {
            let id = resolve::resolve_service_account_id(&api, service_account).await?;
            run_list(&api, tenant_id, &id, *json).await
        }
        ApiKeyCommand::Revoke {
            service_account,
            api_key_id,
            json,
        } => {
            let id = resolve::resolve_service_account_id(&api, service_account).await?;
            run_revoke(&api, tenant_id, &id, api_key_id, *json).await
        }
    }
}
