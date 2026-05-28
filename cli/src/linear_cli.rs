//! `tachyon linear` subcommand for tenant-scoped Linear issue operations.

use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};

#[derive(Debug, Clone, Args)]
pub struct LinearArgs {
    #[command(subcommand)]
    pub command: LinearCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum LinearCommand {
    /// Manage Linear issues
    Issue {
        #[command(subcommand)]
        command: LinearIssueCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum LinearIssueCommand {
    /// Create a Linear issue
    Create {
        /// Linear team UUID
        #[arg(long)]
        team_id: String,
        /// Issue title
        #[arg(long)]
        title: String,
        /// Markdown description
        #[arg(long)]
        description: Option<String>,
        /// Linear user UUID
        #[arg(long)]
        assignee_id: Option<String>,
        /// Linear label UUID. Can be specified multiple times.
        #[arg(long = "label-id")]
        label_ids: Vec<String>,
        /// Priority: 0=none, 1=urgent, 2=high, 3=medium, 4=low
        #[arg(long)]
        priority: Option<i32>,
        /// Linear project UUID
        #[arg(long)]
        project_id: Option<String>,
        /// Due date in YYYY-MM-DD format
        #[arg(long)]
        due_date: Option<String>,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
    /// List Linear issues
    List {
        /// Linear team UUID
        #[arg(long)]
        team_id: String,
        /// Linear project UUID
        #[arg(long)]
        project_id: Option<String>,
        /// Include completed/canceled issues
        #[arg(long)]
        include_completed: bool,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
    /// Update a Linear issue
    Update {
        /// Linear issue UUID or identifier such as PLT-1711
        issue_id: String,
        /// Linear team UUID; required when --state is a name
        #[arg(long)]
        team_id: Option<String>,
        /// Linear workflow state UUID
        #[arg(long)]
        state_id: Option<String>,
        /// Linear workflow state name
        #[arg(long)]
        state: Option<String>,
        /// New issue title
        #[arg(long)]
        title: Option<String>,
        /// Linear user UUID
        #[arg(long)]
        assignee_id: Option<String>,
        /// Priority: 0=none, 1=urgent, 2=high, 3=medium, 4=low
        #[arg(long)]
        priority: Option<i32>,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct CreateLinearIssueRequest {
    team_id: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    label_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct UpdateLinearIssueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    team_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LinearIssueSummary {
    id: String,
    identifier: String,
    title: String,
    url: String,
    state: String,
    priority: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateLinearIssueResponse {
    id: String,
    identifier: String,
    title: String,
    url: String,
    created: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ListLinearIssuesResponse {
    items: Vec<LinearIssueSummary>,
    count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateLinearIssueResponse {
    issue: LinearIssueSummary,
}

fn issues_path(tenant_id: &str) -> String {
    format!("/v1beta/{tenant_id}/roadmap/linear/issues")
}

fn issue_path(tenant_id: &str, issue_id: &str) -> String {
    format!("/v1beta/{tenant_id}/roadmap/linear/issues/{issue_id}")
}

async fn run_create(
    api: &ApiClient,
    tenant_id: &str,
    request: CreateLinearIssueRequest,
    json: bool,
) -> Result<()> {
    let response: CreateLinearIssueResponse = api.post(&issues_path(tenant_id), &request).await?;
    if json {
        return print_json(&response);
    }
    println!(
        "Linear issue {}: {} ({})",
        if response.created { "created" } else { "found" },
        response.identifier,
        response.url
    );
    Ok(())
}

async fn run_list(
    api: &ApiClient,
    tenant_id: &str,
    team_id: &str,
    project_id: Option<&str>,
    include_completed: bool,
    json: bool,
) -> Result<()> {
    let include_completed_value = include_completed.to_string();
    let mut query = vec![
        ("team_id", team_id),
        ("include_completed", include_completed_value.as_str()),
    ];
    if let Some(project_id) = project_id {
        query.push(("project_id", project_id));
    }
    let response: ListLinearIssuesResponse = api.get_query(&issues_path(tenant_id), &query).await?;
    if json {
        return print_json(&response);
    }
    for issue in &response.items {
        println!(
            "{}\t{}\t{}\t{}",
            issue.identifier, issue.state, issue.priority, issue.title
        );
    }
    Ok(())
}

async fn run_update(
    api: &ApiClient,
    tenant_id: &str,
    issue_id: &str,
    request: UpdateLinearIssueRequest,
    json: bool,
) -> Result<()> {
    let response: UpdateLinearIssueResponse = api
        .patch(&issue_path(tenant_id, issue_id), &request)
        .await?;
    if json {
        return print_json(&response);
    }
    println!(
        "Linear issue updated: {} ({})",
        response.issue.identifier, response.issue.url
    );
    Ok(())
}

pub async fn run(args: &LinearArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    match &args.command {
        LinearCommand::Issue { command } => match command {
            LinearIssueCommand::Create {
                team_id,
                title,
                description,
                assignee_id,
                label_ids,
                priority,
                project_id,
                due_date,
                json,
            } => {
                run_create(
                    &api,
                    tenant_id,
                    CreateLinearIssueRequest {
                        team_id: team_id.clone(),
                        title: title.clone(),
                        description: description.clone(),
                        assignee_id: assignee_id.clone(),
                        label_ids: label_ids.clone(),
                        priority: *priority,
                        project_id: project_id.clone(),
                        due_date: due_date.clone(),
                    },
                    *json,
                )
                .await
            }
            LinearIssueCommand::List {
                team_id,
                project_id,
                include_completed,
                json,
            } => {
                run_list(
                    &api,
                    tenant_id,
                    team_id,
                    project_id.as_deref(),
                    *include_completed,
                    *json,
                )
                .await
            }
            LinearIssueCommand::Update {
                issue_id,
                team_id,
                state_id,
                state,
                title,
                assignee_id,
                priority,
                json,
            } => {
                run_update(
                    &api,
                    tenant_id,
                    issue_id,
                    UpdateLinearIssueRequest {
                        team_id: team_id.clone(),
                        state_id: state_id.clone(),
                        state: state.clone(),
                        title: title.clone(),
                        assignee_id: assignee_id.clone(),
                        priority: *priority,
                    },
                    *json,
                )
                .await
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_linear_issue_paths() {
        assert_eq!(
            issues_path("tn_test"),
            "/v1beta/tn_test/roadmap/linear/issues"
        );
        assert_eq!(
            issue_path("tn_test", "PLT-1711"),
            "/v1beta/tn_test/roadmap/linear/issues/PLT-1711"
        );
    }

    #[test]
    fn serializes_create_request() {
        let request = CreateLinearIssueRequest {
            team_id: "team_1".to_string(),
            title: "Test issue".to_string(),
            description: None,
            assignee_id: None,
            label_ids: vec!["label_1".to_string()],
            priority: Some(3),
            project_id: None,
            due_date: None,
        };

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            serde_json::json!({
                "team_id": "team_1",
                "title": "Test issue",
                "label_ids": ["label_1"],
                "priority": 3
            })
        );
    }
}
