use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::Args;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value as JsonValue};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::auth::{AuthProvider, AuthUserPool};
use crate::config::loader;

#[derive(Debug, Clone, Args)]
pub struct IssueAuthArgs {
    /// Provider identifier from tachyon.yml
    pub provider: String,
    /// Rotate existing credentials
    #[arg(long)]
    pub rotate: bool,
    /// Print only the secret reference path/ARN; never prints plaintext secrets
    #[arg(long)]
    pub show_secret: bool,
    /// Issue from the Tachyon shared Cognito User Pool
    #[arg(long)]
    pub shared_pool: bool,
}

#[derive(Debug, Clone, Serialize)]
struct IssueRequest<'a> {
    provider: &'a str,
    #[serde(rename = "type")]
    type_: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    audience: Option<&'a str>,
    expiry_days: u32,
    rotate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_pool: Option<&'a str>,
}

#[derive(Debug, Clone, Deserialize)]
struct IssueResponse {
    client_id: String,
    #[serde(default)]
    client_secret: Option<String>,
    #[serde(default)]
    secret_arn: Option<String>,
    #[serde(default)]
    local_path: Option<String>,
    #[serde(default)]
    user_pool_id: Option<String>,
    expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TachyonEnv {
    Dev,
    Staging,
    Prod,
}

impl TachyonEnv {
    fn detect() -> Result<Self> {
        match std::env::var("TACHYON_ENV")
            .unwrap_or_else(|_| "dev".to_string())
            .as_str()
        {
            "" | "dev" => Ok(Self::Dev),
            "staging" => Ok(Self::Staging),
            "prod" => Ok(Self::Prod),
            other => Err(anyhow!(
                "unsupported TACHYON_ENV '{other}' (expected dev, staging, or prod)"
            )),
        }
    }
}

pub async fn run(
    args: &IssueAuthArgs,
    config_flag: Option<&Path>,
    api_url: &str,
    bearer_token: Option<String>,
    tenant_id: &str,
) -> Result<()> {
    let loaded = loader::load_with_path(config_flag)?
        .ok_or_else(|| anyhow!("tachyon.yml not found. Run `tachyon init` first."))?;
    let provider = find_provider(&loaded.config, &args.provider)?;
    let app_id = loaded
        .config
        .metadata
        .name
        .as_deref()
        .ok_or_else(|| anyhow!("metadata.name is required in tachyon.yml"))?;
    let env = TachyonEnv::detect()?;
    let user_pool = resolve_user_pool(loaded.config.auth.as_ref(), args.shared_pool);

    let response = issue_from_backend(
        api_url,
        app_id,
        provider,
        args.rotate,
        user_pool,
        bearer_token.as_deref(),
        tenant_id,
    )
    .await?;

    match env {
        TachyonEnv::Dev => {
            let local_path = store_dev_credentials(&loaded.path, &args.provider, &response)?;
            let secret_ref_path = loaded
                .path
                .parent()
                .and_then(|project_dir| local_path.strip_prefix(project_dir).ok())
                .unwrap_or(local_path.as_path());
            let secret_ref = format!("{}#{}", secret_ref_path.display(), args.provider);
            update_secret_ref(&loaded.path, &args.provider, &secret_ref)?;
            if args.show_secret {
                println!("{}", local_path.display());
            } else {
                println!("Stored credentials at {}", local_path.display());
            }
        }
        TachyonEnv::Staging | TachyonEnv::Prod => {
            let secret_arn = response
                .secret_arn
                .as_deref()
                .ok_or_else(|| anyhow!("backend response did not include secret_arn"))?;
            update_secret_ref(&loaded.path, &args.provider, secret_arn)?;
            println!("Stored secret reference {secret_arn}");
        }
    }

    Ok(())
}

fn find_provider<'a>(
    config: &'a loader::ProjectConfig,
    provider_name: &str,
) -> Result<&'a AuthProvider> {
    config
        .auth
        .as_ref()
        .and_then(|auth| {
            auth.providers
                .iter()
                .find(|provider| provider.name == provider_name)
        })
        .ok_or_else(|| anyhow!("provider not registered, run tachyon auth init first"))
}

