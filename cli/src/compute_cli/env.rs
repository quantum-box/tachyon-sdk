use super::*;
use std::str::FromStr;

use yaml_edit::{Document, Mapping, Sequence};

// --- Env subcommands ---

#[derive(Debug, Clone, Subcommand)]
pub enum EnvCommand {
    /// List environment variables for an app
    List {
        /// App ID or name
        app_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Set environment variables for an app
    Set {
        /// App ID or name
        app_id: Option<String>,
        /// App ID or name (alternative to positional app_id)
        #[arg(long)]
        app: Option<String>,
        /// Register this key as a Cloudflare Pages secret
        #[arg(long)]
        secret: Option<String>,
        /// Secret value (non-interactive). Use `-` to read from stdin.
        #[arg(
            long,
            value_name = "VALUE",
            requires = "secret",
            conflicts_with = "vars"
        )]
        value: Option<String>,
        /// Target environment
        #[arg(long, default_value = "all")]
        target: String,
        /// Git branch to scope plain variables to
        #[arg(long)]
        branch: Option<String>,
        /// Variables in KEY=VALUE format
        #[arg(num_args = 0..)]
        vars: Vec<String>,
    },
    /// Delete environment variables by key
    Unset {
        /// App ID or name (alternative to positional app_id)
        #[arg(long)]
        app: Option<String>,
        /// Target environment to delete
        #[arg(long)]
        target: Option<String>,
        /// KEY, or APP KEY when no project config is available
        #[arg(num_args = 1..=2)]
        args: Vec<String>,
    },
    /// Delete an environment variable
    Delete {
        /// App ID or name
        app_id: String,
        /// Env var ID to delete
        env_id: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct ListEnvVarsResponse {
    pub(super) env_vars: Vec<EnvVarResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct EnvVarResponse {
    pub(super) id: String,
    pub(super) key: String,
    #[serde(default)]
    pub(super) value: Option<String>,
    #[serde(default)]
    pub(super) target: Option<String>,
    #[serde(default)]
    pub(super) branch: Option<String>,
    #[serde(default)]
    pub(super) is_secret: Option<bool>,
    #[serde(default)]
    pub(super) secret_source: Option<EnvVarSecretSourceResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct EnvVarSecretSourceResponse {
    pub(super) source_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) field: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SetEnvVarsRequest {
    pub(super) env_vars: Vec<SetEnvVarEntry>,
}

#[derive(Serialize)]
struct SetAppSecretRequest {
    key: String,
    value: String,
    target: String,
}

#[derive(Deserialize)]
struct SetAppSecretResponse {
    key: String,
    target: String,
    secret_ref: String,
}

#[derive(Debug, Serialize, Clone)]
pub(super) struct SetEnvVarEntry {
    pub(super) key: String,
    pub(super) value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) is_secret: Option<bool>,
}

pub(super) async fn run_env_list(api: &ApiClient, app_id: &str, json: bool) -> Result<()> {
    let resp: ListEnvVarsResponse = api.get(&format!("/v1/apps/{app_id}/env")).await?;
    if json {
        return print_json(&resp.env_vars);
    }
    if resp.env_vars.is_empty() {
        println!("No environment variables set for app {app_id}");
        return Ok(());
    }
    println!(
        "{:<28}  {:<24}  {:<11}  {:<16}  {:<8}  VALUE",
        "ID", "KEY", "TARGET", "BRANCH", "SECRET"
    );
    println!(
        "{:-<28}  {:-<24}  {:-<11}  {:-<16}  {:-<8}  {:-<40}",
        "", "", "", "", "", ""
    );
    for var in &resp.env_vars {
        let is_secret = var.is_secret.unwrap_or(false);
        let value = if is_secret {
            "****".to_string()
        } else {
            var.value.as_deref().unwrap_or("-").to_string()
        };
        println!(
            "{:<28}  {:<24}  {:<11}  {:<16}  {:<8}  {}",
            var.id,
            var.key,
            var.target.as_deref().unwrap_or("all"),
            var.branch.as_deref().unwrap_or("-"),
            if is_secret { "yes" } else { "no" },
            value,
        );
    }
    Ok(())
}

pub(super) async fn run_env_set(
    api: &ApiClient,
    app_id: &str,
    vars: &[String],
    target: &str,
    branch: Option<&str>,
) -> Result<()> {
    if vars.is_empty() {
        return Err(anyhow!(
            "at least one KEY=VALUE pair is required unless --secret is used"
        ));
    }
    validate_secret_target(target)?;

    let entries: Vec<SetEnvVarEntry> = vars
        .iter()
        .map(|v| {
            let (key, value) = v
                .split_once('=')
                .ok_or_else(|| anyhow!("invalid env var format: '{v}' (expected KEY=VALUE)"))?;
            Ok(SetEnvVarEntry {
                key: key.to_string(),
                value: value.to_string(),
                target: Some(target.to_string()),
                branch: branch.filter(|b| !b.is_empty()).map(ToString::to_string),
                is_secret: None,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let req = SetEnvVarsRequest { env_vars: entries };
    let resp: ListEnvVarsResponse = api.post(&format!("/v1/apps/{app_id}/env"), &req).await?;
    println!("Set {} environment variable(s).", resp.env_vars.len());
    Ok(())
}

pub(super) async fn run_env_unset_key(
    api: &ApiClient,
    app_id: &str,
    key: &str,
    target: Option<&str>,
) -> Result<()> {
    if let Some(target) = target {
        validate_secret_target(target)?;
    }
    let suffix = target
        .map(|target| format!("?target={target}"))
        .unwrap_or_default();
    api.delete(&format!("/v1/apps/{app_id}/env/{key}{suffix}"))
        .await?;
    println!("Environment variable {key} deleted.");
    Ok(())
}

pub(super) async fn run_env_set_secret(
    api: &ApiClient,
    app_id: &str,
    key: &str,
    target: &str,
    value_flag: Option<&str>,
    config_flag: Option<&Path>,
) -> Result<()> {
    validate_secret_key(key)?;
    validate_secret_target(target)?;
    let value = read_secret_value(key, value_flag)?;
    let req = SetAppSecretRequest {
        key: key.to_string(),
        value,
        target: target.to_string(),
    };
    let resp: SetAppSecretResponse = api
        .post(&format!("/v1/apps/{app_id}/secrets"), &req)
        .await?;

    let secret_path = manifest_secret_path(&resp.secret_ref, &resp.key)?;
    update_manifest_secret_ref(config_flag, &resp.key, &resp.target, secret_path)?;
    println!("Set secret {} for target {}.", resp.key, resp.target);
    println!("Updated tachyon.yml with valueFrom.secret: {secret_path}");
    Ok(())
}

pub(crate) fn validate_secret_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(anyhow!("secret key must not be empty"));
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(anyhow!(
            "secret key must contain only uppercase ASCII letters, digits, and underscores"
        ));
    }
    Ok(())
}

pub(super) fn looks_like_secret_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

fn validate_secret_target(target: &str) -> Result<()> {
    match target {
        "production" | "preview" | "all" => Ok(()),
        _ => Err(anyhow!(
            "invalid target '{target}' (expected production, preview, or all)"
        )),
    }
}

/// Resolve a secret value for `env set --secret`.
///
/// Priority: `--value` / `--value -` (stdin) → `TACHYON_SECRET_VALUE` → piped
/// stdin when non-interactive → interactive prompt.
fn read_secret_value(key: &str, value_flag: Option<&str>) -> Result<String> {
    if let Some(flag) = value_flag {
        if flag == "-" {
            return read_secret_value_from_stdin();
        }
        if flag.is_empty() {
            return Err(anyhow!("--value must not be empty"));
        }
        return Ok(flag.to_string());
    }

    if let Ok(value) = std::env::var("TACHYON_SECRET_VALUE") {
        if value.is_empty() {
            return Err(anyhow!("TACHYON_SECRET_VALUE must not be empty"));
        }
        return Ok(value);
    }

    if !io::stdin().is_terminal() {
        return read_secret_value_from_stdin();
    }

    let value = Password::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Enter value for {key}"))
        .allow_empty_password(false)
        .interact()?;
    Ok(value)
}

fn read_secret_value_from_stdin() -> Result<String> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|error| anyhow!("failed to read secret value from stdin: {error}"))?;
    let value = buf.trim_end_matches(['\r', '\n']).to_string();
    if value.is_empty() {
        return Err(anyhow!("secret value from stdin must not be empty"));
    }
    Ok(value)
}

fn manifest_secret_path<'a>(secret_ref: &'a str, key: &str) -> Result<&'a str> {
    let path = secret_ref
        .strip_prefix("$secret_ref:")
        .ok_or_else(|| anyhow!("API returned an invalid app secret reference"))?;
    let mut parts = path.split('/');
    let app_name = parts.next().unwrap_or_default();
    let secret_key = parts.next().unwrap_or_default();
    if app_name.is_empty() || secret_key != key || parts.next().is_some() {
        return Err(anyhow!(
            "API returned an invalid app secret reference for {key}"
        ));
    }
    Ok(path)
}

