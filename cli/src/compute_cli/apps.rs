use super::*;
use anyhow::Context;

const DOCUMENT_TENANT_ID_KEY: &str = "__tachyonDocumentTenantId";

// --- Apps subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum AppsCommand {
    /// List all compute apps
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get details of a compute app
    Get {
        /// App ID or name
        app_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Delete a compute app
    Delete {
        /// App ID or name
        app_id: Option<String>,
    },
    /// Create or update compute apps from tachyon.yml
    Apply {
        /// Manifest file path
        #[arg(short = 'f', long, default_value = "tachyon.yml")]
        file: PathBuf,
        /// App name to select from a multi-app CloudApps manifest
        #[arg(long)]
        app: Option<String>,
        /// Target environment label for this apply operation
        #[arg(long, default_value = "sandbox")]
        environment: String,
        /// Required approval token for production apply.
        ///
        /// This only gates write execution. The token is never printed or sent
        /// to the Cloud Apps API by the CLI.
        #[arg(
            long = "change-control-token",
            env = "TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN",
            hide_env_values = true
        )]
        change_control_token: Option<String>,
        /// Preview changes without mutating Cloud Apps
        #[arg(long)]
        dry_run: bool,
    },
    /// Re-sync manifest secret references from server-side sources of truth
    SyncSecrets {
        /// Manifest file path
        #[arg(short = 'f', long, default_value = "tachyon.yml")]
        file: PathBuf,
        /// App name to select from a multi-app CloudApps manifest
        #[arg(long)]
        app: Option<String>,
        /// Target environment label for this sync operation
        #[arg(long)]
        environment: String,
        /// Preview the sync request without mutating Cloud Apps
        #[arg(long)]
        dry_run: bool,
    },
    /// Generate a user feedback report for a compute app
    Feedback(FeedbackArgs),
}

#[derive(Debug, Clone, Args)]
pub struct FeedbackArgs {
    /// App ID or name the feedback is about.
    pub app_id: String,
    /// Feedback body from the user.
    #[arg(long, short = 'm')]
    pub message: String,
    /// Feedback kind.
    #[arg(long, value_enum, default_value_t = FeedbackKind::Other)]
    pub kind: FeedbackKind,
    /// Feedback severity.
    #[arg(long, value_enum, default_value_t = FeedbackSeverity::Medium)]
    pub severity: FeedbackSeverity,
    /// URL where the user observed the issue or request.
    #[arg(long)]
    pub url: Option<String>,
    /// Build ID related to the feedback.
    #[arg(long)]
    pub build_id: Option<String>,
    /// Deployment ID related to the feedback.
    #[arg(long)]
    pub deployment_id: Option<String>,
    /// Optional contact information for follow-up.
    #[arg(long)]
    pub contact: Option<String>,
    /// Additional KEY=VALUE metadata. Secret-like keys are rejected.
    #[arg(long = "metadata")]
    pub metadata: Vec<String>,
    /// Emit a JSON payload instead of Markdown.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackKind {
    Bug,
    Feature,
    Question,
    Other,
}

