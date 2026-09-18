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
    /// Attach a policy to a user in the acting tenant (--tenant-id)
    AttachPolicy {
        /// User ID (e.g. us_xxx)
        user_id: String,
        /// Policy name (e.g. field:staff) or policy ID (pol_xxx)
        #[arg(long)]
        policy: String,
        /// Narrow the grant to one resource, in TRN format
        /// (e.g. trn:library:repo:rp_xxx). Omit to grant the policy
        /// across the whole tenant.
        #[arg(long)]
        resource_scope: Option<String>,
    },
    /// Detach a policy from a user in the acting tenant (--tenant-id).
    /// Without --resource-scope this removes every grant of the policy for
    /// the user in that tenant, including grants narrowed to a resource.
    DetachPolicy {
        /// User ID (e.g. us_xxx)
        user_id: String,
        /// Policy name (e.g. field:staff) or policy ID (pol_xxx)
        #[arg(long)]
        policy: String,
        /// Remove only the grant narrowed to this TRN resource scope
        #[arg(long)]
        resource_scope: Option<String>,
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

/// Body shared by the four user-policy write endpoints. `attach` and
/// `detach` ignore `resourceScope`; `attach-with-scope` and
/// `detach-with-scope` require it.
///
/// `tenantId` is required by the API schema, but the server derives the
/// tenant it actually writes to from the authorized `x-operator-id` scope, so
/// this must be the same tenant the command is acting as.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserPolicyRequest {
    user_id: String,
    policy_id: String,
    tenant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_scope: Option<String>,
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

// --- User policy attach / detach ---

/// A policy resolved from `--policy`, carrying both the ID the API needs and
/// the name the operator typed, so output can name the grant unambiguously.
struct ResolvedPolicy {
    id: String,
    name: String,
}

impl std::fmt::Display for ResolvedPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

/// Describe a policy by ID and owner, for disambiguating same-named policies.
fn describe_policy_owner(policy: &PolicyResponse) -> String {
    let owner = policy
        .owner_tenant_id
        .as_deref()
        .or(policy.tenant_id.as_deref())
        .unwrap_or(if policy.is_system { "system" } else { "-" });
    format!("{} (owner: {owner})", policy.id)
}

/// Resolve `--policy` to a policy the acting tenant can grant.
///
/// A `pol_` ID is read back so the command can print the policy's name;
/// anything else is matched by exact name against the policies visible from
/// the acting tenant. Names are not unique across owners — two tenants can
/// both publish `field:admin` — so an ambiguous name fails instead of
/// silently picking one.
async fn resolve_policy(
    api: &ApiClient,
    tenant_id: &str,
    name_or_id: &str,
) -> Result<ResolvedPolicy> {
    if name_or_id.starts_with("pol_") && resolve::looks_like_id(name_or_id) {
        return resolve_policy_by_id(api, tenant_id, name_or_id).await;
    }

    let response: PolicyListResponse = api.get("/v1/auth/policies").await?;
    let matches: Vec<&PolicyResponse> = response
        .policies
        .iter()
        .filter(|policy| policy.name == name_or_id)
        .collect();

    match matches.as_slice() {
        [] => Err(anyhow!(
            "no policy named '{name_or_id}' is visible from tenant {tenant_id}. \
             Run `tachyon org policies list --tenant-id {tenant_id}` to see the \
             policies this tenant can grant, or pass the ID as `--policy pol_...`."
        )),
        [policy] => Ok(ResolvedPolicy {
            id: policy.id.clone(),
            name: policy.name.clone(),
        }),
        ambiguous => Err(anyhow!(
            "policy name '{name_or_id}' is ambiguous in tenant {tenant_id}: {}. \
             Pass the ID as `--policy pol_...` to choose one.",
            ambiguous
                .iter()
                .map(|policy| describe_policy_owner(policy))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

async fn resolve_policy_by_id(
    api: &ApiClient,
    tenant_id: &str,
    policy_id: &str,
) -> Result<ResolvedPolicy> {
    match api
        .get::<PolicyResponse>(&format!("/v1/auth/policies/{policy_id}"))
        .await
    {
        Ok(policy) => Ok(ResolvedPolicy {
            id: policy.id,
            name: policy.name,
        }),
        Err(err) if http_error_status(&err) == Some(reqwest::StatusCode::NOT_FOUND) => Err(err
            .context(format!(
                "policy {policy_id} does not exist. Run \
                 `tachyon org policies list --tenant-id {tenant_id}` to see the \
                 policies this tenant can grant."
            ))),
        // The lookup only supplies a human-readable name for the output, so a
        // read the caller is not allowed to make must not block the write.
        Err(err) => {
            eprintln!("Warning: could not read policy {policy_id}: {err}");
            Ok(ResolvedPolicy {
                id: policy_id.to_string(),
                name: "-".to_string(),
            })
        }
    }
}

/// Confirm the target user and return a label for the output.
///
/// `user_policies` has a foreign key on `user_id`, so attaching to an unknown
/// user fails deep inside the API; a 404 here turns that into an answerable
/// message before anything is written.
async fn resolve_user_label(api: &ApiClient, tenant_id: &str, user_id: &str) -> Result<String> {
    match api
        .get::<UserResponse>(&format!("/v1/auth/users/{user_id}"))
        .await
    {
        Ok(user) => Ok(user.email.or(user.name).unwrap_or_else(|| "-".to_string())),
        Err(err) if http_error_status(&err) == Some(reqwest::StatusCode::NOT_FOUND) => Err(err
            .context(format!(
                "user {user_id} does not exist. Run \
                 `tachyon org users list --tenant-id {tenant_id}` to see the users \
                 of this tenant, or invite the user first with \
                 `tachyon org users invite <email> --tenant-id {tenant_id}`."
            ))),
        Err(err) => {
            eprintln!("Warning: could not read user {user_id}: {err}");
            Ok("-".to_string())
        }
    }
}

/// Whether `policy_id` is already listed for the user in the acting tenant,
/// or `None` when the read failed.
///
/// The API reports a user's policies without their resource scopes, so this
/// answers "is this policy listed at all", not "is this exact grant present".
async fn policy_is_listed_for_user(
    api: &ApiClient,
    user_id: &str,
    policy_id: &str,
) -> Option<bool> {
    match api
        .get::<UserPolicyListResponse>(&format!("/v1/auth/users/{user_id}/policies"))
        .await
    {
        Ok(response) => Some(response.policy_ids.iter().any(|id| id == policy_id)),
        Err(err) => {
            eprintln!("Warning: could not read the current policies of user {user_id}: {err}");
            None
        }
    }
}

/// Writing a grant needs an explicit tenant: the API derives the tenant it
/// writes to from the authorized `x-operator-id` scope, and an empty value
/// would be rejected far from its cause.
fn require_tenant<'a>(tenant_id: &'a str, command: &str) -> Result<&'a str> {
    if tenant_id.trim().is_empty() {
        return Err(anyhow!(
            "`tachyon org users {command}` needs a tenant. Pass \
             `--tenant-id <tn_...>`, set TACHYON_TENANT_ID, or use a profile \
             whose login carries an operator."
        ));
    }
    Ok(tenant_id)
}

/// The scope endpoints store `resourceScope` verbatim without validating it,
/// so a typo is accepted by the API and then matches nothing. Reject anything
/// that is not a TRN before it is written.
fn validate_resource_scope(scope: &str) -> Result<()> {
    if scope == "*" {
        return Ok(());
    }
    let parts: Vec<&str> = scope.split(':').collect();
    if parts.len() == 4 && parts[0] == "trn" && parts[1..].iter().all(|part| !part.is_empty()) {
        return Ok(());
    }
    Err(anyhow!(
        "invalid --resource-scope '{scope}'. Expected a TRN of the form \
         `trn:<service>:<resource-type>:<resource-id>` (for example \
         `trn:library:repo:rp_xxx`), where any part may be `*`, or `*` on its \
         own for every resource."
    ))
}

/// Turn the API's status into the next thing the operator can do.
fn user_policy_write_hint(
    err: anyhow::Error,
    verb: &str,
    tenant_id: &str,
    user_id: &str,
    policy: &ResolvedPolicy,
) -> anyhow::Error {
    let hint = match http_error_status(&err) {
        Some(reqwest::StatusCode::NOT_FOUND) => format!(
            "could not {verb} policy {policy} for user {user_id}: the API reports \
             the user or the policy as missing in tenant {tenant_id}. Check both \
             with `tachyon org users list --tenant-id {tenant_id}` and \
             `tachyon org policies list --tenant-id {tenant_id}`."
        ),
        Some(reqwest::StatusCode::FORBIDDEN) => format!(
            "could not {verb} policy {policy} for user {user_id} in tenant \
             {tenant_id}: the acting profile is not allowed to grant it. A policy \
             can only be granted from the tenant scope that owns or inherits it, \
             and AdministratorAccess additionally requires the caller to hold \
             AdministratorAccess in the same tenant. Check the caller's own grants \
             with `tachyon org users policies <your-user-id> --tenant-id \
             {tenant_id}`, or re-run with `--profile <admin-profile>`."
        ),
        Some(reqwest::StatusCode::BAD_REQUEST) => format!(
            "could not {verb} policy {policy} for user {user_id}: the API rejected \
             the identifiers. The user argument must be a `us_...` ID rather than \
             an email address, and `--policy` must resolve to a `pol_...` ID."
        ),
        _ => format!("could not {verb} policy {policy} for user {user_id} in tenant {tenant_id}."),
    };
    err.context(hint)
}

async fn run_users_attach_policy(
    api: &ApiClient,
    tenant_id: &str,
    user_id: &str,
    policy: &str,
    resource_scope: Option<&str>,
) -> Result<()> {
    let tenant_id = require_tenant(tenant_id, "attach-policy")?;
    if let Some(scope) = resource_scope {
        validate_resource_scope(scope)?;
    }

    let policy = resolve_policy(api, tenant_id, policy).await?;
    let user_label = resolve_user_label(api, tenant_id, user_id).await?;
    let listed_before = policy_is_listed_for_user(api, user_id, &policy.id).await;

    let request = UserPolicyRequest {
        user_id: user_id.to_string(),
        policy_id: policy.id.clone(),
        tenant_id: tenant_id.to_string(),
        resource_scope: resource_scope.map(str::to_string),
    };
    let path = if resource_scope.is_some() {
        "/v1/auth/user-policies/attach-with-scope"
    } else {
        "/v1/auth/user-policies/attach"
    };
    let _: serde_json::Value = api
        .post(path, &request)
        .await
        .map_err(|err| user_policy_write_hint(err, "attach", tenant_id, user_id, &policy))?;

    println!("Attached policy {policy} to user {user_id} ({user_label}) in tenant {tenant_id}.");
    if let Some(scope) = resource_scope {
        println!("Resource scope: {scope}");
    }
    if listed_before == Some(true) {
        println!(
            "Note: the policy was already listed for this user in this tenant \
             before the call, and attaching is idempotent."
        );
    }
    println!("Verify with: tachyon org users policies {user_id} --tenant-id {tenant_id}");
    Ok(())
}

async fn run_users_detach_policy(
    api: &ApiClient,
    tenant_id: &str,
    user_id: &str,
    policy: &str,
    resource_scope: Option<&str>,
) -> Result<()> {
    let tenant_id = require_tenant(tenant_id, "detach-policy")?;
    if let Some(scope) = resource_scope {
        validate_resource_scope(scope)?;
    }

    let policy = resolve_policy(api, tenant_id, policy).await?;
    let user_label = resolve_user_label(api, tenant_id, user_id).await?;
    let listed_before = policy_is_listed_for_user(api, user_id, &policy.id).await;

    let request = UserPolicyRequest {
        user_id: user_id.to_string(),
        policy_id: policy.id.clone(),
        tenant_id: tenant_id.to_string(),
        resource_scope: resource_scope.map(str::to_string),
    };
    let path = if resource_scope.is_some() {
        "/v1/auth/user-policies/detach-with-scope"
    } else {
        "/v1/auth/user-policies/detach"
    };
    let _: serde_json::Value = api
        .post(path, &request)
        .await
        .map_err(|err| user_policy_write_hint(err, "detach", tenant_id, user_id, &policy))?;

    println!("Detached policy {policy} from user {user_id} ({user_label}) in tenant {tenant_id}.");
    match resource_scope {
        Some(scope) => {
            println!("Resource scope: {scope} (grants with any other scope are left in place)")
        }
        None => println!("Every scope of this policy for the user in this tenant was removed."),
    }
    if listed_before == Some(false) {
        println!(
            "Note: the policy was not listed for this user in this tenant before \
             the call, so nothing was removed."
        );
    }
    println!("Verify with: tachyon org users policies {user_id} --tenant-id {tenant_id}");
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
            UsersCommand::AttachPolicy {
                user_id,
                policy,
                resource_scope,
            } => {
                run_users_attach_policy(&api, tenant_id, user_id, policy, resource_scope.as_deref())
                    .await
            }
            UsersCommand::DetachPolicy {
                user_id,
                policy,
                resource_scope,
            } => {
                run_users_detach_policy(&api, tenant_id, user_id, policy, resource_scope.as_deref())
                    .await
            }
        },
        OrgCommand::ServiceAccounts { command } => match command {
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
