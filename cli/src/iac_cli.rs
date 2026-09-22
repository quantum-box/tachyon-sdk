use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand};
use dialoguer::{theme::ColorfulTheme, Password};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};
use crate::resolve;

#[derive(Debug, Clone, Args)]
pub struct IacArgs {
    #[command(subcommand)]
    pub command: IacCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum IacCommand {
    /// Show manifest revision history
    History {
        #[arg(long)]
        kind: String,
        #[arg(long)]
        name: String,
        #[arg(long, default_value_t = 20)]
        limit: i32,
        #[arg(long)]
        json: bool,
    },
    /// Compare a local manifest file with the latest saved revision
    Diff {
        #[arg(long)]
        file: String,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        name: Option<String>,
        /// App name to select from a multi-app CloudApps manifest
        #[arg(long)]
        app: Option<String>,
    },
    /// Preview whether applying a local manifest will create/update/no-change
    Plan {
        #[arg(long)]
        file: String,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        name: Option<String>,
        /// App name to select from a multi-app CloudApps manifest
        #[arg(long)]
        app: Option<String>,
        /// Override state file path
        #[arg(long)]
        state: Option<String>,
    },
    /// Save and apply a local manifest
    Apply {
        #[arg(long)]
        file: String,
        /// App name to select from a multi-app CloudApps manifest
        #[arg(long)]
        app: Option<String>,
        /// Override state file path
        #[arg(long)]
        state: Option<String>,
        /// Prompt interactively for newly added or changed `$secret_ref` values.
        /// Values are sent only in memory; the server stores them and the
        /// local IaC state keeps the reference instead of the value.
        #[arg(long)]
        prompt_secrets: bool,
        /// Change-control approval token for protected production mutations.
        ///
        /// Prefer TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN so the token does not
        /// appear in shell history or process arguments. Optional during the
        /// compatibility rollout; when present, verified before API access
        /// and forwarded only as a dedicated request header.
        #[arg(
            long = "change-control-token",
            env = "TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN",
            hide_env_values = true
        )]
        change_control_token: Option<String>,
    },
    /// Roll back manifest to a specific revision
    Rollback {
        #[arg(long)]
        kind: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        revision: i32,
        /// Change-control approval token for protected production mutations.
        ///
        /// Prefer TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN so the token does not
        /// appear in shell history or process arguments. Optional during the
        /// compatibility rollout; when present, verified before API access
        /// and forwarded only as a dedicated request header.
        #[arg(
            long = "change-control-token",
            env = "TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN",
            hide_env_values = true
        )]
        change_control_token: Option<String>,
    },
    /// Import IAC manifests from 003-iac-manifests.yaml through the API
    ImportSeed {
        #[arg(long)]
        file: String,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Change-control approval token for protected production mutations.
        ///
        /// Prefer TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN so the token does not
        /// appear in shell history or process arguments. Optional during the
        /// compatibility rollout; ignored for --dry-run because no API
        /// mutation is sent.
        #[arg(
            long = "change-control-token",
            env = "TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN",
            hide_env_values = true
        )]
        change_control_token: Option<String>,
    },
    /// Verify drift between seed manifests and current IAC API state
    VerifySeed {
        #[arg(long)]
        file: String,
    },
    /// Show the current local IaC state file contents
    State {
        /// Override state file path
        #[arg(long)]
        state: Option<String>,
    },
    /// Show OAuth provider configurations (GitHub, Linear, etc.)
    OauthProviders {
        /// Tenant ID to query (uses default if not specified)
        #[arg(long)]
        tenant_id: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// List integrations (connected external services)
    Integrations {
        #[command(subcommand)]
        command: IntegrationsCommand,
    },
    /// List integration connections
    Connections {
        #[command(subcommand)]
        command: ConnectionsCommand,
    },
    /// Manage tenant grants on integration installations
    Grants {
        #[command(subcommand)]
        command: GrantsCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum IntegrationsCommand {
    /// List available integrations
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get integration details
    Get {
        /// Integration ID or name
        id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConnectionsCommand {
    /// List integration connections
    List {
        #[arg(long)]
        json: bool,
    },
    /// Get connection details
    Get {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Disconnect an integration
    Disconnect { id: String },
}

#[derive(Debug, Clone, Subcommand)]
pub enum GrantsCommand {
    /// List tenant grants on integration installations
    List {
        #[arg(long)]
        json: bool,
    },
    /// Create a tenant grant for an installation verified by the caller
    Create {
        #[arg(long)]
        provider: String,
        #[arg(long)]
        installation_id: String,
        /// Value of github_verification returned after GitHub OAuth
        #[arg(long)]
        verification: String,
        /// Grant every resource reachable by the installation
        #[arg(long, conflicts_with = "repos", required_unless_present = "repos")]
        all: bool,
        /// owner/repo, repeatable; an empty list is not a valid CLI scope
        #[arg(long = "repo", conflicts_with = "all", required_unless_present = "all")]
        repos: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Change the resource scope of a tenant grant
    SetScope {
        id: String,
        #[arg(long, conflicts_with = "repos", required_unless_present = "repos")]
        all: bool,
        /// owner/repo, repeatable; an empty list is not a valid CLI scope
        #[arg(long = "repo", conflicts_with = "all", required_unless_present = "all")]
        repos: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Revoke a tenant grant
    Revoke {
        id: String,
        #[arg(long)]
        json: bool,
    },
}

// ---- Response types ----

#[derive(Debug, Deserialize, Serialize)]
struct OAuthProvidersResponse {
    providers: Vec<OAuthProviderConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OAuthProviderConfig {
    provider: String,
    client_id: String,
    redirect_uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct IntegrationResponse {
    id: String,
    name: String,
    description: String,
    category: String,
    provider: String,
    icon_url: Option<String>,
    is_enabled: bool,
    is_featured: bool,
    requires_oauth: bool,
    requires_setup: bool,
}

#[derive(Debug, Deserialize)]
struct IntegrationListResponse {
    integrations: Vec<IntegrationResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IntegrationDetailResponse {
    id: String,
    name: String,
    description: String,
    category: String,
    provider: String,
    icon_url: Option<String>,
    sync_capability: String,
    supported_objects: Vec<String>,
    is_enabled: bool,
    is_featured: bool,
    requires_oauth: bool,
    oauth_scopes: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConnectionResponse {
    id: String,
    integration_id: String,
    provider: String,
    status: String,
    external_account_id: Option<String>,
    external_account_name: Option<String>,
    connected_at: String,
    last_synced_at: Option<String>,
    error_message: Option<String>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ConnectionListResponse {
    connections: Vec<ConnectionResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ResourceScope {
    All,
    List { resources: Vec<String> },
}

#[derive(Debug, Deserialize, Serialize)]
struct TenantGrantResponse {
    id: String,
    status: String,
    resource_scope: ResourceScope,
    managed_by_connection: bool,
    installation: InstallationResponse,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct InstallationResponse {
    id: String,
    provider: String,
    kind: String,
    status: String,
    external_installation_id: Option<String>,
    external_account_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TenantGrantListResponse {
    grants: Vec<TenantGrantResponse>,
}

#[derive(Debug, Serialize)]
struct CreateTenantGrantRequest<'a> {
    provider: &'a str,
    installation_id: &'a str,
    verification: &'a str,
    resource_scope: ResourceScope,
}

#[derive(Debug, Serialize)]
struct UpdateTenantGrantScopeRequest {
    resource_scope: ResourceScope,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestHistoryItem {
    revision: i32,
    content_hash: String,
    applied_by: String,
    applied_at: String,
    manifest: String,
}

#[derive(Debug, Clone)]
struct ManifestIdentity {
    kind: String,
    name: String,
}

struct PlannedManifestWrite {
    manifest: Value,
    identity: ManifestIdentity,
    expected_revision: i32,
    prompted_secrets: Vec<PromptedSecret>,
}

struct PlannedManifestApply {
    write: PlannedManifestWrite,
    action: ChangeAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SecretPromptCandidate {
    provider_index: usize,
    field: String,
    secret_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptedSecret {
    provider_index: usize,
    field: String,
    secret_ref: String,
}

#[derive(Debug, Deserialize)]
struct SeedFile {
    tables: Vec<SeedTable>,
}

#[derive(Debug, Deserialize)]
struct SeedTable {
    name: String,
    rows: Vec<SeedRow>,
}

#[derive(Debug, Deserialize)]
struct SeedRow {
    manifest: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IacState {
    version: u32,
    serial: u64,
    lineage: String,
    resources: Vec<StateResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StateResource {
    kind: String,
    name: String,
    content_hash: String,
    manifest: Value,
    applied_at: DateTime<Utc>,
}

impl Default for IacState {
    fn default() -> Self {
        Self {
            version: 1,
            serial: 0,
            lineage: format!("ln_{}", chrono::Utc::now().timestamp_millis()),
            resources: Vec::new(),
        }
    }
}

impl IacState {
    fn find_resource(&self, kind: &str, name: &str) -> Option<&StateResource> {
        self.resources
            .iter()
            .find(|resource| resource.kind == kind && resource.name == name)
    }

    fn upsert_resource(&mut self, identity: &ManifestIdentity, manifest: Value) {
        let content_hash = content_hash(&manifest);
        let resource = StateResource {
            kind: identity.kind.clone(),
            name: identity.name.clone(),
            content_hash,
            manifest,
            applied_at: Utc::now(),
        };

        if let Some(existing) = self
            .resources
            .iter_mut()
            .find(|r| r.kind == identity.kind && r.name == identity.name)
        {
            *existing = resource;
        } else {
            self.resources.push(resource);
        }
        self.serial += 1;
    }
}

#[derive(Debug, PartialEq)]
enum ChangeAction {
    Create,
    Update,
    NoChange,
}

// ---- Handlers ----

async fn run_oauth_providers(
    api: &ApiClient,
    tenant_id: Option<&str>,
    default_tenant_id: &str,
    json: bool,
) -> Result<()> {
    let tid = tenant_id.unwrap_or(default_tenant_id);
    let resp: OAuthProvidersResponse = api
        .get_query("/v1/iac/oauth-providers", &[("tenant_id", tid)])
        .await?;
    if json {
        return print_json(&resp);
    }
    println!("OAuth Provider Configurations:");
    for provider in &resp.providers {
        println!();
        println!("  {}:", provider.provider);
        println!("    Client ID:    {}", provider.client_id);
        println!("    Redirect URI: {}", provider.redirect_uri);
    }
    Ok(())
}

async fn run_integrations_list(api: &ApiClient, json: bool) -> Result<()> {
    let response: IntegrationListResponse = api.get("/v1/integrations").await?;
    let integrations = response.integrations;
    if json {
        return print_json(&integrations);
    }
    if integrations.is_empty() {
        println!("No integrations found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<20}  {:<16}  {:<8}  REQUIRES SETUP",
        "ID", "NAME", "PROVIDER", "ENABLED"
    );
    println!(
        "{:-<28}  {:-<20}  {:-<16}  {:-<8}  {:-<14}",
        "", "", "", "", ""
    );
    for i in &integrations {
        println!(
            "{:<28}  {:<20}  {:<16}  {:<8}  {}",
            i.id, i.name, i.provider, i.is_enabled, i.requires_setup,
        );
    }
    Ok(())
}

async fn run_integrations_get(api: &ApiClient, id: &str, json: bool) -> Result<()> {
    let i: IntegrationDetailResponse = api.get(&format!("/v1/integrations/{id}")).await?;
    if json {
        return print_json(&i);
    }
    println!("ID:               {}", i.id);
    println!("Name:             {}", i.name);
    println!("Description:      {}", i.description);
    println!("Category:         {}", i.category);
    println!("Provider:         {}", i.provider);
    println!("Sync capability:  {}", i.sync_capability);
    println!("Supported objects: {}", i.supported_objects.join(", "));
    println!("Enabled:          {}", i.is_enabled);
    println!("Featured:         {}", i.is_featured);
    println!("Requires OAuth:   {}", i.requires_oauth);
    println!("OAuth scopes:     {}", i.oauth_scopes.join(", "));
    Ok(())
}

async fn run_connections_list(api: &ApiClient, json: bool) -> Result<()> {
    let response: ConnectionListResponse = api.get("/v1/integrations/connections").await?;
    let conns = response.connections;
    if json {
        return print_json(&conns);
    }
    if conns.is_empty() {
        println!("No connections found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<16}  {:<12}  {:<20}  CONNECTED AT",
        "ID", "PROVIDER", "STATUS", "ACCOUNT"
    );
    println!(
        "{:-<28}  {:-<16}  {:-<12}  {:-<20}  {:-<19}",
        "", "", "", "", ""
    );
    for c in &conns {
        println!(
            "{:<28}  {:<16}  {:<12}  {:<20}  {}",
            c.id,
            c.provider,
            c.status,
            c.external_account_name.as_deref().unwrap_or("-"),
            c.connected_at,
        );
    }
    Ok(())
}

async fn run_connections_get(api: &ApiClient, id: &str, json: bool) -> Result<()> {
    let c: ConnectionResponse = api
        .get(&format!("/v1/integrations/connections/{id}"))
        .await?;
    if json {
        return print_json(&c);
    }
    println!("ID:       {}", c.id);
    println!("Provider:  {}", c.provider);
    println!("Status:    {}", c.status);
    println!(
        "Account:   {}",
        c.external_account_name.as_deref().unwrap_or("-")
    );
    println!("Connected: {}", c.connected_at);
    Ok(())
}

fn resource_scope(all: bool, repos: &[String]) -> Result<ResourceScope> {
    if all {
        return Ok(ResourceScope::All);
    }
    if repos.is_empty() {
        return Err(anyhow!("at least one --repo is required for a list scope"));
    }
    Ok(ResourceScope::List {
        resources: repos.to_vec(),
    })
}

fn render_resource_scope(scope: &ResourceScope) -> String {
    match scope {
        ResourceScope::All => "all".to_string(),
        ResourceScope::List { resources } => format!("list({})", resources.join(",")),
    }
}

fn print_grant(grant: &TenantGrantResponse) {
    println!("ID:        {}", grant.id);
    println!("Provider:  {}", grant.installation.provider);
    println!(
        "Account:   {}",
        grant
            .installation
            .external_account_name
            .as_deref()
            .unwrap_or("-")
    );
    println!("Status:    {}", grant.status);
    println!(
        "Scope:     {}",
        render_resource_scope(&grant.resource_scope)
    );
    println!("Managed:   {}", grant.managed_by_connection);
}

async fn run_grants_list(api: &ApiClient, json: bool) -> Result<()> {
    let response: TenantGrantListResponse = api.get("/v1/integrations/grants").await?;
    if json {
        return print_json(&response.grants);
    }
    if response.grants.is_empty() {
        println!("No tenant grants found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<16}  {:<12}  {:<24}  SCOPE",
        "ID", "PROVIDER", "STATUS", "ACCOUNT"
    );
    println!(
        "{:-<28}  {:-<16}  {:-<12}  {:-<24}  {:-<20}",
        "", "", "", "", ""
    );
    for grant in &response.grants {
        println!(
            "{:<28}  {:<16}  {:<12}  {:<24}  {}",
            grant.id,
            grant.installation.provider,
            grant.status,
            grant
                .installation
                .external_account_name
                .as_deref()
                .unwrap_or("-"),
            render_resource_scope(&grant.resource_scope),
        );
    }
    Ok(())
}

async fn run_grants_create(
    api: &ApiClient,
    provider: &str,
    installation_id: &str,
    verification: &str,
    all: bool,
    repos: &[String],
    json: bool,
) -> Result<()> {
    let response: TenantGrantResponse = api
        .post(
            "/v1/integrations/grants",
            &CreateTenantGrantRequest {
                provider,
                installation_id,
                verification,
                resource_scope: resource_scope(all, repos)?,
            },
        )
        .await?;
    if json {
        return print_json(&response);
    }
    print_grant(&response);
    Ok(())
}

async fn run_grants_set_scope(
    api: &ApiClient,
    id: &str,
    all: bool,
    repos: &[String],
    json: bool,
) -> Result<()> {
    let response: TenantGrantResponse = api
        .patch(
            &format!("/v1/integrations/grants/{id}/scope"),
            &UpdateTenantGrantScopeRequest {
                resource_scope: resource_scope(all, repos)?,
            },
        )
        .await?;
    if json {
        return print_json(&response);
    }
    print_grant(&response);
    Ok(())
}

async fn run_grants_revoke(api: &ApiClient, id: &str, json: bool) -> Result<()> {
    let response: TenantGrantResponse = api
        .delete_json(&format!("/v1/integrations/grants/{id}"))
        .await?;
    if json {
        return print_json(&response);
    }
    println!("Tenant grant {id} revoked.");
    Ok(())
}

async fn run_connections_disconnect(api: &ApiClient, id: &str) -> Result<()> {
    api.delete(&format!("/v1/integrations/connections/{id}"))
        .await?;
    println!("Connection {id} disconnected.");
    Ok(())
}

fn state_path(override_path: Option<&str>) -> PathBuf {
    override_path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tachyon.tfstate"))
}

fn load_state(path: &Path) -> Result<IacState> {
    if !path.exists() {
        return Ok(IacState::default());
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

fn save_state(path: &Path, state: &IacState) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(state)?;
    fs::write(path, format!("{text}\n")).with_context(|| format!("write {}", path.display()))
}

fn content_hash(manifest: &Value) -> String {
    let normalized = serde_json::to_vec(manifest).unwrap_or_default();
    format!("{:x}", Sha256::digest(normalized))
}

fn compute_change(state: &IacState, identity: &ManifestIdentity, manifest: &Value) -> ChangeAction {
    match state.find_resource(&identity.kind, &identity.name) {
        None => ChangeAction::Create,
        Some(existing) if existing.manifest == *manifest => ChangeAction::NoChange,
        Some(_) => ChangeAction::Update,
    }
}

fn should_save_manifest(action: &ChangeAction, prompted_secrets: &[PromptedSecret]) -> bool {
    *action != ChangeAction::NoChange || !prompted_secrets.is_empty()
}

fn load_manifest_files(path: &str, app: Option<&str>) -> Result<Vec<Value>> {
    let content = fs::read_to_string(path).with_context(|| format!("read {path}"))?;
    let path_lower = path.to_lowercase();
    let docs = if path_lower.ends_with(".yaml") || path_lower.ends_with(".yml") {
        let mut docs = Vec::new();
        for doc in serde_yaml::Deserializer::from_str(&content) {
            let value = Value::deserialize(doc)
                .with_context(|| format!("manifest must be valid YAML: {path}"))?;
            if value.is_null() {
                continue;
            }
            docs.extend(normalize_manifest_value(value, app)?);
        }
        docs
    } else {
        let value: Value = serde_json::from_str(&content)
            .with_context(|| format!("manifest must be valid JSON: {path}"))?;
        normalize_manifest_value(value, app)?
    };

    let docs = filter_manifests_for_selected_app(docs, app);
    if docs.is_empty() {
        if let Some(app) = app {
            return Err(anyhow!("no manifest documents matched app {app} in {path}"));
        }
        return Err(anyhow!("no manifest documents found in {path}"));
    }
    Ok(docs)
}

fn normalize_manifest_value(value: Value, app: Option<&str>) -> Result<Vec<Value>> {
    if value.get("kind").and_then(Value::as_str) != Some("CloudApps") {
        return Ok(vec![value]);
    }
    let metadata = value.get("metadata").cloned().unwrap_or_else(|| json!({}));
    let tenant_id = metadata.get("tenantId").cloned();
    let apps = value
        .get("spec")
        .and_then(|s| s.get("apps"))
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("CloudApps manifest must contain spec.apps[]"))?;

    let mut out = Vec::new();
    for entry in apps {
        let app_name = entry
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("CloudApps spec.apps[] entry is missing name"))?;
        if app.is_some_and(|selected| selected != app_name) {
            continue;
        }
        let mut spec = entry.clone();
        let spec_obj = spec
            .as_object_mut()
            .ok_or_else(|| anyhow!("CloudApps app entry must be an object"))?;
        spec_obj.remove("name");
        let mut cloud_metadata = serde_json::Map::new();
        cloud_metadata.insert("name".to_string(), Value::String(app_name.to_string()));
        if let Some(tenant_id) = tenant_id.clone() {
            cloud_metadata.insert("tenantId".to_string(), tenant_id);
        }
        out.push(json!({
            "apiVersion": "apps.tachy.one/v1alpha",
            "kind": "CloudApp",
            "metadata": Value::Object(cloud_metadata),
            "spec": spec,
        }));
    }
    if out.is_empty() && app.is_none() {
        return Err(anyhow!(
            "CloudApps manifest has no app entry matching {}",
            app.unwrap_or("<all apps>")
        ));
    }
    Ok(out)
}

fn filter_manifests_for_selected_app(manifests: Vec<Value>, app: Option<&str>) -> Vec<Value> {
    if app.is_none() {
        return manifests;
    }

    let oauth2_client_refs = collect_oauth2_client_refs(&manifests);
    manifests
        .into_iter()
        .filter(|manifest| {
            if manifest.get("kind").and_then(Value::as_str) != Some("OAuth2Client") {
                return true;
            }
            manifest
                .get("metadata")
                .and_then(|metadata| metadata.get("name"))
                .and_then(Value::as_str)
                .is_some_and(|name| oauth2_client_refs.contains(name))
        })
        .collect()
}

fn collect_oauth2_client_refs(manifests: &[Value]) -> HashSet<String> {
    manifests
        .iter()
        .filter(|manifest| manifest.get("kind").and_then(Value::as_str) == Some("CloudApp"))
        .flat_map(|manifest| {
            manifest
                .get("spec")
                .and_then(|spec| spec.get("envVars"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|env_var| {
            env_var
                .get("valueFrom")
                .and_then(|value_from| value_from.get("oauth2ClientRef"))
                .and_then(|ref_value| ref_value.get("name"))
                .and_then(Value::as_str)
        })
        .map(ToString::to_string)
        .collect()
}

fn manifest_apply_order(manifest: &Value) -> u8 {
    match manifest.get("kind").and_then(Value::as_str) {
        Some("CloudApp") | Some("CloudApps") => 10,
        _ => 0,
    }
}

fn sort_manifests_for_apply(manifests: &mut [Value]) {
    manifests.sort_by_key(manifest_apply_order);
}

fn infer_identity(
    manifest: &Value,
    kind: Option<&str>,
    name: Option<&str>,
) -> Result<ManifestIdentity> {
    let inferred_kind = kind
        .map(ToString::to_string)
        .or_else(|| {
            manifest
                .get("kind")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .ok_or_else(|| anyhow!("missing kind (use --kind or manifest.kind)"))?;
    let inferred_name = name
        .map(ToString::to_string)
        .or_else(|| {
            manifest
                .get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .ok_or_else(|| anyhow!("missing name (use --name or manifest.metadata.name)"))?;

    Ok(ManifestIdentity {
        kind: inferred_kind,
        name: inferred_name,
    })
}

fn inject_tenant_id(manifest: &Value, tenant_id: &str) -> Value {
    let mut manifest = manifest.clone();
    if let Some(metadata) = manifest.get_mut("metadata") {
        if metadata.get("tenantId").is_none() {
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("tenantId".to_string(), Value::String(tenant_id.to_string()));
            }
        }
    }
    manifest
}

fn find_provider<'a>(manifest: Option<&'a Value>, provider_name: &str) -> Option<&'a Value> {
    manifest
        .and_then(|manifest| manifest.pointer("/spec/providers"))
        .and_then(Value::as_array)
        .and_then(|providers| {
            providers.iter().find(|provider| {
                provider.get("name").and_then(Value::as_str) == Some(provider_name)
            })
        })
}

fn collect_secret_prompt_candidates(
    manifest: &Value,
    current_manifest: Option<&Value>,
) -> Vec<SecretPromptCandidate> {
    let Some(providers) = manifest
        .pointer("/spec/providers")
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    for (provider_index, provider) in providers.iter().enumerate() {
        let Some(provider_name) = provider.get("name").and_then(Value::as_str) else {
            continue;
        };
        let Some(config) = provider.get("config").and_then(Value::as_object) else {
            continue;
        };
        let current_provider = find_provider(current_manifest, provider_name);

        for (field, value) in config {
            let Some(secret_ref) = value.get("$secret_ref").and_then(Value::as_str) else {
                continue;
            };

            let current_ref = current_provider
                .and_then(|provider| provider.get("config"))
                .and_then(Value::as_object)
                .and_then(|config| config.get(field))
                .and_then(|value| value.get("$secret_ref"))
                .and_then(Value::as_str);

            // Existing references are intentionally left untouched. This
            // makes re-applying a full manifest non-interactive for secrets
            // that are already part of the current desired state.
            if current_ref == Some(secret_ref) {
                continue;
            }

            candidates.push(SecretPromptCandidate {
                provider_index,
                field: field.clone(),
                secret_ref: secret_ref.to_string(),
            });
        }
    }

    candidates
}

fn nonempty_secret_input(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn prompt_for_manifest_secrets(
    manifest: &mut Value,
    current_manifest: Option<&Value>,
) -> Result<Vec<PromptedSecret>> {
    let candidates = collect_secret_prompt_candidates(manifest, current_manifest);
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    if !std::io::stdin().is_terminal() {
        return Err(anyhow!(
            "--prompt-secrets requires an interactive terminal; omit the flag for non-interactive apply"
        ));
    }

    let theme = ColorfulTheme::default();
    let mut prompted = Vec::new();
    for candidate in candidates {
        let value = Password::with_theme(&theme)
            .with_prompt(format!(
                "Secret for {} (hidden; Enter keeps the $secret_ref)",
                candidate.secret_ref
            ))
            .allow_empty_password(true)
            .interact()
            .with_context(|| format!("read secret for {}", candidate.secret_ref))?;
        let Some(value) = nonempty_secret_input(value) else {
            println!(
                "Skipped {}; the reference will be kept.",
                candidate.secret_ref
            );
            continue;
        };

        let Some(provider) = manifest
            .pointer_mut("/spec/providers")
            .and_then(Value::as_array_mut)
            .and_then(|providers| providers.get_mut(candidate.provider_index))
        else {
            return Err(anyhow!(
                "provider index {} disappeared while prompting for {}",
                candidate.provider_index,
                candidate.secret_ref
            ));
        };
        let Some(config) = provider.get_mut("config").and_then(Value::as_object_mut) else {
            return Err(anyhow!(
                "provider config disappeared while prompting for {}",
                candidate.secret_ref
            ));
        };
        config.insert(candidate.field.clone(), Value::String(value));
        prompted.push(PromptedSecret {
            provider_index: candidate.provider_index,
            field: candidate.field,
            secret_ref: candidate.secret_ref,
        });
    }

    Ok(prompted)
}

fn restore_prompted_secret_refs(manifest: &mut Value, prompted: &[PromptedSecret]) {
    for secret in prompted {
        let Some(provider) = manifest
            .pointer_mut("/spec/providers")
            .and_then(Value::as_array_mut)
            .and_then(|providers| providers.get_mut(secret.provider_index))
        else {
            continue;
        };
        let Some(config) = provider.get_mut("config").and_then(Value::as_object_mut) else {
            continue;
        };
        config.insert(
            secret.field.clone(),
            json!({ "$secret_ref": secret.secret_ref }),
        );
    }
}

async fn graphql_request(api: &ApiClient, body: Value) -> Result<Value> {
    graphql_request_with_change_control(api, body, None).await
}

/// Header carrying the approval token to the authoritative server-side gate.
/// Keeping it out of GraphQL variables prevents request-body logs from
/// retaining the credential.
const CHANGE_CONTROL_TOKEN_HEADER: &str = "x-tachyon-change-control-token";

async fn graphql_request_with_change_control(
    api: &ApiClient,
    body: Value,
    change_control_token: Option<&str>,
) -> Result<Value> {
    let url = format!("{}/v1/graphql", api.base_url);
    let mut request = api.client.post(url).json(&body);
    if let Some(token) = change_control_token
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        request = request.header(CHANGE_CONTROL_TOKEN_HEADER, token);
    }
    let response = request.send().await?;
    let status = response.status();
    let payload: Value = response.json().await?;
    if !status.is_success() {
        return Err(anyhow!(
            "graphql request failed: status={status}, body={payload}"
        ));
    }
    if let Some(errors) = payload.get("errors") {
        return Err(anyhow!("graphql error: {errors}"));
    }
    payload
        .get("data")
        .cloned()
        .ok_or_else(|| anyhow!("missing data in graphql response"))
}

/// Validate an optional token before the first API request. This is only a
/// fail-fast client check; the server remains the authorization boundary.
fn verify_iac_change_control_token(change_control_token: Option<&str>) -> Result<()> {
    let Some(token) = change_control_token
        .map(str::trim)
        .filter(|token| !token.is_empty())
    else {
        return Ok(());
    };

    let verification_key = std::env::var(crate::compute_cli::change_control::VERIFICATION_KEY_ENV)
        .ok()
        .filter(|key| !key.is_empty());
    crate::compute_cli::change_control::verify_change_control_token(
        token,
        "production",
        verification_key.as_deref().map(str::as_bytes),
        Utc::now().timestamp(),
    )?;
    Ok(())
}

async fn fetch_history(
    api: &ApiClient,
    tenant_id: &str,
    kind: &str,
    name: &str,
    limit: i32,
) -> Result<Vec<ManifestHistoryItem>> {
    let body = json!({
        "query": r#"
          query ManifestHistory($operatorId: ID!, $kind: String!, $name: String!, $limit: Int) {
            manifestHistory(operatorId: $operatorId, kind: $kind, name: $name, limit: $limit) {
              revision
              contentHash
              appliedBy
              appliedAt
              manifest
            }
          }
        "#,
        "variables": {
            "operatorId": tenant_id,
            "kind": kind,
            "name": name,
            "limit": limit,
        }
    });
    let data = graphql_request(api, body).await?;
    let value = data
        .get("manifestHistory")
        .ok_or_else(|| anyhow!("manifestHistory not found in response"))?;
    Ok(serde_json::from_value(value.clone())?)
}

async fn fetch_expected_revision(
    api: &ApiClient,
    tenant_id: &str,
    identity: &ManifestIdentity,
) -> Result<i32> {
    Ok(fetch_latest_manifest(api, tenant_id, identity).await?.0)
}

async fn fetch_latest_manifest(
    api: &ApiClient,
    tenant_id: &str,
    identity: &ManifestIdentity,
) -> Result<(i32, Option<Value>)> {
    let history = fetch_history(api, tenant_id, &identity.kind, &identity.name, 1).await?;
    let Some(item) = history.first() else {
        return Ok((0, None));
    };
    let manifest = serde_json::from_str(&item.manifest).with_context(|| {
        format!(
            "parse latest manifest {} / {}",
            identity.kind, identity.name
        )
    })?;
    Ok((item.revision, Some(manifest)))
}

async fn save_manifest(
    api: &ApiClient,
    tenant_id: &str,
    manifest: &Value,
    expected_revision: i32,
    change_control_token: Option<&str>,
) -> Result<()> {
    let body = json!({
        "query": r#"
          mutation SaveManifest($input: SaveManifestInput!) {
            saveManifest(input: $input) { kind }
          }
        "#,
        "variables": {
            "input": {
                "tenantId": tenant_id,
                "manifest": serde_json::to_string(manifest)?,
                "expectedRevision": expected_revision,
            }
        }
    });
    graphql_request_with_change_control(api, body, change_control_token).await?;
    Ok(())
}

async fn apply_manifest_resource(
    api: &ApiClient,
    kind: &str,
    name: &str,
    change_control_token: Option<&str>,
) -> Result<Value> {
    let body = json!({
        "query": r#"
          mutation ApplyManifest($input: ApplyManifestInput!) {
            applyManifest(input: $input) {
              success
              serviceAccountsCreated
              serviceAccountsModified
              providersApplied
              seedDataTables { tableName created updated skipped }
            }
          }
        "#,
        "variables": {
            "input": {
                "kind": kind,
                "name": name,
                "dryRun": false,
            }
        }
    });
    let data = graphql_request_with_change_control(api, body, change_control_token).await?;
    data.get("applyManifest")
        .cloned()
        .ok_or_else(|| anyhow!("applyManifest not found in response"))
}

async fn rollback_manifest(
    api: &ApiClient,
    kind: &str,
    name: &str,
    revision: i32,
    expected_revision: i32,
    change_control_token: Option<&str>,
) -> Result<()> {
    let body = json!({
        "query": r#"
          mutation RollbackManifest($input: RollbackManifestInput!) {
            rollbackManifest(input: $input) { kind }
          }
        "#,
        "variables": {
            "input": {
                "kind": kind,
                "name": name,
                "revision": revision,
                "expectedRevision": expected_revision,
            }
        }
    });
    graphql_request_with_change_control(api, body, change_control_token).await?;
    Ok(())
}

fn extract_iac_manifests(seed_file: SeedFile) -> Vec<Value> {
    seed_file
        .tables
        .into_iter()
        .filter(|table| table.name == "tachyon_apps_iac.manifests")
        .flat_map(|table| table.rows.into_iter().map(|row| row.manifest))
        .collect()
}

fn load_seed_manifests(file: &str) -> Result<Vec<Value>> {
    let content = fs::read_to_string(file).with_context(|| format!("read {file}"))?;
    let seed_file: SeedFile =
        serde_yaml::from_str(&content).with_context(|| format!("parse yaml {file}"))?;
    Ok(extract_iac_manifests(seed_file))
}

async fn run_history(
    api: &ApiClient,
    tenant_id: &str,
    kind: &str,
    name: &str,
    limit: i32,
    json: bool,
) -> Result<()> {
    let history = fetch_history(api, tenant_id, kind, name, limit).await?;
    if json {
        return print_json(&history);
    }
    if history.is_empty() {
        println!("No revisions found.");
        return Ok(());
    }
    for item in history {
        println!(
            "#{:<4} {} {} {}",
            item.revision, item.applied_at, item.applied_by, item.content_hash
        );
    }
    Ok(())
}

async fn run_diff(
    api: &ApiClient,
    tenant_id: &str,
    file: &str,
    kind: Option<&str>,
    name: Option<&str>,
    app: Option<&str>,
) -> Result<()> {
    for manifest in load_manifest_files(file, app)? {
        let manifest = inject_tenant_id(&manifest, tenant_id);
        let identity = infer_identity(&manifest, kind, name)?;
        let history = fetch_history(api, tenant_id, &identity.kind, &identity.name, 1).await?;
        let Some(latest) = history.first() else {
            println!("{} / {}: create", identity.kind, identity.name);
            continue;
        };
        let latest_manifest: Value = serde_json::from_str(&latest.manifest)?;
        if latest_manifest == manifest {
            println!("{} / {}: no changes", identity.kind, identity.name);
        } else {
            println!("{} / {}: update", identity.kind, identity.name);
            println!("  remote_hash: {}", latest.content_hash);
            println!("  local_hash:  {}", content_hash(&manifest));
        }
    }
    Ok(())
}

async fn run_plan(
    tenant_id: &str,
    file: &str,
    kind: Option<&str>,
    name: Option<&str>,
    app: Option<&str>,
    state: Option<&str>,
) -> Result<()> {
    let state_path = state_path(state);
    let state = load_state(&state_path)?;
    println!("State: {}", state_path.display());
    let mut manifests = load_manifest_files(file, app)?;
    sort_manifests_for_apply(&mut manifests);
    for manifest in manifests {
        let manifest = inject_tenant_id(&manifest, tenant_id);
        let identity = infer_identity(&manifest, kind, name)?;
        let action = compute_change(&state, &identity, &manifest);
        println!("{} / {}: {:?}", identity.kind, identity.name, action);
    }
    Ok(())
}

async fn run_apply(
    api: &ApiClient,
    tenant_id: &str,
    file: &str,
    app: Option<&str>,
    state: Option<&str>,
    prompt_secrets: bool,
    change_control_token: Option<&str>,
) -> Result<()> {
    verify_iac_change_control_token(change_control_token)?;
    let state_path = state_path(state);
    let mut iac_state = load_state(&state_path)?;
    let mut manifests = load_manifest_files(file, app)?;
    sort_manifests_for_apply(&mut manifests);

    // Capture every serving revision before the first write so a concurrent
    // update after this planning window is rejected by the server-side CAS.
    let mut planned = Vec::with_capacity(manifests.len());
    for manifest in manifests {
        let mut manifest = inject_tenant_id(&manifest, tenant_id);
        let identity = infer_identity(&manifest, None, None)?;
        let (expected_revision, current_manifest) =
            fetch_latest_manifest(api, tenant_id, &identity).await?;
        let prompted_secrets = if prompt_secrets {
            prompt_for_manifest_secrets(&mut manifest, current_manifest.as_ref())?
        } else {
            Vec::new()
        };
        let action = compute_change(&iac_state, &identity, &manifest);
        planned.push(PlannedManifestApply {
            write: PlannedManifestWrite {
                manifest,
                identity,
                expected_revision,
                prompted_secrets,
            },
            action,
        });
    }

    for plan in planned {
        let PlannedManifestWrite {
            mut manifest,
            identity,
            expected_revision,
            prompted_secrets,
        } = plan.write;
        let action = plan.action;
        if should_save_manifest(&action, &prompted_secrets) {
            save_manifest(
                api,
                tenant_id,
                &manifest,
                expected_revision,
                change_control_token,
            )
            .await?;
        }
        let result =
            apply_manifest_resource(api, &identity.kind, &identity.name, change_control_token)
                .await?;
        restore_prompted_secret_refs(&mut manifest, &prompted_secrets);
        iac_state.upsert_resource(&identity, manifest);
        if action == ChangeAction::NoChange {
            println!(
                "Reconciled: {} / {} (no manifest changes)",
                identity.kind, identity.name
            );
        } else {
            println!(
                "Applied: {} / {} ({action:?})",
                identity.kind, identity.name
            );
        }
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    save_state(&state_path, &iac_state)?;
    Ok(())
}

async fn run_import_seed(
    api: &ApiClient,
    tenant_id: &str,
    file: &str,
    dry_run: bool,
    change_control_token: Option<&str>,
) -> Result<()> {
    let manifests = load_seed_manifests(file)?;
    if dry_run {
        println!(
            "Import dry-run completed: {} manifests found.",
            manifests.len()
        );
        return Ok(());
    }
    verify_iac_change_control_token(change_control_token)?;

    // Preflight the complete seed set before any mutation. Later concurrent
    // changes are detected by each SaveManifest CAS instead of overwritten.
    let mut planned = Vec::with_capacity(manifests.len());
    for manifest in manifests {
        let manifest = inject_tenant_id(&manifest, tenant_id);
        let identity = infer_identity(&manifest, None, None)?;
        let expected_revision = fetch_expected_revision(api, tenant_id, &identity).await?;
        planned.push(PlannedManifestWrite {
            manifest,
            identity,
            expected_revision,
            prompted_secrets: Vec::new(),
        });
    }

    let manifest_count = planned.len();
    for plan in planned {
        save_manifest(
            api,
            tenant_id,
            &plan.manifest,
            plan.expected_revision,
            change_control_token,
        )
        .await?;
        let identity = plan.identity;
        println!("Imported: {} / {}", identity.kind, identity.name);
    }
    println!("Import completed: {manifest_count} manifests saved.");
    Ok(())
}

async fn run_verify_seed(api: &ApiClient, tenant_id: &str, file: &str) -> Result<()> {
    let manifests = load_seed_manifests(file)?;
    let mut drift_messages = Vec::new();
    for expected in manifests {
        let expected = inject_tenant_id(&expected, tenant_id);
        let identity = infer_identity(&expected, None, None)?;
        let history = fetch_history(api, tenant_id, &identity.kind, &identity.name, 1).await?;
        let Some(latest) = history.first() else {
            drift_messages.push(format!(
                "missing manifest: kind={} name={}",
                identity.kind, identity.name
            ));
            continue;
        };
        let actual: Value = serde_json::from_str(&latest.manifest)?;
        if actual != expected {
            drift_messages.push(format!(
                "content drift: kind={} name={}",
                identity.kind, identity.name
            ));
        }
    }
    if drift_messages.is_empty() {
        println!("Drift check passed.");
        return Ok(());
    }
    for message in &drift_messages {
        eprintln!("drift: {message}");
    }
    Err(anyhow!(
        "detected {} IAC manifest drift(s)",
        drift_messages.len()
    ))
}

fn run_state(state: Option<&str>) -> Result<()> {
    let state_path = state_path(state);
    let iac_state = load_state(&state_path)?;
    println!("State:    {}", state_path.display());
    println!("Version:  {}", iac_state.version);
    println!("Serial:   {}", iac_state.serial);
    println!("Lineage:  {}", iac_state.lineage);
    println!("Resources ({}):", iac_state.resources.len());
    for resource in &iac_state.resources {
        println!(
            "  - {} / {} (applied at: {})",
            resource.kind,
            resource.name,
            resource.applied_at.format("%Y-%m-%dT%H:%M:%SZ")
        );
    }
    Ok(())
}

// ---- Entry point ----

pub async fn run(args: &IacArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    match &args.command {
        IacCommand::History {
            kind,
            name,
            limit,
            json,
        } => run_history(&api, tenant_id, kind, name, *limit, *json).await,
        IacCommand::Diff {
            file,
            kind,
            name,
            app,
        } => {
            run_diff(
                &api,
                tenant_id,
                file,
                kind.as_deref(),
                name.as_deref(),
                app.as_deref(),
            )
            .await
        }
        IacCommand::Plan {
            file,
            kind,
            name,
            app,
            state,
        } => {
            run_plan(
                tenant_id,
                file,
                kind.as_deref(),
                name.as_deref(),
                app.as_deref(),
                state.as_deref(),
            )
            .await
        }
        IacCommand::Apply {
            file,
            app,
            state,
            prompt_secrets,
            change_control_token,
        } => {
            run_apply(
                &api,
                tenant_id,
                file,
                app.as_deref(),
                state.as_deref(),
                *prompt_secrets,
                change_control_token.as_deref(),
            )
            .await
        }
        IacCommand::Rollback {
            kind,
            name,
            revision,
            change_control_token,
        } => {
            verify_iac_change_control_token(change_control_token.as_deref())?;
            let identity = ManifestIdentity {
                kind: kind.clone(),
                name: name.clone(),
            };
            let expected_revision = fetch_expected_revision(&api, tenant_id, &identity).await?;
            rollback_manifest(
                &api,
                kind,
                name,
                *revision,
                expected_revision,
                change_control_token.as_deref(),
            )
            .await?;
            println!("Rollback completed: {kind} / {name} => revision {revision}");
            Ok(())
        }
        IacCommand::ImportSeed {
            file,
            dry_run,
            change_control_token,
        } => {
            run_import_seed(
                &api,
                tenant_id,
                file,
                *dry_run,
                change_control_token.as_deref(),
            )
            .await
        }
        IacCommand::VerifySeed { file } => run_verify_seed(&api, tenant_id, file).await,
        IacCommand::State { state } => run_state(state.as_deref()),
        IacCommand::OauthProviders {
            tenant_id: tid,
            json,
        } => run_oauth_providers(&api, tid.as_deref(), tenant_id, *json).await,
        IacCommand::Integrations { command } => match command {
            IntegrationsCommand::List { json } => run_integrations_list(&api, *json).await,
            IntegrationsCommand::Get { id, json } => {
                let resolved = resolve::resolve_integration_id(&api, id).await?;
                run_integrations_get(&api, &resolved, *json).await
            }
        },
        IacCommand::Connections { command } => match command {
            ConnectionsCommand::List { json } => run_connections_list(&api, *json).await,
            ConnectionsCommand::Get { id, json } => run_connections_get(&api, id, *json).await,
            ConnectionsCommand::Disconnect { id } => run_connections_disconnect(&api, id).await,
        },
        IacCommand::Grants { command } => match command {
            GrantsCommand::List { json } => run_grants_list(&api, *json).await,
            GrantsCommand::Create {
                provider,
                installation_id,
                verification,
                all,
                repos,
                json,
            } => {
                run_grants_create(
                    &api,
                    provider,
                    installation_id,
                    verification,
                    *all,
                    repos,
                    *json,
                )
                .await
            }
            GrantsCommand::SetScope {
                id,
                all,
                repos,
                json,
            } => run_grants_set_scope(&api, id, *all, repos, *json).await,
            GrantsCommand::Revoke { id, json } => run_grants_revoke(&api, id, *json).await,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn load_manifest_files_selects_app_across_multi_document_cloudapps() {
        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join("tachyon.yml");
        fs::write(
            &manifest_path,
            r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: fieldadmin
  tenantId: tn_fieldadmin
spec:
  apps:
    - name: fieldadmin
      framework: next_js
---
apiVersion: apps.tachy.one/v1alpha
kind: OAuth2Client
metadata:
  name: fieldadmin-web
  tenantId: tn_fieldadmin
spec:
  redirectUris:
    - https://fieldadmin.txcloud.app/api/auth/callback/cognito
---
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: tachyonfield-storefront
  tenantId: tn_storefront
spec:
  apps:
    - name: tachyonfield
      framework: next_js
"#,
        )
        .unwrap();

        let manifests =
            load_manifest_files(manifest_path.to_str().unwrap(), Some("tachyonfield")).unwrap();

        let names = manifests
            .iter()
            .map(|manifest| {
                manifest
                    .get("metadata")
                    .and_then(|metadata| metadata.get("name"))
                    .and_then(Value::as_str)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["tachyonfield"]);
        assert_eq!(
            manifests[0]
                .get("metadata")
                .and_then(|metadata| metadata.get("tenantId"))
                .and_then(Value::as_str),
            Some("tn_storefront")
        );
    }

    #[test]
    fn load_manifest_files_keeps_referenced_oauth2_client_for_selected_app() {
        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join("tachyon.yml");
        fs::write(
            &manifest_path,
            r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: fieldadmin
  tenantId: tn_fieldadmin
spec:
  apps:
    - name: fieldadmin
      framework: next_js
      envVars:
        - name: COGNITO_CLIENT_ID
          valueFrom:
            oauth2ClientRef:
              name: fieldadmin-web
              field: clientId
---
apiVersion: apps.tachy.one/v1alpha
kind: OAuth2Client
metadata:
  name: fieldadmin-web
  tenantId: tn_fieldadmin
spec:
  redirectUris:
    - https://fieldadmin.txcloud.app/api/auth/callback/cognito
---
apiVersion: apps.tachy.one/v1alpha
kind: OAuth2Client
metadata:
  name: unrelated-web
  tenantId: tn_fieldadmin
spec:
  redirectUris:
    - https://unrelated.txcloud.app/api/auth/callback/cognito
"#,
        )
        .unwrap();

        let manifests =
            load_manifest_files(manifest_path.to_str().unwrap(), Some("fieldadmin")).unwrap();

        let names = manifests
            .iter()
            .map(|manifest| {
                manifest
                    .get("metadata")
                    .and_then(|metadata| metadata.get("name"))
                    .and_then(Value::as_str)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["fieldadmin", "fieldadmin-web"]);
    }

    #[test]
    fn load_manifest_files_errors_when_selected_app_does_not_exist() {
        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join("tachyon.yml");
        fs::write(
            &manifest_path,
            r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: fieldadmin
spec:
  apps:
    - name: fieldadmin
      framework: next_js
"#,
        )
        .unwrap();

        let error = load_manifest_files(manifest_path.to_str().unwrap(), Some("missing"))
            .expect_err("missing app should fail after all documents are inspected");

        assert!(error
            .to_string()
            .contains("no manifest documents matched app missing"));
    }

    #[test]
    fn sort_manifests_for_apply_keeps_prerequisites_before_cloud_apps() {
        let mut manifests = vec![
            json!({
                "kind": "CloudApp",
                "metadata": { "name": "fieldadmin" },
            }),
            json!({
                "kind": "OAuth2Client",
                "metadata": { "name": "fieldadmin-web" },
            }),
        ];

        sort_manifests_for_apply(&mut manifests);

        let kinds = manifests
            .iter()
            .map(|manifest| manifest.get("kind").and_then(Value::as_str).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(kinds, vec!["OAuth2Client", "CloudApp"]);
    }

    #[test]
    fn secret_prompt_candidates_only_include_new_or_changed_references() {
        let current = json!({
            "spec": {
                "providers": [{
                    "name": "openai",
                    "config": {
                        "api_key": { "$secret_ref": "openai/api_key" }
                    }
                }]
            }
        });
        let desired = json!({
            "spec": {
                "providers": [{
                    "name": "openai",
                    "config": {
                        "api_key": { "$secret_ref": "openai/api_key" }
                    }
                }, {
                    "name": "typesafeai",
                    "config": {
                        "api_key": { "$secret_ref": "typesafeai/api_key" }
                    }
                }]
            }
        });

        assert_eq!(
            collect_secret_prompt_candidates(&desired, Some(&current)),
            vec![SecretPromptCandidate {
                provider_index: 1,
                field: "api_key".to_string(),
                secret_ref: "typesafeai/api_key".to_string(),
            }]
        );
    }

    #[test]
    fn prompted_secret_is_restored_to_a_reference_before_local_state_save() {
        let mut manifest = json!({
            "spec": {
                "providers": [{
                    "name": "typesafeai",
                    "config": { "api_key": "in-memory-only" }
                }]
            }
        });
        let prompted = vec![PromptedSecret {
            provider_index: 0,
            field: "api_key".to_string(),
            secret_ref: "typesafeai/api_key".to_string(),
        }];

        restore_prompted_secret_refs(&mut manifest, &prompted);

        assert_eq!(
            manifest["spec"]["providers"][0]["config"]["api_key"],
            json!({ "$secret_ref": "typesafeai/api_key" })
        );
        assert!(!manifest.to_string().contains("in-memory-only"));
    }

    #[test]
    fn nonempty_secret_input_preserves_whitespace() {
        let value = "  secret with whitespace  ".to_string();

        assert_eq!(nonempty_secret_input(value.clone()), Some(value));
        assert_eq!(nonempty_secret_input(String::new()), None);
    }

    #[test]
    fn prompted_secret_forces_save_when_manifest_matches_local_state() {
        let prompted = vec![PromptedSecret {
            provider_index: 0,
            field: "api_key".to_string(),
            secret_ref: "typesafeai/api_key".to_string(),
        }];

        assert!(should_save_manifest(&ChangeAction::NoChange, &prompted));
        assert!(!should_save_manifest(&ChangeAction::NoChange, &[]));
        assert!(should_save_manifest(&ChangeAction::Create, &[]));
    }
}
