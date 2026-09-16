//! Commands for project status updates (the posts on a project's
//! "Updates" tab). These are distinct from `project update`, which edits
//! the project itself.

use std::io::Read;

use anyhow::{anyhow, Context, Result};
use clap::{Subcommand, ValueEnum};
use serde::Serialize;
use serde_json::{Map, Value};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};

const RESOURCE: &str = "project-updates";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProjectHealth {
    #[value(alias = "onTrack")]
    OnTrack,
    #[value(alias = "atRisk")]
    AtRisk,
    #[value(alias = "offTrack")]
    OffTrack,
}

impl ProjectHealth {
    fn provider_value(self) -> &'static str {
        match self {
            Self::OnTrack => "onTrack",
            Self::AtRisk => "atRisk",
            Self::OffTrack => "offTrack",
        }
    }
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum ProjectUpdateCommand {
    /// Post a status update to a project
    Create {
        #[arg(long)]
        provider: Option<String>,
        /// Project id the update is posted to
        #[arg(long = "project", value_name = "PROJECT_ID")]
        project_id: String,
        #[arg(long, value_enum)]
        health: Option<ProjectHealth>,
        /// Markdown body
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the Markdown body from a file (`-` for stdin)
        #[arg(long, value_name = "PATH")]
        body_file: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// List status updates
    List {
        #[arg(long)]
        provider: Option<String>,
        /// Only list updates posted to this project
        #[arg(long = "project", value_name = "PROJECT_ID")]
        project_id: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Get a status update
    Get {
        update_id: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Edit the body or health of a status update
    Update {
        update_id: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long, value_enum)]
        health: Option<ProjectHealth>,
        /// Markdown body
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the Markdown body from a file (`-` for stdin)
        #[arg(long, value_name = "PATH")]
        body_file: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Archive a status update
    Delete {
        update_id: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize)]
struct MutationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    fields: Map<String, Value>,
}

fn collection_path(tenant_id: &str) -> String {
    format!("/v1beta/{tenant_id}/pm/{RESOURCE}")
}

fn item_path(tenant_id: &str, update_id: &str) -> String {
    format!(
        "{}/{}",
        collection_path(tenant_id),
        urlencoding::encode(update_id)
    )
}

fn read_body(body: Option<&str>, body_file: Option<&str>) -> Result<Option<String>> {
    match (body, body_file) {
        (Some(body), _) => Ok(Some(body.to_string())),
        (None, Some("-")) => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .context("failed to read body from stdin")?;
            Ok(Some(buffer))
        }
        (None, Some(path)) => std::fs::read_to_string(path)
            .with_context(|| format!("failed to read --body-file {path}"))
            .map(Some),
        (None, None) => Ok(None),
    }
}

fn build_fields(
    project_id: Option<&str>,
    health: Option<ProjectHealth>,
    body: Option<String>,
) -> Map<String, Value> {
    let mut fields = Map::new();
    if let Some(project_id) = project_id {
        fields.insert(
            "projectId".to_string(),
            Value::String(project_id.to_string()),
        );
    }
    if let Some(health) = health {
        fields.insert(
            "health".to_string(),
            Value::String(health.provider_value().to_string()),
        );
    }
    if let Some(body) = body {
        fields.insert("body".to_string(), Value::String(body));
    }
    fields
}

fn attribute<'a>(value: &'a Value, pointer: &str) -> &'a str {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or("-")
}

fn print_update(value: &Value, json: bool) -> Result<()> {
    if json {
        return print_json(value);
    }
    println!("Project update {}", attribute(value, "/id"));
    println!(
        "  project: {} ({})",
        attribute(value, "/attributes/project/name"),
        attribute(value, "/attributes/project/id")
    );
    println!("  health:  {}", attribute(value, "/attributes/health"));
    println!("  author:  {}", attribute(value, "/attributes/user/name"));
    println!("  created: {}", attribute(value, "/attributes/createdAt"));
    println!("  url:     {}", attribute(value, "/url"));
    if let Some(body) = value.pointer("/attributes/body").and_then(Value::as_str) {
        println!();
        println!("{body}");
    }
    Ok(())
}

fn provider_with_alias(provider: &Option<String>, alias: Option<&str>) -> Option<String> {
    provider.clone().or_else(|| alias.map(str::to_string))
}

