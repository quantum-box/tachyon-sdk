use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{http_error_status, print_json, truncate, ApiClient};
use crate::resolve;
use crate::response_contract::{
    registered_contract, tachyon_response_contract, ContractRegistration,
};

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
    /// Delete an operator (hard delete, for cleaning up empty organizations).
    /// The acting scope (--tenant-id) must be the target operator itself or
    /// its parent platform.
    Delete { operator_id: String },
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
    /// Invite a user by ID or email address
    Invite {
        /// User ID (e.g. us_xxx) or email address
        identifier: String,
        /// Notify the user by email
        #[arg(long)]
        notify: bool,
        /// Platform ID (optional, resolved from tenant hierarchy
        /// if omitted)
        #[arg(long)]
        platform_id: Option<String>,
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
    /// Create a service account in the acting tenant (--tenant-id)
    ///
    /// The account is created empty: it holds no API key and no policy, so it
    /// can do nothing until both are granted. Issue a key with
    /// `tachyon api-key create`.
    Create {
        /// Display name, e.g. gha.auth-manifest-drift-guard
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// List service accounts
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get service account details
    Get {
        /// Service account ID or name
        service_account_id: String,
        #[arg(long)]
        json: bool,
    },
    /// List API keys for a service account
    ApiKeys {
        /// Service account ID or name
        service_account_id: String,
        #[arg(long)]
        json: bool,
    },
    /// Delete a service account and every API key issued to it
    ///
    /// Destructive and immediate: anything authenticating with one of the
    /// account's keys starts failing as soon as this returns. Without --yes
    /// the command only reports what it would delete.
    Delete {
        /// Service account ID or name
        service_account_id: String,
        /// Delete instead of only reporting what would be deleted
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        json: bool,
    },
}

// --- Policies ---

#[derive(Debug, Clone, Subcommand)]
pub enum PoliciesCommand {
    /// List policies visible to the current tenant
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get a policy by ID
    Get {
        policy_id: String,
        #[arg(long)]
        json: bool,
    },
    /// Delete an unreferenced custom policy
    Delete { policy_id: String },
    /// List available actions
    Actions {
        #[arg(long)]
        json: bool,
    },
    /// Find user-policy mappings by resource scope
    Mappings {
        #[arg(long)]
        resource_scope: String,
        #[arg(long)]
        json: bool,
    },
}

// ---- Response types ----

// response-contract:auth.operators.list:start
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct OperatorResponse {
    id: String,
    name: String,
    operator_name: String,
    platform_id: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
struct OperatorListResponse(Vec<OperatorResponse>);

tachyon_response_contract! {
    root: OperatorListResponse,
    id: "auth.operators.list",
    operation: ("GET", "/v1/auth/operators/by-user", 200),
    api_owner: "packages/auth",
    cli_owner: "cli/org",
}
// response-contract:auth.operators.list:end

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteOperatorResponse {
    #[allow(dead_code)]
    success: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserResponse {
    id: String,
    email: Option<String>,
    name: Option<String>,
    role: String,
    tenants: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListedUserResponse {
    id: Option<String>,
    email: Option<String>,
    name: Option<String>,
    role: Option<String>,
    tenants: Vec<String>,
    status: String,
    created_at: Option<String>,
    expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserListResponse {
    users: Vec<ListedUserResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServiceAccountResponse {
    id: String,
    tenant_id: String,
    name: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceAccountListResponse {
    service_accounts: Vec<ServiceAccountResponse>,
}

/// Body of `POST /v1/auth/service-accounts`.
///
/// `tenantId` is required by the API schema, but the server compares it with
/// the authorized `x-operator-id` scope and answers 403 on a mismatch, so it
/// is always the tenant the command is already acting as rather than a
/// separate option.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateServiceAccountRequest {
    tenant_id: String,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteServiceAccountResponse {
    id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiKeyResponse {
    id: String,
    service_account_id: String,
    name: String,
    value: String,
    created_at: String,
    expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiKeyListResponse {
    api_keys: Vec<ApiKeyResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PolicyResponse {
    id: String,
    name: String,
    description: Option<String>,
    is_system: bool,
    tenant_id: Option<String>,
    shared_with_descendants: bool,
    owner_tenant_id: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PolicyListResponse {
    policies: Vec<PolicyResponse>,
    total_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserPolicyListResponse {
    policy_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserPolicyMappingResponse {
    user_id: String,
    tenant_id: String,
    policy_id: String,
    resource_scope: Option<String>,
    assigned_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserPolicyMappingListResponse {
    mappings: Vec<UserPolicyMappingResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InviteUserRequest {
    tenant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    invitee_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notify_user: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GrantTenantAccessRequest {
    tenant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_id: Option<String>,
}

// response-contract:auth.actions.list:start
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ActionResponse {
    id: String,
    platform_id: Option<String>,
    shared_with_descendants: bool,
    owner_tenant_id: Option<String>,
    context: String,
    name: String,
    full_name: String,
    description: Option<String>,
    resource_pattern: Option<String>,
    sandbox_restriction: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ActionListResponse {
    actions: Vec<ActionResponse>,
    total_count: usize,
}

tachyon_response_contract! {
    root: ActionListResponse,
    id: "auth.actions.list",
    operation: ("GET", "/v1/auth/actions", 200),
    api_owner: "packages/auth",
    cli_owner: "cli/org",
}
// response-contract:auth.actions.list:end

#[allow(dead_code)]
pub(crate) fn auth_list_contracts() -> Vec<ContractRegistration> {
    vec![
        registered_contract::<OperatorListResponse>(),
        registered_contract::<ActionListResponse>(),
    ]
}

// ---- Handlers ----

async fn run_operators_list(api: &ApiClient, json: bool) -> Result<()> {
    let response: OperatorListResponse = api.get_contract("/v1/auth/operators/by-user").await?;
    let ops = response.0;
    if json {
        return print_json(&ops);
    }
    if ops.is_empty() {
        println!("No operators found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<24}  {:<24}  PLATFORM ID",
        "ID", "NAME", "OPERATOR NAME"
    );
    println!("{:-<28}  {:-<24}  {:-<24}  {:-<28}", "", "", "", "");
    for op in &ops {
        println!(
            "{:<28}  {:<24}  {:<24}  {}",
            op.id,
            truncate(&op.name, 24),
            truncate(&op.operator_name, 24),
            op.platform_id,
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
    println!("Name:    {}", op.name);
    println!("Operator name: {}", op.operator_name);
    println!("Platform ID:   {}", op.platform_id);
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
    println!("Name:    {}", op.name);
    println!("Operator name: {}", op.operator_name);
    println!("Platform ID:   {}", op.platform_id);
    Ok(())
}

async fn run_operators_delete(api: &ApiClient, id: &str) -> Result<()> {
    let _: DeleteOperatorResponse = api.delete_json(&format!("/v1/auth/operators/{id}")).await?;
    println!("Operator {id} deleted.");
    Ok(())
}

async fn run_users_list(api: &ApiClient, tenant_id: &str, json: bool) -> Result<()> {
    let response: UserListResponse = api
        .get_query("/v1/auth/users", &[("operator_id", tenant_id)])
        .await?;
    let users = response.users;
    if json {
        return print_json(&users);
    }
    if users.is_empty() {
        println!("No users found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<24}  {:<30}  {:<10}  CREATED AT",
        "ID", "NAME", "EMAIL", "ROLE"
    );
    println!(
        "{:-<28}  {:-<24}  {:-<30}  {:-<10}  {:-<19}",
        "", "", "", "", ""
    );
    for u in &users {
        println!(
            "{:<28}  {:<24}  {:<30}  {:<10}  {}",
            u.id.as_deref().unwrap_or("-"),
            truncate(u.name.as_deref().unwrap_or("-"), 24),
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
    println!("Name:     {}", u.name.as_deref().unwrap_or("-"));
    println!("Email:    {}", u.email.as_deref().unwrap_or("-"));
    println!("Role:     {}", u.role);
    println!("Tenants:  {}", u.tenants.join(", "));
    Ok(())
}

async fn run_users_invite(
    api: &ApiClient,
    tenant_id: &str,
    identifier: &str,
    notify: bool,
    platform_id: Option<&str>,
) -> Result<()> {
    let is_email = identifier.contains('@');
    if is_email {
        let req = InviteUserRequest {
            tenant_id: tenant_id.to_string(),
            invitee_email: Some(identifier.to_string()),
            platform_id: platform_id.map(|s| s.to_string()),
            notify_user: if notify { Some(true) } else { None },
        };
        let resp: serde_json::Value = api.post("/v1/auth/users/invite", &req).await?;
        println!("Invitation sent to {identifier}.");
        if let Some(email) = resp.get("email").and_then(|v| v.as_str()) {
            println!("Email: {email}");
        }
    } else {
        let req = GrantTenantAccessRequest {
            tenant_id: tenant_id.to_string(),
            user_id: Some(identifier.to_string()),
            email: None,
            platform_id: platform_id.map(|s| s.to_string()),
        };
        let resp: serde_json::Value = api.post("/v1/auth/users/grant-tenant-access", &req).await?;
        println!("User {identifier} invited to tenant.");
        if let Some(id) = resp.get("id").and_then(|v| v.as_str()) {
            println!("User ID: {id}");
        }
    }
    Ok(())
}

async fn run_users_policies(api: &ApiClient, user_id: &str, json: bool) -> Result<()> {
    let response: UserPolicyListResponse = api
        .get(&format!("/v1/auth/users/{user_id}/policies"))
        .await?;
    let policies = response.policy_ids;
    if json {
        return print_json(&policies);
    }
    if policies.is_empty() {
        println!("No policies attached to user {user_id}");
        return Ok(());
    }
    println!("POLICY ID");
    println!("{:-<28}", "");
    for policy_id in &policies {
        println!("{policy_id}");
    }
    Ok(())
}

// --- Service account create / delete ---

/// The API stores a service account name as a `Text` value object: non-empty
/// after trimming, and at most this many bytes.
const SERVICE_ACCOUNT_NAME_MAX_BYTES: usize = 191;

/// Writing a service account needs an explicit tenant. The API derives the
/// tenant it writes to from the authorized `x-operator-id` scope, and an empty
/// scope is rejected far from its cause.
fn require_service_account_tenant<'a>(tenant_id: &'a str, command: &str) -> Result<&'a str> {
    if tenant_id.trim().is_empty() {
        return Err(anyhow!(
            "`tachyon org service-accounts {command}` needs a tenant. Pass \
             `--tenant-id <tn_...>`, set TACHYON_TENANT_ID, or use a profile \
             whose login carries an operator."
        ));
    }
    Ok(tenant_id)
}

/// Check the name against the value object the API will parse it into.
///
/// The create usecase parses the name after the policy check, and that parse
/// failure converts into a 500 rather than a 400, so an empty or over-long
/// name comes back as an internal error that says nothing about the name.
fn validate_service_account_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(anyhow!(
            "service account name must not be empty or only whitespace."
        ));
    }
    if name.len() > SERVICE_ACCOUNT_NAME_MAX_BYTES {
        return Err(anyhow!(
            "service account name is {} bytes; the API accepts at most \
             {SERVICE_ACCOUNT_NAME_MAX_BYTES}.",
            name.len()
        ));
    }
    Ok(())
}

/// The ID of a service account that already carries this exact name in the
/// tenant, or `None` when there is none.
///
/// The API has no uniqueness constraint on (tenant, name), so this is a
/// client-side check rather than a read of a constraint the server enforces.
async fn existing_service_account_named(
    api: &ApiClient,
    tenant_id: &str,
    name: &str,
) -> Option<String> {
    match api
        .get_query::<ServiceAccountListResponse>(
            "/v1/auth/service-accounts",
            &[("operator_id", tenant_id)],
        )
        .await
    {
        Ok(response) => response
            .service_accounts
            .into_iter()
            .find(|sa| sa.name == name)
            .map(|sa| sa.id),
        // The read exists only to catch a duplicate name early. A caller that
        // may create an account but not list them must still be able to
        // create one.
        Err(err) => {
            eprintln!("Warning: could not list the service accounts of tenant {tenant_id}: {err}");
            None
        }
    }
}

/// Turn the API's status into the next thing the operator can do.
fn create_service_account_hint(err: anyhow::Error, tenant_id: &str, name: &str) -> anyhow::Error {
    let hint = match http_error_status(&err) {
        Some(reqwest::StatusCode::FORBIDDEN) => format!(
            "could not create service account '{name}' in tenant {tenant_id}: the \
             acting profile is not allowed to. Creating one needs a policy that \
             grants `auth:CreateServiceAccount` in this tenant, and the API also \
             refuses a tenant other than the authorized `--tenant-id` scope. \
             Check the caller's own grants with `tachyon org users policies \
             <your-user-id> --tenant-id {tenant_id}`, or re-run with \
             `--profile <admin-profile>`."
        ),
        Some(reqwest::StatusCode::BAD_REQUEST) => format!(
            "could not create service account '{name}': the API rejected the \
             request. `--tenant-id` must resolve to a `tn_...` tenant ID."
        ),
        _ => format!("could not create service account '{name}' in tenant {tenant_id}."),
    };
    err.context(hint)
}

async fn run_service_accounts_create(
    api: &ApiClient,
    tenant_id: &str,
    name: &str,
    json: bool,
) -> Result<()> {
    let tenant_id = require_service_account_tenant(tenant_id, "create")?;
    validate_service_account_name(name)?;

    if let Some(existing_id) = existing_service_account_named(api, tenant_id, name).await {
        return Err(anyhow!(
            "tenant {tenant_id} already has a service account named '{name}' \
             ({existing_id}). The API would create a second one, and the name \
             would then be ambiguous everywhere the CLI takes a service account \
             by name, such as `tachyon api-key create {name}`. Reuse that \
             account, or choose another name."
        ));
    }

    let request = CreateServiceAccountRequest {
        tenant_id: tenant_id.to_string(),
        name: name.to_string(),
    };
    // Not idempotent: without a uniqueness constraint on (tenant, name), a
    // replayed request creates a second account. Send it exactly once.
    let sa: ServiceAccountResponse = api
        .post_once("/v1/auth/service-accounts", &request)
        .await
        .map_err(|err| create_service_account_hint(err, tenant_id, name))?;

    if json {
        return print_json(&sa);
    }
    println!("Service account created.");
    println!("ID:          {}", sa.id);
    println!("Tenant ID:   {}", sa.tenant_id);
    println!("Name:        {}", sa.name);
    println!("Created:     {}", sa.created_at);
    println!();
    println!("The account holds no API key and no policy yet, so it cannot call anything.");
    println!(
        "Issue a key with: tachyon api-key create {} --name <key-name> --tenant-id {tenant_id}",
        sa.id
    );
    Ok(())
}

/// Turn the API's status into the next thing the operator can do.
fn delete_service_account_hint(err: anyhow::Error, tenant_id: &str, id: &str) -> anyhow::Error {
    let hint = match http_error_status(&err) {
        Some(reqwest::StatusCode::NOT_FOUND) => format!(
            "tenant {tenant_id} has no service account {id}. The API only reads \
             service accounts within the acting tenant, so an account of another \
             tenant looks missing here. List this tenant's accounts with \
             `tachyon org service-accounts list --tenant-id {tenant_id}`."
        ),
        Some(reqwest::StatusCode::FORBIDDEN) => format!(
            "could not delete service account {id} in tenant {tenant_id}: the \
             acting profile is not allowed to. Deleting one needs a policy that \
             grants `auth:DeleteServiceAccount` in this tenant. Check the \
             caller's own grants with `tachyon org users policies <your-user-id> \
             --tenant-id {tenant_id}`, or re-run with `--profile <admin-profile>`."
        ),
        _ => format!("could not delete service account {id} in tenant {tenant_id}."),
    };
    err.context(hint)
}

/// How many API keys the account currently has, or `None` when the list could
/// not be read. Only the count is reported; key material is never read back
/// into the output.
async fn service_account_api_key_count(
    api: &ApiClient,
    tenant_id: &str,
    id: &str,
) -> Option<usize> {
    match api
        .get_query::<ApiKeyListResponse>(
            &format!("/v1/auth/service-accounts/{id}/api-keys"),
            &[("operator_id", tenant_id)],
        )
        .await
    {
        Ok(response) => Some(response.api_keys.len()),
        Err(err) => {
            eprintln!("Warning: could not list the API keys of service account {id}: {err}");
            None
        }
    }
}

async fn run_service_accounts_delete(
    api: &ApiClient,
    tenant_id: &str,
    name_or_id: &str,
    assume_yes: bool,
    json: bool,
) -> Result<()> {
    let tenant_id = require_service_account_tenant(tenant_id, "delete")?;
    if json && !assume_yes {
        // The confirmation step reports in prose, so --json without --yes
        // would hand a script something it cannot parse and a zero exit code.
        return Err(anyhow!(
            "--json requires --yes. Without --yes the command only reports what \
             it would delete, and that report is not JSON."
        ));
    }
    // Resolved here rather than by the caller so the tenant is checked before
    // a name lookup that needs it.
    let id = &resolve::resolve_service_account_id(api, tenant_id, name_or_id).await?;

    let sa: ServiceAccountResponse = api
        .get_query(
            &format!("/v1/auth/service-accounts/{id}"),
            &[("operator_id", tenant_id)],
        )
        .await
        .map_err(|err| delete_service_account_hint(err, tenant_id, id))?;
    let key_count = service_account_api_key_count(api, tenant_id, &sa.id).await;

    if !assume_yes {
        println!("Service account: {} ({})", sa.name, sa.id);
        println!("Tenant ID:       {}", sa.tenant_id);
        println!("Created:         {}", sa.created_at);
        match key_count {
            Some(count) => println!("API keys:        {count} (all destroyed with the account)"),
            None => println!("API keys:        unknown (the list could not be read)"),
        }
        println!();
        println!("Deleting the account also deletes every API key issued to it, and any");
        println!("caller still presenting one of those keys starts failing immediately.");
        println!("Revoking a single key instead: tachyon api-key revoke {} <api-key-id> --tenant-id {tenant_id}", sa.id);
        println!();
        println!("No changes made. Re-run with --yes to delete.");
        return Ok(());
    }

    let deleted: DeleteServiceAccountResponse = api
        .delete_json(&format!("/v1/auth/service-accounts/{}", sa.id))
        .await
        .map_err(|err| delete_service_account_hint(err, tenant_id, &sa.id))?;

    if json {
        return print_json(&deleted);
    }
    println!(
        "Service account {} ({}) deleted from tenant {tenant_id}.",
        sa.name, deleted.id
    );
    if let Some(count) = key_count {
        println!("{count} API key(s) issued to it were deleted with it.");
    }
    Ok(())
}

async fn run_service_accounts_list(api: &ApiClient, tenant_id: &str, json: bool) -> Result<()> {
    let response: ServiceAccountListResponse = api
        .get_query("/v1/auth/service-accounts", &[("operator_id", tenant_id)])
        .await?;
    let accs = response.service_accounts;
    if json {
        return print_json(&accs);
    }
    if accs.is_empty() {
        println!("No service accounts found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<28}  {:<24}  CREATED AT",
        "ID", "TENANT ID", "NAME"
    );
    println!("{:-<28}  {:-<28}  {:-<24}  {:-<19}", "", "", "", "");
    for sa in &accs {
        println!(
            "{:<28}  {:<28}  {:<24}  {}",
            sa.id,
            sa.tenant_id,
            truncate(&sa.name, 24),
            sa.created_at,
        );
    }
    Ok(())
}

async fn run_service_accounts_get(
    api: &ApiClient,
    tenant_id: &str,
    id: &str,
    json: bool,
) -> Result<()> {
    let sa: ServiceAccountResponse = api
        .get_query(
            &format!("/v1/auth/service-accounts/{id}"),
            &[("operator_id", tenant_id)],
        )
        .await?;
    if json {
        return print_json(&sa);
    }
    println!("ID:          {}", sa.id);
    println!("Tenant ID:   {}", sa.tenant_id);
    println!("Name:        {}", sa.name);
    println!("Created:     {}", sa.created_at);
    Ok(())
}

async fn run_service_accounts_api_keys(
    api: &ApiClient,
    tenant_id: &str,
    id: &str,
    json: bool,
) -> Result<()> {
    let response: ApiKeyListResponse = api
        .get_query(
            &format!("/v1/auth/service-accounts/{id}/api-keys"),
            &[("operator_id", tenant_id)],
        )
        .await?;
    let keys = response.api_keys;
    if json {
        return print_json(&keys);
    }
    if keys.is_empty() {
        println!("No API keys found for service account {id}");
        return Ok(());
    }
    println!("{:<28}  {:<20}  {:<16}  CREATED AT", "ID", "NAME", "VALUE");
    println!("{:-<28}  {:-<20}  {:-<16}  {:-<19}", "", "", "", "");
    for k in &keys {
        println!(
            "{:<28}  {:<20}  {:<16}  {}",
            k.id, k.name, k.value, k.created_at,
        );
    }
    Ok(())
}

async fn run_policies_list(api: &ApiClient, json: bool) -> Result<()> {
    let response: PolicyListResponse = api.get("/v1/auth/policies").await?;
    let PolicyListResponse {
        policies,
        total_count,
    } = response;
    if json {
        return print_json(&policies);
    }
    if policies.is_empty() {
        println!("No policies found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<32}  {:<6}  {:<28}  {:<6}  CREATED AT",
        "ID", "NAME", "SYSTEM", "TENANT ID", "SHARED"
    );
    println!(
        "{:-<28}  {:-<32}  {:-<6}  {:-<28}  {:-<6}  {:-<19}",
        "", "", "", "", "", ""
    );
    for policy in &policies {
        println!(
            "{:<28}  {:<32}  {:<6}  {:<28}  {:<6}  {}",
            policy.id,
            truncate(&policy.name, 32),
            policy.is_system,
            policy.tenant_id.as_deref().unwrap_or("-"),
            policy.shared_with_descendants,
            policy.created_at,
        );
    }
    println!("Total: {total_count}");
    Ok(())
}

async fn run_policies_get(api: &ApiClient, policy_id: &str, json: bool) -> Result<()> {
    let p: PolicyResponse = api.get(&format!("/v1/auth/policies/{policy_id}")).await?;
    if json {
        return print_json(&p);
    }
    println!("ID:          {}", p.id);
    println!("Name:        {}", p.name);
    println!("Description: {}", p.description.as_deref().unwrap_or("-"));
    println!("System:      {}", p.is_system);
    println!("Tenant ID:   {}", p.tenant_id.as_deref().unwrap_or("-"));
    println!("Shared:      {}", p.shared_with_descendants);
    println!(
        "Owner tenant: {}",
        p.owner_tenant_id.as_deref().unwrap_or("-")
    );
    println!("Created:     {}", p.created_at);
    println!("Updated:     {}", p.updated_at);
    Ok(())
}

async fn run_policies_delete(api: &ApiClient, policy_id: &str) -> Result<()> {
    api.delete(&format!("/v1/auth/policies/{policy_id}"))
        .await?;
    println!("Policy {policy_id} deleted.");
    Ok(())
}

async fn run_policies_actions(api: &ApiClient, json: bool) -> Result<()> {
    let response: ActionListResponse = api.get_contract("/v1/auth/actions").await?;
    let ActionListResponse {
        actions,
        total_count,
    } = response;
    if json {
        return print_json(&actions);
    }
    if actions.is_empty() {
        println!("No actions found.");
        return Ok(());
    }
    println!("{:<40}  DESCRIPTION", "ACTION");
    println!("{:-<40}  {:-<40}", "", "");
    for a in &actions {
        println!(
            "{:<40}  {}",
            a.full_name,
            a.description.as_deref().unwrap_or("-"),
        );
    }
    println!("Total: {total_count}");
    Ok(())
}

async fn run_policy_mappings(
    api: &ApiClient,
    tenant_id: &str,
    resource_scope: &str,
    json: bool,
) -> Result<()> {
    let response: UserPolicyMappingListResponse = api
        .get_query(
            "/v1/auth/user-policy-mappings",
            &[("tenantId", tenant_id), ("resourceScope", resource_scope)],
        )
        .await?;
    let mappings = response.mappings;
    if json {
        return print_json(&mappings);
    }
    if mappings.is_empty() {
        println!("No user-policy mappings found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<28}  {:<28}  {:<40}  ASSIGNED AT",
        "USER ID", "TENANT ID", "POLICY ID", "RESOURCE SCOPE"
    );
    println!(
        "{:-<28}  {:-<28}  {:-<28}  {:-<40}  {:-<19}",
        "", "", "", "", ""
    );
    for mapping in &mappings {
        println!(
            "{:<28}  {:<28}  {:<28}  {:<40}  {}",
            mapping.user_id,
            mapping.tenant_id,
            mapping.policy_id,
            truncate(mapping.resource_scope.as_deref().unwrap_or("-"), 40),
            mapping.assigned_at,
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
            OperatorsCommand::Delete { operator_id } => {
                run_operators_delete(&api, operator_id).await
            }
        },
        OrgCommand::Users { command } => match command {
            UsersCommand::List { json } => run_users_list(&api, tenant_id, *json).await,
            UsersCommand::Get { user_id, json } => run_users_get(&api, user_id, *json).await,
            UsersCommand::Invite {
                identifier,
                notify,
                platform_id,
            } => {
                run_users_invite(&api, tenant_id, identifier, *notify, platform_id.as_deref()).await
            }
            UsersCommand::Policies { user_id, json } => {
                run_users_policies(&api, user_id, *json).await
            }
        },
        OrgCommand::ServiceAccounts { command } => match command {
            ServiceAccountsCommand::Create { name, json } => {
                run_service_accounts_create(&api, tenant_id, name, *json).await
            }
            ServiceAccountsCommand::List { json } => {
                run_service_accounts_list(&api, tenant_id, *json).await
            }
            ServiceAccountsCommand::Get {
                service_account_id,
                json,
            } => {
                let id = resolve::resolve_service_account_id(&api, tenant_id, service_account_id)
                    .await?;
                run_service_accounts_get(&api, tenant_id, &id, *json).await
            }
            ServiceAccountsCommand::ApiKeys {
                service_account_id,
                json,
            } => {
                let id = resolve::resolve_service_account_id(&api, tenant_id, service_account_id)
                    .await?;
                run_service_accounts_api_keys(&api, tenant_id, &id, *json).await
            }
            ServiceAccountsCommand::Delete {
                service_account_id,
                yes,
                json,
            } => {
                run_service_accounts_delete(&api, tenant_id, service_account_id, *yes, *json).await
            }
        },
        OrgCommand::Policies { command } => match command {
            PoliciesCommand::List { json } => run_policies_list(&api, *json).await,
            PoliciesCommand::Get { policy_id, json } => {
                run_policies_get(&api, policy_id, *json).await
            }
            PoliciesCommand::Delete { policy_id } => run_policies_delete(&api, policy_id).await,
            PoliciesCommand::Actions { json } => run_policies_actions(&api, *json).await,
            PoliciesCommand::Mappings {
                resource_scope,
                json,
            } => run_policy_mappings(&api, tenant_id, resource_scope, *json).await,
        },
    }
}
