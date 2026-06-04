use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::Path;
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::ApiClient;
use crate::compute_cli;
use crate::config::loader::ProjectConfig;
use crate::resolve;

#[derive(Debug, Clone, Args)]
pub struct SecretArgs {
    #[command(subcommand)]
    pub command: SecretCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum SecretCommand {
    /// Set a runtime secret for a Cloud App (CF Pages or ASM by deployment target)
    Set {
        /// App ID or name
        app: String,
        /// Environment variable name (also used as CF Pages secret key)
        key: String,
        /// Target environment
        #[arg(long, default_value = "all")]
        target: String,
    },
}

pub async fn run(
    args: &SecretArgs,
    config: &Configuration,
    tenant_id: &str,
    project_config: Option<&ProjectConfig>,
    config_flag: Option<&Path>,
) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;
    match &args.command {
        SecretCommand::Set { app, key, target } => {
            let app_id = resolve::resolve_app_id(&api, app).await?;
            let app_name = project_config
                .and_then(|cfg| cfg.metadata.name.as_deref())
                .unwrap_or(app.as_str());
            compute_cli::run_env_set_secret(
                &api,
                &app_id,
                app_name,
                key,
                target,
                config_flag,
            )
            .await
        }
    }
}
