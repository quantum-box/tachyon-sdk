mod agent_cli;
mod auth;
mod client;
mod compute_cli;
mod iac_cli;
mod ops_cli;
mod org_cli;
mod resolve;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tachyon_sdk::apis::configuration::Configuration;

/// Default Cognito domain for the Tachyon production environment.
const DEFAULT_COGNITO_DOMAIN: &str = "https://auth-pool.n1.tachy.one";
/// Default Cognito client ID (tachyon-app-client).
const DEFAULT_COGNITO_CLIENT_ID: &str = "5002hok6cj8mjmt3gepdpdq98i";
/// Default Cognito client secret (tachyon-app-client).
const DEFAULT_COGNITO_CLIENT_SECRET: &str = "3epft46iie79jshd4gkpeuj62q1pcmthequ1skbbd9dj1rdojrf";

#[derive(Parser)]
#[command(name = "tachyon", version, about = "Tachyon Platform CLI")]
struct Cli {
    /// Tachyon API base URL
    #[arg(
        long,
        env = "TACHYON_API_URL",
        default_value = "https://api.n1.tachy.one"
    )]
    api_url: String,

    /// Tenant ID or name/alias (x-operator-id header)
    #[arg(long, env = "TACHYON_TENANT_ID", default_value = "")]
    tenant_id: String,

    /// API key for authentication (overrides stored OAuth token)
    #[arg(long, env = "TACHYON_API_KEY")]
    api_key: Option<String>,

    /// Cognito domain URL (e.g. https://your-domain.auth.ap-northeast-1.amazoncognito.com)
    #[arg(long, env = "TACHYON_COGNITO_DOMAIN", default_value = DEFAULT_COGNITO_DOMAIN)]
    cognito_domain: String,

    /// Cognito OAuth client ID
    #[arg(long, env = "TACHYON_COGNITO_CLIENT_ID", default_value = DEFAULT_COGNITO_CLIENT_ID)]
    cognito_client_id: String,

    /// Cognito OAuth client secret
    #[arg(long, env = "TACHYON_COGNITO_CLIENT_SECRET", default_value = DEFAULT_COGNITO_CLIENT_SECRET)]
    cognito_client_secret: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Log in via browser-based OAuth (Cognito Hosted UI)
    Login,
    /// Remove stored credentials
    Logout,
    /// Manage compute apps, builds, deployments, and configuration
    Compute(compute_cli::ComputeArgs),
    /// Manage organizations, users, service accounts, and policies
    Org(org_cli::OrgArgs),
    /// Manage agent sessions, protocols, workers, and memory
    Agent(agent_cli::AgentArgs),
    /// Infrastructure-as-Code: integrations, OAuth providers, connections
    Iac(iac_cli::IacArgs),
    /// Operations: deployment events, scenario reports, and tool jobs
    Ops(ops_cli::OpsArgs),
}

/// Resolve the bearer token from CLI args or stored credentials.
fn resolve_token(cli: &Cli) -> Option<String> {
    if cli.api_key.is_some() {
        return cli.api_key.clone();
    }
    match auth::load_credentials() {
        Ok(Some(creds)) => {
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
}

/// Build SDK configuration with the resolved token.
fn build_config(cli: &Cli) -> Configuration {
    let mut config = Configuration::new();
    config.base_path = cli.api_url.clone();
    config.bearer_access_token = resolve_token(cli);
    config
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Login => {
            let redirect_uri =
                format!("{}/v1/auth/cli/callback", cli.api_url.trim_end_matches('/'));
            let oauth_config = auth::OAuthConfig {
                cognito_domain: cli.cognito_domain.clone(),
                client_id: cli.cognito_client_id.clone(),
                client_secret: cli.cognito_client_secret.clone(),
                redirect_uri,
                scopes: vec!["openid".into(), "profile".into(), "email".into()],
            };
            auth::login(&oauth_config, &cli.api_url).await
        }
        Commands::Logout => auth::logout(),
        Commands::Compute(args) => {
            let config = build_config(&cli);
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            compute_cli::run(args, &config, &tenant_id).await
        }
        Commands::Org(args) => {
            let config = build_config(&cli);
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            org_cli::run(args, &config, &tenant_id).await
        }
        Commands::Agent(args) => {
            let config = build_config(&cli);
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            agent_cli::run(args, &config, &tenant_id).await
        }
        Commands::Iac(args) => {
            let config = build_config(&cli);
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            iac_cli::run(args, &config, &tenant_id).await
        }
        Commands::Ops(args) => {
            let config = build_config(&cli);
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            ops_cli::run(args, &config, &tenant_id).await
        }
    }
}
