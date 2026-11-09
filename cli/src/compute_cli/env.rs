use super::*;
use std::str::FromStr;

use yaml_edit::{Mapping, Sequence, YamlFile};

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
    println!("Set secret {} for target {}.", resp.key, resp.target);
    let app_name = secret_path
        .split_once('/')
        .map(|(app_name, _)| app_name)
        .unwrap_or_default();
    let manifest_path =
        update_manifest_secret_ref(config_flag, app_name, &resp.key, &resp.target, secret_path)?;
    match manifest_path {
        Some(path) => println!(
            "Updated {} with valueFrom.secret: {secret_path}",
            path.display()
        ),
        None => println!(
            "Left the local manifest unchanged: it does not declare app {app_name}. \
             Reference the secret from that app's manifest with valueFrom.secret: {secret_path}"
        ),
    }
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

/// Returns the manifest path when it declares `app_name` and was updated.
fn update_manifest_secret_ref(
    config_flag: Option<&Path>,
    app_name: &str,
    key: &str,
    target: &str,
    secret_path: &str,
) -> Result<Option<PathBuf>> {
    let loaded = crate::config::loader::load_with_path(config_flag)?
        .ok_or_else(|| anyhow!("tachyon.yml not found. Run `tachyon init` first."))?;
    let updated = upsert_manifest_secret_ref(&loaded.path, app_name, key, target, secret_path)?;
    Ok(updated.then_some(loaded.path))
}