impl std::fmt::Display for FeedbackKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Bug => "bug",
            Self::Feature => "feature",
            Self::Question => "question",
            Self::Other => "other",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for FeedbackSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct FeedbackPayload {
    pub(super) app_id: String,
    pub(super) operator_id: String,
    pub(super) kind: FeedbackKind,
    pub(super) severity: FeedbackSeverity,
    pub(super) message: String,
    pub(super) url: Option<String>,
    pub(super) build_id: Option<String>,
    pub(super) deployment_id: Option<String>,
    pub(super) contact: Option<String>,
    pub(super) metadata: BTreeMap<String, String>,
    pub(super) created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct ListAppsResponse {
    pub(super) apps: Vec<AppResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct AppResponse {
    pub(super) id: String,
    pub(super) name: String,
    #[serde(default)]
    pub(super) framework: Option<String>,
    #[serde(default)]
    pub(super) repository_url: Option<String>,
    #[serde(default)]
    pub(super) repository_owner: Option<String>,
    #[serde(default)]
    pub(super) repository_name: Option<String>,
    #[serde(default)]
    pub(super) default_branch: Option<String>,
    #[serde(default)]
    pub(super) deployment_target: Option<String>,
    #[serde(default)]
    pub(super) connection_id: Option<String>,
    #[serde(default)]
    pub(super) root_directory: Option<String>,
    #[serde(default)]
    pub(super) docker_context: Option<String>,
    #[serde(default)]
    pub(super) build_command: Option<String>,
    #[serde(default)]
    pub(super) install_command: Option<String>,
    #[serde(default)]
    pub(super) output_directory: Option<String>,
    #[serde(default)]
    pub(super) node_version: Option<String>,
    #[serde(default)]
    pub(super) buildspec_strategy: Option<String>,
    #[serde(default)]
    pub(super) watch_paths: Option<Vec<String>>,
    #[serde(default)]
    pub(super) paths_ignore: Option<Vec<String>>,
    #[serde(default)]
    pub(super) status: Option<String>,
    #[serde(default)]
    pub(super) created_at: Option<String>,
    #[serde(default)]
    pub(super) updated_at: Option<String>,
}

pub(super) async fn run_apps_list(api: &ApiClient, json: bool) -> Result<()> {
    let resp: ListAppsResponse = api.get("/v1/compute/apps").await?;
    if json {
        return print_json(&resp.apps);
    }
    if resp.apps.is_empty() {
        println!("No apps found.");
        return Ok(());
    }
    println!(
        "{:<28}  {:<24}  {:<12}  {:<10}  CREATED AT",
        "ID", "NAME", "FRAMEWORK", "STATUS"
    );
    println!(
        "{:-<28}  {:-<24}  {:-<12}  {:-<10}  {:-<19}",
        "", "", "", "", ""
    );
    for app in &resp.apps {
        println!(
            "{:<28}  {:<24}  {:<12}  {:<10}  {}",
            app.id,
            truncate(&app.name, 24),
            app.framework.as_deref().unwrap_or("-"),
            app.status.as_deref().unwrap_or("-"),
            app.created_at
                .as_deref()
                .map(format_created_at)
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    Ok(())
}

pub(super) async fn run_apps_get(api: &ApiClient, app_id: &str, json: bool) -> Result<()> {
    let app: AppResponse = api.get(&format!("/v1/compute/apps/{app_id}")).await?;
    if json {
        return print_json(&app);
    }
    println!("ID:         {}", app.id);
    println!("Name:       {}", app.name);
    println!("Framework:  {}", app.framework.as_deref().unwrap_or("-"));
    println!(
        "Repository: {}",
        app.repository_url.as_deref().unwrap_or("-")
    );
    println!("Status:     {}", app.status.as_deref().unwrap_or("-"));
    println!(
        "Created:    {}",
        app.created_at
            .as_deref()
            .map(format_created_at)
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "Updated:    {}",
        app.updated_at
            .as_deref()
            .map(format_created_at)
            .unwrap_or_else(|| "-".to_string())
    );
    Ok(())
}

pub(super) async fn run_apps_delete(api: &ApiClient, app_id: &str) -> Result<()> {
    api.delete(&format!("/v1/compute/apps/{app_id}")).await?;
    println!("App {app_id} deleted.");
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum ApplyAction {
    Create,
    Update,
    NoChange,
}

struct AppApplyPlan {
    name: String,
    body: Value,
    env_plan: EnvPlan,
    iac_manifest: Option<Value>,
    sentry_plan: Option<SentryIntegrationPlan>,
}

#[derive(Debug, Default)]
pub(crate) struct EnvPlan {
    pub(super) plain: Vec<SetEnvVarEntry>,
    pub(super) secret_refs: Vec<SecretEnvRef>,
    pub(super) server_managed_credentials: Vec<ServerManagedCredentialRef>,
    pub(super) sentry_integrations: Vec<SentryIntegrationPlan>,
}

#[derive(Debug)]
pub(super) struct SecretEnvRef {
    pub(super) key: String,
    pub(super) target: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ServerManagedCredentialRef {
    pub(super) key: String,
    pub(super) target: String,
    pub(super) source: ServerManagedCredentialSource,
}

/// Server-managed credential source accepted by `tachyon compute apps apply`.
///
/// The CLI does not resolve these references locally. It preserves the
/// selected `CloudApp` manifest and asks the server-side IaC apply path to
/// materialize the final app-env secret. Keeping this as an enum avoids
/// carrying `"databaseRef"` / `"oauth2ClientRef"` / `"storageRef"` string
/// literals through planner logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ServerManagedCredentialSource {
    /// `valueFrom.databaseRef`
    Database,
    /// `valueFrom.oauth2ClientRef`
    OAuth2Client,
    /// `valueFrom.storageRef`
    Storage,
}

impl ServerManagedCredentialSource {
    /// Returns the manifest field name used for display and diagnostics.
    fn as_str(self) -> &'static str {
        match self {
            Self::Database => "databaseRef",
            Self::OAuth2Client => "oauth2ClientRef",
            Self::Storage => "storageRef",
        }
    }
}

impl std::fmt::Display for ServerManagedCredentialSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct SentryIntegrationPlan {
    pub(super) project: String,
    pub(super) provider: String,
    pub(super) env_vars: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(super) struct SyncSecretsRequest {
    pub(super) app_name: String,
    pub(super) environment: String,
    pub(super) manifest_kind: String,
    pub(super) refs: Vec<SyncSecretRef>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(super) struct SyncSecretRef {
    pub(super) key: String,
    pub(super) target: String,
    pub(super) source: String,
    #[serde(rename = "sourceRef")]
    pub(super) source_ref: Value,
}

#[derive(Debug, Deserialize)]
struct SyncSecretsResponse {
    #[serde(default)]
    synced: usize,
    #[serde(default)]
    skipped: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ListIntegrationsResponse {
    integrations: Vec<IntegrationInfo>,
}

#[derive(Debug, Deserialize)]
struct IntegrationInfo {
    provider: String,
    is_enabled: bool,
    #[serde(default)]
    requires_setup: bool,
}

#[derive(Debug, Deserialize)]
struct ListConnectionsResponse {
    connections: Vec<IntegrationConnection>,
}

#[derive(Debug, Deserialize)]
struct IntegrationConnection {
    provider: String,
    status: String,
}

pub(crate) async fn run_apps_apply(
    api: &ApiClient,
    tenant_id: &str,
    file: &Path,
    selected_app: Option<&str>,
    environment: &str,
    change_control_token: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    require_production_apply_approval(environment, change_control_token, dry_run)?;

    let manifest = load_cloud_apps_manifest(file)?;
    let entries = select_app_entries(&manifest, selected_app)?;
    let plans = build_app_apply_plans(entries, tenant_id, environment)?;
    if plans.iter().any(|plan| plan.sentry_plan.is_some()) {
        validate_sentry_integration(api).await?;
    }
    let live: ListAppsResponse = api.get("/v1/compute/apps").await?;

    println!("Manifest:    {}", file.display());
    println!("Environment: {environment}");
    println!("Mode:        {}", if dry_run { "dry-run" } else { "apply" });
    println!();

    let mut created = 0;
    let mut updated = 0;
    let mut unchanged = 0;
    for plan in plans {
        let existing = live.apps.iter().find(|app| app.name == plan.name);
        let (action, changed_fields) = classify_app_action(existing, &plan.body);
        let app_id = match (existing, &action, dry_run) {
            (Some(app), ApplyAction::Update, false) => {
                let updated: AppResponse = api
                    .patch(&format!("/v1/compute/apps/{}", app.id), &plan.body)
                    .await?;
                updated.id
            }
            (Some(app), _, _) => app.id.clone(),
            (None, ApplyAction::Create, false) => {
                let created: AppResponse = api.post("/v1/compute/apps", &plan.body).await?;
                created.id
            }
            (None, _, true) => "<new app>".to_string(),
            (None, _, false) => unreachable!(),
        };

        let (env_changed, missing_secrets) =
            apply_env_plan(api, &app_id, &plan.env_plan, dry_run).await?;
        match action {
            ApplyAction::Create => created += 1,
            ApplyAction::Update => updated += 1,
            ApplyAction::NoChange => unchanged += 1,
        }
        let label = match action {
            ApplyAction::Create => "CREATED",
            ApplyAction::Update => "UPDATED",
            ApplyAction::NoChange => "UNCHANGED",
        };
        println!("{label} {} ({app_id})", plan.name);
        println!("  environment: {environment}");
        println!("  manifest:    {}", file.display());
        if changed_fields.is_empty() {
            println!("  changed:     <none>");
        } else {
            println!("  changed:     {}", changed_fields.join(", "));
        }
        if !env_changed.is_empty() {
            println!("  env:         {}", env_changed.join(", "));
        }
        if !plan.env_plan.server_managed_credentials.is_empty() {
            let refs = plan
                .env_plan
                .server_managed_credentials
                .iter()
                .map(|credential| {
                    format!(
                        "{}({}; {})",
                        credential.key, credential.target, credential.source
                    )
                })
                .collect::<Vec<_>>();
            println!("  managed env: {}", refs.join(", "));
            if dry_run {
                println!("  next:        run without --dry-run to save/apply the CloudApp manifest server-side");
            } else {
                let manifest = plan
                    .iac_manifest
                    .as_ref()
                    .ok_or_else(|| anyhow!("missing CloudApp IaC manifest for {}", plan.name))?;
                apply_compute_cloud_app_manifest(api, manifest).await?;
                println!("  iac:         applied server-managed credential refs");
            }
        }
        if let Some(sentry) = &plan.sentry_plan {
            println!(
                "  sentry:      project={} provider={} env={}",
                sentry.project,
                sentry.provider,
                sentry.env_vars.join(", ")
            );
        }
        if !missing_secrets.is_empty() {
            println!("  missing secrets: {}", missing_secrets.join(", "));
            println!("  next:        tachyon compute env set {app_id} KEY=<value>");
        }
    }

    println!();
    println!("Summary: {created} created, {updated} updated, {unchanged} unchanged");
    Ok(())
}

fn require_production_apply_approval(
    environment: &str,
    change_control_token: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    if dry_run || !is_production_environment(environment) {
        return Ok(());
    }

    let approved = change_control_token
        .map(str::trim)
        .is_some_and(|token| !token.is_empty());
    if approved {
        return Ok(());
    }

    Err(anyhow!(
        "production Cloud App apply requires change-control approval; pass --change-control-token or set TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN. Use --dry-run to preview without writing."
    ))
}

fn is_production_environment(environment: &str) -> bool {
    matches!(
        environment.trim().to_ascii_lowercase().as_str(),
        "production" | "prod"
    )
}

fn build_app_apply_plans(
    entries: Vec<Value>,
    tenant_id: &str,
    environment: &str,
) -> Result<Vec<AppApplyPlan>> {
    entries
        .into_iter()
        .map(|entry| {
            let name = entry
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("app entry is missing name"))?
                .to_string();
            let body = app_entry_to_api_body(&entry)?;
            let env_plan = plan_env_vars(&entry, environment)?;
            let iac_manifest = if env_plan.server_managed_credentials.is_empty() {
                None
            } else {
                Some(cloud_app_manifest_for_iac(&entry, tenant_id)?)
            };
            let sentry_plan = plan_sentry_integration(&entry)?;
            Ok(AppApplyPlan {
                name,
                body,
                env_plan,
                iac_manifest,
                sentry_plan,
            })
        })
        .collect()
}

pub(crate) fn load_cloud_apps_manifest(path: &Path) -> Result<Value> {
    let content = std::fs::read_to_string(path)?;
    let mut apps = Vec::new();
    let mut oauth2_clients = HashMap::<(Option<String>, String), Value>::new();
    let mut non_app_kinds = Vec::new();

    for (index, doc) in serde_yaml::Deserializer::from_str(&content).enumerate() {
        let value = Value::deserialize(doc)
            .with_context(|| format!("parse YAML document {} in {}", index + 1, path.display()))?;
        if value.is_null() {
            continue;
        }
        match value.get("kind").and_then(Value::as_str) {
            Some("CloudApps") => apps.extend(cloud_apps_entries(&value)?),
            Some("CloudApp") => apps.push(cloud_app_entry(&value)?),
            Some("OAuth2Client") => {
                let dependency = oauth2_client_dependency(&value)?;
                let key = (dependency.tenant_id, dependency.name);
                if oauth2_clients
                    .insert(key.clone(), dependency.spec)
                    .is_some()
                {
                    return Err(anyhow!(
                        "manifest has multiple OAuth2Client documents named {}",
                        key.1
                    ));
                }
                non_app_kinds.push("OAuth2Client".to_string());
            }
            Some(kind) => non_app_kinds.push(kind.to_string()),
            None => non_app_kinds.push("<missing kind>".to_string()),
        }
    }

    if apps.is_empty() {
        return match non_app_kinds.first() {
            Some(kind) if kind == "<missing kind>" => Err(anyhow!("manifest is missing kind")),
            Some(kind) => Err(anyhow!("unsupported manifest kind: {kind}")),
            None => Err(anyhow!("manifest is empty")),
        };
    }
    enrich_oauth2_client_refs(&mut apps, &oauth2_clients)?;

    Ok(json!({
        "apiVersion": "apps.tachy.one/v1alpha",
        "kind": "CloudApps",
        "metadata": {
            "name": path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("cloud-apps")
        },
        "spec": { "apps": apps }
    }))
}

fn cloud_apps_entries(value: &Value) -> Result<Vec<Value>> {
    let metadata = value.get("metadata").cloned().unwrap_or_else(|| json!({}));
    let tenant_id = metadata.get("tenantId").cloned();
    let apps = value
        .get("spec")
        .and_then(|s| s.get("apps"))
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("CloudApps manifest must contain spec.apps[]"))?;

    let mut entries = Vec::new();
    for entry in apps {
        let mut entry = entry.clone();
        let entry_obj = entry
            .as_object_mut()
            .ok_or_else(|| anyhow!("CloudApps app entry must be an object"))?;
        entry_obj
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("CloudApps spec.apps[] entry is missing name"))?;
        if let Some(tenant_id) = tenant_id.clone() {
            entry_obj.insert(DOCUMENT_TENANT_ID_KEY.to_string(), tenant_id);
        }
        entries.push(entry);
    }
    Ok(entries)
}

fn cloud_app_entry(value: &Value) -> Result<Value> {
    let metadata = value.get("metadata").cloned().unwrap_or_else(|| json!({}));
    let name = metadata
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("CloudApp manifest is missing metadata.name"))?;
    let mut entry = value.get("spec").cloned().unwrap_or_else(|| json!({}));
    let entry_obj = entry
        .as_object_mut()
        .ok_or_else(|| anyhow!("CloudApp spec must be an object"))?;
    entry_obj.insert("name".to_string(), Value::String(name.to_string()));
    if let Some(tenant_id) = metadata.get("tenantId").cloned() {
        entry_obj.insert(DOCUMENT_TENANT_ID_KEY.to_string(), tenant_id);
    }
    Ok(entry)
}

struct OAuth2ClientDependency {
    tenant_id: Option<String>,
    name: String,
    spec: Value,
}

fn oauth2_client_dependency(value: &Value) -> Result<OAuth2ClientDependency> {
    let metadata = value.get("metadata").cloned().unwrap_or_else(|| json!({}));
    let name = metadata
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("OAuth2Client manifest is missing metadata.name"))?
        .to_string();
    let tenant_id = metadata
        .get("tenantId")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let spec = value
        .get("spec")
        .cloned()
        .ok_or_else(|| anyhow!("OAuth2Client manifest {name} is missing spec"))?;
    if !spec.is_object() {
        return Err(anyhow!(
            "OAuth2Client manifest {name} spec must be an object"
        ));
    }
    Ok(OAuth2ClientDependency {
        tenant_id,
        name,
        spec,
    })
}

fn enrich_oauth2_client_refs(
    apps: &mut [Value],
    oauth2_clients: &HashMap<(Option<String>, String), Value>,
) -> Result<()> {
    if oauth2_clients.is_empty() {
        return Ok(());
    }

    for app in apps {
        let app_tenant_id = app
            .get(DOCUMENT_TENANT_ID_KEY)
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let Some(env_vars) = app.get_mut("envVars").and_then(Value::as_array_mut) else {
            continue;
        };
        for env_var in env_vars {
            let Some(client_ref) = env_var
                .get_mut("valueFrom")
                .and_then(Value::as_object_mut)
                .and_then(|value_from| value_from.get_mut("oauth2ClientRef"))
                .and_then(Value::as_object_mut)
            else {
                continue;
            };
            let Some(client_name) = client_ref.get("name").and_then(Value::as_str) else {
                continue;
            };
            let Some(spec) = find_oauth2_client_dependency(
                oauth2_clients,
                app_tenant_id.as_deref(),
                client_name,
            )?
            else {
                continue;
            };
            for key in [
                "redirectUris",
                "allowedScopes",
                "grantTypes",
                "useTachyonUserPool",
            ] {
                if !client_ref.contains_key(key) {
                    if let Some(value) = spec.get(key) {
                        client_ref.insert(key.to_string(), value.clone());
                    }
                }
            }
        }
    }

    Ok(())
}

fn find_oauth2_client_dependency<'a>(
    oauth2_clients: &'a HashMap<(Option<String>, String), Value>,
    tenant_id: Option<&str>,
    name: &str,
) -> Result<Option<&'a Value>> {
    let key = (tenant_id.map(ToString::to_string), name.to_string());
    if let Some(spec) = oauth2_clients.get(&key) {
        return Ok(Some(spec));
    }

    let matches = oauth2_clients
        .iter()
        .filter(|((_, client_name), _)| client_name == name)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Ok(None),
        [(_, spec)] => Ok(Some(*spec)),
        _ => Err(anyhow!(
            "OAuth2Client dependency {name} is ambiguous across manifest documents"
        )),
    }
}

