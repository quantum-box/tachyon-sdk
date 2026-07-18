//! Compatibility alias for tenant-scoped Linear issue operations.

use anyhow::Result;
use clap::{Args, Subcommand};
use tachyon_sdk::apis::configuration::Configuration;

use crate::pm_cli::{self, IssueCommand};
use crate::pm_resource_cli::{self, ResourceCommand};

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
        command: Box<IssueCommand>,
    },
    /// Manage Linear projects
    Project {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage Linear initiatives
    Initiative {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage Linear cycles
    Cycle {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage Linear teams
    Team {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage Linear issue labels
    Label {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage Linear documents
    Document {
        #[command(subcommand)]
        command: ResourceCommand,
    },
    /// Manage Linear project milestones
    Milestone {
        #[command(subcommand)]
        command: ResourceCommand,
    },
}

pub async fn run(args: &LinearArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    match &args.command {
        LinearCommand::Issue { command } => {
            pm_cli::run_issue(command, config, tenant_id, Some("linear")).await
        }
        LinearCommand::Project { command } => {
            pm_resource_cli::run_resource("projects", command, config, tenant_id, Some("linear"))
                .await
        }
        LinearCommand::Initiative { command } => {
            pm_resource_cli::run_resource("initiatives", command, config, tenant_id, Some("linear"))
                .await
        }
        LinearCommand::Cycle { command } => {
            pm_resource_cli::run_resource("cycles", command, config, tenant_id, Some("linear"))
                .await
        }
        LinearCommand::Team { command } => {
            pm_resource_cli::run_resource("teams", command, config, tenant_id, Some("linear")).await
        }
        LinearCommand::Label { command } => {
            pm_resource_cli::run_resource("labels", command, config, tenant_id, Some("linear"))
                .await
        }
        LinearCommand::Document { command } => {
            pm_resource_cli::run_resource("documents", command, config, tenant_id, Some("linear"))
                .await
        }
        LinearCommand::Milestone { command } => {
            pm_resource_cli::run_resource("milestones", command, config, tenant_id, Some("linear"))
                .await
        }
    }
}
