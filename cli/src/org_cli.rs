use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Deserialize;
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{api_url, build_client, get_json, get_json_with_query};

#[derive(Debug, Clone, Args)]
pub struct OrgArgs {
    #[command(subcommand)]
    pub command: OrgCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum OrgCommand {
    /// Manage policies and actions
    Policies(PoliciesArgs),
    /// Manage users
    Users(UsersArgs),
    /// Manage operators
    Operators(OperatorsArgs),
    /// Manage service accounts
    #[command(name = "service-accounts")]
    ServiceAccounts(ServiceAccountsArgs),
}

// ── Policies ──

#[derive(Debug, Clone, Args)]
pub struct PoliciesArgs {
    #[command(subcommand)]
    pub command: PoliciesCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum PoliciesCommand {
    /// List all auth actions
    Actions {
        /// Filter by context (e.g. "auth", "llms")
        #[arg(long)]
        context: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActionListResponse {
    actions: Vec<ActionResponse>,
    #[allow(dead_code)]
    total_count: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActionResponse {
    #[allow(dead_code)]
    id: String,
    context: String,
    name: String,
    full_name: String,
    description: Option<String>,
}

// ── Users ──

#[derive(Debug, Clone, Args)]
pub struct UsersArgs {
    #[command(subcommand)]
    pub command: UsersCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum UsersCommand {
    /// List users in the operator
    List,
}

#[derive(Debug, Deserialize)]
struct UserListResponse {
    users: Vec<UserResponse>,
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    id: String,
    email: Option<String>,
    name: Option<String>,
    role: Option<String>,
}

// ── Operators ──

#[derive(Debug, Clone, Args)]
pub struct OperatorsArgs {
    #[command(subcommand)]
    pub command: OperatorsCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum OperatorsCommand {
    /// List operators for a platform
    List {
        /// Platform tenant ID (required for operator listing)
        #[arg(long, env = "TACHYON_PLATFORM_ID")]
        platform_id: String,
        /// User ID to filter by
        #[arg(long, env = "TACHYON_USER_ID")]
        user_id: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct OperatorListResponse {
    operators: Vec<OperatorResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OperatorResponse {
    id: String,
    name: String,
    operator_name: Option<String>,
}

// ── Service Accounts ──

#[derive(Debug, Clone, Args)]
pub struct ServiceAccountsArgs {
    #[command(subcommand)]
    pub command: ServiceAccountsCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServiceAccountsCommand {
    /// List service accounts
    List,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceAccountListResponse {
    service_accounts: Vec<ServiceAccountResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceAccountResponse {
    id: String,
    #[allow(dead_code)]
    tenant_id: Option<String>,
    name: String,
    created_at: Option<String>,
}

// ── Runners ──

async fn run_actions(config: &Configuration, tenant_id: &str, context: Option<&str>) -> Result<()> {
    let client = build_client(config, tenant_id)?;
    let url = api_url(config, "/v1/auth/actions");

    let resp: ActionListResponse = if let Some(ctx) = context {
        get_json_with_query(&client, &url, &[("context", ctx)]).await?
    } else {
        get_json(&client, &url).await?
    };

    if resp.actions.is_empty() {
        println!("No actions found.");
        return Ok(());
    }

    let ctx_w = 12;
    let name_w = 30;
    let full_w = 40;

    println!(
        "{:<ctx_w$}  {:<name_w$}  {:<full_w$}  DESCRIPTION",
        "CONTEXT", "NAME", "FULL NAME",
    );
    println!(
        "{:-<ctx_w$}  {:-<name_w$}  {:-<full_w$}  -----------",
        "", "", "",
    );

    for a in &resp.actions {
        println!(
            "{:<ctx_w$}  {:<name_w$}  {:<full_w$}  {}",
            a.context,
            a.name,
            a.full_name,
            a.description.as_deref().unwrap_or("-"),
        );
    }

    if let Some(total) = resp.total_count {
        println!("\nTotal: {total}");
    }

    Ok(())
}

async fn run_users_list(config: &Configuration, tenant_id: &str) -> Result<()> {
    let client = build_client(config, tenant_id)?;
    let url = api_url(config, "/v1/auth/users");

    // operator_id query parameter is required
    let resp: UserListResponse =
        get_json_with_query(&client, &url, &[("operator_id", tenant_id)]).await?;

    if resp.users.is_empty() {
        println!("No users found.");
        return Ok(());
    }

    let id_w = 30;
    let email_w = 30;
    let name_w = 20;

    println!(
        "{:<id_w$}  {:<email_w$}  {:<name_w$}  ROLE",
        "ID", "EMAIL", "NAME",
    );
    println!("{:-<id_w$}  {:-<email_w$}  {:-<name_w$}  ----", "", "", "",);

    for u in &resp.users {
        println!(
            "{:<id_w$}  {:<email_w$}  {:<name_w$}  {}",
            u.id,
            u.email.as_deref().unwrap_or("-"),
            u.name.as_deref().unwrap_or("-"),
            u.role.as_deref().unwrap_or("-"),
        );
    }

    Ok(())
}

async fn run_operators_list(
    config: &Configuration,
    tenant_id: &str,
    platform_id: &str,
    user_id: Option<&str>,
) -> Result<()> {
    let client = build_client(config, tenant_id)?;
    let url = api_url(config, "/v1/auth/operators/by-user");

    let mut params = vec![("platform_id", platform_id)];
    // user_id is required by the API; fall back to a
    // well-known dev user when not supplied.
    let uid = user_id.unwrap_or("us_01hs2yepy5hw4rz8pdq2wywnwt");
    params.push(("user_id", uid));

    let resp: OperatorListResponse = get_json_with_query(&client, &url, &params).await?;

    if resp.operators.is_empty() {
        println!("No operators found.");
        return Ok(());
    }

    let id_w = 30;
    let name_w = 30;

    println!("{:<id_w$}  {:<name_w$}  OPERATOR NAME", "ID", "NAME",);
    println!("{:-<id_w$}  {:-<name_w$}  -------------", "", "",);

    for o in &resp.operators {
        println!(
            "{:<id_w$}  {:<name_w$}  {}",
            o.id,
            o.name,
            o.operator_name.as_deref().unwrap_or("-"),
        );
    }

    Ok(())
}

async fn run_service_accounts_list(config: &Configuration, tenant_id: &str) -> Result<()> {
    let client = build_client(config, tenant_id)?;
    let url = api_url(config, "/v1/auth/service-accounts");

    let resp: ServiceAccountListResponse =
        get_json_with_query(&client, &url, &[("operator_id", tenant_id)]).await?;

    if resp.service_accounts.is_empty() {
        println!("No service accounts found.");
        return Ok(());
    }

    let id_w = 30;
    let name_w = 30;

    println!("{:<id_w$}  {:<name_w$}  CREATED AT", "ID", "NAME",);
    println!("{:-<id_w$}  {:-<name_w$}  ----------", "", "",);

    for sa in &resp.service_accounts {
        println!(
            "{:<id_w$}  {:<name_w$}  {}",
            sa.id,
            sa.name,
            sa.created_at.as_deref().unwrap_or("-"),
        );
    }

    Ok(())
}

pub async fn run(args: &OrgArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    match &args.command {
        OrgCommand::Policies(p) => match &p.command {
            PoliciesCommand::Actions { context } => {
                run_actions(config, tenant_id, context.as_deref()).await
            }
        },
        OrgCommand::Users(u) => match &u.command {
            UsersCommand::List => run_users_list(config, tenant_id).await,
        },
        OrgCommand::Operators(o) => match &o.command {
            OperatorsCommand::List {
                platform_id,
                user_id,
            } => run_operators_list(config, tenant_id, platform_id, user_id.as_deref()).await,
        },
        OrgCommand::ServiceAccounts(sa) => match &sa.command {
            ServiceAccountsCommand::List => run_service_accounts_list(config, tenant_id).await,
        },
    }
}
