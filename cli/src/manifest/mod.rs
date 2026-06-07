use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};
use crate::commands::auth::manifest as auth_manifest;
use crate::compute_cli;

#[derive(Debug, Clone, Args)]
pub struct ManifestArgs {
    #[command(subcommand)]
    pub command: ManifestCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ManifestCommand {
    /// Validate local manifest syntax and supported schemas without API calls
    Validate(ValidateArgs),
    /// Show desired-vs-live manifest changes without mutating resources
    Plan(ApplyArgs),
    /// Apply local manifest desired state
    Apply(ApplyArgs),
    /// Reconcile local manifest desired state with live resources
    Reconcile(ApplyArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ValidateArgs {
    /// Manifest file path. When omitted, tachyon.yml and .tachyon/manifests are discovered.
    #[arg(short = 'f', long)]
    pub file: Option<PathBuf>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ApplyArgs {
    /// Manifest file path. When omitted, tachyon.yml and .tachyon/manifests are discovered.
    #[arg(short = 'f', long)]
    pub file: Option<PathBuf>,
    /// Target app name to select from a multi-app CloudApps manifest
    #[arg(long)]
    pub app: Option<String>,
    /// Environment label for CloudApps manifest operations
    #[arg(long, default_value = "sandbox")]
    pub environment: String,
    /// Remove resources absent from manifest where supported
    #[arg(long)]
    pub prune: bool,
    /// Preview changes without mutating resources
    #[arg(long)]
    pub dry_run: bool,
    /// Output as JSON where supported
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ManifestKind {
    CloudApps,
    Auth,
    Iac,
    Unsupported,
}

#[derive(Debug, Clone, Serialize)]
struct ManifestSource {
    path: PathBuf,
    kind: ManifestKind,
    detail: String,
}

#[derive(Debug, Serialize)]
struct ValidationItem {
    path: String,
    kind: ManifestKind,
    status: &'static str,
    message: String,
}

pub async fn run(
    args: &ManifestArgs,
    config: Option<&Configuration>,
    tenant_id: Option<&str>,
) -> Result<()> {
    match &args.command {
        ManifestCommand::Validate(validate_args) => validate(validate_args),
        ManifestCommand::Plan(apply_args) => {
            let config = config.ok_or_else(|| anyhow!("manifest plan requires tenant context"))?;
            let tenant_id =
                tenant_id.ok_or_else(|| anyhow!("manifest plan requires tenant context"))?;
            run_apply_like(apply_args, config, tenant_id, true, "plan").await
        }
        ManifestCommand::Apply(apply_args) => {
            let config = config.ok_or_else(|| anyhow!("manifest apply requires tenant context"))?;
            let tenant_id =
                tenant_id.ok_or_else(|| anyhow!("manifest apply requires tenant context"))?;
            run_apply_like(apply_args, config, tenant_id, apply_args.dry_run, "apply").await
        }
        ManifestCommand::Reconcile(apply_args) => {
            let config =
                config.ok_or_else(|| anyhow!("manifest reconcile requires tenant context"))?;
            let tenant_id =
                tenant_id.ok_or_else(|| anyhow!("manifest reconcile requires tenant context"))?;
            run_apply_like(
                apply_args,
                config,
                tenant_id,
                apply_args.dry_run,
                "reconcile",
            )
            .await
        }
    }
}

pub fn context_file(args: &ManifestArgs) -> Option<&Path> {
    match &args.command {
        ManifestCommand::Validate(args) => args.file.as_deref(),
        ManifestCommand::Plan(args)
        | ManifestCommand::Apply(args)
        | ManifestCommand::Reconcile(args) => args.file.as_deref(),
    }
}

pub fn needs_tenant(args: &ManifestArgs) -> bool {
    !matches!(args.command, ManifestCommand::Validate(_))
}

pub async fn reconcile_alias(
    args: &crate::reconcile_cli::ReconcileArgs,
    config: &Configuration,
    tenant_id: &str,
) -> Result<()> {
    let manifest_args = ManifestArgs {
        command: ManifestCommand::Reconcile(ApplyArgs {
            file: args.file.clone(),
            app: args.app.clone(),
            environment: args.environment.clone(),
            prune: args.prune,
            dry_run: args.dry_run,
            json: args.json,
        }),
    };
    run(&manifest_args, Some(config), Some(tenant_id)).await
}

fn validate(args: &ValidateArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let sources = discover(args.file.as_deref(), &cwd)?;
    if sources.is_empty() {
        println!("No manifests found.");
        return Ok(());
    }

    let mut items = Vec::new();
    let mut errors = 0;
    for source in sources {
        match validate_source(&source, None, "sandbox") {
            Ok(message) => items.push(ValidationItem {
                path: source.path.display().to_string(),
                kind: source.kind,
                status: "valid",
                message,
            }),
            Err(error) => {
                errors += 1;
                items.push(ValidationItem {
                    path: source.path.display().to_string(),
                    kind: source.kind,
                    status: "invalid",
                    message: error.to_string(),
                });
            }
        }
    }

    if args.json {
        print_json(&items)?;
    } else {
        for item in &items {
            let label = if item.status == "valid" {
                "Valid"
            } else {
                "Invalid"
            };
            println!(
                "{label}: {} ({:?}) - {}",
                item.path, item.kind, item.message
            );
        }
    }

    if errors > 0 {
        return Err(anyhow!("{errors} manifest(s) failed validation"));
    }
    Ok(())
}

async fn run_apply_like(
    args: &ApplyArgs,
    config: &Configuration,
    tenant_id: &str,
    dry_run: bool,
    mode: &str,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let sources = discover(args.file.as_deref(), &cwd)?;
    if sources.is_empty() {
        println!("No manifests found. Nothing to {mode}.");
        return Ok(());
    }

    let api = ApiClient::new(config, tenant_id)?;
    let mut fatal_errors = 0;

    for source in sources.iter().filter(|s| s.kind == ManifestKind::Iac) {
        validate_source(source, args.app.as_deref(), &args.environment)?;
        println!(
            "IaC manifest: {} is recognized but apply/reconcile is not supported yet.",
            source.path.display()
        );
    }

    let auth_files = sources
        .iter()
        .filter(|s| s.kind == ManifestKind::Auth)
        .map(|s| s.path.clone())
        .collect::<Vec<_>>();
    if !auth_files.is_empty() {
        let loaded = load_auth_manifests(&auth_files, args.file.is_some(), &cwd)?;
        let merged = auth_manifest::merge_manifests(loaded);
        auth_manifest::validate_manifest(&merged)?;
        if dry_run {
            let report = auth_manifest::build_plan(&api, &merged, args.prune).await?;
            if args.json {
                print_json(&report)?;
            } else {
                println!("=== Auth Manifest Plan ===");
                auth_manifest::print_plan(&report);
            }
        } else {
            let result =
                auth_manifest::apply_manifest(&api, &merged, args.prune, tenant_id).await?;
            if args.json {
                print_json(&result)?;
            } else {
                println!("=== Auth Manifest Apply ===");
                auth_manifest::print_apply_result(&result);
            }
        }
    }

    for source in sources.iter().filter(|s| s.kind == ManifestKind::CloudApps) {
        if let Err(error) = validate_source(source, args.app.as_deref(), &args.environment) {
            fatal_errors += 1;
            eprintln!(
                "Invalid Cloud Apps manifest {}: {error}",
                source.path.display()
            );
            continue;
        }
        if !args.json {
            println!(
                "=== Cloud Apps Manifest {} ===",
                if dry_run { "Plan" } else { "Apply" }
            );
        }
        compute_cli::run_apps_apply(
            &api,
            &source.path,
            args.app.as_deref(),
            &args.environment,
            dry_run,
        )
        .await?;
    }

    for source in sources
        .iter()
        .filter(|s| s.kind == ManifestKind::Unsupported)
    {
        println!(
            "Unsupported manifest: {} ({}) skipped.",
            source.path.display(),
            source.detail
        );
    }

    if fatal_errors > 0 {
        return Err(anyhow!("{fatal_errors} manifest step(s) failed"));
    }
    Ok(())
}

fn validate_source(
    source: &ManifestSource,
    app: Option<&str>,
    environment: &str,
) -> Result<String> {
    match source.kind {
        ManifestKind::CloudApps => {
            let manifest = compute_cli::load_cloud_apps_manifest(&source.path)?;
            let entries = compute_cli::select_app_entries(&manifest, app)?;
            for entry in &entries {
                let _ = compute_cli::app_entry_to_api_body(entry)?;
                let _ = compute_cli::plan_env_vars(entry, environment)?;
            }
            Ok(format!("{} Cloud Apps entry(s)", entries.len()))
        }
        ManifestKind::Auth => {
            let cwd = source.path.parent().unwrap_or_else(|| Path::new("."));
            let loaded = auth_manifest::discover_manifests(Some(&source.path), cwd)?;
            let merged = auth_manifest::merge_manifests(loaded);
            auth_manifest::validate_manifest(&merged)?;
            Ok("auth actions/policies".to_string())
        }
        ManifestKind::Iac => {
            Ok("IaC v1alpha manifest recognized; apply not supported yet".to_string())
        }
        ManifestKind::Unsupported => Ok(format!("unsupported manifest skipped: {}", source.detail)),
    }
}

fn load_auth_manifests(
    paths: &[PathBuf],
    explicit_file: bool,
    cwd: &Path,
) -> Result<Vec<auth_manifest::LoadedManifest>> {
    if explicit_file {
        return auth_manifest::discover_manifests(paths.first().map(PathBuf::as_path), cwd);
    }

    let mut loaded = Vec::new();
    for path in paths {
        let base = path.parent().unwrap_or(cwd);
        loaded.extend(auth_manifest::discover_manifests(
            Some(path.as_path()),
            base,
        )?);
    }
    Ok(loaded)
}

fn discover(explicit_file: Option<&Path>, cwd: &Path) -> Result<Vec<ManifestSource>> {
    if let Some(file) = explicit_file {
        let path = absolutize(cwd, file);
        return classify_file(&path);
    }

    let mut sources = Vec::new();
    let project_root = if let Some(path) = find_tachyon_yml(cwd) {
        sources.extend(classify_file(&path)?);
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| cwd.to_path_buf())
    } else {
        find_repo_root(cwd).unwrap_or_else(|| cwd.to_path_buf())
    };

    let manifests_dir = project_root.join(".tachyon").join("manifests");
    if manifests_dir.is_dir() {
        let mut paths = collect_yaml_files(&manifests_dir)?;
        paths.sort();
        for path in paths {
            sources.extend(classify_file(&path)?);
        }
    }

    sources.sort_by(|a, b| {
        source_order(a)
            .cmp(&source_order(b))
            .then(a.path.cmp(&b.path))
    });
    Ok(sources)
}

fn classify_file(path: &Path) -> Result<Vec<ManifestSource>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value: Value =
        serde_yaml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    let mut sources = Vec::new();

    let is_cloud_apps = matches!(
        value.get("kind").and_then(Value::as_str),
        Some("CloudApps") | Some("CloudApp")
    );
    if is_cloud_apps {
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::CloudApps,
            detail: "Cloud Apps".to_string(),
        });
    }

    if value
        .get("auth")
        .and_then(|auth| auth.get("manifest"))
        .is_some()
        || looks_like_auth(&value)
    {
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::Auth,
            detail: "auth manifest".to_string(),
        });
    } else if !is_cloud_apps
        && value.get("apiVersion").and_then(Value::as_str) == Some("apps.tachy.one/v1alpha")
    {
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::Iac,
            detail: value
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
        });
    } else if sources.is_empty() {
        sources.push(ManifestSource {
            path: path.to_path_buf(),
            kind: ManifestKind::Unsupported,
            detail: value
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
        });
    }

    Ok(sources)
}

