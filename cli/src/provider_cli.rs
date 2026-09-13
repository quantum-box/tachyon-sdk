use std::io::{IsTerminal, Read};

use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm, Password};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::{print_json, ApiClient};

const STRIPE_CREDENTIALS_PATH: &str = "/v1/providers/stripe/credentials";

#[derive(Debug, Clone, Args)]
pub struct ProviderArgs {
    #[command(subcommand)]
    pub command: ProviderCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ProviderCommand {
    /// Manage Stripe provider configuration
    Stripe {
        #[command(subcommand)]
        command: StripeCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum StripeCommand {
    /// Manage Stripe credentials
    Credentials {
        #[command(subcommand)]
        command: StripeCredentialsCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum StripeCredentialsCommand {
    /// Store this tenant's own Stripe platform credentials
    Set {
        /// Read one JSON object from stdin instead of prompting
        #[arg(long)]
        from_stdin: bool,
        /// Skip the target-tenant confirmation
        #[arg(long)]
        yes: bool,
        /// Output the non-secret response as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Plaintext exists only while the request is being sent. Do not derive
/// `Debug`, because this value carries Stripe credentials.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StripeCredentialsRequest {
    secret_key: String,
    publishable_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    webhook_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    connect_webhook_secret: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct StripeCredentialsResponse {
    tenant_id: String,
    environment: String,
    has_webhook_secret: bool,
    has_connect_webhook_secret: bool,
}

fn prompt_credentials() -> Result<StripeCredentialsRequest> {
    let theme = ColorfulTheme::default();
    let secret_key = Password::with_theme(&theme)
        .with_prompt("Stripe secret key (sk_test_... or sk_live_...)")
        .allow_empty_password(false)
        .interact()?;
    let publishable_key = Password::with_theme(&theme)
        .with_prompt("Stripe publishable key (pk_test_... or pk_live_...)")
        .allow_empty_password(false)
        .interact()?;
    let webhook_secret = Password::with_theme(&theme)
        .with_prompt("Account webhook signing secret (optional; blank preserves existing)")
        .allow_empty_password(true)
        .interact()?;
    let connect_webhook_secret = Password::with_theme(&theme)
        .with_prompt(
            "Connected-account webhook signing secret (optional; blank preserves existing)",
        )
        .allow_empty_password(true)
        .interact()?;

    normalize_and_validate(StripeCredentialsRequest {
        secret_key,
        publishable_key,
        webhook_secret: Some(webhook_secret),
        connect_webhook_secret: Some(connect_webhook_secret),
    })
}

fn read_credentials_from_stdin() -> Result<StripeCredentialsRequest> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("read Stripe credentials JSON from stdin")?;
    let request =
        serde_json::from_str(&input).context("parse Stripe credentials JSON from stdin")?;
    normalize_and_validate(request)
}

fn normalize_and_validate(
    mut request: StripeCredentialsRequest,
) -> Result<StripeCredentialsRequest> {
    request.secret_key = request.secret_key.trim().to_string();
    request.publishable_key = request.publishable_key.trim().to_string();
    request.webhook_secret = normalize_optional(request.webhook_secret);
    request.connect_webhook_secret = normalize_optional(request.connect_webhook_secret);

    let environment = validate_key(&request.secret_key, "secret_key", "sk_test_", "sk_live_")?;
    let publishable_environment = validate_key(
        &request.publishable_key,
        "publishable_key",
        "pk_test_",
        "pk_live_",
    )?;
    if environment != publishable_environment {
        return Err(anyhow!(
            "publishable_key must belong to the same Stripe environment as secret_key"
        ));
    }
    validate_webhook_secret("webhook_secret", request.webhook_secret.as_deref())?;
    validate_webhook_secret(
        "connect_webhook_secret",
        request.connect_webhook_secret.as_deref(),
    )?;

    Ok(request)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn validate_key<'a>(
    value: &str,
    field: &str,
    test_prefix: &'a str,
    live_prefix: &'a str,
) -> Result<&'a str> {
    let (environment, suffix) = if let Some(suffix) = value.strip_prefix(test_prefix) {
        ("test", suffix)
    } else if let Some(suffix) = value.strip_prefix(live_prefix) {
        ("live", suffix)
    } else {
        return Err(anyhow!(
            "{field} must start with {test_prefix} or {live_prefix}"
        ));
    };
    if suffix.len() < 24
        || !suffix
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        return Err(anyhow!("{field} has an invalid Stripe key format"));
    }
    Ok(environment)
}

fn validate_webhook_secret(field: &str, value: Option<&str>) -> Result<()> {
    if value.is_some_and(|value| {
        !value
            .strip_prefix("whsec_")
            .is_some_and(|suffix| !suffix.is_empty())
    }) {
        return Err(anyhow!(
            "{field} must be a Stripe webhook signing secret (whsec_...)"
        ));
    }
    Ok(())
}

async fn run_set(
    api: &ApiClient,
    tenant_id: &str,
    from_stdin: bool,
    yes: bool,
    json: bool,
) -> Result<()> {
    if from_stdin && !yes {
        return Err(anyhow!(
            "--from-stdin requires --yes after verifying the target tenant"
        ));
    }
    if !yes {
        if !std::io::stdin().is_terminal() {
            return Err(anyhow!(
                "interactive confirmation requires a terminal; pass --from-stdin --yes for automation"
            ));
        }
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Register this tenant as a Stripe platform: {tenant_id}?"
            ))
            .default(false)
            .interact()?;
        if !confirmed {
            println!("Cancelled; no credentials were sent.");
            return Ok(());
        }
    }

    let request = if from_stdin {
        read_credentials_from_stdin()?
    } else {
        prompt_credentials()?
    };
    let response: StripeCredentialsResponse = api.put(STRIPE_CREDENTIALS_PATH, &request).await?;

    if json {
        return print_json(&response);
    }
    println!("Stripe credentials registered.");
    println!("Tenant:                            {}", response.tenant_id);
    println!(
        "Environment:                       {}",
        response.environment
    );
    println!(
        "Account webhook secret provided:    {}",
        response.has_webhook_secret
    );
    println!(
        "Connect webhook secret provided:    {}",
        response.has_connect_webhook_secret
    );
    println!("Key material was not stored by the CLI.");
    Ok(())
}

pub async fn run(args: &ProviderArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;
    match &args.command {
        ProviderCommand::Stripe { command } => match command {
            StripeCommand::Credentials { command } => match command {
                StripeCredentialsCommand::Set {
                    from_stdin,
                    yes,
                    json,
                } => run_set(&api, tenant_id, *from_stdin, *yes, *json).await,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(secret_key: &str, publishable_key: &str) -> StripeCredentialsRequest {
        StripeCredentialsRequest {
            secret_key: secret_key.to_string(),
            publishable_key: publishable_key.to_string(),
            webhook_secret: None,
            connect_webhook_secret: None,
        }
    }

    fn key(prefix: &str) -> String {
        format!("{prefix}abcdefghijklmnopqrstuvwxyz")
    }

    #[test]
    fn validates_matching_stripe_environments() {
        let test = request(
            "sk_test_abcdefghijklmnopqrstuvwxyz",
            "pk_test_abcdefghijklmnopqrstuvwxyz",
        );
        assert!(normalize_and_validate(test).is_ok());

        let live_secret = key("sk_live_");
        let live_publishable = key("pk_live_");
        let live = request(&live_secret, &live_publishable);
        assert!(normalize_and_validate(live).is_ok());
    }

    #[test]
    fn rejects_mismatched_stripe_environments() {
        let live_secret = key("sk_live_");
        let test_publishable = key("pk_test_");
        let mismatched = request(&live_secret, &test_publishable);
        let error = match normalize_and_validate(mismatched) {
            Ok(_) => panic!("mismatched Stripe environments must be rejected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("same Stripe environment"));
    }

    #[test]
    fn rejects_malformed_webhook_secret() {
        let mut malformed = request(
            "sk_test_abcdefghijklmnopqrstuvwxyz",
            "pk_test_abcdefghijklmnopqrstuvwxyz",
        );
        malformed.webhook_secret = Some("secret_without_prefix".to_string());
        let error = match normalize_and_validate(malformed) {
            Ok(_) => panic!("malformed webhook secret must be rejected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("webhook_secret"));
    }
}
