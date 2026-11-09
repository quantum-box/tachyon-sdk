use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::auth;

pub const PM_NO_DELEGATE_ENV: &str = "TACHYON_PM_NO_DELEGATE";
pub const PM_DEFAULT_TEAM_ENV: &str = "TACHYON_PM_DEFAULT_TEAM";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    #[serde(default)]
    pub profiles: BTreeMap<String, ProfileSettings>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSettings {
    #[serde(default, skip_serializing_if = "PmSettings::is_empty")]
    pub pm: PmSettings,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

impl ProfileSettings {
    fn is_empty(&self) -> bool {
        self.pm.is_empty() && self.extra.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PmSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_delegate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_team: Option<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

impl PmSettings {
    fn is_empty(&self) -> bool {
        self.no_delegate.is_none() && self.default_team.is_none() && self.extra.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedPmSettings {
    pub no_delegate: Option<bool>,
    pub default_team: Option<String>,
}

pub fn settings_path() -> Result<PathBuf> {
    Ok(auth::config_dir()?.join("settings.json"))
}

pub fn load() -> Result<Settings> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(Settings::default());
    }

    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn save(settings: &Settings) -> Result<()> {
    let path = settings_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("settings path has no parent: {}", path.display()))?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("failed to create {}", parent.display()))?;

    let temporary_path = path.with_extension("tmp");
    let data = serde_json::to_string_pretty(settings)?;
    std::fs::write(&temporary_path, format!("{data}\n"))
        .with_context(|| format!("failed to write {}", temporary_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temporary_path, std::fs::Permissions::from_mode(0o600))?;
    }

    std::fs::rename(&temporary_path, &path).with_context(|| {
        format!(
            "failed to replace {} using {}",
            path.display(),
            temporary_path.display()
        )
    })?;
    Ok(())
}

pub fn profile(settings: &Settings, profile: &str) -> Result<ProfileSettings> {
    auth::validate_profile_name(profile)?;
    Ok(settings.profiles.get(profile).cloned().unwrap_or_default())
}

pub fn profile_mut<'a>(
    settings: &'a mut Settings,
    profile: &str,
) -> Result<&'a mut ProfileSettings> {
    auth::validate_profile_name(profile)?;
    Ok(settings.profiles.entry(profile.to_string()).or_default())
}

pub fn remove_empty_profile(settings: &mut Settings, profile: &str) {
    if settings
        .profiles
        .get(profile)
        .is_some_and(ProfileSettings::is_empty)
    {
        settings.profiles.remove(profile);
    }
}

pub fn resolve_pm(profile_name: &str) -> Result<ResolvedPmSettings> {
    let stored = profile(&load()?, profile_name)?.pm;
    Ok(ResolvedPmSettings {
        no_delegate: parse_bool_env(PM_NO_DELEGATE_ENV)?.or(stored.no_delegate),
        default_team: parse_non_empty_env(PM_DEFAULT_TEAM_ENV)?.or(stored.default_team),
    })
}

fn parse_bool_env(name: &str) -> Result<Option<bool>> {
    let Some(value) = std::env::var_os(name) else {
        return Ok(None);
    };
    let value = env_value(name, value)?;
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(Some(true)),
        "false" => Ok(Some(false)),
        _ => Err(anyhow!("{name} must be 'true' or 'false'")),
    }
}

fn parse_non_empty_env(name: &str) -> Result<Option<String>> {
    let Some(value) = std::env::var_os(name) else {
        return Ok(None);
    };
    let value = env_value(name, value)?;
    let value = value.trim();
    if value.is_empty() {
        return Err(anyhow!("{name} must not be empty"));
    }
    Ok(Some(value.to_string()))
}

fn env_value(name: &str, value: OsString) -> Result<String> {
    value
        .into_string()
        .map_err(|_| anyhow!("{name} must contain valid UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_profile_is_removed_after_unset() {
        let mut settings = Settings::default();
        profile_mut(&mut settings, "admin").unwrap().pm.no_delegate = Some(true);
        settings.profiles.get_mut("admin").unwrap().pm.no_delegate = None;

        remove_empty_profile(&mut settings, "admin");

        assert!(!settings.profiles.contains_key("admin"));
    }

    #[test]
    fn unknown_fields_survive_a_typed_settings_round_trip() {
        let mut settings: Settings = serde_json::from_value(serde_json::json!({
            "schema_version": 2,
            "profiles": {
                "admin": {
                    "future_profile_setting": "keep",
                    "pm": {
                        "no_delegate": true,
                        "future_pm_setting": ["keep"]
                    }
                }
            }
        }))
        .unwrap();
        profile_mut(&mut settings, "admin").unwrap().pm.default_team =
            Some("Platform Team".to_string());

        let serialized = serde_json::to_value(settings).unwrap();

        assert_eq!(serialized["schema_version"], 2);
        assert_eq!(
            serialized["profiles"]["admin"]["future_profile_setting"],
            "keep"
        );
        assert_eq!(
            serialized["profiles"]["admin"]["pm"]["future_pm_setting"],
            serde_json::json!(["keep"])
        );
    }
}
