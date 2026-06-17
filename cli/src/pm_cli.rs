//! `tachyon pm` subcommands for provider-agnostic PM operations.

use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct PmArgs {
    #[command(subcommand)]
    pub command: PmCommand,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum PmCommand {
    /// Manage issues in a configured project-management provider
    Issue {
        #[command(subcommand)]
        command: IssueCommand,
    },
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct IssueArgs {
    #[command(subcommand)]
    pub command: IssueCommand,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum IssueCommand {
    /// Create an issue
    Create {
        /// PM provider, such as linear
        #[arg(long)]
        provider: Option<String>,
        /// Team id, key, or name
        #[arg(long)]
        team: Option<String>,
        /// Legacy team id option
        #[arg(long)]
        team_id: Option<String>,
        /// Issue title
        #[arg(long)]
        title: String,
        /// Markdown description
        #[arg(long)]
        description: Option<String>,
        /// Provider-specific assignee id
        #[arg(long, visible_alias = "assignee")]
        assignee_id: Option<String>,
        /// Provider-specific label id. Can be specified multiple times.
        #[arg(long = "label-id")]
        label_ids: Vec<String>,
        /// Priority: urgent, high, medium, low, none, or provider alias
        #[arg(long)]
        priority: Option<String>,
        /// Project id or name
        #[arg(long)]
        project: Option<String>,
        /// Legacy project id option
        #[arg(long)]
        project_id: Option<String>,
        /// Due date in YYYY-MM-DD format
        #[arg(long)]
        due_date: Option<String>,
        /// Related issue id. Can be specified multiple times.
        #[arg(long = "related-issue-id")]
        related_issue_ids: Vec<String>,
        /// Return an existing issue with the same title instead of creating
        #[arg(long)]
        skip_if_exists: bool,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
    /// List issues
    List {
        /// PM provider, such as linear
        #[arg(long)]
        provider: Option<String>,
        /// Team id, key, or name
        #[arg(long)]
        team: Option<String>,
        /// Legacy team id option
        #[arg(long)]
        team_id: Option<String>,
        /// Project id or name
        #[arg(long)]
        project: Option<String>,
        /// Legacy project id option
        #[arg(long)]
        project_id: Option<String>,
        /// Include completed/canceled issues
        #[arg(long)]
        include_completed: bool,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
    /// Update an issue
    Update {
        /// Issue id or key, such as PLT-1711
        issue_id: String,
        /// PM provider, such as linear
        #[arg(long)]
        provider: Option<String>,
        /// Team id, key, or name; required when status is a name
        #[arg(long)]
        team: Option<String>,
        /// Legacy team id option
        #[arg(long)]
        team_id: Option<String>,
        /// Provider-specific status id
        #[arg(long)]
        status_id: Option<String>,
        /// Provider-specific status name
        #[arg(long)]
        status: Option<String>,
        /// Legacy Linear workflow state id
        #[arg(long)]
        state_id: Option<String>,
        /// Legacy Linear workflow state name
        #[arg(long)]
        state: Option<String>,
        /// New issue title
        #[arg(long)]
        title: Option<String>,
        /// Provider-specific assignee id
        #[arg(long, visible_alias = "assignee")]
        assignee_id: Option<String>,
        /// Priority: urgent, high, medium, low, none, or provider alias
        #[arg(long)]
        priority: Option<String>,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
struct CreatePmIssueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    team: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    team_id: Option<String>,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    label_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    related_issue_ids: Vec<String>,
    #[serde(skip_serializing_if = "is_false")]
    skip_if_exists: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
struct UpdatePmIssueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    team: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    team_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct PmIssue {
    provider: String,
    id: String,
    key: String,
    title: String,
    url: String,
    status: String,
    priority: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct CreatePmIssueResponse {
    issue: PmIssue,
    created: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct ListPmIssuesResponse {
    items: Vec<PmIssue>,
    count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct UpdatePmIssueResponse {
    issue: PmIssue,
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn issues_path(tenant_id: &str) -> String {
    format!("/v1beta/{tenant_id}/pm/issues")
}

fn issue_path(tenant_id: &str, issue_id: &str) -> String {
    format!("/v1beta/{tenant_id}/pm/issues/{issue_id}")
}

fn provider_with_alias(provider: &Option<String>, provider_alias: Option<&str>) -> Option<String> {
    provider
        .clone()
        .or_else(|| provider_alias.map(str::to_string))
}

async fn run_create(
    api: &ApiClient,
    tenant_id: &str,
    request: CreatePmIssueRequest,
    json: bool,
) -> Result<()> {
    let response: CreatePmIssueResponse = api.post(&issues_path(tenant_id), &request).await?;
    if json {
        return print_json(&response);
    }
    println!(
        "Issue {}: {} ({})",
        if response.created { "created" } else { "found" },
        response.issue.key,
        response.issue.url
    );
    Ok(())
}

async fn run_list(
    api: &ApiClient,
    tenant_id: &str,
    query: Vec<(&str, String)>,
    json: bool,
) -> Result<()> {
    let query_refs = query
        .iter()
        .map(|(key, value)| (*key, value.as_str()))
        .collect::<Vec<_>>();
    let response: ListPmIssuesResponse =
        api.get_query(&issues_path(tenant_id), &query_refs).await?;
    if json {
        return print_json(&response);
    }
    for issue in &response.items {
        println!(
            "{}\t{}\t{}\t{}",
            issue.key, issue.status, issue.priority, issue.title
        );
    }
    Ok(())
}

async fn run_update(
    api: &ApiClient,
    tenant_id: &str,
    issue_id: &str,
    request: UpdatePmIssueRequest,
    json: bool,
) -> Result<()> {
    let response: UpdatePmIssueResponse = api
        .patch(&issue_path(tenant_id, issue_id), &request)
        .await?;
    if json {
        return print_json(&response);
    }
    println!(
        "Issue updated: {} ({})",
        response.issue.key, response.issue.url
    );
    Ok(())
}

pub async fn run_issue(
    command: &IssueCommand,
    config: &Configuration,
    tenant_id: &str,
    provider_alias: Option<&str>,
) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    match command {
        IssueCommand::Create {
            provider,
            team,
            team_id,
            title,
            description,
            assignee_id,
            label_ids,
            priority,
            project,
            project_id,
            due_date,
            related_issue_ids,
            skip_if_exists,
            json,
        } => {
            run_create(
                &api,
                tenant_id,
                CreatePmIssueRequest {
                    provider: provider_with_alias(provider, provider_alias),
                    team: team.clone(),
                    team_id: team_id.clone(),
                    title: title.clone(),
                    description: description.clone(),
                    assignee_id: assignee_id.clone(),
                    label_ids: label_ids.clone(),
                    priority: priority.clone(),
                    project: project.clone(),
                    project_id: project_id.clone(),
                    due_date: due_date.clone(),
                    related_issue_ids: related_issue_ids.clone(),
                    skip_if_exists: *skip_if_exists,
                },
                *json,
            )
            .await
        }
        IssueCommand::List {
            provider,
            team,
            team_id,
            project,
            project_id,
            include_completed,
            json,
        } => {
            let mut query = vec![("include_completed", include_completed.to_string())];
            if let Some(provider) = provider_with_alias(provider, provider_alias) {
                query.push(("provider", provider));
            }
            if let Some(team) = team {
                query.push(("team", team.clone()));
            }
            if let Some(team_id) = team_id {
                query.push(("team_id", team_id.clone()));
            }
            if let Some(project) = project {
                query.push(("project", project.clone()));
            }
            if let Some(project_id) = project_id {
                query.push(("project_id", project_id.clone()));
            }
            run_list(&api, tenant_id, query, *json).await
        }
        IssueCommand::Update {
            issue_id,
            provider,
            team,
            team_id,
            status_id,
            status,
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
                UpdatePmIssueRequest {
                    provider: provider_with_alias(provider, provider_alias),
                    team: team.clone(),
                    team_id: team_id.clone(),
                    status_id: status_id.clone(),
                    status: status.clone(),
                    state_id: state_id.clone(),
                    state: state.clone(),
                    title: title.clone(),
                    assignee_id: assignee_id.clone(),
                    priority: priority.clone(),
                },
                *json,
            )
            .await
        }
    }
}

pub async fn run(args: &PmArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    match &args.command {
        PmCommand::Issue { command } => run_issue(command, config, tenant_id, None).await,
    }
}

pub async fn run_top_level_issue(
    args: &IssueArgs,
    config: &Configuration,
    tenant_id: &str,
) -> Result<()> {
    run_issue(&args.command, config, tenant_id, None).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_pm_issue_paths() {
        assert_eq!(issues_path("tn_test"), "/v1beta/tn_test/pm/issues");
        assert_eq!(
            issue_path("tn_test", "PLT-1711"),
            "/v1beta/tn_test/pm/issues/PLT-1711"
        );
    }

    #[test]
    fn serializes_create_request() {
        let request = CreatePmIssueRequest {
            provider: Some("linear".to_string()),
            team: Some("PLT".to_string()),
            team_id: None,
            title: "Test issue".to_string(),
            description: None,
            assignee_id: None,
            label_ids: vec!["label_1".to_string()],
            priority: Some("medium".to_string()),
            project: None,
            project_id: None,
            due_date: None,
            related_issue_ids: Vec::new(),
            skip_if_exists: false,
        };

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            serde_json::json!({
                "provider": "linear",
                "team": "PLT",
                "title": "Test issue",
                "label_ids": ["label_1"],
                "priority": "medium"
            })
        );
    }
}