pub(crate) fn select_app_entries(manifest: &Value, app: Option<&str>) -> Result<Vec<Value>> {
    let apps = manifest
        .get("spec")
        .and_then(|s| s.get("apps"))
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("CloudApps manifest must contain spec.apps[]"))?;
    let entries = apps
        .iter()
        .filter(|entry| {
            app.is_none_or(|name| entry.get("name").and_then(Value::as_str) == Some(name))
        })
        .cloned()
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return Err(anyhow!(
            "no app entry matched {}",
            app.unwrap_or("<all apps>")
        ));
    }
    ensure_unique_app_names(&entries, app)?;
    Ok(entries)
}

fn ensure_unique_app_names(entries: &[Value], selected_app: Option<&str>) -> Result<()> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for entry in entries {
        let Some(name) = entry.get("name").and_then(Value::as_str) else {
            continue;
        };
        if !seen.insert(name.to_string()) {
            duplicates.insert(name.to_string());
        }
    }
    if duplicates.is_empty() {
        return Ok(());
    }
    if let Some(app) = selected_app {
        return Err(anyhow!(
            "multiple CloudApps documents contain app {app}; app names must be unique to avoid ambiguous apply"
        ));
    }
    Err(anyhow!(
        "CloudApps manifest contains duplicate app name(s): {}",
        duplicates.into_iter().collect::<Vec<_>>().join(", ")
    ))
}

