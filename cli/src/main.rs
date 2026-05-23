mod agent_cli;
mod auth;
mod build_reproduce;
mod client;
mod commands;
mod compute_cli;
mod config;
mod iac_cli;
mod image_cli;
mod install_cli;
mod mcp;
mod mcp_cli;
mod ops_cli;
mod org_cli;
mod reconcile_cli;
mod resolve;
mod switch_cli;
mod tts_cli;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
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
        default_value = "",
        global = true
    )]
    tenant_id: String,

    /// API key for authentication (overrides stored OAuth token)
    #[arg(long, env = "TACHYON_API_KEY")]
    api_key: Option<String>,

    /// Auth profile to use for this command (overrides the active profile).
    #[arg(long, global = true, env = "TACHYON_PROFILE")]
    profile: Option<String>,

    /// Project config file (overridden by TACHYON_CONFIG when set)
    #[arg(long, global = true)]
    config: Option<PathBuf>,

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tenant_id_after_nested_ops_command() {
        let cli = Cli::try_parse_from([
            "tachyon",
            "ops",
            "slack",
            "send",
            "--tenant-id",
            "tn_test",
            "--text",
            "hello",
        ])
        .unwrap();

        assert_eq!(cli.tenant_id, "tn_test");
        match cli.command {
            Commands::Ops(ops_cli::OpsArgs {
                command:
                    ops_cli::OpsCommand::Notify {
                        command: ops_cli::NotifyCommand::Send { text, json },
                    },
            }) => {
                assert_eq!(text, "hello");
                assert!(!json);
            }
            _ => panic!("expected ops slack send command"),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Manage authentication profiles (login / logout / list / use)
    Auth(AuthArgs),
    /// Log in via browser-based OAuth (alias for `auth login`)
    Login(LoginArgs),
    /// Remove stored credentials (alias for `auth logout`)
    Logout(LogoutArgs),
    /// Manage compute apps, builds, deployments, and configuration
    Compute(compute_cli::ComputeArgs),
    /// Generate a tachyon.yml project config
    Init(commands::init::InitArgs),
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
    /// Reconcile Cloud Apps and auth manifest with Tachyon API
    Reconcile(reconcile_cli::ReconcileArgs),
}

#[derive(Args)]
struct AuthArgs {
    #[command(subcommand)]
    command: AuthCommand,
}

#[derive(Subcommand)]
enum AuthCommand {
    /// Log in via browser-based OAuth and save tokens to a profile
    Login(LoginArgs),
    /// Register an auth provider in tachyon.yml
    Init(commands::auth::init::InitAuthArgs),
    /// Issue credentials for a registered auth provider
    Issue(commands::auth::issue::IssueAuthArgs),
    /// Remove a profile's stored credentials
    Logout(LogoutArgs),
    /// List registered profiles (active marked with *)
    List,
    /// Switch the active profile (must already be logged in)
    Use(UseArgs),
    /// Manage auth manifest (custom actions and policies as code)
    Manifest(commands::auth::manifest::ManifestArgs),
}

#[derive(Args)]
struct LoginArgs {
    /// Profile name to save tokens under (default: the active profile or "default").
    /// On `auth login`, this overrides the global `--profile`.
    #[arg(long)]
    profile: Option<String>,
}

#[derive(Args)]
struct LogoutArgs {
    /// Profile name to log out of (default: the active profile).
    /// On `auth logout`, this overrides the global `--profile`.
    #[arg(long)]
    profile: Option<String>,
}

#[derive(Args)]
struct UseArgs {
    /// Profile name to make active.
    profile: String,
}

/// Resolve the bearer token from CLI args or stored credentials for the given
/// profile. If the stored token is expired, attempt to refresh it automatically
/// (writing back to the same profile).
async fn resolve_token(cli: &Cli, profile: &str) -> Option<String> {
    if cli.api_key.is_some() {
        return cli.api_key.clone();
    }
    match auth::load_profile(profile) {
        Ok(Some(creds)) => {
            let expired = creds
                .expires_at
                .map(|exp| chrono::Utc::now().timestamp() >= exp)
                .unwrap_or(false);

            if expired {
                let oauth_config = build_oauth_config(cli);
                match auth::refresh_access_token(&oauth_config, profile, &creds).await {
                    Ok(new_creds) => {
                        eprintln!("Token refreshed successfully (profile: {profile}).");
                        return Some(new_creds.access_token);
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: token refresh failed for profile '{profile}': {e}. \
                             Run `tachyon auth login --profile {profile}` to re-authenticate."
                        );
                        return None;
                    }
                }
            }

            Some(creds.access_token)
        }
        Ok(None) => None,
        Err(e) => {
            eprintln!("Warning: failed to load profile '{profile}': {e}");
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

/// Build SDK configuration with the resolved token for the given profile.
async fn build_config(cli: &Cli, profile: &str) -> Configuration {
    let mut config = Configuration::new();
    config.base_path = cli.api_url.clone();
    config.bearer_access_token = resolve_token(cli, profile).await;
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

    // Resolve the profile to use for this invocation. For most commands this
    // is the active profile (or the value of --profile / TACHYON_PROFILE for a
    // one-shot override). `auth login` / `auth use` choose their own target
    // profile, so they ignore `active`.
    let active = auth::resolve_active_profile(cli.profile.as_deref())?;

    match &cli.command {
        Commands::Auth(args) => match &args.command {
            AuthCommand::Login(login_args) => {
                let oauth_config = build_oauth_config(&cli);
                let target = login_args
                    .profile
                    .clone()
                    .or_else(|| cli.profile.clone())
                    .unwrap_or_else(|| auth::DEFAULT_PROFILE.to_string());
                auth::login(&oauth_config, &cli.api_url, &target).await
            }
            AuthCommand::Init(init_args) => {
                commands::auth::init::run(init_args, cli.config.as_deref())
            }
            AuthCommand::Issue(issue_args) => {
                let project_config = config::loader::load(cli.config.as_deref())?;
                let tenant_arg = tenant_arg(&cli, project_config.as_ref()).to_string();
                let token = resolve_token(&cli, &active).await;
                commands::auth::issue::run(
                    issue_args,
                    cli.config.as_deref(),
                    &cli.api_url,
                    token,
                    &tenant_arg,
                )
                .await
            }
            AuthCommand::Logout(logout_args) => {
                let target = logout_args
                    .profile
                    .clone()
                    .or_else(|| cli.profile.clone())
                    .unwrap_or(active.clone());
                auth::logout(&target)
            }
            AuthCommand::List => auth::list_profiles_command(),
            AuthCommand::Use(use_args) => auth::use_profile(&use_args.profile),
            AuthCommand::Manifest(manifest_args) => {
                let project_config = config::loader::load(cli.config.as_deref())?;
                let tenant_arg = tenant_arg(&cli, project_config.as_ref());
                let config = build_config(&cli, &active).await;
                let tenant_id =
                    resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
                commands::auth::manifest::run(manifest_args, &config, &tenant_id).await
            }
        },
        Commands::Login(login_args) => {
            let oauth_config = build_oauth_config(&cli);
            let target = login_args
                .profile
                .clone()
                .or_else(|| cli.profile.clone())
                .unwrap_or_else(|| auth::DEFAULT_PROFILE.to_string());
            auth::login(&oauth_config, &cli.api_url, &target).await
        }
        Commands::Logout(logout_args) => {
            let target = logout_args
                .profile
                .clone()
                .or_else(|| cli.profile.clone())
                .unwrap_or(active.clone());
            auth::logout(&target)
        }
        Commands::Compute(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            compute_cli::run(args, &config, &tenant_id, project_config.as_ref()).await
        }
        Commands::Org(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            org_cli::run(args, &config, &tenant_id).await
        }
        Commands::Agent(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            agent_cli::run(args, &config, &tenant_id).await
        }
        Commands::Iac(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            iac_cli::run(args, &config, &tenant_id).await
        }
        Commands::Ops(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            ops_cli::run(args, &config, &tenant_id).await
        }
        Commands::Image(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            image_cli::run(args, &config, &tenant_id).await
        }
        Commands::Tts(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            tts_cli::run(args, &config, &tenant_id).await
        }
        Commands::Mcp(args) => mcp_cli::run(args).await,
        Commands::SelfUpdate => install_cli::run().await,
        Commands::Init(args) => commands::init::run(args),
        Commands::Switch(args) => {
            let config = build_config(&cli, &active).await;
            switch_cli::run(args, &config, &active).await
        }
        Commands::Reconcile(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            reconcile_cli::run(args, &config, &tenant_id).await
        }
    }
}

fn tenant_arg<'a>(
    cli: &'a Cli,
    project_config: Option<&'a config::loader::ProjectConfig>,
) -> &'a str {
    if !cli.tenant_id.is_empty() {
        return cli.tenant_id.as_str();
    }

    project_config
        .and_then(|config| config.metadata.tenant_id.as_deref())
        .unwrap_or("")
}
