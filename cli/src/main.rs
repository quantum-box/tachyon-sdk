mod agent_cli;
mod client;
mod compute_cli;
mod iac_cli;
mod ops_cli;
mod org_cli;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tachyon_sdk::apis::configuration::Configuration;

#[derive(Parser)]
#[command(name = "tachyon", version, about = "Tachyon Platform CLI")]
struct Cli {
    /// Tachyon API base URL
    #[arg(
        long,
        env = "TACHYON_API_URL",
        default_value = "https://api.tachyon.run"
    )]
    api_url: String,

    /// Tenant ID (x-operator-id header)
    #[arg(long, env = "TACHYON_TENANT_ID", default_value = "")]
    tenant_id: String,

    /// API key for authentication
    #[arg(long, env = "TACHYON_API_KEY")]
    api_key: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage compute apps and builds
    Compute(compute_cli::ComputeArgs),
    /// Manage organization: users, operators, policies
    Org(org_cli::OrgArgs),
    /// Manage AI agent sessions and models
    Agent(agent_cli::AgentArgs),
    /// Manage IaC integrations
    Iac(iac_cli::IacArgs),
    /// Operations: deployments and tool jobs
    Ops(ops_cli::OpsArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = Configuration::new();
    config.base_path = cli.api_url;
    config.bearer_access_token = cli.api_key;

    match cli.command {
        Commands::Compute(args) => compute_cli::run(&args, &config, &cli.tenant_id).await,
        Commands::Org(args) => org_cli::run(&args, &config, &cli.tenant_id).await,
        Commands::Agent(args) => agent_cli::run(&args, &config, &cli.tenant_id).await,
        Commands::Iac(args) => iac_cli::run(&args, &config, &cli.tenant_id).await,
        Commands::Ops(args) => ops_cli::run(&args, &config, &cli.tenant_id).await,
    }
}
