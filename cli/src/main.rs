mod compute_cli;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = Configuration::new();
    config.base_path = cli.api_url;
    config.bearer_access_token = cli.api_key;

    match cli.command {
        Commands::Compute(args) => compute_cli::run(&args, &config, &cli.tenant_id).await,
    }
}
