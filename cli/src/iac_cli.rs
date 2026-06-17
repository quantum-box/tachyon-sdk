use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
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
    },
    /// Roll back manifest to a specific revision
    Rollback {
        #[arg(long)]
        kind: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        revision: i32,
    },
    /// Import IAC manifests from 003-iac-manifests.yaml through the API
    ImportSeed {
        #[arg(long)]
        file: String,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
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

// ---- Response types ----

#[derive(Debug, Deserialize, Serialize)]
struct OAuthProvidersResponse {
    #[serde(default)]
    github: Option<OAuthProviderConfig>,
    #[serde(default)]
    linear: Option<OAuthProviderConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OAuthProviderConfig {
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IntegrationResponse {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConnectionResponse {
    id: String,
    #[serde(default)]
    integration_id: Option<String>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    account_name: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
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
    println!();
    if let Some(gh) = &resp.github {
        println!("  GitHub:");
        println!(
            "    Enabled:   {}",
            gh.enabled
                .map(|v| if v { "yes" } else { "no" })
                .unwrap_or("-")
        );
        println!("    Client ID: {}", gh.client_id.as_deref().unwrap_or("-"));
        if let Some(scopes) = &gh.scopes {
            println!("    Scopes:    {}", scopes.join(", "));
        }
    } else {
        println!("  GitHub: not configured");
    }
    println!();
    if let Some(lr) = &resp.linear {
        println!("  Linear:");
        println!(
            "    Enabled:   {}",
            lr.enabled
                .map(|v| if v { "yes" } else { "no" })
                .unwrap_or("-")
        );
        println!("    Client ID: {}", lr.client_id.as_deref().unwrap_or("-"));
        if let Some(scopes) = &lr.scopes {
            println!("    Scopes:    {}", scopes.join(", "));
        }
    } else {
        println!("  Linear: not configured");
    }
    Ok(())
}

async fn run_integrations_list(api: &ApiClient, json: bool) -> Result<()> {
    let integrations: Vec<IntegrationResponse> = api.get("/v1/integrations").await?;
    if json {
        return print_json(&integrations);
    }
    if integrations.is_empty() {
        println!("No integrations found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<20}  {:<16}  {:<12}  CREATED AT",
        "ID", "NAME", "PROVIDER", "STATUS"
    );
    println!(
        "{:-<28}  {:-<20}  {:-<16}  {:-<12}  {:-<19}",
        "", "", "", "", ""
    );
    for i in &integrations {
        println!(
            "{:<28}  {:<20}  {:<16}  {:<12}  {}",
            i.id,
            i.name.as_deref().unwrap_or("-"),
            i.provider.as_deref().unwrap_or("-"),
            i.status.as_deref().unwrap_or("-"),
            i.created_at.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

async fn run_integrations_get(api: &ApiClient, id: &str, json: bool) -> Result<()> {
    let i: IntegrationResponse = api.get(&format!("/v1/integrations/{id}")).await?;
    if json {
        return print_json(&i);
    }
    println!("ID:       {}", i.id);
    println!("Name:     {}", i.name.as_deref().unwrap_or("-"));
    println!("Provider: {}", i.provider.as_deref().unwrap_or("-"));
    println!("Status:   {}", i.status.as_deref().unwrap_or("-"));
    println!("Created:  {}", i.created_at.as_deref().unwrap_or("-"));
    Ok(())
}

async fn run_connections_list(api: &ApiClient, json: bool) -> Result<()> {
    let conns: Vec<ConnectionResponse> = api.get("/v1/integrations/connections").await?;
    if json {
        return print_json(&conns);
    }
    if conns.is_empty() {
        println!("No connections found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<16}  {:<12}  {:<20}  CREATED AT",
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
            c.provider.as_deref().unwrap_or("-"),
            c.status.as_deref().unwrap_or("-"),
            c.account_name.as_deref().unwrap_or("-"),
            c.created_at.as_deref().unwrap_or("-"),
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
    println!("Provider: {}", c.provider.as_deref().unwrap_or("-"));
    println!("Status:   {}", c.status.as_deref().unwrap_or("-"));
    println!("Account:  {}", c.account_name.as_deref().unwrap_or("-"));
    println!("Created:  {}", c.created_at.as_deref().unwrap_or("-"));
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

fn load_manifest_files(path: &str, app: Option<&str>) -> Result<Vec<Value>> {
    let content = fs::read_to_string(path).with_context(|| format!("read {path}"))?;
    let path_lower = path.to_lowercase();
    if path_lower.ends_with(".yaml") || path_lower.ends_with(".yml") {
        let mut docs = Vec::new();
        for doc in serde_yaml::Deserializer::from_str(&content) {
            let value = Value::deserialize(doc)
                .with_context(|| format!("manifest must be valid YAML: {path}"))?;
            docs.extend(normalize_manifest_value(value, app)?);
        }
        if docs.is_empty() {
            return Err(anyhow!("no YAML documents found in {path}"));
        }
        Ok(docs)
    } else {
        let value: Value = serde_json::from_str(&content)
            .with_context(|| format!("manifest must be valid JSON: {path}"))?;
        normalize_manifest_value(value, app)
    }
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
    if out.is_empty() {
        return Err(anyhow!(
            "CloudApps manifest has no app entry matching {}",
            app.unwrap_or("<all apps>")
        ));
    }
    Ok(out)
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

async fn graphql_request(api: &ApiClient, body: Value) -> Result<Value> {
    let url = format!("{}/v1/graphql", api.base_url);
    let response = api.client.post(url).json(&body).send().await?;
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

async fn save_manifest(api: &ApiClient, tenant_id: &str, manifest: &Value) -> Result<()> {
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
            }
        }
    });
    graphql_request(api, body).await?;
    Ok(())
}

async fn apply_manifest_resource(api: &ApiClient, kind: &str, name: &str) -> Result<Value> {
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
    let data = graphql_request(api, body).await?;
    data.get("applyManifest")
        .cloned()
        .ok_or_else(|| anyhow!("applyManifest not found in response"))
}

async fn rollback_manifest(api: &ApiClient, kind: &str, name: &str, revision: i32) -> Result<()> {
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
            }
        }
    });
    graphql_request(api, body).await?;
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
    for manifest in load_manifest_files(file, app)? {
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
) -> Result<()> {
    let state_path = state_path(state);
    let mut iac_state = load_state(&state_path)?;
    for manifest in load_manifest_files(file, app)? {
        let manifest = inject_tenant_id(&manifest, tenant_id);
        let identity = infer_identity(&manifest, None, None)?;
        let action = compute_change(&iac_state, &identity, &manifest);
        if action != ChangeAction::NoChange {
            save_manifest(api, tenant_id, &manifest).await?;
        }
        let result = apply_manifest_resource(api, &identity.kind, &identity.name).await?;
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
) -> Result<()> {
    let manifests = load_seed_manifests(file)?;
    if dry_run {
        println!(
            "Import dry-run completed: {} manifests found.",
            manifests.len()
        );
        return Ok(());
    }
    for manifest in &manifests {
        let manifest = inject_tenant_id(manifest, tenant_id);
        let identity = infer_identity(&manifest, None, None)?;
        save_manifest(api, tenant_id, &manifest).await?;
        println!("Imported: {} / {}", identity.kind, identity.name);
    }
    println!("Import completed: {} manifests saved.", manifests.len());
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
        IacCommand::Apply { file, app, state } => {
            run_apply(&api, tenant_id, file, app.as_deref(), state.as_deref()).await
        }
        IacCommand::Rollback {
            kind,
            name,
            revision,
        } => {
            rollback_manifest(&api, kind, name, *revision).await?;
            println!("Rollback completed: {kind} / {name} => revision {revision}");
            Ok(())
        }
        IacCommand::ImportSeed { file, dry_run } => {
            run_import_seed(&api, tenant_id, file, *dry_run).await
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
    }
}
