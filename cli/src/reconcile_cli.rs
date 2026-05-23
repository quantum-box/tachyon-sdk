use anyhow::Result;
use clap::Args;
use serde_json::Value;
use std::path::PathBuf;
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::ApiClient;
use crate::commands::auth::manifest as auth_manifest;
use crate::compute_cli;

#[derive(Debug, Clone, Args)]
pub struct ReconcileArgs {
    /// Cloud Apps manifest file (defaults to tachyon.yml; skipped if absent or non-CloudApps)
    #[arg(short = 'f', long, default_value = "tachyon.yml")]
    pub file: PathBuf,

    /// Preview changes without making API calls (plan mode)
    #[arg(long)]
    pub dry_run: bool,

    /// Target app name to select from a multi-app CloudApps manifest
    #[arg(long)]
    pub app: Option<String>,

    /// Environment label for CloudApps reconcile
    #[arg(long, default_value = "sandbox")]
    pub environment: String,

    /// Remove resources absent from the CloudApps manifest
    /// (auth prune is unsupported – no delete endpoint)
    #[arg(long)]
    pub prune: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub async fn run(args: &ReconcileArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    // --- 1. Cloud Apps reconcile ---
    // Only run if the file exists and contains a CloudApps / CloudApp manifest.
    if is_cloud_apps_manifest(&args.file) {
        if !args.json {
            println!("=== Cloud Apps Reconcile ===");
        }
        let apps_args = compute_cli::ComputeArgs {
            command: compute_cli::ComputeCommand::Apps {
                command: compute_cli::AppsCommand::Apply {
                    file: args.file.clone(),
                    app: args.app.clone(),
                    environment: args.environment.clone(),
                    dry_run: args.dry_run,
                },
            },
        };
        compute_cli::run(&apps_args, config, tenant_id, None, None).await?;
        if !args.json {
            println!();
        }
    } else if !args.json {
        println!(
            "Cloud Apps: {} not found or not a CloudApps manifest, skipped.",
            args.file.display()
        );
    }

    // --- 2. Auth manifest reconcile ---
    // Auto-discovers auth manifest from tachyon.yml auth.manifest section
    // and .tachyon/manifests/**/*.yml.  Silently skipped if nothing is found.
    let auth_result = auth_manifest::reconcile(
        &api,
        args.dry_run,
        None, // auto-discovery; --file is for Cloud Apps
        args.prune,
        args.json,
    )
    .await?;

    if auth_result.is_none() && !args.json {
        println!("Auth manifest: none found, skipped.");
    }

    Ok(())
}

/// Returns true when `path` is a YAML file whose top-level `kind` is
/// `CloudApps` or `CloudApp`.  Never panics – any I/O or parse failure
/// returns false so the reconcile step is silently skipped.
fn is_cloud_apps_manifest(path: &std::path::Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(value) = serde_yaml::from_str::<Value>(&content) else {
        return false;
    };
    matches!(
        value.get("kind").and_then(Value::as_str),
        Some("CloudApps") | Some("CloudApp")
    )
}
