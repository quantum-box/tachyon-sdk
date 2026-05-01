use anyhow::{anyhow, Result};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Select};
use serde::Deserialize;
use tachyon_sdk::apis::configuration::Configuration;

use crate::auth;
use crate::client::ApiClient;

const DEFAULT_TACHYON_TENANT_ID: &str = "tn_01hjryxysgey07h5jz5wagqj0m";
const DEFAULT_TACHYON_TENANT_ALIAS: &str = "tachyon";
const DEFAULT_TACHYON_TENANT_NAME: &str = "Tachyon";

#[derive(Debug, Args)]
pub struct SwitchArgs {
    /// Tenant ID, alias, or name to switch to (omit for interactive)
    pub tenant: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OperatorEntry {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    alias: Option<String>,
}

pub async fn run(args: &SwitchArgs, config: &Configuration, profile: &str) -> Result<()> {
    // Build client without operator context — list endpoint doesn't need it
    let api = ApiClient::new(config, "")?;

    match &args.tenant {
        Some(name_or_id) => {
            // Direct switch: resolve then update
            let operator_id = resolve_operator_id(&api, name_or_id).await?;
            update_credentials(profile, &operator_id)?;
            println!("Switched to tenant: {operator_id}");
        }
        None => {
            // Interactive switch
            let ops: Vec<OperatorEntry> = api.get("/v1/auth/operators/by-user").await?;
            let ops = with_default_tachyon_tenant(ops);
            if ops.is_empty() {
                return Err(anyhow!(
                    "No tenants found. Run `tachyon auth login` to authenticate."
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
            update_credentials(profile, &operator_id)?;
            println!("Switched to tenant: {operator_id}");
        }
    }
    Ok(())
}

fn update_credentials(profile: &str, operator_id: &str) -> Result<()> {
    let mut creds = auth::load_profile(profile)?.ok_or_else(|| {
        anyhow!("Not logged in (profile '{profile}'). Run `tachyon auth login --profile {profile}` first.")
    })?;
    creds.operator_id = Some(operator_id.to_string());
    auth::save_profile(profile, &creds)?;
    Ok(())
}

async fn resolve_operator_id(api: &ApiClient, name_or_id: &str) -> Result<String> {
    // If it looks like an ID (prefix_base32), use as-is
    if looks_like_id(name_or_id) {
        return Ok(name_or_id.to_string());
    }
    if is_default_tachyon_tenant_name(name_or_id) {
        return Ok(DEFAULT_TACHYON_TENANT_ID.to_string());
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
    let ops = with_default_tachyon_tenant(ops);
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

fn with_default_tachyon_tenant(mut ops: Vec<OperatorEntry>) -> Vec<OperatorEntry> {
    if ops.iter().any(|op| op.id == DEFAULT_TACHYON_TENANT_ID) {
        return ops;
    }
    ops.push(OperatorEntry {
        id: DEFAULT_TACHYON_TENANT_ID.to_string(),
        name: Some(DEFAULT_TACHYON_TENANT_NAME.to_string()),
        alias: Some(DEFAULT_TACHYON_TENANT_ALIAS.to_string()),
    });
    ops
}

fn is_default_tachyon_tenant_name(value: &str) -> bool {
    value.eq_ignore_ascii_case(DEFAULT_TACHYON_TENANT_ALIAS)
        || value.eq_ignore_ascii_case(DEFAULT_TACHYON_TENANT_NAME)
}

fn looks_like_id(value: &str) -> bool {
    if let Some(pos) = value.find('_') {
        let after = &value[pos + 1..];
        after.len() > 10 && after.chars().all(|c| c.is_ascii_alphanumeric())
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_default_tachyon_tenant_to_switch_candidates() {
        let ops = with_default_tachyon_tenant(vec![OperatorEntry {
            id: "tn_customer".to_string(),
            name: Some("Customer".to_string()),
            alias: None,
        }]);

        assert!(ops.iter().any(|op| op.id == DEFAULT_TACHYON_TENANT_ID));
    }

    #[test]
    fn does_not_duplicate_default_tachyon_tenant() {
        let ops = with_default_tachyon_tenant(vec![OperatorEntry {
            id: DEFAULT_TACHYON_TENANT_ID.to_string(),
            name: Some("Existing Tachyon".to_string()),
            alias: None,
        }]);

        assert_eq!(
            ops.iter()
                .filter(|op| op.id == DEFAULT_TACHYON_TENANT_ID)
                .count(),
            1
        );
        assert_eq!(ops[0].name.as_deref(), Some("Existing Tachyon"));
    }

    #[test]
    fn recognizes_default_tachyon_tenant_name() {
        assert!(is_default_tachyon_tenant_name("tachyon"));
        assert!(is_default_tachyon_tenant_name("Tachyon"));
        assert!(!is_default_tachyon_tenant_name("MOVERENT"));
    }
}