fn update_manifest_secret_ref(
    config_flag: Option<&Path>,
    key: &str,
    target: &str,
    secret_path: &str,
) -> Result<()> {
    let loaded = crate::config::loader::load_with_path(config_flag)?
        .ok_or_else(|| anyhow!("tachyon.yml not found. Run `tachyon init` first."))?;
    upsert_manifest_secret_ref(&loaded.path, key, target, secret_path)
}

fn upsert_manifest_secret_ref(
    path: &Path,
    key: &str,
    target: &str,
    secret_path: &str,
) -> Result<()> {
    let raw = std::fs::read_to_string(path)?;
    let doc = Document::from_str(&raw)?;
    let root = doc
        .as_mapping()
        .ok_or_else(|| anyhow!("manifest root must be an object"))?;
    let kind = yaml_mapping_string(&root, "kind").unwrap_or_else(|| "CloudApp".to_string());

    match kind.as_str() {
        "CloudApp" => {
            let spec = ensure_mapping_child(&root, "spec")?;
            upsert_env_var_ref(&spec, key, target, secret_path)?;
        }
        "CloudApps" => {
            let app_name = root
                .get_mapping("metadata")
                .and_then(|metadata| yaml_mapping_string(&metadata, "name"));
            let apps = root
                .get_mapping("spec")
                .and_then(|spec| spec.get_sequence("apps"))
                .ok_or_else(|| anyhow!("CloudApps manifest is missing spec.apps"))?;
            let entry_index = app_name
                .as_deref()
                .and_then(|name| {
                    (0..apps.len()).find(|index| {
                        apps.get(*index)
                            .and_then(|app| {
                                app.as_mapping()
                                    .and_then(|app| yaml_mapping_string(app, "name"))
                            })
                            .as_deref()
                            == Some(name)
                    })
                })
                .unwrap_or(0);
            let entry = apps
                .get(entry_index)
                .ok_or_else(|| anyhow!("CloudApps manifest has no apps"))?;
            let mapping = entry
                .as_mapping()
                .ok_or_else(|| anyhow!("CloudApps spec.apps entry must be an object"))?;
            upsert_env_var_ref(mapping, key, target, secret_path)?;
        }
        other => return Err(anyhow!("unsupported manifest kind: {other}")),
    }

    std::fs::write(path, doc.to_string())?;
    Ok(())
}

