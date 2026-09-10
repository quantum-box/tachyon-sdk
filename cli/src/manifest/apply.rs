use anyhow::{anyhow, Result};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};
use crate::commands::auth::manifest::{self as auth_manifest, ApplyOutcome};
use crate::compute_cli;

use super::discovery::{discover, ManifestKind};
use super::oauth2_resource;
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

    for source in &sources {
        match source.kind {
            ManifestKind::Iac => {
                if let Err(error) = validate_source(source, args.app.as_deref(), &args.environment)
                {
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
            ManifestKind::Auth => {
                apply_auth_source(source, args, &api, config, tenant_id, dry_run, &mut errors)
                    .await?;
            }
            ManifestKind::OAuth2Resource => {
                apply_oauth2_resource_source(source, args, &api, tenant_id, dry_run, &mut errors)
                    .await;
            }
            ManifestKind::CloudApps => {
                let manifest = match compute_cli::normalize_cloud_apps_document(&source.document) {
                    Ok(Some(manifest)) => manifest,
                    Ok(None) => {
                        errors.record(format!(
                            "Invalid Cloud Apps manifest {}: unsupported document",
                            source.path.display()
                        ));
                        continue;
                    }
                    Err(error) => {
                        errors.record(format!(
                            "Invalid Cloud Apps manifest {}: {error}",
                            source.path.display()
                        ));
                        continue;
                    }
                };
                if let Err(error) =
                    validate_cloud_apps_manifest(&manifest, args.app.as_deref(), &args.environment)
                {
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
                let manifest_label = source.path.display().to_string();
                if let Err(error) =
                    compute_cli::run_apps_apply_manifest(compute_cli::AppsApplyManifestInput {
                        api: &api,
                        tenant_id,
                        manifest: &manifest,
                        manifest_label: &manifest_label,
                        selected_app: args.app.as_deref(),
                        environment: &args.environment,
                        change_control_token: args.change_control_token.as_deref(),
                        release_checks_only: false,
                        dry_run,
                    })
                    .await
                {
                    errors.record(format!(
                        "Cloud Apps manifest {} failed: {error}",
                        source.path.display()
                    ));
                }
            }
            ManifestKind::Unsupported => {
                println!(
                    "Unsupported manifest: {} ({}) skipped.",
                    source.path.display(),
                    source.detail
                );
            }
        }
    }

    errors.finish(mode)
}

async fn apply_auth_source(
    source: &super::discovery::ManifestSource,
    args: &ApplyArgs,
    api: &ApiClient,
    config: &Configuration,
    tenant_id: &str,
    dry_run: bool,
    errors: &mut ManifestApplyErrors,
) -> Result<()> {
    match load_auth_manifest_source(source).and_then(|manifest| {
        auth_manifest::validate_manifest(&manifest)?;
        Ok(manifest)
    }) {
        Ok(merged) if dry_run => {
            match auth_manifest::build_plan(api, &merged, args.prune, tenant_id, false).await {
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
            match auth_manifest::apply_manifest(api, config, &merged, args.prune, tenant_id, false)
                .await
            {
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
    Ok(())
}

async fn apply_oauth2_resource_source(
    source: &super::discovery::ManifestSource,
    args: &ApplyArgs,
    api: &ApiClient,
    tenant_id: &str,
    dry_run: bool,
    errors: &mut ManifestApplyErrors,
) {
    let manifest = match oauth2_resource::parse(&source.document) {
        Ok(manifest) => manifest,
        Err(error) => {
            errors.record(format!(
                "Invalid OAuth2Resource manifest {}: {error}",
                source.path.display()
            ));
            return;
        }
    };
    if dry_run {
        match oauth2_resource::plan(api, &manifest, tenant_id).await {
            Ok(plan) => {
                if args.json {
                    if let Err(error) = print_json(&plan) {
                        errors.record(format!("OAuth2Resource plan output failed: {error}"));
                    }
                } else {
                    println!("=== OAuth2Resource Plan ===");
                    oauth2_resource::print_plan(&plan);
                }
            }
            Err(error) => errors.record(format!(
                "OAuth2Resource manifest {} plan failed: {error}",
                source.path.display()
            )),
        }
        return;
    }
    match oauth2_resource::apply(api, &manifest, tenant_id).await {
        Ok(result) => {
            if args.json {
                if let Err(error) = print_json(&result) {
                    errors.record(format!("OAuth2Resource apply output failed: {error}"));
                }
            } else {
                println!("=== OAuth2Resource Apply ===");
                oauth2_resource::print_apply(&result);
            }
        }
        Err(error) => errors.record(format!(
            "OAuth2Resource manifest {} apply failed: {error}",
            source.path.display()
        )),
    }
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

fn validate_cloud_apps_manifest(
    manifest: &serde_json::Value,
    app: Option<&str>,
    environment: &str,
) -> Result<()> {
    let entries = compute_cli::select_app_entries(manifest, app)?;
    for entry in entries {
        super::validate::validate_cloud_app_cache_declarations(&entry)?;
        let entry = compute_cli::resolve_app_entry_for_environment(&entry, environment)?;
        let _ = compute_cli::app_entry_to_api_body(&entry)?;
        let _ = compute_cli::plan_env_vars(&entry, environment)?;
    }
    Ok(())
}

fn load_auth_manifest_source(
    source: &super::discovery::ManifestSource,
) -> Result<auth_manifest::AuthManifest> {
    let yaml_value = serde_yaml::to_value(&source.document)?;
    auth_manifest::parse_manifest_document_value(yaml_value)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn validates_resolved_environment_overlay_before_base_entry() {
        let manifest = json!({
            "spec": {
                "apps": [{
                    "name": "fieldadmin",
                    "environments": {
                        "preview": {
                            "repository": {
                                "url": "https://github.com/quantum-box/tachyonfield",
                                "owner": "quantum-box",
                                "name": "tachyonfield"
                            }
                        }
                    }
                }]
            }
        });

        validate_cloud_apps_manifest(&manifest, Some("fieldadmin"), "preview").unwrap();
    }

    #[test]
    fn apply_validation_rejects_unsafe_cache_rules() {
        let manifest = json!({
            "spec": {
                "apps": [{
                    "name": "fieldadmin",
                    "cache": {
                        "rules": [{
                            "name": "public-docs",
                            "paths": ["https://other.example/docs/*"],
                            "methods": ["GET"],
                            "edgeTtl": "respect-origin"
                        }]
                    }
                }]
            }
        });

        let error = validate_cloud_apps_manifest(&manifest, Some("fieldadmin"), "preview")
            .expect_err("apply must reject unsafe cache declarations")
            .to_string();
        assert!(error.contains("origin-relative path pattern"), "{error}");
    }
}
