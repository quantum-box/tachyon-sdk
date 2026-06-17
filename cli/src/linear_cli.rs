//! Compatibility alias for tenant-scoped Linear issue operations.

use anyhow::Result;
use clap::{Args, Subcommand};
use tachyon_sdk::apis::configuration::Configuration;

use crate::pm_cli::{self, IssueCommand};

#[derive(Debug, Clone, Args)]
pub struct LinearArgs {
    #[command(subcommand)]
    pub command: LinearCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum LinearCommand {
    /// Manage Linear issues
    Issue {
        #[command(subcommand)]
        command: IssueCommand,
    },
}

pub async fn run(args: &LinearArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    match &args.command {
        LinearCommand::Issue { command } => {
            pm_cli::run_issue(command, config, tenant_id, Some("linear")).await
        }
    }
}