/// Writes the secret reference into the entry for `app_name`. A manifest
/// that does not declare that app is left untouched and returns `false`, so
/// setting another app's secret never edits an unrelated `tachyon.yml`.
fn upsert_manifest_secret_ref(
    path: &Path,
    app_name: &str,
    key: &str,
    target: &str,
    secret_path: &str,
) -> Result<bool> {
    let raw = std::fs::read_to_string(path)?;
    let file = YamlFile::from_str(&raw)?;
    let doc = file
        .document()
        .ok_or_else(|| anyhow!("manifest must contain a YAML document"))?;
    let root = doc
        .as_mapping()
        .ok_or_else(|| anyhow!("manifest root must be an object"))?;
    let kind = yaml_mapping_string(&root, "kind").unwrap_or_else(|| "CloudApp".to_string());

    match kind.as_str() {
        "CloudApp" => {
            let declared_name = root
                .get_mapping("metadata")
                .and_then(|metadata| yaml_mapping_string(&metadata, "name"));
            if declared_name.as_deref() != Some(app_name) {
                return Ok(false);
            }
            let spec = ensure_mapping_child(&root, "spec")?;
            upsert_env_var_ref(&spec, key, target, secret_path)?;
        }
        "CloudApps" => {
            let apps = root
                .get_mapping("spec")
                .and_then(|spec| spec.get_sequence("apps"))
                .ok_or_else(|| anyhow!("CloudApps manifest is missing spec.apps"))?;
            let Some(entry) = (0..apps.len())
                .filter_map(|index| apps.get(index))
                .find(|app| {
                    app.as_mapping()
                        .and_then(|app| yaml_mapping_string(app, "name"))
                        .as_deref()
                        == Some(app_name)
                })
            else {
                return Ok(false);
            };
            let mapping = entry
                .as_mapping()
                .ok_or_else(|| anyhow!("CloudApps spec.apps entry must be an object"))?;
            upsert_env_var_ref(mapping, key, target, secret_path)?;
        }
        other => return Err(anyhow!("unsupported manifest kind: {other}")),
    }

    std::fs::write(path, file.to_string())?;
    Ok(true)
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

    // yaml-edit drops the indentation of every key after the first in a
    // pushed multi-key mapping. Push the entry with `name` only, then fill
    // it in place like an existing entry, which keeps the block indentation.
    let mapping = Mapping::new_pending_block();
    mapping.set("name", key);
    env_vars.push(mapping);
    let pushed = env_vars
        .get(env_vars.len() - 1)
        .and_then(|entry| entry.as_mapping().cloned())
        .ok_or_else(|| anyhow!("failed to append envVars entry for {key}"))?;
    set_secret_env_mapping(&pushed, key, target, secret_path);
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

    let value_from = Mapping::new_pending_block();
    value_from.set("secret", secret_path);
    mapping.set("valueFrom", value_from);
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

    const CLOUD_APPS_MANIFEST: &str = "\
apiVersion: apps.tachy.one/v1alpha
kind: CloudApps
metadata:
  name: tachyon-apps
spec:
  apps:
  - name: tachyon-console
    envVars:
    - name: OPEN_NEXT_DEPLOY
      value: 'true'
    watchPaths:
    - apps/tachyon/**
  - name: field
    framework: worker
";

    fn write_manifest(content: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tachyon.yml");
        std::fs::write(&path, content).unwrap();
        (dir, path)
    }

    fn app_env_vars(path: &Path, app_name: &str) -> serde_json::Value {
        let manifest: serde_json::Value =
            serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        manifest["spec"]["apps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|app| app["name"] == app_name)
            .unwrap()["envVars"]
            .clone()
    }

    #[test]
    fn secret_ref_for_undeclared_app_leaves_manifest_untouched() {
        let (_dir, path) = write_manifest(CLOUD_APPS_MANIFEST);

        let updated = upsert_manifest_secret_ref(
            &path,
            "tachyonfield",
            "NEXTAUTH_URL",
            "preview",
            "tachyonfield/NEXTAUTH_URL",
        )
        .unwrap();

        assert!(!updated);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), CLOUD_APPS_MANIFEST);
    }

    #[test]
    fn secret_ref_for_undeclared_single_app_leaves_manifest_untouched() {
        let manifest = "kind: CloudApp\nmetadata:\n  name: web\nspec:\n  envVars:\n  - name: A\n    value: b\n";
        let (_dir, path) = write_manifest(manifest);

        let updated =
            upsert_manifest_secret_ref(&path, "api", "TOKEN", "all", "api/TOKEN").unwrap();

        assert!(!updated);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), manifest);
    }

    #[test]
    fn secret_ref_is_appended_to_the_named_app_with_block_indentation() {
        let (_dir, path) = write_manifest(CLOUD_APPS_MANIFEST);

        for app_name in ["tachyon-console", "field"] {
            let updated = upsert_manifest_secret_ref(
                &path,
                app_name,
                "AUTH_SECRET",
                "preview",
                &format!("{app_name}/AUTH_SECRET"),
            )
            .unwrap();
            assert!(updated);
        }

        assert_eq!(
            app_env_vars(&path, "tachyon-console"),
            json!([
                {"name": "OPEN_NEXT_DEPLOY", "value": "true"},
                {
                    "name": "AUTH_SECRET",
                    "type": "credential",
                    "target": "preview",
                    "valueFrom": {"secret": "tachyon-console/AUTH_SECRET"}
                }
            ])
        );
        assert_eq!(
            app_env_vars(&path, "field"),
            json!([{
                "name": "AUTH_SECRET",
                "type": "credential",
                "target": "preview",
                "valueFrom": {"secret": "field/AUTH_SECRET"}
            }])
        );
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("      type: credential\n"), "{raw}");
        assert!(
            raw.contains("        secret: tachyon-console/AUTH_SECRET\n"),
            "{raw}"
        );
    }

    #[test]
    fn secret_ref_replaces_an_existing_entry_in_place() {
        let (_dir, path) = write_manifest(CLOUD_APPS_MANIFEST);

        let updated = upsert_manifest_secret_ref(
            &path,
            "tachyon-console",
            "OPEN_NEXT_DEPLOY",
            "all",
            "tachyon-console/OPEN_NEXT_DEPLOY",
        )
        .unwrap();

        assert!(updated);
        assert_eq!(
            app_env_vars(&path, "tachyon-console"),
            json!([{
                "name": "OPEN_NEXT_DEPLOY",
                "type": "credential",
                "valueFrom": {"secret": "tachyon-console/OPEN_NEXT_DEPLOY"}
            }])
        );
    }

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
