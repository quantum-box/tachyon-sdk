use super::*;

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

    update_manifest_secret_ref(config_flag, &resp.key, &resp.target)?;
    println!("Set secret {} for target {}.", resp.key, resp.target);
    println!("Updated tachyon.yml with valueFrom.secret: {}", resp.key);
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

fn update_manifest_secret_ref(config_flag: Option<&Path>, key: &str, target: &str) -> Result<()> {
    let loaded = crate::config::loader::load_with_path(config_flag)?
        .ok_or_else(|| anyhow!("tachyon.yml not found. Run `tachyon init` first."))?;
    upsert_manifest_secret_ref(&loaded.path, key, target)
}

fn upsert_manifest_secret_ref(path: &Path, key: &str, target: &str) -> Result<()> {
    let raw = std::fs::read_to_string(path)?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&raw)?;
    let kind = doc
        .get("kind")
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or("CloudApp");

    match kind {
        "CloudApp" => {
            let spec = ensure_mapping_child(&mut doc, "spec")?;
            upsert_env_var_ref(spec, key, target)?;
        }
        "CloudApps" => {
            let app_name = doc
                .get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(serde_yaml::Value::as_str)
                .map(ToString::to_string);
            let apps = doc
                .get_mut("spec")
                .and_then(|s| s.get_mut("apps"))
                .and_then(serde_yaml::Value::as_sequence_mut)
                .ok_or_else(|| anyhow!("CloudApps manifest is missing spec.apps"))?;
            let entry_index = app_name
                .as_deref()
                .and_then(|name| {
                    apps.iter().position(|app| {
                        app.get("name").and_then(serde_yaml::Value::as_str) == Some(name)
                    })
                })
                .unwrap_or(0);
            let entry = apps
                .get_mut(entry_index)
                .ok_or_else(|| anyhow!("CloudApps manifest has no apps"))?;
            let mapping = entry
                .as_mapping_mut()
                .ok_or_else(|| anyhow!("CloudApps spec.apps entry must be an object"))?;
            upsert_env_var_ref(mapping, key, target)?;
        }
        other => return Err(anyhow!("unsupported manifest kind: {other}")),
    }

    let next = serde_yaml::to_string(&doc)?;
    std::fs::write(path, next)?;
    Ok(())
}

fn ensure_mapping_child<'a>(
    value: &'a mut serde_yaml::Value,
    key: &str,
) -> Result<&'a mut serde_yaml::Mapping> {
    let mapping = value
        .as_mapping_mut()
        .ok_or_else(|| anyhow!("manifest root must be an object"))?;
    let key_value = serde_yaml::Value::String(key.to_string());
    if !mapping.contains_key(&key_value) {
        mapping.insert(
            key_value.clone(),
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
    }
    mapping
        .get_mut(&key_value)
        .and_then(serde_yaml::Value::as_mapping_mut)
        .ok_or_else(|| anyhow!("{key} must be an object"))
}

fn upsert_env_var_ref(spec: &mut serde_yaml::Mapping, key: &str, target: &str) -> Result<()> {
    let env_key = serde_yaml::Value::String("envVars".to_string());
    if !spec.contains_key(&env_key) {
        spec.insert(env_key.clone(), serde_yaml::Value::Sequence(Vec::new()));
    }
    let env_vars = spec
        .get_mut(&env_key)
        .and_then(serde_yaml::Value::as_sequence_mut)
        .ok_or_else(|| anyhow!("spec.envVars must be an array"))?;

    if let Some(existing) = env_vars
        .iter_mut()
        .find(|env| env.get("name").and_then(serde_yaml::Value::as_str) == Some(key))
    {
        let mapping = existing
            .as_mapping_mut()
            .ok_or_else(|| anyhow!("spec.envVars entries must be objects"))?;
        set_secret_env_mapping(mapping, key, target);
        return Ok(());
    }

    let mut mapping = serde_yaml::Mapping::new();
    set_secret_env_mapping(&mut mapping, key, target);
    env_vars.push(serde_yaml::Value::Mapping(mapping));
    Ok(())
}

fn set_secret_env_mapping(mapping: &mut serde_yaml::Mapping, key: &str, target: &str) {
    mapping.insert(yaml_key("name"), serde_yaml::Value::String(key.to_string()));
    mapping.insert(
        yaml_key("type"),
        serde_yaml::Value::String("credential".to_string()),
    );
    mapping.remove(yaml_key("value"));
    if target == "all" {
        mapping.remove(yaml_key("target"));
    } else {
        mapping.insert(
            yaml_key("target"),
            serde_yaml::Value::String(target.to_string()),
        );
    }

    let mut value_from = serde_yaml::Mapping::new();
    value_from.insert(
        yaml_key("secret"),
        serde_yaml::Value::String(key.to_string()),
    );
    mapping.insert(
        yaml_key("valueFrom"),
        serde_yaml::Value::Mapping(value_from),
    );
}

fn yaml_key(key: &str) -> serde_yaml::Value {
    serde_yaml::Value::String(key.to_string())
}

pub(super) async fn run_env_delete(api: &ApiClient, app_id: &str, env_id: &str) -> Result<()> {
    api.delete(&format!("/v1/apps/{app_id}/env/{env_id}"))
        .await?;
    println!("Environment variable {env_id} deleted.");
    Ok(())
}
