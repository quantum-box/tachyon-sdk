use anyhow::{anyhow, Result};
use serde::Serialize;

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
            let manifest = compute_cli::normalize_cloud_apps_document(&source.document)?
                .ok_or_else(|| anyhow!("unsupported Cloud Apps document"))?;
            let entries = compute_cli::select_app_entries(&manifest, app)?;
            let entry_count = entries.len();
            for entry in entries {
                validate_cloud_app_entry(&entry, environment)?;
            }
            Ok(format!("{entry_count} Cloud Apps entry(s)"))
        }
        ManifestKind::Auth => {
            let yaml_value = serde_yaml::to_value(&source.document)?;
            let manifest = auth_manifest::parse_manifest_document_value(yaml_value)?;
            auth_manifest::validate_manifest(&manifest)?;
            Ok("auth actions/policies".to_string())
        }
        ManifestKind::Iac => {
            Ok("IaC v1alpha manifest recognized; apply not supported yet".to_string())
        }
        ManifestKind::Unsupported => Ok(format!("unsupported manifest skipped: {}", source.detail)),
    }
}

fn validate_cloud_app_entry(entry: &serde_json::Value, environment: &str) -> Result<()> {
    if environment == "sandbox" {
        match entry.get("environments") {
            Some(serde_json::Value::Object(overlays)) if !overlays.is_empty() => {
                for overlay_environment in overlays.keys() {
                    let resolved =
                        compute_cli::resolve_app_entry_for_environment(entry, overlay_environment)?;
                    let _ = compute_cli::app_entry_to_api_body(&resolved)?;
                    let _ = compute_cli::plan_env_vars(&resolved, overlay_environment)?;
                    compute_cli::validate_generated_env_target(&resolved, overlay_environment)?;
                }
                return Ok(());
            }
            Some(serde_json::Value::Object(_)) | None => {}
            Some(_) => return Err(anyhow!("app environments must be an object")),
        }
    }

    let resolved = compute_cli::resolve_app_entry_for_environment(entry, environment)?;
    let _ = compute_cli::app_entry_to_api_body(&resolved)?;
    let _ = compute_cli::plan_env_vars(&resolved, environment)?;
    compute_cli::validate_generated_env_target(&resolved, environment)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn sandbox_validation_checks_defined_overlays_individually() {
        let entry = json!({
            "name": "fieldadmin",
            "repository": {
                "url": "https://github.com/quantum-box/tachyonfield",
                "owner": "quantum-box",
                "name": "tachyonfield"
            },
            "environments": {
                "production": { "rootDirectory": "apps/admin-ui" },
                "preview": { "rootDirectory": "apps/admin-ui" }
            }
        });

        validate_cloud_app_entry(&entry, "sandbox").unwrap();
    }

    #[test]
    fn sandbox_validation_rejects_unscoped_staging_auth() {
        let entry = json!({
            "name": "fieldadmin",
            "repository": {
                "url": "https://github.com/quantum-box/tachyonfield",
                "owner": "quantum-box",
                "name": "tachyonfield"
            },
            "environments": {
                "staging": { "auth": { "enabled": true } }
            }
        });

        let error = validate_cloud_app_entry(&entry, "sandbox")
            .unwrap_err()
            .to_string();

        assert!(error.contains("no safe target for generated auth or Sentry env vars"));
    }
}