fn looks_like_auth(value: &Value) -> bool {
    value.get("actions").is_some()
        || value.get("policies").is_some()
        || value
            .get("apiVersion")
            .and_then(Value::as_str)
            .is_some_and(|version| version == "auth.tachyon.io/v1")
        || value
            .get("items")
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("apiVersion")
                        .and_then(Value::as_str)
                        .is_some_and(|version| version == "auth.tachyon.io/v1")
                })
            })
}

fn source_order(source: &ManifestSource) -> (u8, &Path) {
    let rank = match source.kind {
        ManifestKind::Iac => 0,
        ManifestKind::Auth => 1,
        ManifestKind::CloudApps => 2,
        ManifestKind::Unsupported => 3,
    };
    (rank, source.path.as_path())
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

fn find_repo_root(cwd: &Path) -> Option<PathBuf> {
    let mut dir = cwd;
    loop {
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

fn collect_yaml_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            result.extend(collect_yaml_files(&path)?);
        } else if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("yml" | "yaml")
        ) {
            result.push(path);
        }
    }
    Ok(result)
}

fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn discover_uses_single_file_when_explicit_file_is_set() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        write(
            &tmp.path().join("tachyon.yml"),
            "kind: CloudApps\nspec:\n  apps: []\n",
        );
        write(
            &tmp.path().join(".tachyon/manifests/auth.yml"),
            "actions:\n  - context: auth\n    name: Read\npolicies: []\n",
        );

        let sources = discover(Some(Path::new(".tachyon/manifests/auth.yml")), tmp.path()).unwrap();

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].kind, ManifestKind::Auth);
        assert!(sources[0].path.ends_with(".tachyon/manifests/auth.yml"));
    }

    #[test]
    fn discover_orders_iac_auth_then_cloud_apps() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        write(
            &tmp.path().join("tachyon.yml"),
            "kind: CloudApps\nspec:\n  apps: []\n",
        );
        write(
            &tmp.path().join(".tachyon/manifests/z-auth.yml"),
            "actions:\n  - context: auth\n    name: Read\npolicies: []\n",
        );
        write(
            &tmp.path().join(".tachyon/manifests/a-iac.yml"),
            "apiVersion: apps.tachy.one/v1alpha\nkind: Operator\nmetadata:\n  name: op\n",
        );

        let sources = discover(None, tmp.path()).unwrap();
        let kinds = sources
            .iter()
            .map(|source| source.kind.clone())
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                ManifestKind::Iac,
                ManifestKind::Auth,
                ManifestKind::CloudApps
            ]
        );
    }

    #[test]
    fn discover_uses_repo_root_manifests_without_tachyon_yml() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        let nested = tmp.path().join("apps/api");
        fs::create_dir_all(&nested).unwrap();
        write(
            &tmp.path().join(".tachyon/manifests/auth.yml"),
            "actions:\n  - context: root\n    name: Read\npolicies: []\n",
        );

        let sources = discover(None, &nested).unwrap();

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].kind, ManifestKind::Auth);
        assert!(sources[0].path.ends_with(".tachyon/manifests/auth.yml"));
    }
}