fn yaml_mapping_string(mapping: &Mapping, key: &str) -> Option<String> {
    mapping
        .get(key)
        .and_then(|value| value.as_scalar().map(|scalar| scalar.as_string()))
}

fn ensure_mapping_child(value: &Mapping, key: &str) -> Result<Mapping> {
    if !value.contains_key(key) {
        value.set(key, Mapping::new_pending_block());
    }
    value
        .get_mapping(key)
        .ok_or_else(|| anyhow!("{key} must be an object"))
}

fn ensure_sequence_child(value: &Mapping, key: &str) -> Result<Sequence> {
    if !value.contains_key(key) {
        value.set(key, Sequence::new_pending_block());
    }
    value
        .get_sequence(key)
        .ok_or_else(|| anyhow!("{key} must be an array"))
}

fn upsert_env_var_ref(spec: &Mapping, key: &str, target: &str, secret_path: &str) -> Result<()> {
    let env_vars = ensure_sequence_child(spec, "envVars")?;

    for index in 0..env_vars.len() {
        let Some(existing) = env_vars.get(index) else {
            continue;
        };
        let Some(mapping) = existing.as_mapping() else {
            continue;
        };
        if yaml_mapping_string(mapping, "name").as_deref() == Some(key) {
            set_secret_env_mapping(mapping, key, target, secret_path);
            return Ok(());
        }
    }

    let mapping = Mapping::new_pending_block();
    set_secret_env_mapping(&mapping, key, target, secret_path);
    env_vars.push(mapping);
    Ok(())
}

fn set_secret_env_mapping(mapping: &Mapping, key: &str, target: &str, secret_path: &str) {
    mapping.set("name", key);
    mapping.set("type", "credential");
    mapping.remove("value");
    if target == "all" {
        mapping.remove("target");
    } else {
        mapping.set("target", target);
    }

    let value_from = match mapping.get_mapping("valueFrom") {
        Some(value_from) => value_from,
        None => {
            let value_from = Mapping::new_pending_block();
            mapping.set("valueFrom", value_from.clone());
            value_from
        }
    };
    value_from.set("secret", secret_path);
}

pub(super) async fn run_env_delete(api: &ApiClient, app_id: &str, env_id: &str) -> Result<()> {
    api.delete(&format!("/v1/apps/{app_id}/env/{env_id}"))
        .await?;
    println!("Environment variable {env_id} deleted.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn env_response_preserves_secret_source_metadata_in_json() {
        let response: EnvVarResponse = serde_json::from_value(json!({
            "id": "env_01ks18jhh1xvggktfzjx5jqsen",
            "key": "DATABASE_URL",
            "value": "****",
            "target": "preview",
            "branch": null,
            "is_secret": true,
            "secret_source": {
                "source_type": "path",
                "path": "providers/tidb_field_preview",
                "field": "DATABASE_URL"
            }
        }))
        .unwrap();

        let serialized = serde_json::to_value(response).unwrap();
        assert_eq!(serialized["secret_source"]["source_type"], "path");
        assert_eq!(
            serialized["secret_source"]["path"],
            "providers/tidb_field_preview"
        );
        assert_eq!(serialized["secret_source"]["field"], "DATABASE_URL");
        assert!(serialized["secret_source"].get("provider").is_none());
    }
}