async fn issue_from_backend(
    api_url: &str,
    app_id: &str,
    provider: &AuthProvider,
    rotate: bool,
    user_pool: Option<AuthUserPool>,
    bearer_token: Option<&str>,
    tenant_id: &str,
) -> Result<IssueResponse> {
    let url = format!(
        "{}/v1/cloud-apps/{}/auth/credentials",
        api_url.trim_end_matches('/'),
        app_id
    );
    let request = IssueRequest {
        provider: &provider.name,
        type_: provider.type_.as_str(),
        audience: provider.audience.as_deref(),
        expiry_days: provider.expiry_days,
        rotate,
        user_pool: user_pool.map(AuthUserPool::as_str),
    };

    let client = reqwest::Client::new();
    let mut builder = client.post(url).json(&request);
    if let Some(token) = bearer_token {
        builder = builder.bearer_auth(token);
    }
    if !tenant_id.is_empty() {
        builder = builder.header("x-operator-id", tenant_id);
    }

    let response = builder.send().await.context("issue credentials request")?;
    if matches!(
        response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        return Err(anyhow!(
            "provider not registered, run tachyon auth init first"
        ));
    }
    let response = response.error_for_status().context("issue credentials")?;
    response
        .json::<IssueResponse>()
        .await
        .context("parse issue credentials response")
}

