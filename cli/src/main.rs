mod agent_cli;
mod api_key_cli;
mod auth;
mod build_reproduce;
mod client;
mod cloud_app_build_job;
mod commands;
mod compute_cli;
mod config;
mod iac_cli;
mod image_cli;
mod install_cli;
mod linear_cli;
mod manifest;
mod mcp;
mod mcp_cli;
mod ops_cli;
mod org_cli;
mod pm_cli;
mod reconcile_cli;
mod resolve;
mod secret_cli;
mod skills_cli;
mod slack_cli;
mod switch_cli;
mod tts_cli;
mod worker_cli;

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use std::path::{Path, PathBuf};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::AuthDiagnostics;

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
                        command:
                            ops_cli::NotifyCommand::Send {
                                text,
                                mentions,
                                json,
                            },
                    },
            }) => {
                assert_eq!(text, "hello");
                assert!(mentions.is_empty());
                assert!(!json);
            }
            _ => panic!("expected ops slack send command"),
        }
    }

    #[test]
    fn parses_ops_notify_send_mentions() {
        let cli = Cli::try_parse_from([
            "tachyon",
            "ops",
            "notify",
            "send",
            "--text",
            "hello",
            "--mention",
            "U123",
            "--mention",
            "user@example.com",
        ])
        .unwrap();

        match cli.command {
            Commands::Ops(ops_cli::OpsArgs {
                command:
                    ops_cli::OpsCommand::Notify {
                        command:
                            ops_cli::NotifyCommand::Send {
                                text,
                                mentions,
                                json,
                            },
                    },
            }) => {
                assert_eq!(text, "hello");
                assert_eq!(mentions, ["U123", "user@example.com"]);
                assert!(!json);
            }
            _ => panic!("expected ops notify send command"),
        }
    }

    #[test]
    fn parses_ops_notify_users_command() {
        let cli = Cli::try_parse_from(["tachyon", "ops", "notify", "users", "--json"]).unwrap();

        match cli.command {
            Commands::Ops(ops_cli::OpsArgs {
                command:
                    ops_cli::OpsCommand::Notify {
                        command: ops_cli::NotifyCommand::Users { bot_token, json },
                    },
            }) => {
                assert!(bot_token.is_none());
                assert!(json);
            }
            _ => panic!("expected ops notify users command"),
        }
    }

    #[test]
    fn parses_ops_sentry_issues_list_command() {
        let cli = Cli::try_parse_from([
            "tachyon",
            "ops",
            "sentry",
            "issues",
            "list",
            "--project",
            "fieldadmin",
            "--query",
            "is:unresolved",
            "--limit",
            "25",
            "--json",
        ])
        .unwrap();

        match cli.command {
            Commands::Ops(ops_cli::OpsArgs {
                command:
                    ops_cli::OpsCommand::Sentry {
                        command:
                            ops_cli::SentryCommand::Issues {
                                command:
                                    ops_cli::SentryIssuesCommand::List {
                                        project,
                                        query,
                                        limit,
                                        json,
                                    },
                            },
                    },
            }) => {
                assert_eq!(project.as_deref(), Some("fieldadmin"));
                assert_eq!(query.as_deref(), Some("is:unresolved"));
                assert_eq!(limit, Some(25));
                assert!(json);
            }
            _ => panic!("expected ops sentry issues list command"),
        }
    }

    #[test]
    fn parses_ops_sentry_issue_alias_commands() {
        let view =
            Cli::try_parse_from(["tachyon", "ops", "sentry", "issue", "view", "12345"]).unwrap();
        match view.command {
            Commands::Ops(ops_cli::OpsArgs {
                command:
                    ops_cli::OpsCommand::Sentry {
                        command:
                            ops_cli::SentryCommand::Issues {
                                command: ops_cli::SentryIssuesCommand::View { issue_id, json },
                            },
                    },
            }) => {
                assert_eq!(issue_id, "12345");
                assert!(!json);
            }
            _ => panic!("expected ops sentry issue view command"),
        }

        let assign = Cli::try_parse_from([
            "tachyon",
            "ops",
            "sentry",
            "issue",
            "assign",
            "12345",
            "user@example.com",
            "--json",
        ])
        .unwrap();
        match assign.command {
            Commands::Ops(ops_cli::OpsArgs {
                command:
                    ops_cli::OpsCommand::Sentry {
                        command:
                            ops_cli::SentryCommand::Issues {
                                command:
                                    ops_cli::SentryIssuesCommand::Assign {
                                        issue_id,
                                        user,
                                        json,
                                    },
                            },
                    },
            }) => {
                assert_eq!(issue_id, "12345");
                assert_eq!(user, "user@example.com");
                assert!(json);
            }
            _ => panic!("expected ops sentry issue assign command"),
        }
    }

    #[test]
    fn parses_top_level_slack_send_command() {
        let cli = Cli::try_parse_from([
            "tachyon",
            "slack",
            "send",
            "--tenant-id",
            "tn_test",
            "--integration",
            "int_slack",
            "--channel",
            "#tachyon-test",
            "--message",
            "hello",
        ])
        .unwrap();

        assert_eq!(cli.tenant_id, "tn_test");
        match cli.command {
            Commands::Slack(slack_cli::SlackArgs {
                command:
                    slack_cli::SlackCommand::Send {
                        integration,
                        channel,
                        message,
                        json,
                    },
            }) => {
                assert_eq!(integration, "int_slack");
                assert_eq!(channel, "#tachyon-test");
                assert_eq!(message, "hello");
                assert!(!json);
            }
            _ => panic!("expected slack send command"),
        }
    }

    #[test]
    fn parses_top_level_api_key_create_command() {
        let cli = Cli::try_parse_from([
            "tachyon",
            "api-key",
            "create",
            "sa_123456789012",
            "--name",
            "CEO key",
            "--json",
        ])
        .unwrap();

        match cli.command {
            Commands::ApiKey(api_key_cli::ApiKeyArgs {
                command:
                    api_key_cli::ApiKeyCommand::Create {
                        service_account,
                        name,
                        json,
                    },
            }) => {
                assert_eq!(service_account, "sa_123456789012");
                assert_eq!(name, "CEO key");
                assert!(json);
            }
            _ => panic!("expected api-key create command"),
        }
    }

    #[test]
    fn parses_pm_issue_and_top_level_issue_to_same_command() {
        let pm_cli = Cli::try_parse_from([
            "tachyon",
            "pm",
            "issue",
            "create",
            "--title",
            "Implement PM issue CLI",
            "--provider",
            "linear",
            "--team",
            "PLT",
            "--priority",
            "high",
            "--json",
        ])
        .unwrap();
        let issue_cli = Cli::try_parse_from([
            "tachyon",
            "issue",
            "create",
            "--title",
            "Implement PM issue CLI",
            "--provider",
            "linear",
            "--team",
            "PLT",
            "--priority",
            "high",
            "--json",
        ])
        .unwrap();

        let pm_command = match pm_cli.command {
            Commands::Pm(pm_cli::PmArgs {
                command: pm_cli::PmCommand::Issue { command },
            }) => command,
            _ => panic!("expected pm issue command"),
        };
        let issue_command = match issue_cli.command {
            Commands::Issue(pm_cli::IssueArgs { command }) => command,
            _ => panic!("expected top-level issue command"),
        };
        assert_eq!(pm_command, issue_command);
    }

    #[test]
    fn parses_linear_issue_as_compatibility_alias() {
        let cli = Cli::try_parse_from([
            "tachyon",
            "linear",
            "issue",
            "create",
            "--team-id",
            "team_1",
            "--title",
            "Compat issue",
            "--priority",
            "2",
        ])
        .unwrap();

        match cli.command {
            Commands::Linear(linear_cli::LinearArgs {
                command:
                    linear_cli::LinearCommand::Issue {
                        command:
                            pm_cli::IssueCommand::Create {
                                provider,
                                team_id,
                                title,
                                priority,
                                ..
                            },
                    },
            }) => {
                assert_eq!(provider, None);
                assert_eq!(team_id.as_deref(), Some("team_1"));
                assert_eq!(title, "Compat issue");
                assert_eq!(priority.as_deref(), Some("2"));
            }
            _ => panic!("expected linear issue create command"),
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
    /// Manage Cloud App environment variables
    Env(compute_cli::EnvArgs),
    /// Generate a tachyon.yml project config
    Init(commands::init::InitArgs),
    /// Manage organizations, users, service accounts, and policies
    Org(org_cli::OrgArgs),
    /// Manage service-account API keys
    #[command(name = "api-key")]
    ApiKey(api_key_cli::ApiKeyArgs),
    /// Manage Cloudflare Pages secrets
    Secret(secret_cli::SecretArgs),
    /// Manage agent sessions, protocols, workers, and memory
    Agent(agent_cli::AgentArgs),
    /// Install and run the local Tachyon worker daemon
    Worker(worker_cli::WorkerArgs),
    /// Infrastructure-as-Code: integrations, OAuth providers, connections
    Iac(iac_cli::IacArgs),
    /// Validate, plan, apply, and reconcile manifests
    Manifest(manifest::ManifestArgs),
    /// Operations: deployment events, scenario reports, and coding jobs
    Ops(ops_cli::OpsArgs),
    /// Generate AI images from text prompts
    Image(image_cli::ImageArgs),
    /// Send Slack messages through connected integrations
    Slack(slack_cli::SlackArgs),
    /// Manage project-management providers and issues
    Pm(pm_cli::PmArgs),
    /// Manage issues in the default project-management provider
    Issue(pm_cli::IssueArgs),
    /// Manage Linear issues through connected integrations
    Linear(linear_cli::LinearArgs),
    /// Install bundled agent skills
    Skills(skills_cli::SkillsArgs),
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
async fn resolve_token(cli: &Cli, profile: &str) -> Option<auth::ApiBearerToken> {
    if cli.api_key.is_some() {
        return cli.api_key.clone().map(|value| auth::ApiBearerToken {
            value,
            kind: auth::ApiTokenKind::ApiKey,
        });
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
                        let selected = auth::select_api_bearer_token(&new_creds);
                        let token_kind = selected
                            .as_ref()
                            .map(|token| token.kind.as_str())
                            .unwrap_or("none");
                        eprintln!(
                            "Token refreshed successfully (profile: {profile}, api_token={token_kind})."
                        );
                        return selected;
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

            auth::select_api_bearer_token(&creds)
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
        scopes: auth::DEFAULT_OAUTH_SCOPES
            .iter()
            .map(|scope| (*scope).to_string())
            .collect(),
    }
}

/// Build SDK configuration with the resolved token for the given profile.
async fn build_config(cli: &Cli, profile: &str) -> Configuration {
    let mut config = Configuration::new();
    config.base_path = cli.api_url.clone();
    config.bearer_access_token = resolve_token(cli, profile).await.map(|token| token.value);
    config
}

fn profile_operator_id(profile: &str) -> Result<Option<String>> {
    Ok(auth::load_profile(profile)?.and_then(|creds| creds.operator_id))
}

fn profile_for_tenant(cli: &Cli, active: &str, tenant_id: &str) -> Result<String> {
    if cli.profile.is_some() || cli.api_key.is_some() || tenant_id.trim().is_empty() {
        return Ok(active.to_string());
    }

    if profile_operator_id(active)?.as_deref() == Some(tenant_id) {
        return Ok(active.to_string());
    }

    let mut matches = Vec::new();
    for profile in auth::list_profiles()? {
        if profile == active {
            continue;
        }
        if profile_operator_id(&profile)?.as_deref() == Some(tenant_id) {
            matches.push(profile);
        }
    }

    if matches.len() == 1 {
        let selected = matches.remove(0);
        eprintln!("Using auth profile '{selected}' for tenant '{tenant_id}'.");
        return Ok(selected);
    }

    Ok(active.to_string())
}

async fn build_tenant_config(
    cli: &Cli,
    active: &str,
    tenant_arg: &str,
) -> Result<(Configuration, String)> {
    let initial_config = build_config(cli, active).await;
    let tenant_id = resolve::resolve_tenant_id(&initial_config, tenant_arg, active).await?;
    let profile = profile_for_tenant(cli, active, &tenant_id)?;
    if profile == active {
        return Ok((initial_config, tenant_id));
    }

    auth::set_runtime_auth_context(profile.clone(), build_oauth_config(cli));
    Ok((build_config(cli, &profile).await, tenant_id))
}

async fn build_config_with_auth(
    cli: &Cli,
    profile: &str,
) -> (Configuration, Option<AuthDiagnostics>) {
    let mut config = Configuration::new();
    config.base_path = cli.api_url.clone();
    let resolved = resolve_token(cli, profile).await;
    let diagnostics = Some(AuthDiagnostics {
        profile: Some(profile.to_string()),
        token_kind: resolved
            .as_ref()
            .map(|token| token.kind.as_str().to_string()),
        oauth_client_configured: !cli.cognito_client_id.trim().is_empty(),
    });
    config.bearer_access_token = resolved.map(|token| token.value);
    (config, diagnostics)
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
    if cli.api_key.is_none() {
        auth::set_runtime_auth_context(active.clone(), build_oauth_config(&cli));
    }

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
                let token = resolve_token(&cli, &active).await.map(|token| token.value);
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
                let (config, auth_diagnostics) = build_config_with_auth(&cli, &active).await;
                let tenant_id = if auth_manifest_needs_tenant(manifest_args) {
                    let manifest_file = auth_manifest_file(manifest_args);
                    let (project_config, searched_paths) =
                        load_project_config_for_context(&cli, manifest_file)?;
                    let tenant_arg = strict_tenant_arg(
                        &cli,
                        project_config.as_ref(),
                        &searched_paths,
                        "auth manifest",
                    )?;
                    resolve::resolve_tenant_id(&config, tenant_arg, &active).await?
                } else {
                    String::new()
                };
                commands::auth::manifest::run(manifest_args, &config, &tenant_id, auth_diagnostics)
                    .await
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
            let (config, tenant_id) = build_tenant_config(&cli, &active, tenant_arg).await?;
            compute_cli::run(
                args,
                &config,
                &tenant_id,
                project_config.as_ref(),
                cli.config.as_deref(),
            )
            .await
        }
        Commands::Env(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let (config, tenant_id) = build_tenant_config(&cli, &active, tenant_arg).await?;
            compute_cli::run_env(
                args,
                &config,
                &tenant_id,
                project_config.as_ref(),
                cli.config.as_deref(),
            )
            .await
        }
        Commands::Org(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            org_cli::run(args, &config, &tenant_id).await
        }
        Commands::ApiKey(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            api_key_cli::run(args, &config, &tenant_id).await
        }
        Commands::Secret(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let config = build_config(&cli, &active).await;
            secret_cli::run(args, &config, project_config.as_ref()).await
        }
        Commands::Agent(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            agent_cli::run(args, &config, &tenant_id).await
        }
        Commands::Worker(args) => {
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, &cli.tenant_id, &active).await?;
            worker_cli::run(args, &config, &tenant_id, &active).await
        }
        Commands::Iac(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            iac_cli::run(args, &config, &tenant_id).await
        }
        Commands::Manifest(args) => {
            if manifest::needs_tenant(args) {
                let (project_config, searched_paths) =
                    load_project_config_for_context(&cli, manifest::context_file(args))?;
                let tenant_arg =
                    strict_tenant_arg(&cli, project_config.as_ref(), &searched_paths, "manifest")?;
                let config = build_config(&cli, &active).await;
                let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
                manifest::run(args, Some(&config), Some(&tenant_id)).await
            } else {
                manifest::run(args, None, None).await
            }
        }
        Commands::Ops(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            ops_cli::run(args, &config, &tenant_id, cli.config.as_deref()).await
        }
        Commands::Image(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            image_cli::run(args, &config, &tenant_id).await
        }
        Commands::Slack(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            slack_cli::run(args, &config, &tenant_id).await
        }
        Commands::Pm(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            pm_cli::run(args, &config, &tenant_id).await
        }
        Commands::Issue(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            pm_cli::run_top_level_issue(args, &config, &tenant_id).await
        }
        Commands::Linear(args) => {
            let project_config = config::loader::load(cli.config.as_deref())?;
            let tenant_arg = tenant_arg(&cli, project_config.as_ref());
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            linear_cli::run(args, &config, &tenant_id).await
        }
        Commands::Skills(args) => skills_cli::run(args),
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
            let (project_config, searched_paths) =
                load_project_config_for_context(&cli, args.file.as_deref())?;
            let tenant_arg =
                strict_tenant_arg(&cli, project_config.as_ref(), &searched_paths, "reconcile")?;
            let config = build_config(&cli, &active).await;
            let tenant_id = resolve::resolve_tenant_id(&config, tenant_arg, &active).await?;
            manifest::reconcile_alias(args, &config, &tenant_id).await
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

fn auth_manifest_file(args: &commands::auth::manifest::ManifestArgs) -> Option<&Path> {
    use commands::auth::manifest::ManifestCommand;

    match &args.command {
        ManifestCommand::Fmt { file, .. }
        | ManifestCommand::Validate { file }
        | ManifestCommand::Plan { file, .. }
        | ManifestCommand::Apply { file, .. } => file.as_deref(),
    }
}

fn auth_manifest_needs_tenant(args: &commands::auth::manifest::ManifestArgs) -> bool {
    !matches!(
        &args.command,
        commands::auth::manifest::ManifestCommand::Fmt { .. }
    )
}

fn strict_tenant_arg<'a>(
    cli: &'a Cli,
    project_config: Option<&'a config::loader::LoadedProjectConfig>,
    searched_paths: &[PathBuf],
    command_name: &str,
) -> Result<&'a str> {
    if !cli.tenant_id.is_empty() {
        return Ok(cli.tenant_id.as_str());
    }

    if let Some(tenant_id) = project_config
        .and_then(|loaded| loaded.config.metadata.tenant_id.as_deref())
        .filter(|tenant_id| !tenant_id.trim().is_empty())
    {
        return Ok(tenant_id);
    }

    let paths = if searched_paths.is_empty() {
        "none".to_string()
    } else {
        searched_paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    Err(anyhow!(
        "tenant could not be resolved for {command_name}. Tried --tenant-id, \
         TACHYON_TENANT_ID, then tachyon.yml metadata.tenantId/metadata.tenant_id. \
         Searched config path(s): {paths}. Set --tenant-id, export \
         TACHYON_TENANT_ID, or add metadata.tenantId to tachyon.yml."
    ))
}