pub(super) fn select_single_app_entry(manifest: &Value, app: Option<&str>) -> Result<Value> {
    let apps = manifest
        .get("spec")
        .and_then(|s| s.get("apps"))
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("CloudApps manifest must contain spec.apps[]"))?;
    if app.is_none() && apps.len() != 1 {
        return Err(anyhow!(
            "sync-secrets requires --app when the CloudApps manifest contains {} apps",
            apps.len()
        ));
    }
    let entries = select_app_entries(manifest, app)?;
    match entries.as_slice() {
        [entry] => Ok(entry.clone()),
        _ => Err(anyhow!(
            "sync-secrets requires exactly one selected app; pass --app to disambiguate"
        )),
    }
}

pub(crate) fn app_entry_to_api_body(entry: &Value) -> Result<Value> {
    let name = entry
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("app entry is missing name"))?;
    let repo = entry
        .get("repository")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("app entry {name} is missing repository"))?;
    let repo_str = |key: &str| -> Result<String> {
        repo.get(key)
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| anyhow!("app entry {name} repository.{key} is required"))
    };
    let framework = entry
        .get("framework")
        .and_then(Value::as_str)
        .unwrap_or("next_js");
    let deployment_target = entry
        .get("deploymentTarget")
        .and_then(Value::as_str)
        .unwrap_or("cloud_run");
    let build = entry.get("build").and_then(Value::as_object);
    let build_command = build.and_then(|b| {
        b.get("command")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .or_else(|| cargo_lambda_build_command(framework, b))
    });
    let mut body = json!({
        "name": name,
        "repository_url": repo_str("url")?,
        "repository_owner": repo_str("owner")?,
        "repository_name": repo_str("name")?,
        "default_branch": repo.get("defaultBranch").and_then(Value::as_str).unwrap_or("main"),
        "framework": framework,
        "deployment_target": deployment_target,
    });
    let obj = body.as_object_mut().unwrap();
    copy_string_field(entry, obj, "rootDirectory", "root_directory");
    copy_string_field(entry, obj, "dockerContext", "docker_context");
    copy_string_field(entry, obj, "buildspecStrategy", "buildspec_strategy");
    if let Some(command) = build_command {
        obj.insert("build_command".to_string(), Value::String(command));
    }
    if let Some(build) = build {
        copy_string_field_from_map(build, obj, "installCommand", "install_command");
        copy_string_field_from_map(build, obj, "outputDirectory", "output_directory");
        copy_string_field_from_map(build, obj, "nodeVersion", "node_version");
    }
    if let Some(paths) = entry.get("watchPaths").and_then(Value::as_array) {
        let paths: Vec<Value> = paths
            .iter()
            .filter_map(|p| p.as_str().map(|s| Value::String(s.to_string())))
            .collect();
        obj.insert("watch_paths".to_string(), Value::Array(paths));
    }
    if let Some(paths) = entry.get("pathsIgnore").and_then(Value::as_array) {
        let paths: Vec<Value> = paths
            .iter()
            .filter_map(|p| p.as_str().map(|s| Value::String(s.to_string())))
            .collect();
        obj.insert("paths_ignore".to_string(), Value::Array(paths));
    }
    Ok(body)
}

