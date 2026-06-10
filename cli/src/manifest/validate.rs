use anyhow::{anyhow, Result};
use serde::Serialize;
use std::path::Path;

use crate::client::print_json;
use crate::commands::auth::manifest as auth_manifest;
use crate::compute_cli;

use super::discovery::{discover, ManifestKind, ManifestSource};
use super::ValidateArgs;

#[derive(Debug, Serialize)]
struct ValidationItem {
    path: String,
    kind: ManifestKind,
    status: &'static str,
    message: String,
}

pub(crate) fn run(args: &ValidateArgs) -> Result<()> {
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

pub(crate) fn validate_source(
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
