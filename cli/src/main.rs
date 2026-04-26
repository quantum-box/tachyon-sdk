mod agent_cli;
mod auth;
mod build_reproduce;
mod client;
mod compute_cli;
mod iac_cli;
mod image_cli;
mod install_cli;
mod mcp;
mod mcp_cli;
mod ops_cli;
mod org_cli;
mod resolve;
mod switch_cli;
mod tts_cli;

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
    #[arg(
        long,
        visible_alias = "operator",
        env = "TACHYON_TENANT_ID",
        default_value = ""
    )]
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
    /// Generate AI images from text prompts
    Image(image_cli::ImageArgs),
    /// Convert text to speech using AI TTS models
    Tts(tts_cli::TtsArgs),
    /// Run as an MCP (Model Context Protocol) server (stdio or HTTP)
    Mcp(mcp_cli::McpArgs),
    /// Update the Tachyon CLI to the latest version
    #[command(name = "self-update", visible_alias = "install")]
    SelfUpdate,
    /// Switch the active tenant (updates saved credentials)
    Switch(switch_cli::SwitchArgs),
}

/// Resolve the bearer token from CLI args or stored credentials.
/// If the stored token is expired, attempt to refresh it automatically.
async fn resolve_token(cli: &Cli) -> Option<String> {
    if cli.api_key.is_some() {
        return cli.api_key.clone();
    }
    match auth::load_credentials() {
        Ok(Some(creds)) => {
            let expired = creds
                .expires_at
                .map(|exp| chrono::Utc::now().timestamp() >= exp)
                .unwrap_or(false);

            if expired {
                let oauth_config = build_oauth_config(cli);
                match auth::refresh_access_token(&oauth_config, &creds).await {
                    Ok(new_creds) => {
                        eprintln!("Token refreshed successfully.");
                        return Some(new_creds.access_token);
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: token refresh failed: {e}. \
                             Run `tachyon login` to re-authenticate."
                        );
                        return None;
                    }
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

/// Build an OAuthConfig from CLI args.
fn build_oauth_config(cli: &Cli) -> auth::OAuthConfig {
    let redirect_uri = format!("{}/v1/auth/cli/callback", cli.api_url.trim_end_matches('/'));
    auth::OAuthConfig {
        cognito_domain: cli.cognito_domain.clone(),
        client_id: cli.cognito_client_id.clone(),
        client_secret: cli.cognito_client_secret.clone(),
        redirect_uri,
        scopes: vec!["openid".into(), "profile".into(), "email".into()],
    }
}

/// Build SDK configuration with the resolved token.
async fn build_config(cli: &Cli) -> Configuration {
    let mut config = Configuration::new();
    config.base_path = cli.api_url.clone();
    config.bearer_access_token = resolve_token(cli).await;
    config
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        // Show the full error chain (causes) without a backtrace.
        for cause in e.chain().skip(1) {
            eprintln!("  caused by: {cause}");
        }
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Login => {
            let oauth_config = build_oauth_config(&cli);
            auth::login(&oauth_config, &cli.api_url).await
        }
        Commands::Logout => auth::logout(),
        Commands::Compute(args) => {
            let config = build_config(&cli).await;
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            compute_cli::run(args, &config, &tenant_id).await
        }
        Commands::Org(args) => {
            let config = build_config(&cli).await;
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            org_cli::run(args, &config, &tenant_id).await
        }
        Commands::Agent(args) => {
            let config = build_config(&cli).await;
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            agent_cli::run(args, &config, &tenant_id).await
        }
        Commands::Iac(args) => {
            let config = build_config(&cli).await;
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            iac_cli::run(args, &config, &tenant_id).await
        }
        Commands::Ops(args) => {
            let config = build_config(&cli).await;
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            ops_cli::run(args, &config, &tenant_id).await
        }
        Commands::Image(args) => {
            let config = build_config(&cli).await;
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            image_cli::run(args, &config, &tenant_id).await
        }
        Commands::Tts(args) => {
            let config = build_config(&cli).await;
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id).await?;
            tts_cli::run(args, &config, &tenant_id).await
        }
        Commands::Mcp(args) => mcp_cli::run(args).await,
        Commands::SelfUpdate => install_cli::run().await,
        Commands::Switch(args) => {
            let config = build_config(&cli).await;
            switch_cli::run(args, &config).await
        }
    }
}
