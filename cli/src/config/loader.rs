use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "tachyon.yml";
const CONFIG_ENV: &str = "TACHYON_CONFIG";

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ProjectConfig {
    #[serde(default)]
    pub metadata: ProjectMetadata,
    #[serde(default)]
    pub auth: Option<crate::config::auth::AuthConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ProjectMetadata {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, alias = "tenantId")]
    pub tenant_id: Option<String>,
}

pub fn load(config_flag: Option<&Path>) -> Result<Option<ProjectConfig>> {
    let cwd = env::current_dir()?;
    load_from(&cwd, config_flag)
}

pub fn load_with_path(config_flag: Option<&Path>) -> Result<Option<LoadedProjectConfig>> {
    let cwd = env::current_dir()?;
    load_with_path_from(&cwd, config_flag)
}

pub fn load_with_path_from_dir(
    cwd: &Path,
    config_flag: Option<&Path>,
) -> Result<Option<LoadedProjectConfig>> {
    load_with_path_from(cwd, config_flag)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedProjectConfig {
    pub path: PathBuf,
    pub config: ProjectConfig,
}

fn load_from(cwd: &Path, config_flag: Option<&Path>) -> Result<Option<ProjectConfig>> {
    Ok(load_with_path_from(cwd, config_flag)?.map(|loaded| loaded.config))
}

fn load_with_path_from(
    cwd: &Path,
    config_flag: Option<&Path>,
) -> Result<Option<LoadedProjectConfig>> {
    if let Some(path) = env::var_os(CONFIG_ENV) {
        let path = resolve_path(cwd, Path::new(&path));
        return load_path(&path).map(|config| Some(LoadedProjectConfig { path, config }));
    }

    if let Some(path) = config_flag {
        let path = resolve_path(cwd, path);
        return load_path(&path).map(|config| Some(LoadedProjectConfig { path, config }));
    }

    match discover(cwd) {
        Some(path) => load_path(&path).map(|config| Some(LoadedProjectConfig { path, config })),
        None => Ok(None),
    }
}

fn resolve_path(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn load_path(path: &Path) -> Result<ProjectConfig> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read Tachyon config: {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parse Tachyon config: {}", path.display()))
}

fn discover(cwd: &Path) -> Option<PathBuf> {
    let mut dir = cwd;
    loop {
        let candidate = dir.join(CONFIG_FILE);
        if candidate.is_file() {
            return Some(candidate);
        }
        if dir.join(".git").exists() {
            return None;
        }
        dir = dir.parent()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        key: &'static str,
        old: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn remove(key: &'static str) -> Self {
            let old = env::var_os(key);
            env::remove_var(key);
            Self { key, old }
        }

        fn set(key: &'static str, value: &Path) -> Self {
            let old = env::var_os(key);
            env::set_var(key, value);
            Self { key, old }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }

    fn write_config(path: &Path, name: &str, tenant_id: &str) {
        std::fs::write(
            path,
            format!(
                "apiVersion: tachyon/v1\nkind: CloudApp\nmetadata:\n  name: {name}\n  tenant_id: {tenant_id}\nspec:\n  framework: vite\n"
            ),
        )
        .unwrap();
    }

    fn write_camel_config(path: &Path, name: &str, tenant_id: &str) {
        std::fs::write(
            path,
            format!(
                "apiVersion: tachyon/v1\nkind: CloudApp\nmetadata:\n  name: {name}\n  tenantId: {tenant_id}\nspec:\n  framework: vite\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn discovers_config_from_parent_until_git_root() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::remove(CONFIG_ENV);
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();
        std::fs::create_dir_all(tmp.path().join("a/b")).unwrap();
        write_config(&tmp.path().join(CONFIG_FILE), "from-parent", "tn_parent");

        let loaded = load_from(&tmp.path().join("a/b"), None).unwrap().unwrap();
        assert_eq!(loaded.metadata.name.as_deref(), Some("from-parent"));
        assert_eq!(loaded.metadata.tenant_id.as_deref(), Some("tn_parent"));
    }

    #[test]
    fn parses_camel_case_tenant_id_alias() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::remove(CONFIG_ENV);
        let tmp = TempDir::new().unwrap();
        write_camel_config(&tmp.path().join(CONFIG_FILE), "camel", "tn_camel");

        let loaded = load_from(tmp.path(), None).unwrap().unwrap();
        assert_eq!(loaded.metadata.name.as_deref(), Some("camel"));
        assert_eq!(loaded.metadata.tenant_id.as_deref(), Some("tn_camel"));
    }

    #[test]
    fn does_not_search_above_git_root() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::remove(CONFIG_ENV);
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();
        std::fs::create_dir_all(tmp.path().join("repo/sub")).unwrap();
        std::fs::create_dir(tmp.path().join("repo/.git")).unwrap();
        write_config(&tmp.path().join(CONFIG_FILE), "above", "tn_above");

        let loaded = load_from(&tmp.path().join("repo/sub"), None).unwrap();
        assert_eq!(loaded, None);
    }

    #[test]
    fn precedence_is_env_then_flag_then_discovery() {
        let _lock = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();
        let env_path = tmp.path().join("env.yml");
        let flag_path = tmp.path().join("flag.yml");
        write_config(&tmp.path().join(CONFIG_FILE), "discovered", "tn_discovered");
        write_config(&flag_path, "flag", "tn_flag");
        write_config(&env_path, "env", "tn_env");

        let guard = EnvGuard::set(CONFIG_ENV, &env_path);
        let loaded = load_from(tmp.path(), Some(&flag_path)).unwrap().unwrap();
        assert_eq!(loaded.metadata.name.as_deref(), Some("env"));
        drop(guard);

        let _guard = EnvGuard::remove(CONFIG_ENV);
        let loaded = load_from(tmp.path(), Some(&flag_path)).unwrap().unwrap();
        assert_eq!(loaded.metadata.name.as_deref(), Some("flag"));

        let loaded = load_from(tmp.path(), None).unwrap().unwrap();
        assert_eq!(loaded.metadata.name.as_deref(), Some("discovered"));
    }
}