fn copy_string_field(
    source: &Value,
    target: &mut serde_json::Map<String, Value>,
    from: &str,
    to: &str,
) {
    if let Some(value) = source.get(from).and_then(Value::as_str) {
        target.insert(to.to_string(), Value::String(value.to_string()));
    }
}

fn copy_string_field_from_map(
    source: &serde_json::Map<String, Value>,
    target: &mut serde_json::Map<String, Value>,
    from: &str,
    to: &str,
) {
    if let Some(value) = source.get(from).and_then(Value::as_str) {
        target.insert(to.to_string(), Value::String(value.to_string()));
    }
}

fn cargo_lambda_build_command(
    framework: &str,
    build: &serde_json::Map<String, Value>,
) -> Option<String> {
    if framework != "cargo_lambda" {
        return None;
    }
    let package = build.get("package").and_then(Value::as_str)?;
    let binary = build.get("binary").and_then(Value::as_str);
    let release = build
        .get("release")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let arch = build.get("arch").and_then(Value::as_str).unwrap_or("arm64");
    let mut command = format!("cargo lambda build --package {package}");
    if let Some(binary) = binary {
        command.push_str(&format!(" --bin {binary}"));
    }
    if release {
        command.push_str(" --release");
    }
    if arch == "arm64" {
        command.push_str(" --arm64");
    }
    Some(command)
}

fn classify_app_action(existing: Option<&AppResponse>, body: &Value) -> (ApplyAction, Vec<String>) {
    match existing {
        None => (ApplyAction::Create, manifest_body_fields(body)),
        Some(app) => {
            let fields = manifest_body_fields(body)
                .into_iter()
                .filter(|field| app_field_value(app, field) != body[field])
                .collect::<Vec<_>>();
            let action = if fields.is_empty() {
                ApplyAction::NoChange
            } else {
                ApplyAction::Update
            };
            (action, fields)
        }
    }
}

fn manifest_body_fields(body: &Value) -> Vec<String> {
    body.as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

fn app_field_value(app: &AppResponse, field: &str) -> Value {
    match field {
        "name" => json!(app.name),
        "repository_url" => opt_string_value(app.repository_url.as_deref()),
        "repository_owner" => opt_string_value(app.repository_owner.as_deref()),
        "repository_name" => opt_string_value(app.repository_name.as_deref()),
        "default_branch" => opt_string_value(app.default_branch.as_deref()),
        "framework" => opt_string_value(app.framework.as_deref()),
        "deployment_target" => opt_string_value(app.deployment_target.as_deref()),
        "connection_id" => opt_string_value(app.connection_id.as_deref()),
        "root_directory" => opt_string_value(app.root_directory.as_deref()),
        "docker_context" => opt_string_value(app.docker_context.as_deref()),
        "build_command" => opt_string_value(app.build_command.as_deref()),
        "install_command" => opt_string_value(app.install_command.as_deref()),
        "output_directory" => opt_string_value(app.output_directory.as_deref()),
        "node_version" => opt_string_value(app.node_version.as_deref()),
        "buildspec_strategy" => {
            opt_string_value(app.buildspec_strategy.as_deref().or(Some("inline")))
        }
        "watch_paths" => match &app.watch_paths {
            Some(paths) if !paths.is_empty() => {
                Value::Array(paths.iter().map(|p| Value::String(p.clone())).collect())
            }
            _ => Value::Null,
        },
        "paths_ignore" => match &app.paths_ignore {
            Some(paths) if !paths.is_empty() => {
                Value::Array(paths.iter().map(|p| Value::String(p.clone())).collect())
            }
            _ => Value::Null,
        },
        _ => Value::Null,
    }
}

fn opt_string_value(value: Option<&str>) -> Value {
    match value.filter(|v| !v.is_empty()) {
        Some(value) => Value::String(value.to_string()),
        None => Value::Null,
    }
}

#[cfg(test)]
mod manifest_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_manifest(content: &str) -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("tachyon.yml");
        fs::write(&path, content).unwrap();
        (tmp, path)
    }

    #[test]
    fn load_cloud_apps_manifest_reads_apps_from_all_yaml_documents() {
        let (_tmp, path) = write_manifest(
            r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: first
spec:
  apps:
    - name: api
      repository:
        url: https://github.com/quantum-box/tachyonfield
        owner: quantum-box
        name: tachyonfield
---
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: second
spec:
  apps:
    - name: fieldadmin
      repository:
        url: https://github.com/quantum-box/tachyonfield
        owner: quantum-box
        name: tachyonfield
"#,
        );

        let manifest = load_cloud_apps_manifest(&path).unwrap();
        let first = select_app_entries(&manifest, Some("api")).unwrap();
        let second = select_app_entries(&manifest, Some("fieldadmin")).unwrap();

        assert_eq!(first[0]["name"], "api");
        assert_eq!(second[0]["name"], "fieldadmin");
    }

    #[test]
    fn load_cloud_apps_manifest_enriches_oauth2_client_refs_from_dependency_documents() {
        let (_tmp, path) = write_manifest(
            r#"
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: first
  tenantId: tn_01hjjn348rn3t49zz6hvmfq67p
spec:
  apps:
    - name: fieldadmin
      repository:
        url: https://github.com/quantum-box/tachyonfield
        owner: quantum-box
        name: tachyonfield
      framework: vinext
      deploymentTarget: cloudflare_pages
      envVars:
        - name: COGNITO_CLIENT_ID
          type: credential
          valueFrom:
            oauth2ClientRef:
              name: fieldadmin-web
              field: clientId
---
apiVersion: apps.tachy.one/v1alpha
kind: OAuth2Client
metadata:
  name: fieldadmin-web
  tenantId: tn_01hjjn348rn3t49zz6hvmfq67p
spec:
  redirectUris:
    - https://fieldadmin.txcloud.app/api/auth/callback/cognito
  allowedScopes:
    - openid
    - profile
  grantTypes:
    - authorization_code
    - password
  useTachyonUserPool: true
"#,
        );

        let manifest = load_cloud_apps_manifest(&path).unwrap();
        let entries = select_app_entries(&manifest, Some("fieldadmin")).unwrap();
        let client_ref = &entries[0]["envVars"][0]["valueFrom"]["oauth2ClientRef"];

        assert_eq!(
            client_ref["redirectUris"][0],
            "https://fieldadmin.txcloud.app/api/auth/callback/cognito"
        );
        assert_eq!(client_ref["allowedScopes"], json!(["openid", "profile"]));
        assert_eq!(
            client_ref["grantTypes"],
            json!(["authorization_code", "password"])
        );
        assert_eq!(client_ref["useTachyonUserPool"], true);

        let iac_manifest =
            cloud_app_manifest_for_iac(&entries[0], "tn_01hjjn348rn3t49zz6hvmfq67p").unwrap();
        let iac_ref = &iac_manifest["spec"]["envVars"][0]["valueFrom"]["oauth2ClientRef"];
        assert_eq!(iac_ref["allowedScopes"], json!(["openid", "profile"]));
        assert!(iac_manifest["spec"].get(DOCUMENT_TENANT_ID_KEY).is_none());
    }

    #[test]
    fn load_cloud_apps_manifest_skips_unsupported_documents() {
        let (_tmp, path) = write_manifest(
            r#"
apiVersion: iac.tachy.one/v1alpha
kind: TerraformStack
metadata:
  name: infra
---
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
spec:
  apps:
    - name: field
      repository:
        url: https://github.com/quantum-box/tachyonfield
        owner: quantum-box
        name: tachyonfield
"#,
        );

        let manifest = load_cloud_apps_manifest(&path).unwrap();
        let entries = select_app_entries(&manifest, Some("field")).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["name"], "field");
    }

    #[test]
    fn select_app_entries_rejects_duplicate_app_names_across_documents() {
        let (_tmp, path) = write_manifest(
            r#"
kind: CloudApps
spec:
  apps:
    - name: fieldadmin
      repository:
        url: https://github.com/quantum-box/tachyonfield
        owner: quantum-box
        name: tachyonfield
---
kind: CloudApps
spec:
  apps:
    - name: fieldadmin
      repository:
        url: https://github.com/quantum-box/tachyonfield
        owner: quantum-box
        name: tachyonfield
"#,
        );

        let manifest = load_cloud_apps_manifest(&path).unwrap();
        let error = select_app_entries(&manifest, Some("fieldadmin"))
            .unwrap_err()
            .to_string();

        assert!(error.contains("multiple CloudApps documents contain app fieldadmin"));
    }
}

