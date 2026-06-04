use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use dialoguer::{theme::ColorfulTheme, Password};
use reqwest::{header, Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use tachyon_sdk::apis::configuration::Configuration;

use crate::compute_cli::{validate_secret_key, PagesApp};
use crate::config::loader::ProjectConfig;

#[derive(Debug, Clone, Args)]
pub struct SecretArgs {
    #[command(subcommand)]
    pub command: SecretCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum SecretCommand {
    /// Set a Cloudflare Pages encrypted secret binding
    Set {
        /// Secret binding name
        name: String,
        /// Secret value. Prefer TACHYON_SECRET_VALUE or --from-stdin in scripts.
        #[arg(
            long,
            conflicts_with = "from_stdin",
            help = "Secret value. Prefer TACHYON_SECRET_VALUE or --from-stdin to avoid shell history leaks."
        )]
        value: Option<String>,
        /// Read the secret value from stdin
        #[arg(long)]
        from_stdin: bool,
        /// Cloudflare account ID
        #[arg(long, env = "CLOUDFLARE_ACCOUNT_ID")]
        account_id: Option<String>,
        /// Cloudflare API token
        #[arg(long, env = "CLOUDFLARE_API_TOKEN", hide_env_values = true)]
        api_token: Option<String>,
        /// Cloudflare Pages project name
        #[arg(long)]
        project_name: Option<String>,
        /// Known Pages app mapping from the local build command
        #[arg(long, value_enum)]
        app: Option<PagesApp>,
        /// Pages deployment environment
        #[arg(long, value_enum, default_value = "production")]
        environment: PagesEnvironment,
        /// Cloudflare API base URL
        #[arg(
            long,
            env = "TACHYON_CLOUDFLARE_API_URL",
            default_value = "https://api.cloudflare.com/client/v4"
        )]
        cloudflare_api_url: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PagesEnvironment {
    Production,
    Preview,
}

impl PagesEnvironment {
    fn as_str(self) -> &'static str {
        match self {
            PagesEnvironment::Production => "production",
            PagesEnvironment::Preview => "preview",
        }
    }
}

#[derive(Debug, Deserialize)]
struct CloudflareEnvelope<T> {
    success: bool,
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
struct PagesProject {
    #[serde(default)]
    deployment_configs: BTreeMap<String, DeploymentConfig>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
struct DeploymentConfig {
    #[serde(default)]
    env_vars: BTreeMap<String, PagesEnvVar>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PagesEnvVar {
    #[serde(rename = "type")]
    binding_type: String,
    value: String,
}

#[derive(Debug, Serialize)]
struct UpdateProjectRequest {
    deployment_configs: BTreeMap<String, DeploymentConfig>,
}

struct CloudflarePagesClient {
    client: Client,
    base_url: String,
}

impl CloudflarePagesClient {
    fn new(base_url: &str, api_token: &str) -> Result<Self> {
        let token = api_token.trim();
        if token.is_empty() {
            return Err(anyhow!(
                "Cloudflare API token is required. Set CLOUDFLARE_API_TOKEN or pass --api-token."
            ));
        }

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {token}"))?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        let client = Client::builder().default_headers(headers).build()?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    async fn get_project(&self, account_id: &str, project_name: &str) -> Result<PagesProject> {
        let path = pages_project_path(account_id, project_name);
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET Cloudflare Pages project {project_name}"))?;
        decode_cloudflare_response("GET", &path, resp).await
    }

    async fn update_project(
        &self,
        account_id: &str,
        project_name: &str,
        req: &UpdateProjectRequest,
    ) -> Result<PagesProject> {
        let path = pages_project_path(account_id, project_name);
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .patch(&url)
            .json(req)
            .send()
            .await
            .with_context(|| format!("PATCH Cloudflare Pages project {project_name}"))?;
        decode_cloudflare_response("PATCH", &path, resp).await
    }
}

fn pages_project_path(account_id: &str, project_name: &str) -> String {
    format!(
        "/accounts/{}/pages/projects/{}",
        urlencoding::encode(account_id),
        urlencoding::encode(project_name)
    )
}

async fn decode_cloudflare_response<T: for<'de> Deserialize<'de>>(
    method: &str,
    path: &str,
    resp: reqwest::Response,
) -> Result<T> {
    let status = resp.status();
    if !status.is_success() {
        return Err(cloudflare_http_error(method, path, status));
    }
    let envelope: CloudflareEnvelope<T> = resp
        .json()
        .await
        .with_context(|| format!("parse Cloudflare {method} {path} response"))?;
    if !envelope.success {
        return Err(anyhow!("Cloudflare {method} {path} failed"));
    }
    envelope
        .result
        .ok_or_else(|| anyhow!("Cloudflare {method} {path} response did not include result"))
}

fn cloudflare_http_error(method: &str, path: &str, status: StatusCode) -> anyhow::Error {
    anyhow!("Cloudflare {method} {path} failed with status {status}")
}

fn resolve_project_name(
    project_name: Option<&str>,
    app: Option<&PagesApp>,
    project_config: Option<&ProjectConfig>,
) -> Result<String> {
    if let Some(project_name) = project_name.filter(|value| !value.trim().is_empty()) {
        return Ok(project_name.to_string());
    }
    if let Some(app) = app {
        return Ok(app.cf_project_name().to_string());
    }
    if let Some(config) = project_config {
        if is_cloud_apps_config(config) {
            if let Some(name) = single_cloud_app_name(config) {
                return Ok(name.to_string());
            }
        }
        if let Some(name) = config
            .metadata
            .name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            return Ok(name.to_string());
        }
        if let Some(name) = single_cloud_app_name(config) {
            return Ok(name.to_string());
        }
    }

