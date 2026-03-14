mod auth;
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

    /// API key for authentication (overrides stored OAuth token)
    #[arg(long, env = "TACHYON_API_KEY")]
    api_key: Option<String>,

    /// Cognito domain URL (e.g. https://your-domain.auth.ap-northeast-1.amazoncognito.com)
    #[arg(long, env = "TACHYON_COGNITO_DOMAIN")]
    cognito_domain: Option<String>,

    /// Cognito OAuth client ID
    #[arg(long, env = "TACHYON_COGNITO_CLIENT_ID")]
    cognito_client_id: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Log in via browser-based OAuth (Cognito Hosted UI)
    Login,
    /// Remove stored credentials
    Logout,
    /// Manage compute apps and builds
    Compute(compute_cli::ComputeArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login => {
            let cognito_domain = cli.cognito_domain.ok_or_else(|| {
                anyhow::anyhow!(
                    "Cognito domain is required for login. Set --cognito-domain or TACHYON_COGNITO_DOMAIN"
                )
            })?;
            let client_id = cli.cognito_client_id.ok_or_else(|| {
                anyhow::anyhow!(
                    "Cognito client ID is required for login. Set --cognito-client-id or TACHYON_COGNITO_CLIENT_ID"
                )
            })?;

            let oauth_config = auth::OAuthConfig {
                cognito_domain,
                client_id,
                scopes: vec!["openid".into(), "profile".into(), "email".into()],
            };
            auth::login(&oauth_config).await
        }
        Commands::Logout => auth::logout(),
        Commands::Compute(args) => {
            let mut config = Configuration::new();
            config.base_path = cli.api_url;

            // API key takes precedence; otherwise try stored OAuth token
            config.bearer_access_token = if cli.api_key.is_some() {
                cli.api_key
            } else {
                match auth::load_credentials() {
                    Ok(Some(creds)) => {
                        // Warn if token is expired
                        if let Some(exp) = creds.expires_at {
                            let now = chrono::Utc::now().timestamp();
                            if now >= exp {
                                eprintln!(
                                    "Warning: stored token has expired. Run `tachyon login` to re-authenticate."
                                );
                            }
                        }
                        Some(creds.access_token)
                    }
                    Ok(None) => None,
                    Err(e) => {
                        eprintln!("Warning: failed to load stored credentials: {e}");
                        None
                    }
                }
            };

            compute_cli::run(&args, &config, &cli.tenant_id).await
        }
    }
}
