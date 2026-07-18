//! Generic CRUD commands for PM provider resources.

use anyhow::{anyhow, Context, Result};
use clap::Subcommand;
use serde::Serialize;
use serde_json::{Map, Value};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum ResourceCommand {
    /// Create a resource
    Create {
        #[arg(long)]
        provider: Option<String>,
        /// JSON object containing provider fields
        #[arg(long, value_name = "JSON")]
        input_json: Option<String>,
        /// Provider field as KEY=JSON. Can be specified multiple times.
        #[arg(long = "field", value_name = "KEY=JSON")]
        fields: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// List resources
    List {
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Get a resource
    Get {
        resource_id: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Update a resource
    Update {
        resource_id: String,
        #[arg(long)]
        provider: Option<String>,
        /// JSON object containing provider fields
        #[arg(long, value_name = "JSON")]
        input_json: Option<String>,
        /// Provider field as KEY=JSON. Can be specified multiple times.
        #[arg(long = "field", value_name = "KEY=JSON")]
        fields: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Delete or archive a resource according to provider capabilities
    Delete {
        resource_id: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize)]
struct ResourceMutationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    fields: Map<String, Value>,
}

fn collection_path(tenant_id: &str, resource: &str) -> String {
    format!("/v1beta/{tenant_id}/pm/{resource}")
}

fn item_path(tenant_id: &str, resource: &str, resource_id: &str) -> String {
    format!("{}/{resource_id}", collection_path(tenant_id, resource))
}

fn with_provider(path: String, provider: Option<&str>) -> String {
    match provider {
        Some(provider) => format!("{path}?provider={}", urlencoding::encode(provider)),
        None => path,
    }
}

fn merge_fields(input_json: Option<&str>, field_args: &[String]) -> Result<Map<String, Value>> {
    let mut fields = match input_json {
        Some(value) => serde_json::from_str::<Map<String, Value>>(value)
            .context("--input-json must be a JSON object")?,
        None => Map::new(),
    };
    for field in field_args {
        let (key, raw_value) = field
            .split_once('=')
            .ok_or_else(|| anyhow!("--field must use KEY=JSON syntax: {field}"))?;
        if key.trim().is_empty() {
            return Err(anyhow!("--field key must not be empty"));
        }
        let value = serde_json::from_str(raw_value)
            .unwrap_or_else(|_| Value::String(raw_value.to_string()));
        fields.insert(key.to_string(), value);
    }
    Ok(fields)
}

fn provider_with_alias(provider: &Option<String>, alias: Option<&str>) -> Option<String> {
    provider.clone().or_else(|| alias.map(str::to_string))
}

fn print_resource(value: &Value, json: bool) -> Result<()> {
    if json {
        return print_json(value);
    }
    let id = value.get("id").and_then(Value::as_str).unwrap_or("-");
    let name = value.get("name").and_then(Value::as_str).unwrap_or("-");
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("resource");
    println!("{kind} {id}: {name}");
    Ok(())
}

pub async fn run_resource(
    resource: &str,
    command: &ResourceCommand,
    config: &Configuration,
    tenant_id: &str,
    provider_alias: Option<&str>,
) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;
    match command {
        ResourceCommand::Create {
            provider,
            input_json,
            fields,
            json,
        } => {
            let request = ResourceMutationRequest {
                provider: provider_with_alias(provider, provider_alias),
                fields: merge_fields(input_json.as_deref(), fields)?,
            };
            let response: Value = api
                .post(&collection_path(tenant_id, resource), &request)
                .await?;
            print_resource(&response, *json)
        }
        ResourceCommand::List { provider, json } => {
            let provider = provider_with_alias(provider, provider_alias);
            let query = provider
                .as_deref()
                .map(|value| vec![("provider", value)])
                .unwrap_or_default();
            let response: Value = api
                .get_query(&collection_path(tenant_id, resource), &query)
                .await?;
            if *json {
                return print_json(&response);
            }
            for item in response
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let id = item.get("id").and_then(Value::as_str).unwrap_or("-");
                let name = item.get("name").and_then(Value::as_str).unwrap_or("-");
                println!("{id}\t{name}");
            }
            Ok(())
        }
        ResourceCommand::Get {
            resource_id,
            provider,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let query = provider
                .as_deref()
                .map(|value| vec![("provider", value)])
                .unwrap_or_default();
            let response: Value = api
                .get_query(&item_path(tenant_id, resource, resource_id), &query)
                .await?;
            print_resource(&response, *json)
        }
        ResourceCommand::Update {
            resource_id,
            provider,
            input_json,
            fields,
            json,
        } => {
            let request = ResourceMutationRequest {
                provider: provider_with_alias(provider, provider_alias),
                fields: merge_fields(input_json.as_deref(), fields)?,
            };
            let response: Value = api
                .patch(&item_path(tenant_id, resource, resource_id), &request)
                .await?;
            print_resource(&response, *json)
        }
        ResourceCommand::Delete {
            resource_id,
            provider,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let path = with_provider(
                item_path(tenant_id, resource, resource_id),
                provider.as_deref(),
            );
            let response: Value = api.delete_json(&path).await?;
            if *json {
                print_json(&response)
            } else {
                let mode = response
                    .get("deletion_mode")
                    .and_then(Value::as_str)
                    .unwrap_or("deleted");
                println!("{resource} {resource_id}: {mode}");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_json_and_field_arguments() {
        let fields = merge_fields(
            Some(r#"{"name":"Roadmap","priority":2}"#),
            &["priority=3".to_string(), "teamIds=[\"team_1\"]".to_string()],
        )
        .unwrap();
        assert_eq!(fields["name"], "Roadmap");
        assert_eq!(fields["priority"], 3);
        assert_eq!(fields["teamIds"], serde_json::json!(["team_1"]));
    }

    #[test]
    fn builds_resource_paths() {
        assert_eq!(
            collection_path("tn_1", "projects"),
            "/v1beta/tn_1/pm/projects"
        );
        assert_eq!(
            item_path("tn_1", "projects", "prj_1"),
            "/v1beta/tn_1/pm/projects/prj_1"
        );
    }
}
