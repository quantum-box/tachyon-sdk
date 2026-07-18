//! `tachyon pm` subcommands for provider-agnostic PM operations.

use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};
use crate::pm_resource_cli::{self, ResourceCommand};

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
        command: Box<IssueCommand>,
    },
    /// Manage projects
    Project {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage initiatives
    Initiative {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage cycles
    Cycle {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage teams
    Team {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage issue labels
    Label {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage documents
    Document {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage project milestones
    Milestone {
        #[command(subcommand)]
        command: ResourceCommand,
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
        /// Provider-specific delegate agent user id
        #[arg(long)]
        delegate_id: Option<String>,
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
        /// Cycle id
        #[arg(long)]
        cycle_id: Option<String>,
        /// Project milestone id
        #[arg(long)]
        project_milestone_id: Option<String>,
        /// Parent issue id or key. Creates this issue as a sub-issue.
        #[arg(long, visible_alias = "parent")]
        parent_issue_id: Option<String>,
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
    /// Get an issue by id or key
    Get {
        issue_id: String,
        #[arg(long)]
        provider: Option<String>,
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
        /// New Markdown description
        #[arg(long)]
        description: Option<String>,
        /// Provider-specific assignee id
        #[arg(long, visible_alias = "assignee")]
        assignee_id: Option<String>,
        /// Provider-specific delegate agent user id
        #[arg(long)]
        delegate_id: Option<String>,
        /// Priority: urgent, high, medium, low, none, or provider alias
        #[arg(long)]
        priority: Option<String>,
        /// Replace all label ids. Can be specified multiple times.
        #[arg(long = "label-id")]
        label_ids: Vec<String>,
        /// Add a label id. Can be specified multiple times.
        #[arg(long = "add-label-id")]
        added_label_ids: Vec<String>,
        /// Remove a label id. Can be specified multiple times.
        #[arg(long = "remove-label-id")]
        removed_label_ids: Vec<String>,
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        cycle_id: Option<String>,
        #[arg(long)]
        project_milestone_id: Option<String>,
        #[arg(long)]
        due_date: Option<String>,
        /// Set the parent issue id or key
        #[arg(long, visible_alias = "parent")]
        parent_issue_id: Option<String>,
        /// Remove the current parent and promote the issue to top level
        #[arg(long, conflicts_with = "parent_issue_id")]
        remove_parent: bool,
        /// Print the API response as JSON
        #[arg(long)]
        json: bool,
    },
    /// Delete an issue
    Delete {
        issue_id: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// List direct sub-issues
    Children {
        issue_id: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Manage issue comments
    Comment {
        #[command(subcommand)]
        command: CommentCommand,
    },
    /// Manage issue relations
    Relation {
        #[command(subcommand)]
        command: RelationCommand,
    },
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum CommentCommand {
    List {
        issue_id: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Create {
        issue_id: String,
        #[arg(long)]
        body: String,
        #[arg(long)]
        parent_id: Option<String>,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Update {
        issue_id: String,
        comment_id: String,
        #[arg(long)]
        body: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Delete {
        issue_id: String,
        comment_id: String,
        #[arg(long)]
        provider: Option<String>,
    },
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum RelationCommand {
    List {
        issue_id: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Create {
        issue_id: String,
        related_issue_id: String,
        #[arg(long, default_value = "related")]
        relation_type: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Delete {
        issue_id: String,
        relation_id: String,
        #[arg(long)]
        provider: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    delegate_id: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    auto_delegate_to_linear_agent: bool,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    cycle_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_milestone_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_issue_id: Option<String>,
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
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegate_id: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    auto_delegate_to_linear_agent: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    added_label_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    removed_label_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cycle_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_milestone_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_issue_id: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    remove_parent: bool,
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

fn issue_subresource_path(tenant_id: &str, issue_id: &str, subresource: &str) -> String {
    format!("{}/{subresource}", issue_path(tenant_id, issue_id))
}

fn provider_query(provider: Option<&str>) -> Vec<(&str, &str)> {
    provider
        .map(|value| vec![("provider", value)])
        .unwrap_or_default()
}

fn provider_with_alias(provider: &Option<String>, provider_alias: Option<&str>) -> Option<String> {
    provider
        .clone()
        .or_else(|| provider_alias.map(str::to_string))
}

fn is_linear_provider(provider: &Option<String>) -> bool {
    provider
        .as_deref()
        .is_some_and(|provider| provider.eq_ignore_ascii_case("linear"))
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

fn print_collection(
    response: &serde_json::Value,
    json: bool,
    id_field: &str,
    text_field: &str,
) -> Result<()> {
    if json {
        return print_json(response);
    }
    for item in response
        .get("items")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        let id = item
            .get(id_field)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("-");
        let text = item
            .get(text_field)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("-");
        println!("{id}\t{text}");
    }
    Ok(())
}

async fn run_comment(
    api: &ApiClient,
    tenant_id: &str,
    command: &CommentCommand,
    provider_alias: Option<&str>,
) -> Result<()> {
    match command {
        CommentCommand::List {
            issue_id,
            provider,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let response: serde_json::Value = api
                .get_query(
                    &issue_subresource_path(tenant_id, issue_id, "comments"),
                    &provider_query(provider.as_deref()),
                )
                .await?;
            print_collection(&response, *json, "id", "body")
        }
        CommentCommand::Create {
            issue_id,
            body,
            parent_id,
            provider,
            json,
        } => {
            let request = serde_json::json!({
                "provider": provider_with_alias(provider, provider_alias),
                "body": body,
                "parent_id": parent_id,
            });
            let response: serde_json::Value = api
                .post(
                    &issue_subresource_path(tenant_id, issue_id, "comments"),
                    &request,
                )
                .await?;
            if *json {
                print_json(&response)
            } else {
                println!(
                    "Comment created: {}",
                    response
                        .get("id")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("-")
                );
                Ok(())
            }
        }
        CommentCommand::Update {
            issue_id,
            comment_id,
            body,
            provider,
            json,
        } => {
            let request = serde_json::json!({
                "provider": provider_with_alias(provider, provider_alias),
                "body": body,
            });
            let path = format!(
                "{}/{}",
                issue_subresource_path(tenant_id, issue_id, "comments"),
                comment_id
            );
            let response: serde_json::Value = api.patch(&path, &request).await?;
            if *json {
                print_json(&response)
            } else {
                println!("Comment updated: {comment_id}");
                Ok(())
            }
        }
        CommentCommand::Delete {
            issue_id,
            comment_id,
            provider,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let mut path = format!(
                "{}/{}",
                issue_subresource_path(tenant_id, issue_id, "comments"),
                comment_id
            );
            if let Some(provider) = provider.as_deref() {
                path.push_str(&format!("?provider={}", urlencoding::encode(provider)));
            }
            api.delete(&path).await?;
            println!("Comment deleted: {comment_id}");
            Ok(())
        }
    }
}

async fn run_relation(
    api: &ApiClient,
    tenant_id: &str,
    command: &RelationCommand,
    provider_alias: Option<&str>,
) -> Result<()> {
    match command {
        RelationCommand::List {
            issue_id,
            provider,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let response: serde_json::Value = api
                .get_query(
                    &issue_subresource_path(tenant_id, issue_id, "relations"),
                    &provider_query(provider.as_deref()),
                )
                .await?;
            if *json {
                return print_json(&response);
            }
            for item in response
                .get("items")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
            {
                let id = item
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("-");
                let relation_type = item
                    .get("relation_type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("related");
                let related = item
                    .get("related_issue")
                    .and_then(|v| v.get("key"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("-");
                println!("{id}\t{relation_type}\t{related}");
            }
            Ok(())
        }
        RelationCommand::Create {
            issue_id,
            related_issue_id,
            relation_type,
            provider,
            json,
        } => {
            let request = serde_json::json!({
                "provider": provider_with_alias(provider, provider_alias),
                "related_issue_id": related_issue_id,
                "relation_type": relation_type,
            });
            let response: serde_json::Value = api
                .post(
                    &issue_subresource_path(tenant_id, issue_id, "relations"),
                    &request,
                )
                .await?;
            if *json {
                print_json(&response)
            } else {
                println!(
                    "Relation created: {}",
                    response
                        .get("id")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("-")
                );
                Ok(())
            }
        }
        RelationCommand::Delete {
            issue_id,
            relation_id,
            provider,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let mut path = format!(
                "{}/{}",
                issue_subresource_path(tenant_id, issue_id, "relations"),
                relation_id
            );
            if let Some(provider) = provider.as_deref() {
                path.push_str(&format!("?provider={}", urlencoding::encode(provider)));
            }
            api.delete(&path).await?;
            println!("Relation deleted: {relation_id}");
            Ok(())
        }
    }
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
            delegate_id,
            label_ids,
            priority,
            project,
            project_id,
            due_date,
            cycle_id,
            project_milestone_id,
            parent_issue_id,
            related_issue_ids,
            skip_if_exists,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let auto_delegate_to_linear_agent =
                delegate_id.is_none() && is_linear_provider(&provider);
            run_create(
                &api,
                tenant_id,
                CreatePmIssueRequest {
                    provider,
                    team: team.clone(),
                    team_id: team_id.clone(),
                    title: title.clone(),
                    description: description.clone(),
                    assignee_id: assignee_id.clone(),
                    delegate_id: delegate_id.clone(),
                    auto_delegate_to_linear_agent,
                    label_ids: label_ids.clone(),
                    priority: priority.clone(),
                    project: project.clone(),
                    project_id: project_id.clone(),
                    due_date: due_date.clone(),
                    cycle_id: cycle_id.clone(),
                    project_milestone_id: project_milestone_id.clone(),
                    parent_issue_id: parent_issue_id.clone(),
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
        IssueCommand::Get {
            issue_id,
            provider,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let response: PmIssue = api
                .get_query(
                    &issue_path(tenant_id, issue_id),
                    &provider_query(provider.as_deref()),
                )
                .await?;
            if *json {
                print_json(&response)
            } else {
                println!(
                    "{}\t{}\t{}\t{}",
                    response.key, response.status, response.priority, response.title
                );
                Ok(())
            }
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
            description,
            assignee_id,
            delegate_id,
            priority,
            label_ids,
            added_label_ids,
            removed_label_ids,
            project_id,
            cycle_id,
            project_milestone_id,
            due_date,
            parent_issue_id,
            remove_parent,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let auto_delegate_to_linear_agent =
                delegate_id.is_none() && is_linear_provider(&provider);
            run_update(
                &api,
                tenant_id,
                issue_id,
                UpdatePmIssueRequest {
                    provider,
                    team: team.clone(),
                    team_id: team_id.clone(),
                    status_id: status_id.clone(),
                    status: status.clone(),
                    state_id: state_id.clone(),
                    state: state.clone(),
                    title: title.clone(),
                    description: description.clone(),
                    assignee_id: assignee_id.clone(),
                    delegate_id: delegate_id.clone(),
                    auto_delegate_to_linear_agent,
                    priority: priority.clone(),
                    label_ids: (!label_ids.is_empty()).then(|| label_ids.clone()),
                    added_label_ids: added_label_ids.clone(),
                    removed_label_ids: removed_label_ids.clone(),
                    project_id: project_id.clone(),
                    cycle_id: cycle_id.clone(),
                    project_milestone_id: project_milestone_id.clone(),
                    due_date: due_date.clone(),
                    parent_issue_id: parent_issue_id.clone(),
                    remove_parent: *remove_parent,
                },
                *json,
            )
            .await
        }
        IssueCommand::Delete {
            issue_id,
            provider,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let path = match provider.as_deref() {
                Some(value) => format!(
                    "{}?provider={}",
                    issue_path(tenant_id, issue_id),
                    urlencoding::encode(value)
                ),
                None => issue_path(tenant_id, issue_id),
            };
            let response: serde_json::Value = api.delete_json(&path).await?;
            if *json {
                print_json(&response)
            } else {
                println!(
                    "Issue {issue_id}: {}",
                    response
                        .get("deletion_mode")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("deleted")
                );
                Ok(())
            }
        }
        IssueCommand::Children {
            issue_id,
            provider,
            json,
        } => {
            let provider = provider_with_alias(provider, provider_alias);
            let response: serde_json::Value = api
                .get_query(
                    &issue_subresource_path(tenant_id, issue_id, "children"),
                    &provider_query(provider.as_deref()),
                )
                .await?;
            print_collection(&response, *json, "key", "title")
        }
        IssueCommand::Comment { command } => {
            run_comment(&api, tenant_id, command, provider_alias).await
        }
        IssueCommand::Relation { command } => {
            run_relation(&api, tenant_id, command, provider_alias).await
        }
    }
}

pub async fn run(args: &PmArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    match &args.command {
        PmCommand::Issue { command } => run_issue(command, config, tenant_id, None).await,
        PmCommand::Project { command } => {
            pm_resource_cli::run_resource("projects", command, config, tenant_id, None).await
        }
        PmCommand::Initiative { command } => {
            pm_resource_cli::run_resource("initiatives", command, config, tenant_id, None).await
        }
        PmCommand::Cycle { command } => {
            pm_resource_cli::run_resource("cycles", command, config, tenant_id, None).await
        }
        PmCommand::Team { command } => {
            pm_resource_cli::run_resource("teams", command, config, tenant_id, None).await
        }
        PmCommand::Label { command } => {
            pm_resource_cli::run_resource("labels", command, config, tenant_id, None).await
        }
        PmCommand::Document { command } => {
            pm_resource_cli::run_resource("documents", command, config, tenant_id, None).await
        }
        PmCommand::Milestone { command } => {
            pm_resource_cli::run_resource("milestones", command, config, tenant_id, None).await
        }
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
            delegate_id: None,
            auto_delegate_to_linear_agent: false,
            label_ids: vec!["label_1".to_string()],
            priority: Some("medium".to_string()),
            project: None,
            project_id: None,
            due_date: None,
            cycle_id: None,
            project_milestone_id: None,
            parent_issue_id: None,
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

    #[test]
    fn serializes_linear_auto_delegate_create_request() {
        let request = CreatePmIssueRequest {
            provider: Some("linear".to_string()),
            team: Some("PLT".to_string()),
            team_id: None,
            title: "Test issue".to_string(),
            description: None,
            assignee_id: None,
            delegate_id: None,
            auto_delegate_to_linear_agent: true,
            label_ids: Vec::new(),
            priority: None,
            project: None,
            project_id: None,
            due_date: None,
            cycle_id: None,
            project_milestone_id: None,
            parent_issue_id: None,
            related_issue_ids: Vec::new(),
            skip_if_exists: false,
        };

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            serde_json::json!({
                "provider": "linear",
                "team": "PLT",
                "title": "Test issue",
                "auto_delegate_to_linear_agent": true
            })
        );
    }

    #[test]
    fn serializes_explicit_delegate_update_request() {
        let request = UpdatePmIssueRequest {
            provider: Some("linear".to_string()),
            team: None,
            team_id: None,
            status_id: None,
            status: None,
            state_id: None,
            state: None,
            title: None,
            description: None,
            assignee_id: None,
            delegate_id: Some("agent_user_1".to_string()),
            auto_delegate_to_linear_agent: false,
            priority: None,
            label_ids: None,
            added_label_ids: Vec::new(),
            removed_label_ids: Vec::new(),
            project_id: None,
            cycle_id: None,
            project_milestone_id: None,
            due_date: None,
            parent_issue_id: None,
            remove_parent: false,
        };

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            serde_json::json!({
                "provider": "linear",
                "delegate_id": "agent_user_1"
            })
        );
    }

    #[test]
    fn detects_linear_provider_case_insensitively() {
        assert!(is_linear_provider(&Some("Linear".to_string())));
        assert!(!is_linear_provider(&Some("jira".to_string())));
        assert!(!is_linear_provider(&None));
    }
}
