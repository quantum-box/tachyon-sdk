mod apply;
mod discovery;
mod plan;
mod reconcile;
mod validate;

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};
use tachyon_sdk::apis::configuration::Configuration;

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
    /// Required approval token for production CloudApps apply.
    ///
    /// This only gates write execution. The token is never printed or sent
    /// to the Cloud Apps API by the CLI.
    #[arg(
        long = "change-control-token",
        env = "TACHYON_CHANGE_CONTROL_APPROVAL_TOKEN",
        hide_env_values = true
    )]
    pub change_control_token: Option<String>,
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

pub async fn run(
    args: &ManifestArgs,
    config: Option<&Configuration>,
    tenant_id: Option<&str>,
) -> Result<()> {
    match &args.command {
        ManifestCommand::Validate(validate_args) => validate::run(validate_args),
        ManifestCommand::Plan(apply_args) => {
            let config = config.ok_or_else(|| anyhow!("manifest plan requires tenant context"))?;
            let tenant_id =
                tenant_id.ok_or_else(|| anyhow!("manifest plan requires tenant context"))?;
            plan::run(apply_args, config, tenant_id).await
        }
        ManifestCommand::Apply(apply_args) => {
            let config = config.ok_or_else(|| anyhow!("manifest apply requires tenant context"))?;
            let tenant_id =
                tenant_id.ok_or_else(|| anyhow!("manifest apply requires tenant context"))?;
            apply::run(apply_args, config, tenant_id, apply_args.dry_run, "apply").await
        }
        ManifestCommand::Reconcile(apply_args) => {
            let config =
                config.ok_or_else(|| anyhow!("manifest reconcile requires tenant context"))?;
            let tenant_id =
                tenant_id.ok_or_else(|| anyhow!("manifest reconcile requires tenant context"))?;
            reconcile::run(apply_args, config, tenant_id).await
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
    reconcile::run_alias(args, config, tenant_id).await
}