pub(crate) fn plan_env_vars(entry: &Value, environment: &str) -> Result<EnvPlan> {
    let mut plan = EnvPlan::default();

    if let Some(env_vars) = entry.get("envVars") {
        let env_vars = env_vars
            .as_array()
            .ok_or_else(|| anyhow!("envVars must be an array"))?;
        for env in env_vars {
            let key = env
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("env var entry is missing name"))?;
            let target = env
                .get("target")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| default_env_target(environment).to_string());
            let env_type = env.get("type").and_then(Value::as_str).unwrap_or("plain");
            let value = env.get("value").and_then(Value::as_str);
            let value_from = env.get("valueFrom");
            if env_type == "credential" || value_from.is_some() {
                if value.is_some() {
                    return Err(anyhow!("credential env var {key} must use valueFrom; literal values are not allowed"));
                }
                let value_from = value_from
                    .ok_or_else(|| anyhow!("credential env var {key} is missing valueFrom"))?;
                if let Some(secret) = extract_secret_ref(value_from)? {
                    plan.secret_refs.push(SecretEnvRef {
                        key: secret,
                        target,
                    });
                    continue;
                }
                if let Some(source) = server_managed_credential_source(value_from) {
                    plan.server_managed_credentials
                        .push(ServerManagedCredentialRef {
                            key: key.to_string(),
                            target,
                            source,
                        });
                    continue;
                }
                return Err(anyhow!(
                    "credential env var {key} supports valueFrom.secret, valueFrom.databaseRef, valueFrom.oauth2ClientRef, or valueFrom.storageRef"
                ));
            } else {
                plan.plain.push(SetEnvVarEntry {
                    key: key.to_string(),
                    value: value
                        .ok_or_else(|| anyhow!("plain env var {key} must define value"))?
                        .to_string(),
                    target: Some(target),
                    branch: None,
                    is_secret: Some(false),
                });
            }
        }
    }

    if let Some(sentry) = plan_sentry_integration(entry)? {
        plan.sentry_integrations.push(sentry);
    }

    Ok(plan)
}

