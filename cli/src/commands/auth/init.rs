use anyhow::{anyhow, Context, Result};
use clap::Args;
use dialoguer::{Input, Select};
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::Path;

use crate::config::auth::{default_expiry_days, AuthProvider, AuthProviderType, AuthUserPool};
use crate::config::loader;

const PENDING_SECRET_REF: &str = "<pending; populated by tachyon auth issue>";

#[derive(Debug, Clone, Args)]
pub struct InitAuthArgs {
    /// Provider identifier to add to tachyon.yml
    pub provider: Option<String>,
    /// Provider name override
    #[arg(long)]
    pub name: Option<String>,
    /// Credential provider type
    #[arg(long, value_enum)]
    pub r#type: Option<AuthProviderType>,
    /// Token issuance audience or scope
    #[arg(long)]
    pub audience: Option<String>,
    /// Credential expiry in days
    #[arg(long)]
    pub expiry_days: Option<u32>,
    /// User Pool strategy for this app
    #[arg(long, value_enum)]
    pub user_pool: Option<AuthUserPool>,
    /// Skip prompts and use provided flags/defaults
    #[arg(long)]
    pub non_interactive: bool,
    /// Replace an existing provider with the same name
    #[arg(long)]
    pub force: bool,
}

pub fn run(args: &InitAuthArgs, config_flag: Option<&Path>) -> Result<()> {
    let loaded = loader::load_with_path(config_flag)?
        .ok_or_else(|| anyhow!("tachyon.yml not found. Run `tachyon init` first."))?;
    let provider = resolve_provider(args)?;

    upsert_auth(&loaded.path, &provider, args.user_pool, args.force)?;
    println!(
        "Updated {} with auth provider '{}'.",
        loaded.path.display(),
        provider.name
    );
    Ok(())
}

fn resolve_provider(args: &InitAuthArgs) -> Result<AuthProvider> {
    let default_name = args
        .provider
        .clone()
        .unwrap_or_else(|| "cognito-default".to_string());

    let name = match (&args.name, args.non_interactive) {
        (Some(name), _) => name.clone(),
        (None, true) => default_name,
        (None, false) => Input::<String>::new()
            .with_prompt("Provider name")
            .default(default_name)
            .interact_text()?,
    };
    if name.trim().is_empty() {
        return Err(anyhow!("provider name must not be empty"));
    }

    let type_ = match (args.r#type, args.non_interactive) {
        (Some(type_), _) => type_,
        (None, true) => AuthProviderType::Oauth2ClientCredentials,
        (None, false) => {
            let selected = Select::new()
                .with_prompt("Provider type")
                .items(AuthProviderType::choices())
                .default(0)
                .interact()?;
            AuthProviderType::from_index(selected)
        }
    };

    let audience = match (&args.audience, args.non_interactive) {
        (Some(audience), _) if !audience.trim().is_empty() => Some(audience.clone()),
        (Some(_), _) => None,
        (None, true) => None,
        (None, false) => {
            let value = Input::<String>::new()
                .with_prompt("Audience")
                .allow_empty(true)
                .interact_text()?;
            if value.trim().is_empty() {
                None
            } else {
                Some(value)
            }
        }
    };

    Ok(AuthProvider {
        name,
        type_,
        audience,
        expiry_days: args.expiry_days.unwrap_or_else(default_expiry_days),
        secret_ref: Some(PENDING_SECRET_REF.to_string()),
    })
}

