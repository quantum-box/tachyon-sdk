//! `tachyon auth workload-identity` — exchange a workload's OIDC token for a
//! Tachyon access token, and manage the bindings and federation providers
//! that allow the exchange.

use std::collections::BTreeMap;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use tachyon_sdk::workload_identity::{WorkloadIdentityConfig, DEFAULT_AUDIENCE};

use crate::client::{print_json, ApiClient};
use crate::workload_identity as wif;

/// Issuer of GitHub Actions OIDC tokens.
pub const GITHUB_ACTIONS_ISSUER: &str = "https://token.actions.githubusercontent.com";

const BINDINGS_PATH: &str = "/v1/auth/workload-identities";
const PROVIDERS_PATH: &str = "/v1/auth/workload-identities/federation-providers";

#[derive(Args, Debug)]
pub struct WorkloadIdentityArgs {
    #[command(subcommand)]
    pub command: WorkloadIdentityCommand,
}

#[derive(Subcommand, Debug)]
pub enum WorkloadIdentityCommand {
    /// Exchange the workload's OIDC token for a Tachyon access token and
    /// print it (refuses to print to a terminal). Without
    /// --subject-token-file/--subject-token-env, the GitHub Actions OIDC
    /// token is used.
    Token(TokenArgs),
    /// List workload identity bindings of the tenant
    List {
        #[arg(long)]
        json: bool,
    },
    /// Create or update the binding of a Cloud App environment to a service
    /// account and the actions it may request
    Ensure {
        /// Cloud App ID
        #[arg(long)]
        app_id: String,
        /// Binding environment, e.g. production
        #[arg(long)]
        environment: String,
        /// Service account ID or name the exchanged token acts as
        #[arg(long)]
        service_account: String,
        /// Action the workload may request (repeatable)
        #[arg(long = "allowed-action", required = true)]
        allowed_actions: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Manage OIDC federation providers trusted by a binding
    Provider {
        #[command(subcommand)]
        command: ProviderCommand,
    },
}

#[derive(Args, Debug)]
pub struct TokenArgs {
    /// Cloud App ID of the binding
    #[arg(long, env = wif::ENV_APP_ID)]
    pub app_id: String,
    /// Binding environment, e.g. production
    #[arg(long, env = wif::ENV_ENVIRONMENT)]
    pub environment: String,
    /// Action to authorize (one of the binding's allowed actions)
    #[arg(long, env = wif::ENV_ACTION)]
    pub action: String,
    /// Audience requested from GitHub Actions OIDC
    #[arg(long, env = wif::ENV_AUDIENCE, default_value = DEFAULT_AUDIENCE)]
    pub audience: String,
    /// Read the subject (OIDC) token from this file
    #[arg(long, env = wif::ENV_SUBJECT_TOKEN_FILE, conflicts_with = "subject_token_env")]
    pub subject_token_file: Option<PathBuf>,
    /// Read the subject (OIDC) token from this environment variable
    #[arg(long, env = wif::ENV_SUBJECT_TOKEN_ENV)]
    pub subject_token_env: Option<String>,
    /// Requested token lifetime in seconds (the server clamps it to 60-900)
    #[arg(long, env = wif::ENV_TTL, value_parser = clap::value_parser!(i64).range(1..))]
    pub ttl: Option<i64>,
    /// Output format: the bare token, or the exchange response as JSON
    #[arg(long, value_enum, default_value_t = TokenFormat::Text)]
    pub format: TokenFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum TokenFormat {
    Text,
    Json,
}

#[derive(Subcommand, Debug)]
pub enum ProviderCommand {
    /// Create or update a federation provider (keyed by app, environment,
    /// and issuer)
    Add {
        /// Cloud App ID of the binding
        #[arg(long)]
        app_id: String,
        /// Binding environment
        #[arg(long)]
        environment: String,
        /// OIDC issuer URL
        #[arg(long, default_value = GITHUB_ACTIONS_ISSUER)]
        issuer: String,
        /// Expected `aud` claim
        #[arg(long, default_value = DEFAULT_AUDIENCE)]
        audience: String,
        /// Pattern the `sub` claim must match, e.g.
        /// repo:my-org/my-repo:ref:refs/heads/main
        #[arg(long)]
        subject_pattern: String,
        /// Extra claim condition `claim=value[,value...]` (repeatable)
        #[arg(long = "claim", value_parser = parse_claim_condition)]
        claims: Vec<(String, Vec<String>)>,
        /// Replay protection for the token `jti`
        #[arg(long, value_enum, default_value_t = ReplayProtectionMode::Required)]
        replay_protection: ReplayProtectionMode,
        #[arg(long)]
        json: bool,
    },
    /// List federation providers of the tenant
    List {
        #[arg(long)]
        json: bool,
    },
    /// Delete a federation provider
    Delete {
        /// Federation provider ID (wif_...)
        provider_id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "camelCase")]
pub enum ReplayProtectionMode {
    Required,
    AllowMissingJti,
}

fn parse_claim_condition(value: &str) -> Result<(String, Vec<String>), String> {
    let (claim, values) = value
        .split_once('=')
        .ok_or_else(|| format!("expected claim=value[,value...], got '{value}'"))?;
    let claim = claim.trim();
    let values: Vec<String> = values
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .collect();
    if claim.is_empty() || values.is_empty() {
        return Err(format!("expected claim=value[,value...], got '{value}'"));
    }
    Ok((claim.to_string(), values))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnsureBindingRequest<'a> {
    app_id: &'a str,
    operator_id: &'a str,
    environment: &'a str,
    service_account_id: &'a str,
    allowed_actions: &'a [String],
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BindingResponse {
    id: String,
    app_id: String,
    operator_id: String,
    environment: String,
    service_account_id: String,
    allowed_actions: Vec<String>,
    enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnsureProviderRequest<'a> {
    app_id: &'a str,
    operator_id: &'a str,
    environment: &'a str,
    issuer: &'a str,
    audience: &'a str,
    subject_pattern: &'a str,
    claim_conditions: BTreeMap<String, Vec<String>>,
    replay_protection_mode: ReplayProtectionMode,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProviderResponse {
    id: String,
    app_id: String,
    operator_id: String,
    environment: String,
    issuer: String,
    audience: String,
    subject_pattern: String,
    #[serde(default)]
    claim_conditions: BTreeMap<String, Vec<String>>,
    replay_protection_mode: ReplayProtectionMode,
    enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteProviderOutput<'a> {
    id: &'a str,
    deleted: bool,
}

/// `tachyon auth workload-identity token`.
pub async fn run_token(args: &TokenArgs, api_url: &str, tenant: &str) -> Result<()> {
    if std::io::stdout().is_terminal() {
        bail!(
            "refusing to print the token to a terminal. Capture it instead, e.g. \
             `TOKEN=$(tachyon auth workload-identity token ...)`."
        );
    }
    let audience = Some(args.audience.as_str());
    let config = WorkloadIdentityConfig {
        api_url: api_url.to_string(),
        app_id: args.app_id.clone(),
        operator_id: wif::operator_id(tenant)?,
        environment: args.environment.clone(),
        action: args.action.clone(),
        subject_token: wif::subject_token_source(
            args.subject_token_file.clone(),
            args.subject_token_env.clone(),
            audience,
            |name| std::env::var(name).ok(),
        )?,
        ttl_seconds: args.ttl,
    };
    let response = config
        .exchange(&reqwest12::Client::new())
        .await
        .map_err(|err| anyhow!("workload identity token exchange failed: {err}"))?;
    wif::mask_in_github_actions(&response.access_token);

    let mut stdout = std::io::stdout().lock();
    match args.format {
        TokenFormat::Text => writeln!(stdout, "{}", response.access_token)?,
        TokenFormat::Json => writeln!(stdout, "{}", serde_json::to_string(&response)?)?,
    }
    stdout.flush()?;
    eprintln!(
        "Exchanged a workload identity token for {} ({}, action {}), valid for {}s.",
        config.app_id, config.environment, config.action, response.expires_in
    );
    Ok(())
}

/// Management subcommands (`list`, `ensure`, `provider ...`).
pub async fn run_management(
    args: &WorkloadIdentityArgs,
    api: &ApiClient,
    tenant_id: &str,
) -> Result<()> {
    match &args.command {
        WorkloadIdentityCommand::Token(_) => unreachable!("token is handled by run_token"),
        WorkloadIdentityCommand::List { json } => {
            let bindings: Vec<BindingResponse> = api.get(BINDINGS_PATH).await?;
            if *json {
                return print_json(&bindings);
            }
            if bindings.is_empty() {
                println!("No workload identity bindings found.");
                return Ok(());
            }
            println!(
                "{:<30}  {:<30}  {:<12}  {:<30}  {:<7}  ALLOWED ACTIONS",
                "ID", "APP", "ENVIRONMENT", "SERVICE ACCOUNT", "ENABLED"
            );
            for binding in &bindings {
                println!(
                    "{:<30}  {:<30}  {:<12}  {:<30}  {:<7}  {}",
                    binding.id,
                    binding.app_id,
                    binding.environment,
                    binding.service_account_id,
                    binding.enabled,
                    binding.allowed_actions.join(","),
                );
            }
            Ok(())
        }
        WorkloadIdentityCommand::Ensure {
            app_id,
            environment,
            service_account,
            allowed_actions,
            json,
        } => {
            let service_account_id =
                crate::resolve::resolve_service_account_id(api, tenant_id, service_account).await?;
            let request = EnsureBindingRequest {
                app_id,
                operator_id: tenant_id,
                environment,
                service_account_id: &service_account_id,
                allowed_actions,
            };
            let binding: BindingResponse = api.put(BINDINGS_PATH, &request).await?;
            if *json {
                return print_json(&binding);
            }
            println!("Workload identity binding saved.");
            println!("ID:               {}", binding.id);
            println!("App:              {}", binding.app_id);
            println!("Environment:      {}", binding.environment);
            println!("Service account:  {}", binding.service_account_id);
            println!("Allowed actions:  {}", binding.allowed_actions.join(", "));
            Ok(())
        }
        WorkloadIdentityCommand::Provider { command } => {
            run_provider(command, api, tenant_id).await
        }
    }
}

async fn run_provider(command: &ProviderCommand, api: &ApiClient, tenant_id: &str) -> Result<()> {
    match command {
        ProviderCommand::Add {
            app_id,
            environment,
            issuer,
            audience,
            subject_pattern,
            claims,
            replay_protection,
            json,
        } => {
            let request = EnsureProviderRequest {
                app_id,
                operator_id: tenant_id,
                environment,
                issuer,
                audience,
                subject_pattern,
                claim_conditions: claims.iter().cloned().collect(),
                replay_protection_mode: *replay_protection,
            };
            let provider: ProviderResponse = api.put(PROVIDERS_PATH, &request).await?;
            if *json {
                return print_json(&provider);
            }
            println!("Federation provider saved.");
            println!("ID:               {}", provider.id);
            println!("App:              {}", provider.app_id);
            println!("Environment:      {}", provider.environment);
            println!("Issuer:           {}", provider.issuer);
            println!("Audience:         {}", provider.audience);
            println!("Subject pattern:  {}", provider.subject_pattern);
            Ok(())
        }
        ProviderCommand::List { json } => {
            let providers: Vec<ProviderResponse> = api.get(PROVIDERS_PATH).await?;
            if *json {
                return print_json(&providers);
            }
            if providers.is_empty() {
                println!("No federation providers found.");
                return Ok(());
            }
            println!(
                "{:<30}  {:<30}  {:<12}  {:<45}  SUBJECT PATTERN",
                "ID", "APP", "ENVIRONMENT", "ISSUER"
            );
            for provider in &providers {
                println!(
                    "{:<30}  {:<30}  {:<12}  {:<45}  {}",
                    provider.id,
                    provider.app_id,
                    provider.environment,
                    provider.issuer,
                    provider.subject_pattern,
                );
            }
            Ok(())
        }
        ProviderCommand::Delete { provider_id, json } => {
            api.delete(&format!(
                "{PROVIDERS_PATH}/{}",
                urlencoding::encode(provider_id)
            ))
            .await?;
            if *json {
                return print_json(&DeleteProviderOutput {
                    id: provider_id,
                    deleted: true,
                });
            }
            println!("Federation provider {provider_id} deleted.");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_claim_conditions() {
        assert_eq!(
            parse_claim_condition("repository_owner=my-org").unwrap(),
            ("repository_owner".into(), vec!["my-org".into()])
        );
        assert_eq!(
            parse_claim_condition("ref=refs/heads/main, refs/heads/release").unwrap(),
            (
                "ref".into(),
                vec!["refs/heads/main".into(), "refs/heads/release".into()]
            )
        );
        assert!(parse_claim_condition("ref").is_err());
        assert!(parse_claim_condition("ref=").is_err());
    }
}
