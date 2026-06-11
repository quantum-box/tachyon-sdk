use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};
use crate::commands::auth::manifest as auth_manifest;
use crate::compute_cli;

use super::discovery::{discover, ManifestKind};
use super::validate::validate_source;
use super::ApplyArgs;

pub(crate) async fn run(
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