    Err(anyhow!(
        "Cloudflare Pages project name is required. Pass --project-name, --app, \
         set metadata.name in tachyon.yml, or use a CloudApps config with one app."
    ))
}

fn is_cloud_apps_config(config: &ProjectConfig) -> bool {
    config.kind.as_deref() == Some("CloudApps")
}

fn single_cloud_app_name(config: &ProjectConfig) -> Option<&str> {
    if config.spec.apps.len() != 1 {
        return None;
    }
    config.spec.apps[0]
        .name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
}

fn required_option(value: Option<&String>, label: &str, hint: &str) -> Result<String> {
    let value = value
        .map(String::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    if value.is_empty() {
        return Err(anyhow!("{label} is required. {hint}"));
    }
    Ok(value)
}

fn read_secret_value(name: &str, value: Option<&str>, from_stdin: bool) -> Result<String> {
    if let Some(value) = value {
        return validate_secret_value(value.to_string());
    }
    if from_stdin {
        let mut value = String::new();
        std::io::stdin()
            .read_to_string(&mut value)
            .context("read secret value from stdin")?;
        while value.ends_with('\n') || value.ends_with('\r') {
            value.pop();
        }
        return validate_secret_value(value);
    }
    if let Ok(value) = std::env::var("TACHYON_SECRET_VALUE") {
        return validate_secret_value(value);
    }

    let value = Password::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Enter value for {name}"))
        .allow_empty_password(false)
        .interact()?;
    validate_secret_value(value)
}

fn validate_secret_value(value: String) -> Result<String> {
    if value.is_empty() {
        return Err(anyhow!("secret value must not be empty"));
    }
    Ok(value)
}

fn secret_patch(
    mut project: PagesProject,
    environment: PagesEnvironment,
    name: &str,
    value: String,
) -> UpdateProjectRequest {
    let env_key = environment.as_str().to_string();
    let mut env_config = project
        .deployment_configs
        .remove(&env_key)
        .unwrap_or_default();
    env_config.env_vars.insert(
        name.to_string(),
        PagesEnvVar {
            binding_type: "secret_text".to_string(),
            value,
        },
    );

    let mut deployment_configs = BTreeMap::new();
    deployment_configs.insert(env_key, env_config);
    UpdateProjectRequest { deployment_configs }
}

pub async fn run(
    args: &SecretArgs,
    _config: &Configuration,
    project_config: Option<&ProjectConfig>,
) -> Result<()> {
    match &args.command {
        SecretCommand::Set {
            name,
            value,
            from_stdin,
            account_id,
            api_token,
            project_name,
            app,
            environment,
            cloudflare_api_url,
        } => {
            validate_secret_key(name)?;
            let account_id = required_option(
                account_id.as_ref(),
                "Cloudflare account ID",
                "Set CLOUDFLARE_ACCOUNT_ID or pass --account-id.",
            )?;
            let api_token = required_option(
                api_token.as_ref(),
                "Cloudflare API token",
                "Set CLOUDFLARE_API_TOKEN or pass --api-token.",
            )?;
            let project_name =
                resolve_project_name(project_name.as_deref(), app.as_ref(), project_config)?;
            let secret_value = read_secret_value(name, value.as_deref(), *from_stdin)?;

            let client = CloudflarePagesClient::new(cloudflare_api_url, &api_token)?;
            let project = client.get_project(&account_id, &project_name).await?;
            let patch = secret_patch(project, *environment, name, secret_value);
            client
                .update_project(&account_id, &project_name, &patch)
                .await?;

            println!(
                "Set secret {name} for Cloudflare Pages project {project_name} ({environment}).",
                environment = environment.as_str()
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_patch_preserves_existing_env_vars_for_target_environment() {
        let mut deployment_configs = BTreeMap::new();
        let mut production = DeploymentConfig::default();
        production.env_vars.insert(
            "EXISTING".to_string(),
            PagesEnvVar {
                binding_type: "plain_text".to_string(),
                value: "plain".to_string(),
            },
        );
        deployment_configs.insert("production".to_string(), production);

        let patch = secret_patch(
            PagesProject { deployment_configs },
            PagesEnvironment::Production,
            "API_KEY",
            "redacted".to_string(),
        );

        let env_vars = &patch.deployment_configs["production"].env_vars;
        assert_eq!(env_vars["EXISTING"].binding_type, "plain_text");
        assert_eq!(env_vars["API_KEY"].binding_type, "secret_text");
        assert_eq!(env_vars["API_KEY"].value, "redacted");
    }
}
