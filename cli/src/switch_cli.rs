use anyhow::{anyhow, Result};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Select};
use serde::Deserialize;
use tachyon_sdk::apis::configuration::Configuration;

use crate::auth;
use crate::client::ApiClient;

#[derive(Debug, Args)]
pub struct SwitchArgs {
    /// Tenant ID, alias, or name to switch to (omit for interactive)
    pub tenant: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OperatorEntry {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    alias: Option<String>,
}

pub async fn run(args: &SwitchArgs, config: &Configuration) -> Result<()> {
    // Build client without operator context — list endpoint doesn't need it
    let api = ApiClient::new(config, "")?;

    match &args.tenant {
        Some(name_or_id) => {
            // Direct switch: resolve then update
            let operator_id = resolve_operator_id(&api, name_or_id).await?;
            update_credentials(&operator_id)?;
            println!("Switched to tenant: {operator_id}");
        }
        None => {
            // Interactive switch
            let ops: Vec<OperatorEntry> = api.get("/v1/auth/operators/by-user").await?;
            if ops.is_empty() {
                return Err(anyhow!(
                    "No tenants found. Run `tachyon login` to authenticate."
                ));
            }
            let labels: Vec<String> = ops
                .iter()
                .map(|op| {
                    let label = op.alias.as_deref().or(op.name.as_deref()).unwrap_or(&op.id);
                    format!("{} ({})", label, op.id)
                })
                .collect();
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select tenant")
                .items(&labels)
                .default(0)
                .interact()?;
            let operator_id = ops[selection].id.clone();
            update_credentials(&operator_id)?;
            println!("Switched to tenant: {operator_id}");
        }
    }
    Ok(())
}

fn update_credentials(operator_id: &str) -> Result<()> {
    let mut creds = auth::load_credentials()?
        .ok_or_else(|| anyhow!("Not logged in. Run `tachyon login` first."))?;
    creds.operator_id = Some(operator_id.to_string());
    auth::save_credentials(&creds)?;
    Ok(())
}

async fn resolve_operator_id(api: &ApiClient, name_or_id: &str) -> Result<String> {
    // If it looks like an ID (prefix_base32), use as-is
    if looks_like_id(name_or_id) {
        return Ok(name_or_id.to_string());
    }
    // Try by-alias exact match
    #[derive(Deserialize)]
    struct ByAliasResult {
        id: String,
    }
    if let Ok(op) = api
        .get_query::<ByAliasResult>("/v1/auth/operators/by-alias", &[("alias", name_or_id)])
        .await
    {
        return Ok(op.id);
    }
    // Fall back to listing and matching by name or alias
    let ops: Vec<OperatorEntry> = api.get("/v1/auth/operators/by-user").await?;
    let matches: Vec<_> = ops
        .iter()
        .filter(|o| o.alias.as_deref() == Some(name_or_id) || o.name.as_deref() == Some(name_or_id))
        .collect();
    match matches.len() {
        0 => Err(anyhow!("No tenant found with name or alias '{name_or_id}'")),
        1 => Ok(matches[0].id.clone()),
        _ => {
            let ids: Vec<_> = matches.iter().map(|o| o.id.as_str()).collect();
            Err(anyhow!(
                "Multiple tenants match '{}': {}",
                name_or_id,
                ids.join(", ")
            ))
        }
    }
}

fn looks_like_id(value: &str) -> bool {
    if let Some(pos) = value.find('_') {
        let after = &value[pos + 1..];
        after.len() > 10 && after.chars().all(|c| c.is_ascii_alphanumeric())
    } else {
        false
    }
}