pub async fn run_project_update(
    command: &ProjectUpdateCommand,
    config: &Configuration,
    tenant_id: &str,
    provider_alias: Option<&str>,
) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;
    match command {
        ProjectUpdateCommand::Create {
            provider,
            project_id,
            health,
            body,
            body_file,
            json,
        } => {
            let body = read_body(body.as_deref(), body_file.as_deref())?
                .ok_or_else(|| anyhow!("--body or --body-file is required"))?;
            let request = MutationRequest {
                provider: provider_with_alias(provider, provider_alias),
                fields: build_fields(Some(project_id), *health, Some(body)),
            };
            let response: Value = api.post(&collection_path(tenant_id), &request).await?;
            print_update(&response, *json)
        }
        ProjectUpdateCommand::List {
            provider,
            project_id,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let mut query = Vec::new();
            if let Some(provider) = provider.as_deref() {
                query.push(("provider", provider));
            }
            if let Some(project_id) = project_id.as_deref() {
                query.push(("project_id", project_id));
            }
            let response: Value = api.get_query(&collection_path(tenant_id), &query).await?;
            if *json {
                return print_json(&response);
            }
            for item in response
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                println!(
                    "{}\t{}\t{}\t{}",
                    attribute(item, "/id"),
                    attribute(item, "/attributes/createdAt"),
                    attribute(item, "/attributes/health"),
                    attribute(item, "/attributes/project/name"),
                );
            }
            Ok(())
        }
        ProjectUpdateCommand::Get {
            update_id,
            provider,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let query = provider
                .as_deref()
                .map(|value| vec![("provider", value)])
                .unwrap_or_default();
            let response: Value = api
                .get_query(&item_path(tenant_id, update_id), &query)
                .await?;
            print_update(&response, *json)
        }
        ProjectUpdateCommand::Update {
            update_id,
            provider,
            health,
            body,
            body_file,
            json,
        } => {
            let body = read_body(body.as_deref(), body_file.as_deref())?;
            let fields = build_fields(None, *health, body);
            if fields.is_empty() {
                return Err(anyhow!(
                    "at least one of --health, --body or --body-file is required"
                ));
            }
            let request = MutationRequest {
                provider: provider_with_alias(provider, provider_alias),
                fields,
            };
            let response: Value = api
                .patch(&item_path(tenant_id, update_id), &request)
                .await?;
            print_update(&response, *json)
        }
        ProjectUpdateCommand::Delete {
            update_id,
            provider,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let mut path = item_path(tenant_id, update_id);
            if let Some(provider) = provider.as_deref() {
                path = format!("{path}?provider={}", urlencoding::encode(provider));
            }
            let response: Value = api.delete_json(&path).await?;
            if *json {
                return print_json(&response);
            }
            let mode = response
                .get("deletion_mode")
                .and_then(Value::as_str)
                .unwrap_or("archived");
            println!("project update {update_id}: {mode}");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Debug, Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: ProjectUpdateCommand,
    }

    #[test]
    fn parses_create_with_kebab_case_health() {
        let cli = TestCli::try_parse_from([
            "test",
            "create",
            "--provider",
            "linear",
            "--project",
            "prj_1",
            "--health",
            "on-track",
            "--body-file",
            "update.md",
        ])
        .unwrap();
        assert_eq!(
            cli.command,
            ProjectUpdateCommand::Create {
                provider: Some("linear".to_string()),
                project_id: "prj_1".to_string(),
                health: Some(ProjectHealth::OnTrack),
                body: None,
                body_file: Some("update.md".to_string()),
                json: false,
            }
        );
    }

    #[test]
    fn accepts_provider_spelling_of_health() {
        let cli =
            TestCli::try_parse_from(["test", "update", "upd_1", "--health", "atRisk"]).unwrap();
        match cli.command {
            ProjectUpdateCommand::Update { health, .. } => {
                assert_eq!(health, Some(ProjectHealth::AtRisk))
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn rejects_body_and_body_file_together() {
        let result = TestCli::try_parse_from([
            "test",
            "create",
            "--project",
            "prj_1",
            "--body",
            "text",
            "--body-file",
            "update.md",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn builds_provider_fields() {
        let fields = build_fields(
            Some("prj_1"),
            Some(ProjectHealth::OffTrack),
            Some("## Progress".to_string()),
        );
        assert_eq!(fields["projectId"], "prj_1");
        assert_eq!(fields["health"], "offTrack");
        assert_eq!(fields["body"], "## Progress");
        assert!(build_fields(None, None, None).is_empty());
    }

    #[test]
    fn builds_paths() {
        assert_eq!(collection_path("tn_1"), "/v1beta/tn_1/pm/project-updates");
        assert_eq!(
            item_path("tn_1", "upd_1"),
            "/v1beta/tn_1/pm/project-updates/upd_1"
        );
    }
}
