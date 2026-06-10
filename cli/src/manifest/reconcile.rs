use anyhow::Result;
use tachyon_sdk::apis::configuration::Configuration;

use super::{apply, ApplyArgs, ManifestArgs, ManifestCommand};

pub(crate) async fn run(args: &ApplyArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    apply::run(args, config, tenant_id, args.dry_run, "reconcile").await
}

pub(crate) async fn run_alias(
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
    super::run(&manifest_args, Some(config), Some(tenant_id)).await
}
