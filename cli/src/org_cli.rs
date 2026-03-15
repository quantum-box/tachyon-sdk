use anyhow::Result;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, truncate, ApiClient};

#[derive(Debug, Clone, Args)]
pub struct OrgArgs {
    #[command(subcommand)]
    pub command: OrgCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum OrgCommand {
    /// Manage operators (organizations)
    Operators {
        #[command(subcommand)]
        command: OperatorsCommand,
    },
    /// Manage users
    Users {
        #[command(subcommand)]
        command: UsersCommand,
    },
    /// Manage service accounts
    ServiceAccounts {
        #[command(subcommand)]
        command: ServiceAccountsCommand,
    },
    /// Manage policies
    Policies {
        #[command(subcommand)]
        command: PoliciesCommand,
    },
}

// --- Operators ---

#[derive(Debug, Clone, Subcommand)]
pub enum OperatorsCommand {
    /// List operators for the current user
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get operator details by ID
    Get {
        operator_id: String,
        #[arg(long)]
        json: bool,
    },
    /// Get operator by alias
    ByAlias {
        /// Alias to look up
        alias: String,
        #[arg(long)]
        json: bool,
    },
}

// --- Users ---

#[derive(Debug, Clone, Subcommand)]
pub enum UsersCommand {
    /// List users
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get user details
    Get {
        user_id: String,
        #[arg(long)]
        json: bool,
    },
    /// Invite a user
    Invite {
        /// Email address
        email: String,
        /// Role (e.g. admin, member)
        #[arg(long, default_value = "member")]
        role: String,
    },
    /// List policies attached to a user
    Policies {
        user_id: String,
        #[arg(long)]
        json: bool,
    },
}

// --- Service Accounts ---

#[derive(Debug, Clone, Subcommand)]
pub enum ServiceAccountsCommand {
    /// List service accounts
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get service account details
    Get {
        service_account_id: String,
        #[arg(long)]
        json: bool,
    },
    /// List API keys for a service account
    ApiKeys {
        service_account_id: String,
        #[arg(long)]
        json: bool,
    },
}

// --- Policies ---

#[derive(Debug, Clone, Subcommand)]
pub enum PoliciesCommand {
    /// Get a policy by ID
    Get {
        policy_id: String,
        #[arg(long)]
        json: bool,
    },
    /// List available actions
    Actions {
        #[arg(long)]
        json: bool,
    },
}

// ---- Response types ----

#[derive(Debug, Deserialize, Serialize)]
struct OperatorResponse {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    alias: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserResponse {
    id: String,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ServiceAccountResponse {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiKeyResponse {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PolicyResponse {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    actions: Option<Vec<String>>,
    #[serde(default)]
    resources: Option<Vec<String>>,
    #[serde(default)]
    effect: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserPolicyResponse {
    #[serde(default)]
    policy_id: Option<String>,
    #[serde(default)]
    policy_name: Option<String>,
    #[serde(default)]
    scope: Option<String>,
}

#[derive(Debug, Serialize)]
struct InviteUserRequest {
    email: String,
    role: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ActionResponse {
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

// ---- Handlers ----

async fn run_operators_list(api: &ApiClient, json: bool) -> Result<()> {
    let ops: Vec<OperatorResponse> = api.get("/v1/auth/operators/by-user").await?;
    if json {
        return print_json(&ops);
    }
    if ops.is_empty() {
        println!("No operators found.");
        return Ok(());
    }
    println!("{:<28}  {:<24}  {:<20}  {}", "ID", "NAME", "ALIAS", "CREATED AT");
    println!("{:-<28}  {:-<24}  {:-<20}  {:-<19}", "", "", "", "");
    for op in &ops {
        println!(
            "{:<28}  {:<24}  {:<20}  {}",
            op.id,
            truncate(op.name.as_deref().unwrap_or("-"), 24),
            op.alias.as_deref().unwrap_or("-"),
            op.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_operators_get(api: &ApiClient, id: &str, json: bool) -> Result<()> {
    let op: OperatorResponse = api.get(&format!("/v1/auth/operators/{id}")).await?;
    if json {
        return print_json(&op);
    }
    println!("ID:      {}", op.id);
    println!("Name:    {}", op.name.as_deref().unwrap_or("-"));
    println!("Alias:   {}", op.alias.as_deref().unwrap_or("-"));
    println!(
        "Created: {}",
        op.created_at.as_deref().unwrap_or("-")
    );
    Ok(())
}

async fn run_operators_by_alias(api: &ApiClient, alias: &str, json: bool) -> Result<()> {
    let op: OperatorResponse = api
        .get_query("/v1/auth/operators/by-alias", &[("alias", alias)])
        .await?;
    if json {
        return print_json(&op);
    }
    println!("ID:      {}", op.id);
    println!("Name:    {}", op.name.as_deref().unwrap_or("-"));
    println!("Alias:   {}", op.alias.as_deref().unwrap_or("-"));
    println!(
        "Created: {}",
        op.created_at.as_deref().unwrap_or("-")
    );
    Ok(())
}

async fn run_users_list(api: &ApiClient, json: bool) -> Result<()> {
    let users: Vec<UserResponse> = api.get("/v1/auth/users").await?;
    if json {
        return print_json(&users);
    }
    if users.is_empty() {
        println!("No users found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<24}  {:<30}  {:<10}  {}",
        "ID", "USERNAME", "EMAIL", "ROLE", "CREATED AT"
    );
    println!(
        "{:-<28}  {:-<24}  {:-<30}  {:-<10}  {:-<19}",
        "", "", "", "", ""
    );
    for u in &users {
        println!(
            "{:<28}  {:<24}  {:<30}  {:<10}  {}",
            u.id,
            truncate(u.username.as_deref().unwrap_or("-"), 24),
            truncate(u.email.as_deref().unwrap_or("-"), 30),
            u.role.as_deref().unwrap_or("-"),
            u.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_users_get(api: &ApiClient, user_id: &str, json: bool) -> Result<()> {
    let u: UserResponse = api.get(&format!("/v1/auth/users/{user_id}")).await?;
    if json {
        return print_json(&u);
    }
    println!("ID:       {}", u.id);
    println!("Username: {}", u.username.as_deref().unwrap_or("-"));
    println!("Email:    {}", u.email.as_deref().unwrap_or("-"));
    println!("Role:     {}", u.role.as_deref().unwrap_or("-"));
    println!(
        "Created:  {}",
        u.created_at.as_deref().unwrap_or("-")
    );
    Ok(())
}

async fn run_users_invite(api: &ApiClient, email: &str, role: &str) -> Result<()> {
    let req = InviteUserRequest {
        email: email.to_string(),
        role: role.to_string(),
    };
    let resp: serde_json::Value = api.post("/v1/auth/users/invite", &req).await?;
    println!("Invitation sent to {email}.");
    if let Some(id) = resp.get("id").and_then(|v| v.as_str()) {
        println!("User ID: {id}");
    }
    Ok(())
}

async fn run_users_policies(api: &ApiClient, user_id: &str, json: bool) -> Result<()> {
    let policies: Vec<UserPolicyResponse> = api
        .get(&format!("/v1/auth/users/{user_id}/policies"))
        .await?;
    if json {
        return print_json(&policies);
    }
    if policies.is_empty() {
        println!("No policies attached to user {user_id}");
        return Ok(());
    }
    println!("{:<28}  {:<24}  {}", "POLICY ID", "NAME", "SCOPE");
    println!("{:-<28}  {:-<24}  {:-<20}", "", "", "");
    for p in &policies {
        println!(
            "{:<28}  {:<24}  {}",
            p.policy_id.as_deref().unwrap_or("-"),
            p.policy_name.as_deref().unwrap_or("-"),
            p.scope.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_service_accounts_list(api: &ApiClient, json: bool) -> Result<()> {
    let accs: Vec<ServiceAccountResponse> = api.get("/v1/auth/service-accounts").await?;
    if json {
        return print_json(&accs);
    }
    if accs.is_empty() {
        println!("No service accounts found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<24}  {:<40}  {}",
        "ID", "NAME", "DESCRIPTION", "CREATED AT"
    );
    println!("{:-<28}  {:-<24}  {:-<40}  {:-<19}", "", "", "", "");
    for sa in &accs {
        println!(
            "{:<28}  {:<24}  {:<40}  {}",
            sa.id,
            truncate(sa.name.as_deref().unwrap_or("-"), 24),
            truncate(sa.description.as_deref().unwrap_or("-"), 40),
            sa.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_service_accounts_get(api: &ApiClient, id: &str, json: bool) -> Result<()> {
    let sa: ServiceAccountResponse = api.get(&format!("/v1/auth/service-accounts/{id}")).await?;
    if json {
        return print_json(&sa);
    }
    println!("ID:          {}", sa.id);
    println!("Name:        {}", sa.name.as_deref().unwrap_or("-"));
    println!(
        "Description: {}",
        sa.description.as_deref().unwrap_or("-")
    );
    println!(
        "Created:     {}",
        sa.created_at.as_deref().unwrap_or("-")
    );
    Ok(())
}

async fn run_service_accounts_api_keys(api: &ApiClient, id: &str, json: bool) -> Result<()> {
    let keys: Vec<ApiKeyResponse> = api
        .get(&format!("/v1/auth/service-accounts/{id}/api-keys"))
        .await?;
    if json {
        return print_json(&keys);
    }
    if keys.is_empty() {
        println!("No API keys found for service account {id}");
        return Ok(());
    }
    println!(
        "{:<28}  {:<20}  {:<16}  {}",
        "ID", "NAME", "PREFIX", "CREATED AT"
    );
    println!("{:-<28}  {:-<20}  {:-<16}  {:-<19}", "", "", "", "");
    for k in &keys {
        println!(
            "{:<28}  {:<20}  {:<16}  {}",
            k.id,
            k.name.as_deref().unwrap_or("-"),
            k.prefix.as_deref().unwrap_or("-"),
            k.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_policies_get(api: &ApiClient, policy_id: &str, json: bool) -> Result<()> {
    let p: PolicyResponse = api
        .get(&format!("/v1/auth/policies/{policy_id}"))
        .await?;
    if json {
        return print_json(&p);
    }
    println!("ID:          {}", p.id);
    println!("Name:        {}", p.name.as_deref().unwrap_or("-"));
    println!(
        "Description: {}",
        p.description.as_deref().unwrap_or("-")
    );
    println!("Effect:      {}", p.effect.as_deref().unwrap_or("-"));
    if let Some(actions) = &p.actions {
        println!("Actions:     {}", actions.join(", "));
    }
    if let Some(resources) = &p.resources {
        println!("Resources:   {}", resources.join(", "));
    }
    Ok(())
}

async fn run_policies_actions(api: &ApiClient, json: bool) -> Result<()> {
    let actions: Vec<ActionResponse> = api.get("/v1/auth/actions").await?;
    if json {
        return print_json(&actions);
    }
    if actions.is_empty() {
        println!("No actions found.");
        return Ok(());
    }
    println!("{:<40}  {}", "ACTION", "DESCRIPTION");
    println!("{:-<40}  {:-<40}", "", "");
    for a in &actions {
        println!(
            "{:<40}  {}",
            a.action.as_deref().unwrap_or("-"),
            a.description.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

// ---- Entry point ----

pub async fn run(args: &OrgArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    match &args.command {
        OrgCommand::Operators { command } => match command {
            OperatorsCommand::List { json } => run_operators_list(&api, *json).await,
            OperatorsCommand::Get { operator_id, json } => {
                run_operators_get(&api, operator_id, *json).await
            }
            OperatorsCommand::ByAlias { alias, json } => {
                run_operators_by_alias(&api, alias, *json).await
            }
        },
        OrgCommand::Users { command } => match command {
            UsersCommand::List { json } => run_users_list(&api, *json).await,
            UsersCommand::Get { user_id, json } => run_users_get(&api, user_id, *json).await,
            UsersCommand::Invite { email, role } => run_users_invite(&api, email, role).await,
            UsersCommand::Policies { user_id, json } => {
                run_users_policies(&api, user_id, *json).await
            }
        },
        OrgCommand::ServiceAccounts { command } => match command {
            ServiceAccountsCommand::List { json } => run_service_accounts_list(&api, *json).await,
            ServiceAccountsCommand::Get {
                service_account_id,
                json,
            } => run_service_accounts_get(&api, service_account_id, *json).await,
            ServiceAccountsCommand::ApiKeys {
                service_account_id,
                json,
            } => run_service_accounts_api_keys(&api, service_account_id, *json).await,
        },
        OrgCommand::Policies { command } => match command {
            PoliciesCommand::Get { policy_id, json } => {
                run_policies_get(&api, policy_id, *json).await
            }
            PoliciesCommand::Actions { json } => run_policies_actions(&api, *json).await,
        },
    }
}
