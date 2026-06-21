use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};
use crate::commands::auth::manifest::{self as auth_manifest, ApplyOutcome};
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
    let mut errors = ManifestApplyErrors::default();

    for source in sources.iter().filter(|s| s.kind == ManifestKind::Iac) {
        if let Err(error) = validate_source(source, args.app.as_deref(), &args.environment) {
            errors.record(format!(
                "Invalid IaC manifest {}: {error}",
                source.path.display()
            ));
            continue;
        }
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
        match load_auth_manifests(&auth_files, args.file.is_some(), &cwd).and_then(|loaded| {
            let merged = auth_manifest::merge_manifests(loaded);
            auth_manifest::validate_manifest(&merged)?;
            Ok(merged)
        }) {
            Ok(merged) if dry_run => {
                match auth_manifest::build_plan(&api, &merged, args.prune).await {
                    Ok(report) => {
                        if args.json {
                            print_json(&report)?;
                        } else {
                            println!("=== Auth Manifest Plan ===");
                            auth_manifest::print_plan(&report);
                        }
                    }
                    Err(error) => {
                        errors.record(format!("Auth manifest plan failed: {error}"));
                    }
                }
            }
            Ok(merged) => {
                match auth_manifest::apply_manifest(&api, &merged, args.prune, tenant_id).await {
                    Ok(result) => {
                        let has_errors = auth_apply_has_errors(&result);
                        if args.json {
                            print_json(&result)?;
                        } else {
                            println!("=== Auth Manifest Apply ===");
                            auth_manifest::print_apply_result(&result);
                        }
                        if has_errors {
                            errors.record("Auth manifest apply: some resources failed.");
                        }
                    }
                    Err(error) => {
                        errors.record(format!("Auth manifest apply failed: {error}"));
                    }
                }
            }
            Err(error) => {
                if dry_run {
                    errors.record(format!("Auth manifest plan skipped: {error}"));
                } else {
                    errors.record(format!("Auth manifest apply skipped: {error}"));
                }
            }
        }
    }

    for source in sources.iter().filter(|s| s.kind == ManifestKind::CloudApps) {
        if let Err(error) = validate_source(source, args.app.as_deref(), &args.environment) {
            errors.record(format!(
                "Invalid Cloud Apps manifest {}: {error}",
                source.path.display()
            ));
            continue;
        }
        if !args.json {
            println!(
                "=== Cloud Apps Manifest {} ===",
                if dry_run { "Plan" } else { "Apply" }
            );
        }
        if let Err(error) = compute_cli::run_apps_apply(
            &api,
            tenant_id,
            &source.path,
            args.app.as_deref(),
            &args.environment,
            dry_run,
        )
        .await
        {
            errors.record(format!(
                "Cloud Apps manifest {} failed: {error}",
                source.path.display()
            ));
        }
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

    errors.finish(mode)
}

#[derive(Default)]
struct ManifestApplyErrors {
    messages: Vec<String>,
}

impl ManifestApplyErrors {
    fn record(&mut self, message: impl Into<String>) {
        let message = message.into();
        eprintln!("{message}");
        self.messages.push(message);
    }

    fn finish(self, mode: &str) -> Result<()> {
        if self.messages.is_empty() {
            return Ok(());
        }

        eprintln!();
        eprintln!(
            "Manifest {mode} completed with {} error(s):",
            self.messages.len()
        );
        for message in &self.messages {
            eprintln!("  - {message}");
        }

        Err(anyhow!("{} manifest step(s) failed", self.messages.len()))
    }
}

fn auth_apply_has_errors(result: &auth_manifest::ApplyResult) -> bool {
    result
        .actions
        .iter()
        .any(|action| matches!(action.outcome, ApplyOutcome::Error(_)))
        || result
            .policies
            .iter()
            .any(|policy| matches!(policy.outcome, ApplyOutcome::Error(_)))
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