fn plan_sentry_integration(entry: &Value) -> Result<Option<SentryIntegrationPlan>> {
    let Some(project) = entry
        .get("integrations")
        .and_then(|integrations| integrations.get("sentry"))
        .and_then(|sentry| sentry.get("project"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|project| !project.is_empty())
    else {
        return Ok(None);
    };

    let explicit_env_vars = entry
        .get("envVars")
        .map(|env_vars| {
            env_vars
                .as_array()
                .ok_or_else(|| anyhow!("envVars must be an array"))
        })
        .transpose()?
        .into_iter()
        .flatten()
        .filter_map(|env| env.get("name").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();

    let env_vars = ["NEXT_PUBLIC_SENTRY_DSN", "SENTRY_DSN"]
        .into_iter()
        .filter(|name| !explicit_env_vars.contains(*name))
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    Ok(Some(SentryIntegrationPlan {
        project: project.to_string(),
        provider: sentry_provider_name(project),
        env_vars,
    }))
}

pub(super) fn sentry_provider_name(project: &str) -> String {
    let normalized = project
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    format!("sentry_{normalized}")
}

async fn validate_sentry_integration(api: &ApiClient) -> Result<()> {
    let integrations: ListIntegrationsResponse = api.get("/v1/integrations").await?;
    let Some(sentry) = integrations
        .integrations
        .iter()
        .find(|integration| integration.provider.eq_ignore_ascii_case("sentry"))
    else {
        return Err(anyhow!(
            "Sentry integration is not available for this tenant. Enable the Sentry integration before running tachyon compute apps apply."
        ));
    };
    if !sentry.is_enabled {
        return Err(anyhow!(
            "Sentry integration is disabled for this tenant. Enable it before running tachyon compute apps apply."
        ));
    }
    if sentry.requires_setup {
        return Err(anyhow!(
            "Sentry integration requires setup. Configure the Sentry integration provider secrets before running tachyon compute apps apply."
        ));
    }

    let connections: ListConnectionsResponse = api.get("/v1/integrations/connections").await?;
    let active_connection = connections.connections.iter().any(|connection| {
        connection.provider.eq_ignore_ascii_case("sentry")
            && connection.status.eq_ignore_ascii_case("active")
    });
    if !active_connection {
        return Err(anyhow!(
            "Sentry integration is enabled but has no active connection for this tenant. Run tachyon iac integrations get sentry and connect Sentry first."
        ));
    }

    Ok(())
}

fn default_env_target(environment: &str) -> &'static str {
    match environment {
        "production" | "prod" => "production",
        "preview" => "preview",
        _ => "all",
    }
}

fn extract_secret_ref(value_from: &Value) -> Result<Option<String>> {
    let Some(secret) = value_from.get("secret") else {
        return Ok(None);
    };
    let path = if let Some(path) = secret.as_str() {
        path
    } else if let Some(path) = secret.get("path").and_then(Value::as_str) {
        if secret.get("field").is_some() {
            return Err(anyhow!("valueFrom.secret.field is not supported"));
        }
        path
    } else {
        return Err(anyhow!(
            "valueFrom.secret must be a key string or object with path"
        ));
    };
    if path.is_empty() {
        return Err(anyhow!("valueFrom.secret must reference a single env key"));
    }
    Ok(Some(path.to_string()))
}

fn server_managed_credential_source(value_from: &Value) -> Option<ServerManagedCredentialSource> {
    if value_from.get("databaseRef").is_some() {
        return Some(ServerManagedCredentialSource::Database);
    }
    if value_from.get("oauth2ClientRef").is_some() {
        return Some(ServerManagedCredentialSource::OAuth2Client);
    }
    if value_from.get("storageRef").is_some() {
        return Some(ServerManagedCredentialSource::Storage);
    }
    None
}

pub(super) fn cloud_app_manifest_for_iac(entry: &Value, tenant_id: &str) -> Result<Value> {
    let name = entry
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("app entry is missing name"))?;
    let mut spec = entry.clone();
    spec.as_object_mut()
        .ok_or_else(|| anyhow!("CloudApps app entry must be an object"))?
        .remove("name");
    spec.as_object_mut()
        .ok_or_else(|| anyhow!("CloudApps app entry must be an object"))?
        .remove(DOCUMENT_TENANT_ID_KEY);
    Ok(json!({
        "apiVersion": "apps.tachy.one/v1alpha",
        "kind": "CloudApp",
        "metadata": {
            "tenantId": tenant_id,
            "name": name,
        },
        "spec": spec,
    }))
}

async fn graphql_request(api: &ApiClient, body: Value) -> Result<Value> {
    let url = format!("{}/v1/graphql", api.base_url);
    let response = api.client.post(url).json(&body).send().await?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await?;
        return Err(anyhow!(
            "graphql request failed: status={status}, body={body}"
        ));
    }
    let payload: Value = response.json().await?;
    if let Some(errors) = payload.get("errors") {
        return Err(anyhow!("graphql error: {errors}"));
    }
    payload
        .get("data")
        .cloned()
        .ok_or_else(|| anyhow!("missing data in graphql response"))
}

async fn save_compute_cloud_app_manifest(api: &ApiClient, manifest: &Value) -> Result<()> {
    let tenant_id = manifest
        .get("metadata")
        .and_then(|metadata| metadata.get("tenantId"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("CloudApp manifest is missing metadata.tenantId"))?;
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

async fn apply_compute_cloud_app_manifest(api: &ApiClient, manifest: &Value) -> Result<()> {
    save_compute_cloud_app_manifest(api, manifest).await?;
    let name = manifest
        .get("metadata")
        .and_then(|metadata| metadata.get("name"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("CloudApp manifest is missing metadata.name"))?;
    let body = json!({
        "query": r#"
          mutation ApplyManifest($input: ApplyManifestInput!) {
            applyManifest(input: $input) { success }
          }
        "#,
        "variables": {
            "input": {
                "kind": "CloudApp",
                "name": name,
                "dryRun": false,
            }
        }
    });
    graphql_request(api, body).await?;
    Ok(())
}

pub(crate) async fn run_apps_sync_secrets(
    api: &ApiClient,
    file: &Path,
    selected_app: Option<&str>,
    environment: &str,
    dry_run: bool,
) -> Result<()> {
    if environment.trim().is_empty() {
        return Err(anyhow!("--environment must not be empty"));
    }

    let manifest = load_cloud_apps_manifest(file)?;
    let entry = select_single_app_entry(&manifest, selected_app)?;
    let app_name = entry
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("app entry is missing name"))?;
    let request = build_sync_secrets_request(&entry, environment)?;

    println!("Manifest:    {}", file.display());
    println!("App:         {app_name}");
    println!("Environment: {environment}");
    println!("Mode:        {}", if dry_run { "dry-run" } else { "sync" });

    if request.refs.is_empty() {
        println!("Secret refs: <none>");
        println!("No secret references found; nothing to sync.");
        return Ok(());
    }

    println!(
        "Secret refs: {}",
        render_sync_secret_refs(&request.refs).join(", ")
    );
    if dry_run {
        println!("No API request sent.");
        return Ok(());
    }

    let live: ListAppsResponse = api.get("/v1/compute/apps").await?;
    let app = live
        .apps
        .iter()
        .find(|app| app.name == app_name)
        .ok_or_else(|| {
            anyhow!(
                "Cloud App {app_name} was not found in the current tenant; run `tachyon compute apps apply` first"
            )
        })?;
    // Server dependency: Cloud Apps API owns SSoT reads and platform writes for
    // this endpoint. The CLI sends only manifest refs; it never resolves values.
    let response: SyncSecretsResponse = api
        .post(
            &format!("/v1/compute/apps/{}/secrets/sync", app.id),
            &request,
        )
        .await?;

    println!("Synced {} secret reference(s).", response.synced);
    if !response.skipped.is_empty() {
        println!("Skipped: {}", response.skipped.join(", "));
    }
    Ok(())
}

pub(super) fn build_sync_secrets_request(
    entry: &Value,
    environment: &str,
) -> Result<SyncSecretsRequest> {
    let app_name = entry
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("app entry is missing name"))?
        .to_string();
    let _ = plan_env_vars(entry, environment)?;
    let refs = collect_sync_secret_refs(entry, environment)?;
    Ok(SyncSecretsRequest {
        app_name,
        environment: environment.to_string(),
        manifest_kind: "CloudApp".to_string(),
        refs,
    })
}

fn collect_sync_secret_refs(entry: &Value, environment: &str) -> Result<Vec<SyncSecretRef>> {
    let Some(env_vars) = entry.get("envVars") else {
        return Ok(Vec::new());
    };
    let env_vars = env_vars
        .as_array()
        .ok_or_else(|| anyhow!("envVars must be an array"))?;
    let mut refs = Vec::new();
    for env in env_vars {
        let key = env
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("env var entry is missing name"))?;
        let target = env
            .get("target")
            .and_then(Value::as_str)
            .unwrap_or_else(|| default_env_target(environment))
            .to_string();
        let Some(value_from) = env.get("valueFrom") else {
            continue;
        };
        if let Some(source_ref) = value_from.get("secret") {
            refs.push(SyncSecretRef {
                key: key.to_string(),
                target,
                source: "secretRef".to_string(),
                source_ref: source_ref.clone(),
            });
            continue;
        }
        for source in ["oauth2ClientRef", "databaseRef", "storageRef"] {
            if let Some(source_ref) = value_from.get(source) {
                refs.push(SyncSecretRef {
                    key: key.to_string(),
                    target,
                    source: source.to_string(),
                    source_ref: source_ref.clone(),
                });
                break;
            }
        }
    }
    refs.sort_by(|a, b| {
        a.key
            .cmp(&b.key)
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.source.cmp(&b.source))
    });
    Ok(refs)
}

