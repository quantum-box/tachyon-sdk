use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::client::ApiClient;
use tachyon_sdk::apis::configuration::Configuration;

/// Returns true if the value looks like a Tachyon resource ID.
/// Tachyon IDs follow the pattern: prefix_base32chars (e.g., app_01km2dr0f6hvgj0qvcteyydfbe)
fn looks_like_id(value: &str) -> bool {
    if let Some(pos) = value.find('_') {
        let after = &value[pos + 1..];
        after.len() > 10 && after.chars().all(|c| c.is_ascii_alphanumeric())
    } else {
        false
    }
}

// --- Minimal response types for resolution ---

#[derive(Debug, Deserialize)]
struct ListAppsResponse {
    apps: Vec<AppEntry>,
}

#[derive(Debug, Deserialize)]
struct AppEntry {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OperatorEntry {
    id: String,
    name: String,
    operator_name: String,
}

#[derive(Debug, Deserialize)]
struct WorkerEntry {
    id: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorkerListResponse {
    workers: Vec<WorkerEntry>,
}

#[derive(Debug, Deserialize)]
struct ProtocolEntry {
    id: String,
    protocol_name: String,
}

#[derive(Debug, Deserialize)]
struct ProtocolListResponse {
    items: Vec<ProtocolEntry>,
}

#[derive(Debug, Deserialize)]
struct IntegrationEntry {
    id: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IntegrationListResponse {
    integrations: Vec<IntegrationEntry>,
}

#[derive(Debug, Deserialize)]
struct ServiceAccountEntry {
    id: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ServiceAccountListResponse {
    Envelope {
        #[serde(rename = "serviceAccounts")]
        service_accounts: Vec<ServiceAccountEntry>,
    },
    Array(Vec<ServiceAccountEntry>),
}

impl ServiceAccountListResponse {
    fn into_vec(self) -> Vec<ServiceAccountEntry> {
        match self {
            Self::Envelope { service_accounts } => service_accounts,
            Self::Array(service_accounts) => service_accounts,
        }
    }
}

// --- Tenant ID resolution ---

/// Resolve a tenant identifier (ID, alias, or name) to the operator ID.
/// If empty, falls back to the saved operator_id in the active profile's credentials.
/// If the value looks like an ID, it is returned as-is.
/// Otherwise, tries by-alias lookup first, then falls back to listing operators and matching by name.
pub async fn resolve_tenant_id(
    config: &Configuration,
    name_or_id: &str,
    profile: &str,
) -> Result<String> {
    if name_or_id.is_empty() {
        // Fall back to saved operator_id from the active profile's credentials.
        if let Ok(Some(creds)) = crate::auth::load_profile(profile) {
            if let Some(op_id) = creds.operator_id {
                return Ok(op_id);
            }
        }
        return Ok(String::new());
    }
    if looks_like_id(name_or_id) {
        return Ok(name_or_id.to_string());
    }

    // Create a temporary client without tenant-id to resolve
    let api = ApiClient::new(config, "")?;

    // Try by-alias first (exact match via API)
    if let Ok(op) = api
        .get_query::<OperatorEntry>("/v1/auth/operators/by-alias", &[("alias", name_or_id)])
        .await
    {
        eprintln!("Resolved tenant '{}' → {}", name_or_id, op.id);
        return Ok(op.id);
    }

    // Fall back to listing operators and matching by display or operator name.
    let ops: Vec<OperatorEntry> = api.get("/v1/auth/operators/by-user").await?;
    let matches: Vec<_> = ops
        .iter()
        .filter(|o| o.operator_name == name_or_id || o.name == name_or_id)
        .collect();

    match matches.len() {
        0 => Err(anyhow!(
            "no operator found with name or operator name '{name_or_id}'"
        )),
        1 => {
            eprintln!("Resolved tenant '{}' → {}", name_or_id, matches[0].id);
            Ok(matches[0].id.clone())
        }
        _ => {
            let ids: Vec<_> = matches.iter().map(|o| o.id.as_str()).collect();
            Err(anyhow!(
                "multiple operators match '{name_or_id}': {}",
                ids.join(", ")
            ))
        }
    }
}

// --- App ID resolution ---

/// Resolve an app identifier (ID or name) to the app ID.
pub async fn resolve_app_id(api: &ApiClient, name_or_id: &str) -> Result<String> {
    if looks_like_id(name_or_id) {
        return Ok(name_or_id.to_string());
    }

    let resp: ListAppsResponse = api.get("/v1/compute/apps").await?;
    let matches: Vec<_> = resp.apps.iter().filter(|a| a.name == name_or_id).collect();

    match matches.len() {
        0 => Err(anyhow!("no app found with name '{name_or_id}'")),
        1 => {
            eprintln!("Resolved app '{}' → {}", name_or_id, matches[0].id);
            Ok(matches[0].id.clone())
        }
        _ => {
            let ids: Vec<_> = matches.iter().map(|a| a.id.as_str()).collect();
            Err(anyhow!(
                "multiple apps match name '{name_or_id}': {}",
                ids.join(", ")
            ))
        }
    }
}

// --- Worker ID resolution ---

/// Resolve a worker identifier (ID or name) to the worker ID.
pub async fn resolve_worker_id(api: &ApiClient, name_or_id: &str) -> Result<String> {
    if looks_like_id(name_or_id) {
        return Ok(name_or_id.to_string());
    }

    let response: WorkerListResponse = api.get("/v1/agent/workers").await?;
    let workers = response.workers;
    let matches: Vec<_> = workers
        .iter()
        .filter(|w| w.name.as_deref() == Some(name_or_id))
        .collect();

    match matches.len() {
        0 => Err(anyhow!("no worker found with name '{name_or_id}'")),
        1 => {
            eprintln!("Resolved worker '{}' → {}", name_or_id, matches[0].id);
            Ok(matches[0].id.clone())
        }
        _ => {
            let ids: Vec<_> = matches.iter().map(|w| w.id.as_str()).collect();
            Err(anyhow!(
                "multiple workers match name '{name_or_id}': {}",
                ids.join(", ")
            ))
        }
    }
}

// --- Protocol ID resolution ---

/// Resolve a protocol identifier (ID or name) to the protocol ID.
pub async fn resolve_protocol_id(api: &ApiClient, name_or_id: &str) -> Result<String> {
    if looks_like_id(name_or_id) {
        return Ok(name_or_id.to_string());
    }

    let response: ProtocolListResponse = api.get("/v1/llms/agent-protocols").await?;
    let protocols = response.items;
    let matches: Vec<_> = protocols
        .iter()
        .filter(|p| p.protocol_name == name_or_id)
        .collect();

    match matches.len() {
        0 => Err(anyhow!("no protocol found with name '{name_or_id}'")),
        1 => {
            eprintln!("Resolved protocol '{}' → {}", name_or_id, matches[0].id);
            Ok(matches[0].id.clone())
        }
        _ => {
            let ids: Vec<_> = matches.iter().map(|p| p.id.as_str()).collect();
            Err(anyhow!(
                "multiple protocols match name '{name_or_id}': {}",
                ids.join(", ")
            ))
        }
    }
}

// --- Integration ID resolution ---

/// Resolve an integration identifier (ID or name) to the integration ID.
pub async fn resolve_integration_id(api: &ApiClient, name_or_id: &str) -> Result<String> {
    if looks_like_id(name_or_id) {
        return Ok(name_or_id.to_string());
    }

    let response: IntegrationListResponse = api.get("/v1/integrations").await?;
    let integrations = response.integrations;
    let matches: Vec<_> = integrations
        .iter()
        .filter(|i| i.name.as_deref() == Some(name_or_id))
        .collect();

    match matches.len() {
        0 => Err(anyhow!("no integration found with name '{name_or_id}'")),
        1 => {
            eprintln!("Resolved integration '{}' → {}", name_or_id, matches[0].id);
            Ok(matches[0].id.clone())
        }
        _ => {
            let ids: Vec<_> = matches.iter().map(|i| i.id.as_str()).collect();
            Err(anyhow!(
                "multiple integrations match name '{name_or_id}': {}",
                ids.join(", ")
            ))
        }
    }
}

// --- Service Account ID resolution ---

/// Resolve a service account identifier (ID or name) to the service account ID.
pub async fn resolve_service_account_id(
    api: &ApiClient,
    tenant_id: &str,
    name_or_id: &str,
) -> Result<String> {
    if looks_like_id(name_or_id) {
        return Ok(name_or_id.to_string());
    }

    let response: ServiceAccountListResponse = api
        .get_query("/v1/auth/service-accounts", &[("operator_id", tenant_id)])
        .await?;
    let accounts = response.into_vec();
    let matches: Vec<_> = accounts
        .iter()
        .filter(|s| s.name.as_deref() == Some(name_or_id))
        .collect();

    match matches.len() {
        0 => Err(anyhow!("no service account found with name '{name_or_id}'")),
        1 => {
            eprintln!(
                "Resolved service account '{}' → {}",
                name_or_id, matches[0].id
            );
            Ok(matches[0].id.clone())
        }
        _ => {
            let ids: Vec<_> = matches.iter().map(|s| s.id.as_str()).collect();
            Err(anyhow!(
                "multiple service accounts match name '{name_or_id}': {}",
                ids.join(", ")
            ))
        }
    }
}

#[cfg(test)]
mod response_contract_tests {
    use super::{OperatorEntry, ProtocolEntry};

    #[test]
    fn operator_entry_uses_api_field_names() {
        let valid = r#"{"id":"tn_123","name":"Operator","operatorName":"operator-one"}"#;
        let wrong = r#"{"id":"tn_123","name":"Operator","alias":"operator-one"}"#;

        assert!(serde_json::from_str::<OperatorEntry>(valid).is_ok());
        assert!(serde_json::from_str::<OperatorEntry>(wrong).is_err());
    }

    #[test]
    fn protocol_entry_uses_api_field_names() {
        let valid = r#"{"id":"ap_123","protocol_name":"protocol-one"}"#;
        let wrong = r#"{"id":"ap_123","name":"protocol-one"}"#;

        assert!(serde_json::from_str::<ProtocolEntry>(valid).is_ok());
        assert!(serde_json::from_str::<ProtocolEntry>(wrong).is_err());
    }
}