fn store_dev_credentials(
    config_path: &Path,
    provider_name: &str,
    response: &IssueResponse,
) -> Result<PathBuf> {
    let project_dir = config_path
        .parent()
        .ok_or_else(|| anyhow!("could not determine project directory"))?;
    let tachyon_dir = project_dir.join(".tachyon");
    fs::create_dir_all(&tachyon_dir)
        .with_context(|| format!("create {}", tachyon_dir.display()))?;
    let path = response
        .local_path
        .as_deref()
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                project_dir.join(path)
            }
        })
        .unwrap_or_else(|| tachyon_dir.join("credentials.json"));

    let mut root = if path.exists() {
        let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        serde_json::from_str::<JsonValue>(&raw)
            .with_context(|| format!("parse {}", path.display()))?
    } else {
        json!({})
    };
    let providers = root
        .as_object_mut()
        .ok_or_else(|| anyhow!("{} must contain a JSON object", path.display()))?
        .entry("providers")
        .or_insert_with(|| json!({}));
    let providers = providers
        .as_object_mut()
        .ok_or_else(|| anyhow!("providers in {} must be an object", path.display()))?;

    let mut entry = Map::new();
    entry.insert("client_id".to_string(), json!(response.client_id));
    let client_secret = response
        .client_secret
        .as_deref()
        .ok_or_else(|| anyhow!("backend response did not include client_secret"))?;
    entry.insert("client_secret".to_string(), json!(client_secret));
    if let Some(user_pool_id) = &response.user_pool_id {
        entry.insert("user_pool_id".to_string(), json!(user_pool_id));
    }
    entry.insert("expires_at".to_string(), json!(response.expires_at));
    providers.insert(provider_name.to_string(), JsonValue::Object(entry));

    let tmp = path.with_extension("tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(&root)?)
        .with_context(|| format!("write {}", tmp.display()))?;
    set_private_permissions(&tmp)?;
    fs::rename(&tmp, &path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    set_private_permissions(&path)?;
    ensure_gitignore(project_dir)?;
    Ok(path)
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("chmod 600 {}", path.display()))
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

fn ensure_gitignore(project_dir: &Path) -> Result<()> {
    let path = project_dir.join(".gitignore");
    let entry = ".tachyon/credentials.json";
    let existing = fs::read_to_string(&path).unwrap_or_default();
    if existing.lines().any(|line| line.trim() == entry) {
        return Ok(());
    }

    let mut next = existing;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(entry);
    next.push('\n');
    fs::write(&path, next).with_context(|| format!("write {}", path.display()))
}

fn update_secret_ref(config_path: &Path, provider_name: &str, secret_ref: &str) -> Result<()> {
    let raw = fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut doc: YamlValue =
        serde_yaml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    let providers = doc
        .as_mapping_mut()
        .and_then(|root| root.get_mut(YamlValue::String("auth".to_string())))
        .and_then(YamlValue::as_mapping_mut)
        .and_then(|auth| auth.get_mut(YamlValue::String("providers".to_string())))
        .and_then(YamlValue::as_sequence_mut)
        .ok_or_else(|| anyhow!("auth.providers must be a list"))?;

    let provider = providers
        .iter_mut()
        .find(|entry| {
            entry
                .as_mapping()
                .and_then(|mapping| mapping.get(YamlValue::String("name".to_string())))
                .and_then(YamlValue::as_str)
                == Some(provider_name)
        })
        .ok_or_else(|| anyhow!("provider not registered, run tachyon auth init first"))?;
    let provider = provider
        .as_mapping_mut()
        .ok_or_else(|| anyhow!("auth provider entry must be a YAML mapping"))?;
    provider.insert(
        YamlValue::String("secret_ref".to_string()),
        YamlValue::String(secret_ref.to_string()),
    );

    fs::write(config_path, serde_yaml::to_string(&doc)?)
        .with_context(|| format!("write {}", config_path.display()))
}

fn resolve_user_pool(
    auth: Option<&crate::config::auth::AuthConfig>,
    shared_pool: bool,
) -> Option<AuthUserPool> {
    if shared_pool {
        return Some(AuthUserPool::Shared);
    }
    auth.and_then(|auth| auth.user_pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::auth::{AuthProvider, AuthProviderType};
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn response() -> IssueResponse {
        IssueResponse {
            client_id: "dummy-client".to_string(),
            client_secret: Some("dummy-secret".to_string()),
            secret_arn: Some("arn:aws:secretsmanager:ap-northeast-1:123:secret:test".to_string()),
            local_path: None,
            user_pool_id: Some("ap-northeast-1_8Ga4bK5M4".to_string()),
            expires_at: "2026-06-01T00:00:00Z".parse().unwrap(),
        }
    }

    fn config(path: &Path) {
        fs::write(
            path,
            "apiVersion: tachyon/v1\nkind: CloudApp\nmetadata:\n  name: test-app\nauth:\n  providers:\n  - name: cognito-default\n    type: oauth2_client_credentials\n    expiry_days: 90\n    secret_ref: pending\n",
        )
        .unwrap();
    }

    #[test]
    fn dev_storage_writes_private_project_file_and_gitignore() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("tachyon.yml");
        config(&config_path);

        let path = store_dev_credentials(&config_path, "cognito-default", &response()).unwrap();

        assert_eq!(path, tmp.path().join(".tachyon/credentials.json"));
        assert!(fs::read_to_string(tmp.path().join(".gitignore"))
            .unwrap()
            .contains(".tachyon/credentials.json"));
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let raw = fs::read_to_string(path).unwrap();
        assert!(raw.contains("ap-northeast-1_8Ga4bK5M4"));
    }

    #[test]
    fn staging_updates_secret_ref() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("tachyon.yml");
        config(&config_path);

        update_secret_ref(
            &config_path,
            "cognito-default",
            "arn:aws:secretsmanager:ap-northeast-1:123:secret:test",
        )
        .unwrap();

        let yaml = fs::read_to_string(config_path).unwrap();
        assert!(yaml.contains("secret_ref: arn:aws:secretsmanager"));
    }

    #[test]
    fn finds_registered_provider() {
        let config = loader::ProjectConfig {
            kind: Default::default(),
            metadata: Default::default(),
            spec: Default::default(),
            auth: Some(crate::config::auth::AuthConfig {
                user_pool: Some(AuthUserPool::Shared),
                providers: vec![AuthProvider {
                    name: "cognito-default".to_string(),
                    type_: AuthProviderType::Oauth2ClientCredentials,
                    audience: None,
                    expiry_days: 90,
                    secret_ref: None,
                }],
            }),
        };

        assert!(find_provider(&config, "cognito-default").is_ok());
        assert!(find_provider(&config, "missing").is_err());
    }

    #[test]
    fn resolves_shared_pool_from_flag_or_config() {
        assert_eq!(resolve_user_pool(None, true), Some(AuthUserPool::Shared));
        assert_eq!(
            resolve_user_pool(
                Some(&crate::config::auth::AuthConfig {
                    user_pool: Some(AuthUserPool::Shared),
                    providers: vec![],
                }),
                false,
            ),
            Some(AuthUserPool::Shared)
        );
    }
}
