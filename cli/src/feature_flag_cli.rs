//! Tenant-scoped feature flag commands. Keyed writes preserve server-side
//! ownership, publication approvals and inherited-flag rules.
use crate::client::ApiClient;
use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use serde_json::{json, Value};
use tachyon_sdk::apis::configuration::Configuration;

#[derive(Debug, Args)]
pub struct FeatureFlagArgs {
    #[command(subcommand)]
    pub command: FeatureFlagCommand,
    /// Print the server result as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Debug, Subcommand)]
pub enum FeatureFlagCommand {
    /// List effective flags visible in the selected tenant
    List {
        #[arg(long)]
        search: Option<String>,
    },
    /// Get an effective flag by key
    Get { key: String },
    /// Set a flag ON; server approval and ownership rules still apply
    Enable { key: String },
    /// Set a flag OFF; server approval and ownership rules still apply
    Disable { key: String },
}

const FIELDS: &str = "id key name enabled tenantId inheritedFrom baselineEnabled isOverride";

fn request(command: &FeatureFlagCommand) -> Value {
    match command {
        FeatureFlagCommand::List { search } => json!({
            "query": format!("query($filter: FeatureFlagFilter) {{ featureFlags(filter: $filter) {{ edges {{ node {{ {FIELDS} }} }} totalCount }} }}"),
            "variables": { "filter": search.as_ref().map(|s| json!({"search": s})) },
        }),
        FeatureFlagCommand::Get { key } => json!({
            "query": format!("query($key: String!) {{ featureFlagByKey(key: $key) {{ {FIELDS} }} }}"),
            "variables": { "key": key },
        }),
        FeatureFlagCommand::Enable { key } | FeatureFlagCommand::Disable { key } => json!({
            "query": "mutation($key: String!, $enabled: Boolean!) { setFeatureFlagEnabled(key: $key, enabled: $enabled) { key managingScopeId ownerScopeId targetScopeId revision outcome } }",
            "variables": { "key": key, "enabled": matches!(command, FeatureFlagCommand::Enable { .. }) },
        }),
    }
}

async fn execute(api: &ApiClient, command: &FeatureFlagCommand) -> Result<Value> {
    let response: Value = api.post("/v1/graphql", &request(command)).await?;
    if let Some(errors) = response
        .get("errors")
        .and_then(Value::as_array)
        .filter(|e| !e.is_empty())
    {
        let messages: Vec<&str> = errors
            .iter()
            .map(|e| e["message"].as_str().unwrap_or("GraphQL error"))
            .collect();
        return Err(anyhow!("{}", messages.join("; ")));
    }
    let field = match command {
        FeatureFlagCommand::List { .. } => "featureFlags",
        FeatureFlagCommand::Get { .. } => "featureFlagByKey",
        _ => "setFeatureFlagEnabled",
    };
    response
        .get("data")
        .and_then(|d| d.get(field))
        .filter(|v| !v.is_null())
        .cloned()
        .ok_or_else(|| {
            anyhow!("No {field} result returned; check the flag key and access permissions")
        })
}

pub async fn run(args: &FeatureFlagArgs, config: &Configuration, tenant: &str) -> Result<()> {
    let result = execute(&ApiClient::new(config, tenant)?, &args.command).await?;
    if !args.json && matches!(args.command, FeatureFlagCommand::List { .. }) {
        #[derive(serde::Deserialize)]
        struct FlagRow {
            enabled: bool,
            key: String,
            name: String,
        }
        println!("STATE\tKEY\tNAME");
        for edge in result["edges"]
            .as_array()
            .ok_or_else(|| anyhow!("Missing flag list edges"))?
        {
            let flag: FlagRow = serde_json::from_value(edge["node"].clone())?;
            println!(
                "{}\t{}\t{}",
                if flag.enabled { "ON" } else { "OFF" },
                flag.key,
                flag.name
            );
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::post, Json, Router};
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn writes_explicit_state_with_tenant_and_propagates_graphql_errors() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let requests = captured.clone();
        let app = Router::new().route(
            "/v1/graphql",
            post(
                move |headers: axum::http::HeaderMap, Json(body): Json<Value>| {
                    let requests = requests.clone();
                    async move {
                        assert_eq!(headers["x-operator-id"], "tn_test");
                        requests.lock().unwrap().push(body.clone());
                        Json(if body["variables"]["key"] == "forbidden" {
                            json!({"data": null, "errors": [{"message": "approval required"}]})
                        } else {
                            json!({"data": {"setFeatureFlagEnabled": {"outcome": "UNCHANGED"}}})
                        })
                    }
                },
            ),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let mut config = Configuration::new();
        config.base_path = format!("http://{address}");
        let api = ApiClient::new(&config, "tn_test").unwrap();
        for command in [
            FeatureFlagCommand::Enable { key: "test".into() },
            FeatureFlagCommand::Enable { key: "test".into() },
            FeatureFlagCommand::Disable { key: "test".into() },
        ] {
            execute(&api, &command).await.unwrap();
        }
        let error = execute(
            &api,
            &FeatureFlagCommand::Enable {
                key: "forbidden".into(),
            },
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("approval required"));
        let requests = captured.lock().unwrap();
        assert_eq!(requests[0]["variables"]["enabled"], true);
        assert_eq!(requests[1]["variables"]["enabled"], true);
        assert_eq!(requests[2]["variables"]["enabled"], false);
        server.abort();
    }

    #[test]
    fn list_and_get_keep_search_and_keys_in_variables() {
        let list = request(&FeatureFlagCommand::List {
            search: Some("frontier".into()),
        });
        assert_eq!(list["variables"]["filter"]["search"], "frontier");
        let get = request(&FeatureFlagCommand::Get {
            key: "quoted\"key".into(),
        });
        assert_eq!(get["variables"]["key"], "quoted\"key");
        assert!(!get["query"].as_str().unwrap().contains("quoted"));
    }
}