pub(crate) fn upsert_auth(
    path: &Path,
    provider: &AuthProvider,
    user_pool: Option<AuthUserPool>,
    force: bool,
) -> Result<()> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut doc: Value =
        serde_yaml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    let root = doc
        .as_mapping_mut()
        .ok_or_else(|| anyhow!("{} must contain a YAML mapping", path.display()))?;

    let auth = ensure_mapping(root, "auth")?;
    if let Some(user_pool) = user_pool {
        auth.insert(
            Value::String("user_pool".to_string()),
            Value::String(user_pool.as_str().to_string()),
        );
    }
    let providers_key = Value::String("providers".to_string());
    if !auth.contains_key(&providers_key) {
        auth.insert(providers_key.clone(), Value::Sequence(Vec::new()));
    }
    let providers = auth
        .get_mut(&providers_key)
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| anyhow!("auth.providers must be a list"))?;

    if let Some(index) = providers
        .iter()
        .position(|entry| provider_name(entry) == Some(provider.name.as_str()))
    {
        if !force {
            return Err(anyhow!(
                "auth provider '{}' already exists. Re-run with --force to replace it.",
                provider.name
            ));
        }
        providers[index] = provider_to_value(provider)?;
    } else {
        providers.push(provider_to_value(provider)?);
    }

    fs::write(path, serde_yaml::to_string(&doc)?)?;
    Ok(())
}

fn ensure_mapping<'a>(root: &'a mut Mapping, key: &str) -> Result<&'a mut Mapping> {
    let key_value = Value::String(key.to_string());
    if !root.contains_key(&key_value) {
        root.insert(key_value.clone(), Value::Mapping(Mapping::new()));
    }
    root.get_mut(&key_value)
        .and_then(Value::as_mapping_mut)
        .ok_or_else(|| anyhow!("{key} must be a YAML mapping"))
}

fn provider_name(entry: &Value) -> Option<&str> {
    entry
        .as_mapping()?
        .get(Value::String("name".to_string()))?
        .as_str()
}

fn provider_to_value(provider: &AuthProvider) -> Result<Value> {
    serde_yaml::to_value(provider).context("serialize auth provider")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_config(dir: &Path) -> std::path::PathBuf {
        let path = dir.join("tachyon.yml");
        fs::write(
            &path,
            "apiVersion: tachyon/v1\nkind: CloudApp\nmetadata:\n  name: test-app\n  tenant_id: tn_test\nspec:\n  framework: vite\n",
        )
        .unwrap();
        path
    }

    #[test]
    fn non_interactive_uses_flags() {
        let args = InitAuthArgs {
            provider: Some("cognito-default".to_string()),
            name: None,
            r#type: Some(AuthProviderType::ApiKey),
            audience: Some("aud".to_string()),
            expiry_days: Some(30),
            user_pool: Some(AuthUserPool::Shared),
            non_interactive: true,
            force: false,
        };

        let provider = resolve_provider(&args).unwrap();

        assert_eq!(provider.name, "cognito-default");
        assert_eq!(provider.type_, AuthProviderType::ApiKey);
        assert_eq!(provider.audience.as_deref(), Some("aud"));
        assert_eq!(provider.expiry_days, 30);
    }

    #[test]
    fn refuses_duplicate_without_force() {
        let tmp = TempDir::new().unwrap();
        let path = write_config(tmp.path());
        let provider = AuthProvider {
            name: "cognito-default".to_string(),
            type_: AuthProviderType::Oauth2ClientCredentials,
            audience: None,
            expiry_days: 90,
            secret_ref: Some(PENDING_SECRET_REF.to_string()),
        };

        upsert_auth(&path, &provider, None, false).unwrap();
        let err = upsert_auth(&path, &provider, None, false).unwrap_err();

        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn force_replaces_duplicate() {
        let tmp = TempDir::new().unwrap();
        let path = write_config(tmp.path());
        let mut provider = AuthProvider {
            name: "cognito-default".to_string(),
            type_: AuthProviderType::Oauth2ClientCredentials,
            audience: None,
            expiry_days: 90,
            secret_ref: Some(PENDING_SECRET_REF.to_string()),
        };

        upsert_auth(&path, &provider, None, false).unwrap();
        provider.expiry_days = 7;
        upsert_auth(&path, &provider, Some(AuthUserPool::Shared), true).unwrap();
        let yaml = fs::read_to_string(path).unwrap();

        assert!(yaml.contains("expiry_days: 7"));
        assert!(yaml.contains("user_pool: shared"));
    }
}
