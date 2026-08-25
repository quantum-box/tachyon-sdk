use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tachyon_sdk::apis::auth_policies_api;
use tachyon_sdk::apis::configuration::Configuration;
use tachyon_sdk::models::{PolicyActionPatternRequest, PolicyActionRequest, UpdatePolicyRequest};

use crate::client::{print_json, ApiClient, AuthDiagnostics};

// ---- CLI args ----

#[derive(Debug, Clone, Args)]
pub struct ManifestArgs {
    #[command(subcommand)]
    pub command: ManifestCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ManifestCommand {
    /// Normalize manifest YAML formatting
    Fmt {
        /// Exit non-zero if files would change (for CI)
        #[arg(long)]
        check: bool,
        /// Manifest file; defaults to auto-discovery
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,
    },
    /// Validate manifest schema (no API calls)
    Validate {
        /// Manifest file; defaults to auto-discovery
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,
    },
    /// Show diff between desired state and current API state (read-only)
    Plan {
        /// Manifest file; defaults to auto-discovery
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,
        /// Include resources absent from manifest that would be pruned
        #[arg(long)]
        prune: bool,
        /// Register definitions with no owning tenant so every tenant
        /// resolves them. A manifest that states `global` wins over this.
        #[arg(long)]
        global: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Apply manifest to Tachyon API (register / update)
    Apply {
        /// Manifest file; defaults to auto-discovery
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,
        /// Delete definitions the manifest no longer declares, limited to
        /// the contexts this manifest owns
        #[arg(long)]
        prune: bool,
        /// Register definitions with no owning tenant so every tenant
        /// resolves them. A manifest that states `global` wins over this.
        #[arg(long)]
        global: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

// ---- Manifest schema ----

/// Top-level auth manifest.  Can appear embedded inside tachyon.yml under
/// `auth.manifest`, or as a standalone `.tachyon/manifests/**/*.yml` file.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct AuthManifest {
    #[serde(default)]
    pub actions: Vec<ActionSpec>,
    #[serde(default)]
    pub policies: Vec<PolicySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ActionSpec {
    pub context: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_pattern: Option<String>,
    /// Publish to every descendant of the owning platform. Required when a
    /// shared policy in the same manifest names this action.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub shared: bool,
    /// Register with no owning platform so every tenant resolves it.
    ///
    /// `Option` rather than `bool` so an explicit `global: false` can be told
    /// apart from an absent field, which is what makes the manifest able to
    /// override `--global`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global: Option<bool>,
    /// Actions this action depends on within a call context
    /// (full names, e.g. "field:ListCustomers"). Applied declaratively
    /// through PUT /v1/auth/action-dependencies: the server validates
    /// ownership, cycles, and the system-action allowlist.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
}

impl ActionSpec {
    pub fn full_name(&self) -> String {
        format!("{}:{}", self.context, self.name)
    }
}

#[derive(Debug, Serialize)]
struct ActionDependencyEdgeReq {
    action: String,
    #[serde(rename = "dependsOn")]
    depends_on: String,
}

#[derive(Debug, Serialize)]
struct RegisterActionDependenciesRequest {
    edges: Vec<ActionDependencyEdgeReq>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterActionDependenciesResponse {
    added_count: usize,
    removed_count: usize,
    unchanged_count: usize,
}

/// Outcome of the declarative dependency edge apply.
#[derive(Debug, Clone, Serialize)]
pub struct DependencyApplyReport {
    /// Distinct edges the manifest declares.
    pub declared: usize,
    pub added: usize,
    pub removed: usize,
    pub unchanged: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// How a declaration resolves against the `--global` flag.
///
/// The manifest is the source of truth: a spec that states `global` keeps its
/// value even when the flag says otherwise, so applying the same file twice
/// with different flags cannot change what is published. The flag only fills
/// in specs that stay silent.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalScope {
    pub effective: bool,
    /// Set when the manifest and the flag actively disagree. Callers surface
    /// this so a `--global` run cannot look like it did nothing.
    pub conflict: Option<String>,
}

pub fn resolve_global(declared: Option<bool>, flag: bool, label: &str) -> GlobalScope {
    match declared {
        Some(declared) if declared != flag && flag => GlobalScope {
            effective: declared,
            conflict: Some(format!(
                "{label}: manifest declares global: {declared}, ignoring --global"
            )),
        },
        Some(declared) => GlobalScope {
            effective: declared,
            conflict: None,
        },
        None => GlobalScope {
            effective: flag,
            conflict: None,
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct PolicySpec {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Register with no owning tenant so every tenant can see and grant it.
    ///
    /// `Option` rather than `bool` so an explicit `global: false` can be told
    /// apart from an absent field, which is what makes the manifest able to
    /// override `--global`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global: Option<bool>,
    /// Publish to every descendant of the owning tenant. Applying this from
    /// the root tenant is how a definition becomes usable everywhere.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub shared: bool,
    #[serde(default)]
    pub actions: Vec<PolicyActionSpec>,
    #[serde(default)]
    pub action_patterns: Vec<PolicyActionPatternSpec>,
}

impl PolicySpec {
    fn scope_label(&self) -> String {
        self.scope_label_with(self.global == Some(true))
    }

    /// Same as [`Self::scope_label`] but for a scope already resolved against
    /// the `--global` flag, so plan output matches what apply will do.
    fn scope_label_with(&self, global: bool) -> String {
        let base = if global {
            "global".to_string()
        } else if let Some(namespace) = &self.namespace {
            format!("namespace/{namespace}")
        } else {
            "caller-tenant".to_string()
        };
        if self.shared {
            format!("{base}+shared")
        } else {
            base
        }
    }

    fn target_tenant_id<'a>(&'a self, default_tenant_id: &'a str) -> &'a str {
        self.namespace.as_deref().unwrap_or(default_tenant_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct PolicyActionSpec {
    pub action: String,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct PolicyActionPatternSpec {
    pub context_pattern: String,
    pub name_pattern: String,
    pub effect: String,
}

// ---- API response types ----

#[derive(Debug, Deserialize, Serialize)]
struct ActionResponse {
    #[serde(default)]
    id: Option<String>,
    /// Owning platform. `None` marks a system or global action.
    #[serde(rename = "platformId", alias = "platform_id", default)]
    platform_id: Option<String>,
    /// Tenant that registered a global action. Distinguishes one this
    /// manifest created from a system action nobody may touch.
    #[serde(rename = "ownerTenantId", alias = "owner_tenant_id", default)]
    owner_tenant_id: Option<String>,
    context: String,
    name: String,
    #[serde(rename = "fullName", alias = "full_name", default)]
    full_name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(rename = "resourcePattern", alias = "resource_pattern", default)]
    resource_pattern: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ActionListResponse {
    actions: Vec<ActionResponse>,
}

#[derive(Debug, Clone, Deserialize)]
struct PolicyResponse {
    id: String,
    name: String,
    #[serde(rename = "isSystem", alias = "is_system", default)]
    is_system: bool,
    #[serde(rename = "tenantId", alias = "tenant_id", default)]
    tenant_id: Option<String>,
    /// Tenant that registered a global policy. Distinguishes one this
    /// manifest created from a global policy owned by someone else.
    #[serde(rename = "ownerTenantId", alias = "owner_tenant_id", default)]
    owner_tenant_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PolicyListResponse {
    policies: Vec<PolicyResponse>,
}

/// Contexts the manifest declares, used to bound what `--prune` may delete.
///
/// Pruning anything outside these contexts would let a small manifest wipe
/// definitions owned by other apps in the same tenant.
fn manifest_contexts(manifest: &AuthManifest) -> HashSet<String> {
    let mut contexts: HashSet<String> = manifest
        .actions
        .iter()
        .map(|action| action.context.clone())
        .collect();
    for policy in &manifest.policies {
        if let Some((context, _)) = policy.name.split_once(':') {
            contexts.insert(context.to_string());
        }
    }
    contexts
}

/// Live actions this manifest owns but no longer declares.
fn actions_to_prune<'a>(
    live: &'a [ActionResponse],
    manifest: &AuthManifest,
    tenant_id: &str,
) -> Vec<&'a ActionResponse> {
    let contexts = manifest_contexts(manifest);
    let desired: HashSet<String> = manifest.actions.iter().map(|a| a.full_name()).collect();
    live.iter()
        .filter(|action| {
            // Platform-scoped actions belong to the caller's platform. A
            // global action is only ours if we registered it; anything else
            // without a platform is a system action nobody may delete.
            let owned = action.platform_id.is_some()
                || action.owner_tenant_id.as_deref() == Some(tenant_id);
            owned && contexts.contains(&action.context) && !desired.contains(&action.full_name)
        })
        .collect()
}

/// Live policies owned by the applied tenant that the manifest no longer
/// declares. Only `context:Name` policies inside a declared context qualify,
/// so hand-made tenant policies survive.
fn policies_to_prune<'a>(
    live: &'a [PolicyResponse],
    manifest: &AuthManifest,
    tenant_id: &str,
) -> Vec<&'a PolicyResponse> {
    let contexts = manifest_contexts(manifest);
    let desired: HashSet<&str> = manifest.policies.iter().map(|p| p.name.as_str()).collect();
    live.iter()
        .filter(|policy| {
            // Either a policy this tenant owns, or a global one it
            // registered. A global policy from another tenant stays put.
            let owned = policy.tenant_id.as_deref() == Some(tenant_id)
                || (policy.tenant_id.is_none()
                    && policy.owner_tenant_id.as_deref() == Some(tenant_id));
            if policy.is_system || !owned {
                return false;
            }
            let Some((context, _)) = policy.name.split_once(':') else {
                return false;
            };
            contexts.contains(context) && !desired.contains(policy.name.as_str())
        })
        .collect()
}

/// Find the policy in the exact scope targeted by a manifest declaration.
///
/// Global names are visible across tenants, so a matching name and null
/// tenant alone are not sufficient: only the tenant that registered the
/// global policy may update it.
fn existing_policy_for_spec<'a>(
    live: &'a [PolicyResponse],
    spec: &PolicySpec,
    global: bool,
    default_tenant_id: &str,
) -> Option<&'a PolicyResponse> {
    let target_tenant_id = (!global).then(|| spec.target_tenant_id(default_tenant_id));
    live.iter().find(|policy| {
        policy.name == spec.name
            && !policy.is_system
            && if global {
                policy.tenant_id.is_none()
                    && policy.owner_tenant_id.as_deref() == Some(default_tenant_id)
            } else {
                policy.tenant_id.as_deref() == target_tenant_id
            }
    })
}

#[derive(Debug, Serialize)]
struct RegisterActionRequest<'a> {
    context: &'a str,
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    // The server deserializes with rename_all = "camelCase"; a snake_case
    // key is silently ignored and the pattern never converges on apply.
    #[serde(rename = "resourcePattern", skip_serializing_if = "Option::is_none")]
    resource_pattern: Option<&'a str>,
    #[serde(rename = "sharedWithDescendants")]
    shared_with_descendants: bool,
    global: bool,
}

#[derive(Debug, Serialize)]
struct PolicyActionReq<'a> {
    #[serde(rename = "actionFullName")]
    action_full_name: &'a str,
    effect: &'a str,
}

#[derive(Debug, Serialize)]
struct PolicyActionPatternReq<'a> {
    #[serde(rename = "contextPattern")]
    context_pattern: &'a str,
    #[serde(rename = "namePattern")]
    name_pattern: &'a str,
    effect: &'a str,
}

#[derive(Debug, Serialize)]
struct RegisterPolicyRequest<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(rename = "isSystem")]
    is_system: bool,
    global: bool,
    #[serde(rename = "tenantId", skip_serializing_if = "Option::is_none")]
    tenant_id: Option<&'a str>,
    #[serde(rename = "sharedWithDescendants")]
    shared_with_descendants: bool,
    actions: Vec<PolicyActionReq<'a>>,
    #[serde(rename = "actionPatterns")]
    action_patterns: Vec<PolicyActionPatternReq<'a>>,
}

// ---- Plan report ----

#[derive(Debug, Clone, Serialize)]
pub struct PlanReport {
    pub actions: Vec<ActionPlanItem>,
    pub policies: Vec<PolicyPlanItem>,
    /// True when the report includes resources that `apply --prune` deletes.
    pub prune: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionPlanItem {
    pub full_name: String,
    pub change: ChangeKind,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyPlanItem {
    pub name: String,
    pub scope: String,
    pub change: ChangeKind,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Create,
    Update,
    Unchanged,
    Unknown,
    Prune,
}

// ---- Apply result ----

#[derive(Debug, Clone, Serialize)]
pub struct ApplyResult {
    pub actions: Vec<ActionApplyItem>,
    pub policies: Vec<PolicyApplyItem>,
    /// Present only when the manifest declares `requires` edges. A manifest
    /// without `requires` never touches the platform's edge set, so other
    /// manifests of the same platform stay unaffected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<DependencyApplyReport>,
    /// Resources deleted because the manifest no longer declares them.
    pub pruned: usize,
    /// Places where `--global` was overruled by the manifest.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionApplyItem {
    pub full_name: String,
    pub outcome: ApplyOutcome,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyApplyItem {
    pub name: String,
    pub scope: String,
    pub outcome: ApplyOutcome,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApplyOutcome {
    Created,
    Updated,
    Skipped,
    Pruned,
    Error(String),
}

// ---- Manifest discovery ----

/// A loaded manifest along with its source path.
#[derive(Debug)]
pub struct LoadedManifest {
    pub path: PathBuf,
    pub manifest: AuthManifest,
}

/// Wrapper to parse tachyon.yml and extract auth.manifest if present.
#[derive(Debug, Default, Deserialize)]
struct TachyonYmlAuth {
    #[serde(default)]
    manifest: Option<serde_yaml::Value>,
}

#[derive(Debug, Default, Deserialize)]
struct TachyonYmlRoot {
    #[serde(default)]
    auth: Option<TachyonYmlAuth>,
}

pub fn discover_manifests(explicit_file: Option<&Path>, cwd: &Path) -> Result<Vec<LoadedManifest>> {
    if let Some(path) = explicit_file {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            cwd.join(path)
        };
        return load_single_manifest(&path).map(|m| vec![m]);
    }

    let mut found = Vec::new();

    // 1. tachyon.yml – look for auth.manifest section
    let tachyon_yml = find_tachyon_yml(cwd);
    if let Some(yml_path) = tachyon_yml {
        let raw = std::fs::read_to_string(&yml_path)
            .with_context(|| format!("read {}", yml_path.display()))?;
        let root = parse_tachyon_yml_root(&raw, &yml_path)?;
        if let Some(auth) = root.auth {
            if let Some(manifest_value) = auth.manifest {
                let manifest = parse_manifest_value(manifest_value)
                    .with_context(|| format!("parse auth.manifest in {}", yml_path.display()))?;
                if !manifest.actions.is_empty() || !manifest.policies.is_empty() {
                    found.push(LoadedManifest {
                        path: yml_path,
                        manifest,
                    });
                }
            }
        }
    }

    // 2. .tachyon/manifests/**/*.yml
    let manifests_dir = cwd.join(".tachyon").join("manifests");
    if manifests_dir.is_dir() {
        let mut paths = collect_yaml_files(&manifests_dir)?;
        paths.sort();
        for path in paths {
            found.push(load_single_manifest(&path)?);
        }
    }

    Ok(found)
}

fn find_tachyon_yml(cwd: &Path) -> Option<PathBuf> {
    let mut dir = cwd;
    loop {
        let candidate = dir.join("tachyon.yml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if dir.join(".git").exists() {
            return None;
        }
        dir = dir.parent()?;
    }
}

fn collect_yaml_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("read dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            result.extend(collect_yaml_files(&path)?);
        } else if let Some(ext) = path.extension() {
            if ext == "yml" || ext == "yaml" {
                result.push(path);
            }
        }
    }
    Ok(result)
}

fn load_single_manifest(path: &Path) -> Result<LoadedManifest> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read manifest {}", path.display()))?;

    // If the file has an `auth.manifest` key (tachyon.yml style), use that section.
    // Otherwise fall back to parsing the whole file as a flat AuthManifest.
    if let Ok(root) = parse_tachyon_yml_root(&raw, path) {
        if let Some(auth) = root.auth {
            if let Some(manifest) = auth.manifest {
                return Ok(LoadedManifest {
                    path: path.to_path_buf(),
                    manifest: parse_manifest_value(manifest)
                        .with_context(|| format!("parse auth.manifest in {}", path.display()))?,
                });
            }
        }
    }

    let value: serde_yaml::Value =
        serde_yaml::from_str(&raw).with_context(|| format!("parse manifest {}", path.display()))?;
    let manifest = parse_manifest_value(value)
        .with_context(|| format!("parse manifest {}", path.display()))?;
    Ok(LoadedManifest {
        path: path.to_path_buf(),
        manifest,
    })
}

fn parse_tachyon_yml_root(raw: &str, path: &Path) -> Result<TachyonYmlRoot> {
    let first_doc = serde_yaml::Deserializer::from_str(raw)
        .next()
        .with_context(|| format!("empty {}", path.display()))?;
    TachyonYmlRoot::deserialize(first_doc).with_context(|| format!("parse {}", path.display()))
}

pub(crate) fn parse_manifest_document_value(value: serde_yaml::Value) -> Result<AuthManifest> {
    if let Ok(root) = serde_yaml::from_value::<TachyonYmlRoot>(value.clone()) {
        if let Some(auth) = root.auth {
            if let Some(manifest) = auth.manifest {
                return parse_manifest_value(manifest);
            }
        }
    }
    parse_manifest_value(value)
}

fn parse_manifest_value(value: serde_yaml::Value) -> Result<AuthManifest> {
    if let Some(mapping) = value.as_mapping() {
        if mapping.contains_key("actions") || mapping.contains_key("policies") {
            return Ok(serde_yaml::from_value(value)?);
        }
        if mapping.contains_key("apiVersion") && mapping.contains_key("kind") {
            return parse_k8s_documents(vec![value]);
        }
        if let Some(items) = mapping.get("items") {
            if let Some(sequence) = items.as_sequence() {
                return parse_k8s_documents(sequence.clone());
            }
        }
    }

    if let Some(sequence) = value.as_sequence() {
        return parse_k8s_documents(sequence.clone());
    }

    Err(anyhow!(
        "auth manifest must be flat actions/policies or k8s style document(s)"
    ))
}

fn parse_k8s_documents(documents: Vec<serde_yaml::Value>) -> Result<AuthManifest> {
    let mut manifest = AuthManifest::default();
    for document in documents {
        let doc: K8sAuthDocument = serde_yaml::from_value(document)?;
        if doc.api_version != "auth.tachyon.io/v1" {
            return Err(anyhow!(
                "unsupported auth manifest apiVersion '{}'",
                doc.api_version
            ));
        }
        match doc.kind.as_str() {
            "ActionSet" => {
                let spec: ActionSetSpec = serde_yaml::from_value(doc.spec)?;
                // `shared` on the set is a shorthand for every action in it.
                manifest
                    .actions
                    .extend(spec.actions.into_iter().map(|mut action| {
                        action.shared = action.shared || spec.shared;
                        action.global = action.global.or(spec.global);
                        action
                    }));
            }
            "Policy" => {
                let spec: K8sPolicySpec = serde_yaml::from_value(doc.spec)?;
                // A global policy has no owning tenant, so it is the one kind
                // that may omit the namespace.
                let namespace = match doc.metadata.namespace {
                    Some(namespace) => Some(namespace),
                    None if spec.global == Some(true) => None,
                    None => {
                        return Err(anyhow!(
                        "Policy '{}': metadata.namespace is required unless spec.global is true",
                        doc.metadata.name
                    ))
                    }
                };
                manifest.policies.push(PolicySpec {
                    name: doc.metadata.name,
                    description: spec.description,
                    global: spec.global,
                    shared: spec.shared,
                    namespace,
                    actions: spec.actions,
                    action_patterns: spec.action_patterns,
                });
            }
            other => {
                return Err(anyhow!("unsupported auth manifest kind '{other}'"));
            }
        }
    }
    Ok(manifest)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct K8sAuthDocument {
    api_version: String,
    kind: String,
    metadata: K8sMetadata,
    spec: serde_yaml::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct K8sMetadata {
    name: String,
    #[serde(default)]
    namespace: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ActionSetSpec {
    #[serde(default)]
    shared: bool,
    /// Shorthand applied to every action in the set that does not state its
    /// own `global`.
    #[serde(default)]
    global: Option<bool>,
    #[serde(default)]
    actions: Vec<ActionSpec>,
}

#[derive(Debug, Deserialize)]
struct K8sPolicySpec {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    shared: bool,
    #[serde(default)]
    global: Option<bool>,
    #[serde(default)]
    actions: Vec<PolicyActionSpec>,
    #[serde(default)]
    action_patterns: Vec<PolicyActionPatternSpec>,
}

/// Merge multiple manifests into one, deduplicating by full_name / policy name.
pub fn merge_manifests(manifests: Vec<LoadedManifest>) -> AuthManifest {
    let mut merged = AuthManifest::default();
    let mut seen_actions = std::collections::HashSet::new();
    let mut seen_policies = std::collections::HashSet::new();

    for loaded in manifests {
        for action in loaded.manifest.actions {
            let key = action.full_name();
            if seen_actions.insert(key) {
                merged.actions.push(action);
            }
        }
        for policy in loaded.manifest.policies {
            let key = format!("{}:{}", policy.scope_label(), policy.name);
            if seen_policies.insert(key) {
                merged.policies.push(policy);
            }
        }
    }

    merged
}

// ---- Validate ----

pub fn validate_manifest(manifest: &AuthManifest) -> Result<()> {
    for action in &manifest.actions {
        if action.context.is_empty() {
            return Err(anyhow!("action missing context: {action:?}"));
        }
        if action.name.is_empty() {
            return Err(anyhow!(
                "action missing name in context '{}'",
                action.context
            ));
        }
        for required in &action.requires {
            let well_formed = required
                .split_once(':')
                .is_some_and(|(context, name)| !context.is_empty() && !name.is_empty());
            if !well_formed {
                return Err(anyhow!(
                    "action '{}': requires entry '{}' must be a full                      action name like 'context:Name'",
                    action.full_name(),
                    required
                ));
            }
            if *required == action.full_name() {
                return Err(anyhow!(
                    "action '{}': requires must not declare a self edge",
                    action.full_name()
                ));
            }
        }
    }
    for policy in &manifest.policies {
        if policy.name.is_empty() {
            return Err(anyhow!("policy missing name"));
        }
        if policy.global == Some(true) {
            if policy.shared {
                return Err(anyhow!(
                    "policy '{}': `global` is already visible to every tenant, \
                     so `shared` is not applicable",
                    policy.name
                ));
            }
            if policy.namespace.is_some() {
                return Err(anyhow!(
                    "policy '{}': `global` has no owning tenant, so \
                     `namespace` is not applicable",
                    policy.name
                ));
            }
        }
        for pa in &policy.actions {
            if !pa.action.contains(':') {
                return Err(anyhow!(
                    "policy '{}': action '{}' must be in 'context:Name' format",
                    policy.name,
                    pa.action
                ));
            }
            if pa.effect != "allow" && pa.effect != "deny" {
                return Err(anyhow!(
                    "policy '{}': effect must be 'allow' or 'deny', got '{}'",
                    policy.name,
                    pa.effect
                ));
            }
        }
    }
    Ok(())
}

// ---- Plan ----

pub async fn build_plan(
    api: &ApiClient,
    manifest: &AuthManifest,
    prune: bool,
    tenant_id: &str,
    global: bool,
) -> Result<PlanReport> {
    let live: ActionListResponse = api.get("/v1/auth/actions").await?;
    let live_map: std::collections::HashMap<String, &ActionResponse> = live
        .actions
        .iter()
        .map(|a| (a.full_name.clone(), a))
        .collect();

    let mut action_items = Vec::new();

    for spec in &manifest.actions {
        let full = spec.full_name();
        let change = if let Some(live_action) = live_map.get(&full) {
            // Simple equality check on description and resource_pattern
            if live_action.description == spec.description
                && live_action.resource_pattern == spec.resource_pattern
            {
                ChangeKind::Unchanged
            } else {
                ChangeKind::Update
            }
        } else {
            ChangeKind::Create
        };
        action_items.push(ActionPlanItem {
            full_name: full,
            change,
        });
    }

    // Policies: no per-policy diff is available, so their current state is unknown.
    let mut policy_items: Vec<PolicyPlanItem> = manifest
        .policies
        .iter()
        .map(|p| PolicyPlanItem {
            name: p.name.clone(),
            // Show the scope apply will actually use, flag included.
            scope: p.scope_label_with(resolve_global(p.global, global, &p.name).effective),
            change: ChangeKind::Unknown,
        })
        .collect();

    if prune {
        for action in actions_to_prune(&live.actions, manifest, tenant_id) {
            action_items.push(ActionPlanItem {
                full_name: action.full_name.clone(),
                change: ChangeKind::Prune,
            });
        }

        let live_policies: PolicyListResponse = api.get("/v1/auth/policies").await?;
        for policy in policies_to_prune(&live_policies.policies, manifest, tenant_id) {
            policy_items.push(PolicyPlanItem {
                name: policy.name.clone(),
                scope: format!("namespace/{tenant_id}"),
                change: ChangeKind::Prune,
            });
        }
    }

    Ok(PlanReport {
        actions: action_items,
        policies: policy_items,
        prune,
    })
}

pub(crate) fn print_plan(report: &PlanReport) {
    println!("Actions:");
    if report.actions.is_empty() {
        println!("  (none)");
    }
    for item in &report.actions {
        let symbol = change_symbol(&item.change);
        let note = match item.change {
            ChangeKind::Prune => " [prune: will be deleted]",
            _ => "",
        };
        println!("  {symbol} {}{note}", item.full_name);
    }

    println!();
    println!("Policies:");
    if report.policies.is_empty() {
        println!("  (none)");
    }
    for item in &report.policies {
        let symbol = change_symbol(&item.change);
        let note = policy_plan_note(&item.change);
        println!("  {symbol} {} ({}){note}", item.name, item.scope);
    }

    if report.prune {
        println!();
        println!("Note: --prune only considers definitions inside the contexts this");
        println!("      manifest declares. Resources marked '-' are deleted by apply.");
    }

    let creates = report
        .actions
        .iter()
        .filter(|a| a.change == ChangeKind::Create)
        .count();
    let updates = report
        .actions
        .iter()
        .filter(|a| a.change == ChangeKind::Update)
        .count();
    let unchanged = report
        .actions
        .iter()
        .filter(|a| a.change == ChangeKind::Unchanged)
        .count();
    let policy_creates = report
        .policies
        .iter()
        .filter(|p| p.change == ChangeKind::Create)
        .count();
    let policy_unknown = report
        .policies
        .iter()
        .filter(|p| p.change == ChangeKind::Unknown)
        .count();
    println!();
    println!(
        "Summary: {} action(s) to add, {} to update, {} unchanged, {} policy(ies) to register, {} with current state unknown.",
        creates,
        updates,
        unchanged,
        policy_creates,
        policy_unknown
    );
}

fn change_symbol(change: &ChangeKind) -> &'static str {
    match change {
        ChangeKind::Create => "+",
        ChangeKind::Update => "~",
        ChangeKind::Unchanged => "=",
        ChangeKind::Unknown => "?",
        ChangeKind::Prune => "!",
    }
}

fn policy_plan_note(change: &ChangeKind) -> &'static str {
    match change {
        ChangeKind::Unknown => " [note: current state unknown – no list endpoint]",
        _ => "",
    }
}

// ---- Apply ----

pub async fn apply_manifest(
    api: &ApiClient,
    config: &Configuration,
    manifest: &AuthManifest,
    prune: bool,
    default_tenant_id: &str,
    global: bool,
) -> Result<ApplyResult> {
    validate_manifest(manifest)?;

    let mut action_items = Vec::new();
    let mut policy_items = Vec::new();
    let mut warnings = Vec::new();
    let mut pruned = 0usize;

    for spec in &manifest.actions {
        let scope = resolve_global(spec.global, global, &spec.full_name());
        warnings.extend(scope.conflict);
        let req = RegisterActionRequest {
            context: &spec.context,
            name: &spec.name,
            description: spec.description.as_deref(),
            resource_pattern: spec.resource_pattern.as_deref(),
            shared_with_descendants: spec.shared,
            global: scope.effective,
        };
        let outcome = match api
            .post::<_, serde_json::Value>("/v1/auth/actions", &req)
            .await
        {
            Ok(_) => ApplyOutcome::Created,
            Err(e) => {
                let msg = e.to_string();
                // 409 / duplicate means it already exists – treat as skipped
                if msg.contains("409")
                    || msg.contains("already exists")
                    || msg.contains("duplicate")
                {
                    ApplyOutcome::Skipped
                } else {
                    ApplyOutcome::Error(msg)
                }
            }
        };
        action_items.push(ActionApplyItem {
            full_name: spec.full_name(),
            outcome,
        });
    }

    let live_policies = if manifest.policies.is_empty() {
        None
    } else {
        Some(api.get::<PolicyListResponse>("/v1/auth/policies").await?)
    };
    let sdk_config = policy_sdk_configuration(config, default_tenant_id)?;
    let mut warned_incomplete_policy_reconciliation = false;

    for spec in &manifest.policies {
        let scope = resolve_global(spec.global, global, &spec.name);
        warnings.extend(scope.conflict);
        let target_tenant_id = (!scope.effective).then(|| spec.target_tenant_id(default_tenant_id));
        let existing = live_policies.as_ref().and_then(|response| {
            existing_policy_for_spec(&response.policies, spec, scope.effective, default_tenant_id)
        });
        let req = RegisterPolicyRequest {
            name: &spec.name,
            description: spec.description.as_deref(),
            is_system: false,
            global: scope.effective,
            // A global policy has no owning tenant, so sending one would be
            // rejected as a contradiction.
            tenant_id: target_tenant_id,
            shared_with_descendants: spec.shared,
            actions: spec
                .actions
                .iter()
                .map(|a| PolicyActionReq {
                    action_full_name: &a.action,
                    effect: &a.effect,
                })
                .collect(),
            action_patterns: spec
                .action_patterns
                .iter()
                .map(|p| PolicyActionPatternReq {
                    context_pattern: &p.context_pattern,
                    name_pattern: &p.name_pattern,
                    effect: &p.effect,
                })
                .collect(),
        };
        let outcome = if let Some(existing) = existing {
            // The current API response includes policy metadata, but not the
            // action or action-pattern membership needed to calculate safe
            // removals. Upsert every declared entry and explicitly report the
            // convergence limitation instead of claiming a no-op or silently
            // treating the existing policy as applied.
            if !warned_incomplete_policy_reconciliation {
                warnings.push(
                    "Policy reconciliation is partial: the API does not expose current policy \
                     actions or action patterns. Declared entries are sent for upsert, but stale \
                     entries were not removed and exact manifest convergence was not verified."
                        .to_string(),
                );
                warned_incomplete_policy_reconciliation = true;
            }
            let update = UpdatePolicyRequest {
                actions_to_add: (!spec.actions.is_empty()).then(|| {
                    spec.actions
                        .iter()
                        .map(|action| {
                            PolicyActionRequest::new(action.action.clone(), action.effect.clone())
                        })
                        .collect()
                }),
                actions_to_remove: None,
                action_patterns_to_add: (!spec.action_patterns.is_empty()).then(|| {
                    spec.action_patterns
                        .iter()
                        .map(|pattern| {
                            PolicyActionPatternRequest::new(
                                pattern.context_pattern.clone(),
                                pattern.effect.clone(),
                                pattern.name_pattern.clone(),
                            )
                        })
                        .collect()
                }),
                action_patterns_to_remove: None,
                // Always send the declared description (including null). It
                // gives an otherwise-empty policy a concrete PATCH operation,
                // so Updated is only reported after the API acknowledges it.
                description: Some(spec.description.clone()),
            };
            match auth_policies_api::update_policy(&sdk_config, &existing.id, update).await {
                Ok(_) => ApplyOutcome::Updated,
                Err(error) => ApplyOutcome::Error(error.to_string()),
            }
        } else {
            match api
                .post::<_, serde_json::Value>("/v1/auth/policies", &req)
                .await
            {
                Ok(_) => ApplyOutcome::Created,
                Err(error) => ApplyOutcome::Error(error.to_string()),
            }
        };
        policy_items.push(PolicyApplyItem {
            name: spec.name.clone(),
            scope: spec.scope_label_with(scope.effective),
            outcome,
        });
    }

    // Declarative dependency edges. Only sent when the manifest declares
    // at least one `requires`: the endpoint replaces the platform's whole
    // edge set, so a manifest that says nothing must not clear edges
    // declared elsewhere.
    let mut edges: Vec<ActionDependencyEdgeReq> = Vec::new();
    let mut seen_edges = std::collections::HashSet::new();
    for spec in &manifest.actions {
        for required in &spec.requires {
            if seen_edges.insert((spec.full_name(), required.clone())) {
                edges.push(ActionDependencyEdgeReq {
                    action: spec.full_name(),
                    depends_on: required.clone(),
                });
            }
        }
    }
    let dependencies = if edges.is_empty() {
        None
    } else {
        let declared = edges.len();
        match api
            .put::<_, RegisterActionDependenciesResponse>(
                "/v1/auth/action-dependencies",
                &RegisterActionDependenciesRequest { edges },
            )
            .await
        {
            Ok(response) => Some(DependencyApplyReport {
                declared,
                added: response.added_count,
                removed: response.removed_count,
                unchanged: response.unchanged_count,
                error: None,
            }),
            Err(e) => Some(DependencyApplyReport {
                declared,
                added: 0,
                removed: 0,
                unchanged: 0,
                error: Some(e.to_string()),
            }),
        }
    };

    if prune {
        pruned = prune_absent_resources(
            api,
            manifest,
            default_tenant_id,
            &mut action_items,
            &mut policy_items,
        )
        .await?;
    }

    Ok(ApplyResult {
        actions: action_items,
        policies: policy_items,
        dependencies,
        pruned,
        warnings,
    })
}

/// Build the generated SDK's reqwest 0.12 client with the same authentication
/// context used by the CLI's reqwest 0.13 ApiClient.
fn policy_sdk_configuration(config: &Configuration, tenant_id: &str) -> Result<Configuration> {
    let mut headers = reqwest12::header::HeaderMap::new();
    headers.insert(
        "x-operator-id",
        reqwest12::header::HeaderValue::from_str(tenant_id)?,
    );
    if let Some(token) = &config.bearer_access_token {
        headers.insert(
            reqwest12::header::AUTHORIZATION,
            reqwest12::header::HeaderValue::from_str(&format!("Bearer {token}"))?,
        );
    }
    let client = reqwest12::Client::builder()
        .default_headers(headers)
        .build()
        .context("build SDK client for policy update")?;
    Ok(Configuration {
        client,
        ..config.clone()
    })
}

/// Delete definitions the manifest no longer declares.
///
/// The API has no bulk endpoint, so each resource is deleted individually and
/// recorded in the same report as the applied ones. A policy still attached to
/// a user or service account comes back 409; that is surfaced as an error
/// rather than silently skipped, because a stale grant is the thing prune is
/// meant to remove.
async fn prune_absent_resources(
    api: &ApiClient,
    manifest: &AuthManifest,
    tenant_id: &str,
    action_items: &mut Vec<ActionApplyItem>,
    policy_items: &mut Vec<PolicyApplyItem>,
) -> Result<usize> {
    let mut pruned = 0usize;

    let live_policies: PolicyListResponse = api.get("/v1/auth/policies").await?;
    for policy in policies_to_prune(&live_policies.policies, manifest, tenant_id) {
        let path = format!("/v1/auth/policies/{}", policy.id);
        let outcome = match api.delete(&path).await {
            Ok(()) => {
                pruned += 1;
                ApplyOutcome::Pruned
            }
            Err(e) => ApplyOutcome::Error(e.to_string()),
        };
        policy_items.push(PolicyApplyItem {
            name: policy.name.clone(),
            scope: format!("namespace/{tenant_id}"),
            outcome,
        });
    }

    let live_actions: ActionListResponse = api.get("/v1/auth/actions").await?;
    for action in actions_to_prune(&live_actions.actions, manifest, tenant_id) {
        let Some(id) = action.id.as_deref() else {
            continue;
        };
        let path = format!("/v1/auth/actions/{id}");
        let outcome = match api.delete(&path).await {
            Ok(()) => {
                pruned += 1;
                ApplyOutcome::Pruned
            }
            Err(e) => ApplyOutcome::Error(e.to_string()),
        };
        action_items.push(ActionApplyItem {
            full_name: action.full_name.clone(),
            outcome,
        });
    }

    Ok(pruned)
}

pub(crate) fn print_apply_result(result: &ApplyResult) {
    if !result.warnings.is_empty() {
        println!("Warnings:");
        for warning in &result.warnings {
            println!("  ! {warning}");
        }
        println!();
    }
    println!("Actions:");
    for item in &result.actions {
        let (symbol, note) = match &item.outcome {
            ApplyOutcome::Created => ("+", String::new()),
            ApplyOutcome::Updated => ("~", " (updated)".to_string()),
            ApplyOutcome::Skipped => ("=", " (already exists)".to_string()),
            ApplyOutcome::Pruned => ("-", " (pruned)".to_string()),
            ApplyOutcome::Error(e) => ("!", format!(" ERROR: {e}")),
        };
        println!("  {symbol} {}{note}", item.full_name);
    }
    println!();
    println!("Policies:");
    for item in &result.policies {
        let (symbol, note) = match &item.outcome {
            ApplyOutcome::Created => ("+", String::new()),
            ApplyOutcome::Updated => ("~", " (updated)".to_string()),
            ApplyOutcome::Skipped => ("=", " (already exists)".to_string()),
            ApplyOutcome::Pruned => ("-", " (pruned)".to_string()),
            ApplyOutcome::Error(e) => ("!", format!(" ERROR: {e}")),
        };
        println!("  {symbol} {} ({}){note}", item.name, item.scope);
    }

    let a_created = result
        .actions
        .iter()
        .filter(|a| a.outcome == ApplyOutcome::Created)
        .count();
    let a_skipped = result
        .actions
        .iter()
        .filter(|a| a.outcome == ApplyOutcome::Skipped)
        .count();
    let a_updated = result
        .actions
        .iter()
        .filter(|a| a.outcome == ApplyOutcome::Updated)
        .count();
    let a_errors = result
        .actions
        .iter()
        .filter(|a| matches!(a.outcome, ApplyOutcome::Error(_)))
        .count();
    let p_created = result
        .policies
        .iter()
        .filter(|p| p.outcome == ApplyOutcome::Created)
        .count();
    let p_skipped = result
        .policies
        .iter()
        .filter(|p| p.outcome == ApplyOutcome::Skipped)
        .count();
    let p_updated = result
        .policies
        .iter()
        .filter(|p| p.outcome == ApplyOutcome::Updated)
        .count();
    let p_errors = result
        .policies
        .iter()
        .filter(|p| matches!(p.outcome, ApplyOutcome::Error(_)))
        .count();

    if let Some(dependencies) = &result.dependencies {
        println!();
        println!("Dependency edges:");
        match &dependencies.error {
            Some(error) => println!(
                "  ! {} edge(s) declared, apply failed: {error}",
                dependencies.declared
            ),
            None => println!(
                "  {} edge(s) declared: {} added, {} removed, {} unchanged",
                dependencies.declared,
                dependencies.added,
                dependencies.removed,
                dependencies.unchanged
            ),
        }
    }

    println!();
    println!(
        "Applied: {a_created} action(s) created, {a_updated} updated, {a_skipped} skipped, \
         {a_errors} error(s); {p_created} policy(ies) created, {p_updated} updated, \
         {p_skipped} skipped, {p_errors} error(s)."
    );

    if result.pruned > 0 {
        println!("Pruned: {} resource(s) removed.", result.pruned);
    }
}

// ---- Fmt ----

fn fmt_manifest(manifest: &AuthManifest) -> Result<String> {
    serde_yaml::to_string(manifest).context("serialize manifest to YAML")
}

// ---- Public run entry point ----

pub async fn run(
    args: &ManifestArgs,
    config: &Configuration,
    tenant_id: &str,
    auth_diagnostics: Option<AuthDiagnostics>,
) -> Result<()> {
    let cwd = std::env::current_dir()?;

    match &args.command {
        ManifestCommand::Fmt { check, file } => {
            let manifests = discover_manifests(file.as_deref(), &cwd)?;
            if manifests.is_empty() {
                println!("No auth manifests found.");
                return Ok(());
            }
            let mut needs_change = false;
            for loaded in &manifests {
                let formatted = fmt_manifest(&loaded.manifest)?;
                let original = std::fs::read_to_string(&loaded.path)?;
                if original != formatted {
                    needs_change = true;
                    if *check {
                        println!("Would reformat: {}", loaded.path.display());
                    } else {
                        std::fs::write(&loaded.path, &formatted)?;
                        println!("Reformatted: {}", loaded.path.display());
                    }
                } else {
                    println!("OK: {}", loaded.path.display());
                }
            }
            if *check && needs_change {
                return Err(anyhow!(
                    "Some manifest files are not properly formatted. \
                     Run `tachyon auth manifest fmt` to fix."
                ));
            }
        }

        ManifestCommand::Validate { file } => {
            let manifests = discover_manifests(file.as_deref(), &cwd)?;
            if manifests.is_empty() {
                println!("No auth manifests found.");
                return Ok(());
            }
            let mut errors = 0;
            for loaded in &manifests {
                match validate_manifest(&loaded.manifest) {
                    Ok(()) => println!("Valid: {}", loaded.path.display()),
                    Err(e) => {
                        eprintln!("Invalid: {} — {e}", loaded.path.display());
                        errors += 1;
                    }
                }
            }
            if errors > 0 {
                return Err(anyhow!("{errors} manifest(s) failed validation"));
            }
            println!("All manifests are valid.");
        }

        ManifestCommand::Plan {
            file,
            prune,
            global,
            json,
        } => {
            let manifests = discover_manifests(file.as_deref(), &cwd)?;
            if manifests.is_empty() {
                println!("No auth manifests found. Nothing to plan.");
                return Ok(());
            }
            let merged = merge_manifests(manifests);
            validate_manifest(&merged)?;

            let api =
                ApiClient::new_with_auth_diagnostics(config, tenant_id, auth_diagnostics.clone())?;
            let report = build_plan(&api, &merged, *prune, tenant_id, *global).await?;

            if *json {
                print_json(&report)?;
            } else {
                print_plan(&report);
            }
        }

        ManifestCommand::Apply {
            file,
            prune,
            global,
            json,
        } => {
            let manifests = discover_manifests(file.as_deref(), &cwd)?;
            if manifests.is_empty() {
                println!("No auth manifests found. Nothing to apply.");
                return Ok(());
            }
            let merged = merge_manifests(manifests);
            validate_manifest(&merged)?;

            let api =
                ApiClient::new_with_auth_diagnostics(config, tenant_id, auth_diagnostics.clone())?;
            let result = apply_manifest(&api, config, &merged, *prune, tenant_id, *global).await?;

            let has_errors = result
                .actions
                .iter()
                .any(|a| matches!(a.outcome, ApplyOutcome::Error(_)))
                || result
                    .policies
                    .iter()
                    .any(|p| matches!(p.outcome, ApplyOutcome::Error(_)));

            if *json {
                print_json(&result)?;
            } else {
                print_apply_result(&result);
            }

            if has_errors {
                return Err(anyhow!("Some resources failed to apply. See output above."));
            }
        }
    }

    Ok(())
}

// ---- Public helpers for reconcile_cli ----

/// Run plan (or apply) as part of a broader reconcile flow.
/// Returns None if no manifests found (skip gracefully).
#[allow(dead_code)]
pub async fn reconcile(
    config: &Configuration,
    default_tenant_id: &str,
    dry_run: bool,
    file: Option<&Path>,
    prune: bool,
    json: bool,
) -> Result<Option<()>> {
    let cwd = std::env::current_dir()?;
    reconcile_in(config, default_tenant_id, dry_run, file, prune, json, &cwd).await
}

#[allow(dead_code)]
pub async fn reconcile_in(
    config: &Configuration,
    default_tenant_id: &str,
    dry_run: bool,
    file: Option<&Path>,
    prune: bool,
    json: bool,
    cwd: &Path,
) -> Result<Option<()>> {
    let manifests = discover_manifests(file, cwd)?;
    if manifests.is_empty() {
        return Ok(None);
    }
    let merged = merge_manifests(manifests);
    validate_manifest(&merged)?;
    let api = ApiClient::new(config, default_tenant_id)?;

    if dry_run {
        let report = build_plan(&api, &merged, prune, default_tenant_id, false).await?;
        if json {
            print_json(&report)?;
        } else {
            println!("=== Auth Manifest Plan ===");
            print_plan(&report);
        }
    } else {
        let result = apply_manifest(&api, config, &merged, prune, default_tenant_id, false).await?;
        let has_errors = result
            .actions
            .iter()
            .any(|a| matches!(a.outcome, ApplyOutcome::Error(_)))
            || result
                .policies
                .iter()
                .any(|p| matches!(p.outcome, ApplyOutcome::Error(_)));
        if json {
            print_json(&result)?;
        } else {
            println!("=== Auth Manifest Apply ===");
            print_apply_result(&result);
        }
        if has_errors {
            return Err(anyhow!("Auth manifest apply: some resources failed."));
        }
    }

    Ok(Some(()))
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_manifest() -> AuthManifest {
        AuthManifest {
            actions: vec![
                ActionSpec {
                    context: "myapp".into(),
                    name: "ListItems".into(),
                    description: Some("List items".into()),
                    resource_pattern: None,
                    shared: false,
                    global: None,
                    requires: vec![],
                },
                ActionSpec {
                    context: "myapp".into(),
                    name: "CreateItem".into(),
                    description: None,
                    resource_pattern: None,
                    shared: false,
                    global: None,
                    requires: vec![],
                },
            ],
            policies: vec![PolicySpec {
                name: "MyAppReadOnly".into(),
                description: None,
                namespace: None,
                global: None,
                shared: false,
                actions: vec![PolicyActionSpec {
                    action: "myapp:ListItems".into(),
                    effect: "allow".into(),
                }],
                action_patterns: vec![],
            }],
        }
    }

    fn live_action(
        id: &str,
        context: &str,
        name: &str,
        platform_id: Option<&str>,
    ) -> ActionResponse {
        ActionResponse {
            id: Some(id.to_string()),
            platform_id: platform_id.map(str::to_string),
            context: context.to_string(),
            name: name.to_string(),
            full_name: format!("{context}:{name}"),
            description: None,
            resource_pattern: None,
            owner_tenant_id: None,
        }
    }

    fn live_policy(
        id: &str,
        name: &str,
        is_system: bool,
        tenant_id: Option<&str>,
    ) -> PolicyResponse {
        PolicyResponse {
            id: id.to_string(),
            name: name.to_string(),
            is_system,
            tenant_id: tenant_id.map(str::to_string),
            owner_tenant_id: None,
        }
    }

    #[test]
    fn plan_symbols_preserve_known_changes_and_distinguish_unknown() {
        assert_eq!(change_symbol(&ChangeKind::Create), "+");
        assert_eq!(change_symbol(&ChangeKind::Update), "~");
        assert_eq!(change_symbol(&ChangeKind::Unchanged), "=");
        assert_eq!(change_symbol(&ChangeKind::Unknown), "?");
        assert_eq!(change_symbol(&ChangeKind::Prune), "!");
    }

    #[test]
    fn policy_plan_note_is_only_shown_for_unknown_state() {
        assert!(!policy_plan_note(&ChangeKind::Unknown).is_empty());
        for known in [
            ChangeKind::Create,
            ChangeKind::Update,
            ChangeKind::Unchanged,
            ChangeKind::Prune,
        ] {
            assert_eq!(policy_plan_note(&known), "");
        }
    }

    #[test]
    fn prune_only_touches_contexts_the_manifest_declares() {
        // sample_manifest owns the `myapp` context only.
        let manifest = sample_manifest();
        let live = vec![
            live_action("act_1", "myapp", "Removed", Some("tn_own")),
            live_action("act_2", "otherapp", "Untouched", Some("tn_own")),
            live_action("act_3", "myapp", "ListItems", Some("tn_own")),
        ];

        let pruned = actions_to_prune(&live, &manifest, "tn_root");

        assert_eq!(pruned.len(), 1);
        assert_eq!(pruned[0].full_name, "myapp:Removed");
    }

    #[test]
    fn prune_skips_system_actions() {
        let manifest = sample_manifest();
        let live = vec![live_action("act_1", "myapp", "Removed", None)];

        assert!(actions_to_prune(&live, &manifest, "tn_root").is_empty());
    }

    #[test]
    fn policy_prune_is_limited_to_owned_namespaced_policies() {
        let mut manifest = sample_manifest();
        manifest.policies[0].name = "myapp:ReadOnly".into();
        let live = vec![
            // Same tenant, declared context, absent from the manifest.
            live_policy("pol_1", "myapp:Removed", false, Some("tn_own")),
            // Still declared.
            live_policy("pol_2", "myapp:ReadOnly", false, Some("tn_own")),
            // Another tenant owns it.
            live_policy("pol_3", "myapp:Other", false, Some("tn_other")),
            // System policy.
            live_policy("pol_4", "myapp:System", true, None),
            // Hand-made policy without a context prefix.
            live_policy("pol_5", "AdHocPolicy", false, Some("tn_own")),
        ];

        let pruned = policies_to_prune(&live, &manifest, "tn_own");

        assert_eq!(pruned.len(), 1);
        assert_eq!(pruned[0].name, "myapp:Removed");
    }

    #[test]
    fn shared_policies_report_their_scope() {
        let mut manifest = sample_manifest();
        manifest.policies[0].shared = true;
        manifest.policies[0].namespace = Some("tn_root".into());

        assert_eq!(
            manifest.policies[0].scope_label(),
            "namespace/tn_root+shared"
        );
    }

    #[test]
    fn action_set_shared_applies_to_every_action() {
        let yaml = r#"
apiVersion: auth.tachyon.io/v1
kind: ActionSet
metadata:
  name: field-actions
spec:
  shared: true
  actions:
    - context: field
      name: ListCustomers
    - context: field
      name: ManageCustomers
"#;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let manifest = parse_manifest_value(value).unwrap();

        assert_eq!(manifest.actions.len(), 2);
        assert!(manifest.actions.iter().all(|action| action.shared));
    }

    #[test]
    fn register_requests_carry_the_shared_flag() {
        let action = RegisterActionRequest {
            context: "field",
            name: "ListCustomers",
            description: None,
            resource_pattern: None,
            shared_with_descendants: true,
            global: false,
        };
        let value = serde_json::to_value(action).expect("serializable");
        assert_eq!(value["sharedWithDescendants"], true);

        let policy = RegisterPolicyRequest {
            name: "field:admin",
            description: None,
            is_system: false,
            global: false,
            tenant_id: Some("tn_root"),
            shared_with_descendants: true,
            actions: vec![],
            action_patterns: vec![],
        };
        let value = serde_json::to_value(policy).expect("serializable");
        assert_eq!(value["sharedWithDescendants"], true);
        assert_eq!(value["tenantId"], "tn_root");
    }

    #[test]
    fn validate_valid_manifest() {
        assert!(validate_manifest(&sample_manifest()).is_ok());
    }

    #[test]
    fn validate_empty_context_fails() {
        let mut m = sample_manifest();
        m.actions[0].context = String::new();
        assert!(validate_manifest(&m).is_err());
    }

    #[test]
    fn validate_bad_effect_fails() {
        let mut m = sample_manifest();
        m.policies[0].actions[0].effect = "permit".into();
        assert!(validate_manifest(&m).is_err());
    }

    #[test]
    fn validate_bad_action_format_fails() {
        let mut m = sample_manifest();
        m.policies[0].actions[0].action = "no-colon".into();
        assert!(validate_manifest(&m).is_err());
    }

    #[test]
    fn full_name_format() {
        let action = ActionSpec {
            context: "foo".into(),
            name: "Bar".into(),
            description: None,
            resource_pattern: None,
            shared: false,
            global: None,
            requires: vec![],
        };
        assert_eq!(action.full_name(), "foo:Bar");
    }

    #[test]
    fn requires_parses_and_roundtrips() {
        let yaml = "actions:\n  - context: field\n    name: SyncStorefront\n    requires:\n      - field:ListCustomers\n      - field:ManageCustomers\npolicies: []\n";
        let manifest: AuthManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            manifest.actions[0].requires,
            vec!["field:ListCustomers", "field:ManageCustomers"]
        );
        assert!(validate_manifest(&manifest).is_ok());
        // An absent `requires` stays absent when reserialized.
        let bare: AuthManifest = serde_yaml::from_str(
            "actions:\n  - context: field\n    name: SyncStorefront\npolicies: []\n",
        )
        .unwrap();
        assert!(bare.actions[0].requires.is_empty());
        let reserialized = serde_yaml::to_string(&bare).unwrap();
        assert!(!reserialized.contains("requires"));
    }

    #[test]
    fn requires_must_be_a_full_action_name() {
        let yaml = "actions:\n  - context: field\n    name: SyncStorefront\n    requires:\n      - ListCustomers\npolicies: []\n";
        let manifest: AuthManifest = serde_yaml::from_str(yaml).unwrap();
        let error = validate_manifest(&manifest).unwrap_err();
        assert!(
            error.to_string().contains("action name like"),
            "unexpected: {error}"
        );
    }

    #[test]
    fn requires_rejects_a_self_edge() {
        let yaml = "actions:\n  - context: field\n    name: SyncStorefront\n    requires:\n      - field:SyncStorefront\npolicies: []\n";
        let manifest: AuthManifest = serde_yaml::from_str(yaml).unwrap();
        let error = validate_manifest(&manifest).unwrap_err();
        assert!(
            error.to_string().contains("self edge"),
            "unexpected: {error}"
        );
    }

    #[test]
    fn register_action_request_sends_camel_case_resource_pattern() {
        let request = RegisterActionRequest {
            context: "field",
            name: "SyncStorefront",
            description: None,
            resource_pattern: Some("trn:field:storefront:*"),
            shared_with_descendants: false,
            global: false,
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json["resourcePattern"], "trn:field:storefront:*",
            "the server expects camelCase; snake_case is silently dropped"
        );
        assert!(json.get("resource_pattern").is_none());
    }

    #[test]
    fn dependency_edges_serialize_for_the_registration_api() {
        let request = RegisterActionDependenciesRequest {
            edges: vec![ActionDependencyEdgeReq {
                action: "field:SyncStorefront".to_string(),
                depends_on: "field:ListCustomers".to_string(),
            }],
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["edges"][0]["action"], "field:SyncStorefront");
        assert_eq!(json["edges"][0]["dependsOn"], "field:ListCustomers");
    }

    #[test]
    fn discover_from_dotachyon_dir() {
        let tmp = TempDir::new().unwrap();
        let manifests_dir = tmp.path().join(".tachyon").join("manifests");
        std::fs::create_dir_all(&manifests_dir).unwrap();
        let content = "actions:\n  - context: test\n    name: List\npolicies: []\n";
        std::fs::write(manifests_dir.join("auth.yml"), content).unwrap();

        let found = discover_manifests(None, tmp.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].manifest.actions[0].context, "test");
    }

    #[test]
    fn discover_from_tachyon_yml() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();
        let content = "metadata:\n  name: test\nauth:\n  manifest:\n    actions:\n      - context: myapp\n        name: List\n    policies: []\n";
        std::fs::write(tmp.path().join("tachyon.yml"), content).unwrap();

        let found = discover_manifests(None, tmp.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].manifest.actions[0].full_name(), "myapp:List");
    }

    #[test]
    fn discover_no_manifests() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();
        let found = discover_manifests(None, tmp.path()).unwrap();
        assert!(found.is_empty());
    }

    #[test]
    fn discover_explicit_tachyon_yml_with_auth_section() {
        // --file tachyon.yml where the file has an `auth.manifest` section
        let tmp = TempDir::new().unwrap();
        let content = "metadata:\n  name: myapp\nauth:\n  manifest:\n    actions:\n      - context: myapp\n        name: List\n    policies: []\n";
        let tachyon_yml = tmp.path().join("tachyon.yml");
        std::fs::write(&tachyon_yml, content).unwrap();

        let found = discover_manifests(Some(&tachyon_yml), tmp.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].manifest.actions[0].full_name(), "myapp:List");
    }

    #[tokio::test]
    async fn reconcile_noop_on_empty_directory() {
        // reconcile_in returns Ok(None) when no manifests exist, without touching the API.
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();

        // Use a dummy config; the API is never called if manifests are empty.
        let dummy_config = tachyon_sdk::apis::configuration::Configuration::default();
        let result = reconcile_in(
            &dummy_config,
            "tn_test",
            false,
            None,
            false,
            false,
            tmp.path(),
        )
        .await;
        assert!(
            matches!(result, Ok(None)),
            "expected Ok(None) but got {:?}",
            result
        );
    }

    #[test]
    fn merge_deduplicates() {
        let m1 = LoadedManifest {
            path: PathBuf::from("a.yml"),
            manifest: AuthManifest {
                actions: vec![ActionSpec {
                    context: "app".into(),
                    name: "List".into(),
                    description: None,
                    resource_pattern: None,
                    shared: false,
                    global: None,
                    requires: vec![],
                }],
                policies: vec![],
            },
        };
        let m2 = LoadedManifest {
            path: PathBuf::from("b.yml"),
            manifest: AuthManifest {
                actions: vec![ActionSpec {
                    context: "app".into(),
                    name: "List".into(), // duplicate
                    description: None,
                    resource_pattern: None,
                    shared: false,
                    global: None,
                    requires: vec![],
                }],
                policies: vec![],
            },
        };
        let merged = merge_manifests(vec![m1, m2]);
        assert_eq!(merged.actions.len(), 1);
    }

    #[test]
    fn fmt_manifest_roundtrip() {
        let m = sample_manifest();
        let yaml = fmt_manifest(&m).unwrap();
        let parsed: AuthManifest = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, m);
    }

    #[test]
    fn parse_k8s_policy_without_namespace_fails() {
        let yaml = r#"
- apiVersion: auth.tachyon.io/v1
  kind: ActionSet
  metadata:
    name: billing-actions
  spec:
    actions:
      - context: billing
        name: ViewInvoices
- apiVersion: auth.tachyon.io/v1
  kind: Policy
  metadata:
    name: BillingViewer
  spec:
    description: View invoices
    actions:
      - action: billing:ViewInvoices
        effect: allow
"#;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let error = parse_manifest_value(value).expect_err("policy namespace must be required");
        assert!(error.to_string().contains("metadata.namespace is required"));
    }

    #[test]
    fn parse_k8s_namespaced_policy() {
        let yaml = r#"
apiVersion: auth.tachyon.io/v1
kind: Policy
metadata:
  name: TenantBillingViewer
  namespace: tn_01hjryxysgey07h5jz5wagqj0m
spec:
  action_patterns:
    - context_pattern: billing
      name_pattern: '*'
      effect: allow
"#;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let manifest = parse_manifest_value(value).unwrap();
        let policy = &manifest.policies[0];
        assert_ne!(policy.global, Some(true));
        assert_eq!(
            policy.target_tenant_id("tn_default"),
            "tn_01hjryxysgey07h5jz5wagqj0m"
        );
    }

    #[test]
    fn flat_policy_uses_caller_tenant_scope() {
        let manifest: AuthManifest =
            serde_yaml::from_str("actions: []\npolicies:\n  - name: FlatPolicy\n").unwrap();
        let policy = &manifest.policies[0];
        assert_ne!(policy.global, Some(true));
        assert_eq!(policy.target_tenant_id("tn_default"), "tn_default");
    }

    /// Prune must reach the global definitions this tenant registered, or
    /// `--global` would create rows the same manifest can never clean up. It
    /// must still leave another tenant's global definitions alone.
    #[test]
    fn prune_claims_only_its_own_global_definitions() {
        let manifest = sample_manifest();
        let mut ours = live_policy("pol_ours", "myapp:gone", false, None);
        ours.owner_tenant_id = Some("tn_root".to_string());
        let mut theirs = live_policy("pol_theirs", "myapp:other", false, None);
        theirs.owner_tenant_id = Some("tn_someone_else".to_string());
        let orphan = live_policy("pol_orphan", "myapp:orphan", false, None);
        let live = vec![ours, theirs, orphan];

        let pruned = policies_to_prune(&live, &manifest, "tn_root");

        let ids: Vec<&str> = pruned.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["pol_ours"],
            "only our global policy is ours to delete"
        );
    }

    #[test]
    fn global_apply_does_not_claim_another_tenants_policy() {
        let mut manifest = sample_manifest();
        manifest.policies[0].global = Some(true);
        let mut foreign = live_policy("pol_foreign", "MyAppReadOnly", false, None);
        foreign.owner_tenant_id = Some("tn_other".to_string());
        let mut ours = live_policy("pol_ours", "MyAppReadOnly", false, None);
        ours.owner_tenant_id = Some("tn_root".to_string());

        assert!(existing_policy_for_spec(
            &[foreign.clone()],
            &manifest.policies[0],
            true,
            "tn_root"
        )
        .is_none());
        assert_eq!(
            existing_policy_for_spec(&[foreign, ours], &manifest.policies[0], true, "tn_root")
                .map(|policy| policy.id.as_str()),
            Some("pol_ours")
        );
    }

    #[test]
    fn validate_rejects_global_combined_with_shared() {
        let mut manifest = sample_manifest();
        manifest.policies[0].global = Some(true);
        manifest.policies[0].shared = true;

        let error = validate_manifest(&manifest).expect_err("global + shared must be rejected");

        assert!(error.to_string().contains("not applicable"));
    }

    #[test]
    fn validate_rejects_global_combined_with_namespace() {
        let mut manifest = sample_manifest();
        manifest.policies[0].global = Some(true);
        manifest.policies[0].namespace = Some("tn_root".to_string());

        let error = validate_manifest(&manifest).expect_err("global + namespace must be rejected");

        assert!(error.to_string().contains("namespace"));
    }

    #[tokio::test]
    async fn apply_validates_global_combination_before_api_request() {
        let mut manifest = sample_manifest();
        manifest.policies[0].global = Some(true);
        manifest.policies[0].shared = true;
        let configuration = tachyon_sdk::apis::configuration::Configuration::default();
        let api = ApiClient::new(&configuration, "tn_test").unwrap();

        let error = apply_manifest(&api, &configuration, &manifest, false, "tn_test", false)
            .await
            .expect_err("apply must validate before sending requests");

        assert!(error.to_string().contains("not applicable"));
    }

    #[test]
    fn tenant_scoped_request_carries_its_tenant() {
        let request = RegisterPolicyRequest {
            name: "TenantPolicy",
            description: None,
            is_system: false,
            global: false,
            tenant_id: Some("tn_test"),
            shared_with_descendants: false,
            actions: vec![],
            action_patterns: vec![],
        };

        let value = serde_json::to_value(request).expect("serializable");
        assert_eq!(value["global"], false);
        assert_eq!(value["tenantId"], "tn_test");
    }

    /// A global policy has no owning tenant, so sending one would contradict
    /// the scope and the API rejects it.
    #[test]
    fn global_request_omits_the_tenant() {
        let request = RegisterPolicyRequest {
            name: "GlobalPolicy",
            description: None,
            is_system: false,
            global: true,
            tenant_id: None,
            shared_with_descendants: false,
            actions: vec![],
            action_patterns: vec![],
        };

        let value = serde_json::to_value(request).expect("serializable");
        assert_eq!(value["global"], true);
        assert!(value.get("tenantId").is_none());
    }

    #[test]
    fn manifest_global_declaration_beats_the_flag() {
        // Silent spec: the flag decides.
        assert!(resolve_global(None, true, "x").effective);
        assert!(!resolve_global(None, false, "x").effective);

        // Declared spec: the manifest decides, either way.
        assert!(resolve_global(Some(true), false, "x").effective);
        assert!(!resolve_global(Some(false), true, "x").effective);
    }

    #[test]
    fn overruling_the_flag_is_reported() {
        // Only an active disagreement is worth reporting: the flag asked for
        // global and the manifest refused.
        let overruled = resolve_global(Some(false), true, "field:admin");
        assert!(!overruled.effective);
        assert!(
            overruled
                .conflict
                .as_deref()
                .is_some_and(|c| c.contains("field:admin")),
            "conflict must name the resource, got {:?}",
            overruled.conflict
        );

        // Agreement, and a manifest that widens on its own, are silent.
        assert!(resolve_global(Some(true), true, "x").conflict.is_none());
        assert!(resolve_global(Some(true), false, "x").conflict.is_none());
        assert!(resolve_global(None, true, "x").conflict.is_none());
    }
}