pub(super) fn render_sync_secret_refs(refs: &[SyncSecretRef]) -> Vec<String> {
    refs.iter()
        .map(|reference| {
            format!(
                "{}({}; {})",
                reference.key, reference.target, reference.source
            )
        })
        .collect()
}

async fn apply_env_plan(
    api: &ApiClient,
    app_id: &str,
    plan: &EnvPlan,
    dry_run: bool,
) -> Result<(Vec<String>, Vec<String>)> {
    let changed = plan
        .plain
        .iter()
        .map(|entry| {
            format!(
                "{}({})",
                entry.key,
                entry.target.as_deref().unwrap_or("all")
            )
        })
        .collect::<Vec<_>>();
    if !plan.plain.is_empty() && !dry_run && app_id != "<new app>" {
        let req = SetEnvVarsRequest {
            env_vars: plan.plain.clone(),
        };
        let _: ListEnvVarsResponse = api
            .put(&format!("/v1/compute/apps/{app_id}/env"), &req)
            .await?;
    }

    let mut missing = Vec::new();
    if !plan.secret_refs.is_empty() && !dry_run && app_id != "<new app>" {
        let resp: ListEnvVarsResponse = api
            .get(&format!("/v1/compute/apps/{app_id}/env"))
            .await
            .unwrap_or(ListEnvVarsResponse { env_vars: vec![] });
        let current = resp
            .env_vars
            .into_iter()
            .filter(|var| var.is_secret.unwrap_or(false))
            .map(|var| (var.key, var.target.unwrap_or_else(|| "all".to_string())))
            .collect::<BTreeSet<_>>();
        for secret in &plan.secret_refs {
            if !current.contains(&(secret.key.clone(), secret.target.clone()))
                && !current.contains(&(secret.key.clone(), "all".to_string()))
            {
                missing.push(format!("{}({})", secret.key, secret.target));
            }
        }
    } else {
        missing.extend(
            plan.secret_refs
                .iter()
                .map(|secret| format!("{}({})", secret.key, secret.target)),
        );
    }
    Ok((changed, missing))
}

pub(super) fn run_apps_feedback(tenant_id: &str, app_id: &str, args: &FeedbackArgs) -> Result<()> {
    let payload = build_feedback_payload(tenant_id, app_id, args)?;
    if args.json {
        print_json(&payload)?;
    } else {
        println!("{}", render_feedback_markdown(&payload));
    }
    Ok(())
}

pub(super) fn build_feedback_payload(
    tenant_id: &str,
    app_id: &str,
    args: &FeedbackArgs,
) -> Result<FeedbackPayload> {
    let metadata = parse_feedback_metadata(&args.metadata)?;
    Ok(FeedbackPayload {
        app_id: app_id.to_string(),
        operator_id: tenant_id.to_string(),
        kind: args.kind,
        severity: args.severity,
        message: args.message.clone(),
        url: args.url.clone(),
        build_id: args.build_id.clone(),
        deployment_id: args.deployment_id.clone(),
        contact: args.contact.clone(),
        metadata,
        created_at: Utc::now().to_rfc3339(),
    })
}

pub(super) fn parse_feedback_metadata(entries: &[String]) -> Result<BTreeMap<String, String>> {
    let mut metadata = BTreeMap::new();
    for entry in entries {
        let (key, value) = entry
            .split_once('=')
            .ok_or_else(|| anyhow!("metadata must be KEY=VALUE, got `{entry}`"))?;
        let key = key.trim();
        if key.is_empty() {
            return Err(anyhow!("metadata key must not be empty"));
        }
        if is_secret_like_key(key) {
            return Err(anyhow!(
                "metadata key `{key}` looks secret-like; do not pass secret values to feedback"
            ));
        }
        metadata.insert(key.to_string(), value.trim().to_string());
    }
    Ok(metadata)
}

fn is_secret_like_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '.'], "_");
    [
        "secret",
        "token",
        "password",
        "passwd",
        "api_key",
        "apikey",
        "private_key",
        "credential",
        "authorization",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

pub(super) fn render_feedback_markdown(payload: &FeedbackPayload) -> String {
    let mut lines = vec![
        "# Cloud App Feedback".to_string(),
        String::new(),
        format!("- App ID: {}", payload.app_id),
        format!("- Operator ID: {}", payload.operator_id),
        format!("- Kind: {}", payload.kind),
        format!("- Severity: {}", payload.severity),
        format!("- Created At: {}", payload.created_at),
    ];

    if let Some(url) = &payload.url {
        lines.push(format!("- URL: {url}"));
    }
    if let Some(build_id) = &payload.build_id {
        lines.push(format!("- Build ID: {build_id}"));
    }
    if let Some(deployment_id) = &payload.deployment_id {
        lines.push(format!("- Deployment ID: {deployment_id}"));
    }
    if let Some(contact) = &payload.contact {
        lines.push(format!("- Contact: {contact}"));
    }
    if !payload.metadata.is_empty() {
        lines.push("- Metadata:".to_string());
        for (key, value) in &payload.metadata {
            lines.push(format!("  - {key}: {value}"));
        }
    }

    lines.extend([
        String::new(),
        "## Message".to_string(),
        String::new(),
        payload.message.clone(),
    ]);

    lines.join("\n")
}