fn load_project_config_for_context(
    cli: &Cli,
    context_file: Option<&Path>,
) -> Result<(Option<config::loader::LoadedProjectConfig>, Vec<PathBuf>)> {
    if std::env::var_os("TACHYON_CONFIG").is_some() || cli.config.is_some() {
        let loaded = config::loader::load_with_path(cli.config.as_deref())?;
        let searched_paths = loaded
            .as_ref()
            .map(|loaded| vec![loaded.path.clone()])
            .unwrap_or_default();
        return Ok((loaded, searched_paths));
    }

    let start = project_config_search_start(context_file)?;
    let searched_paths = tachyon_yml_search_paths(&start);
    let loaded = config::loader::load_with_path_from_dir(&start, None)?;
    Ok((loaded, searched_paths))
}

fn project_config_search_start(context_file: Option<&Path>) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let Some(path) = context_file else {
        return Ok(cwd);
    };
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    Ok(resolved.parent().unwrap_or(&cwd).to_path_buf())
}

fn tachyon_yml_search_paths(start: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut dir = start;
    loop {
        paths.push(dir.join("tachyon.yml"));
        if dir.join(".git").exists() {
            break;
        }
        let Some(parent) = dir.parent() else {
            break;
        };
        dir = parent;
    }
    paths
}
