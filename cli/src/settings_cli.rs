use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};

use crate::settings;

#[derive(Debug, Clone, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommand {
    /// Set a value for the selected profile
    Set {
        /// Setting key: pm.no_delegate or pm.default_team
        key: String,
        /// Setting value
        value: String,
    },
    /// Get one value or all values for the selected profile
    Get {
        /// Optional setting key: pm.no_delegate or pm.default_team
        key: Option<String>,
    },
    /// Unset a value for the selected profile
    Unset {
        /// Setting key: pm.no_delegate or pm.default_team
        key: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingKey {
    PmNoDelegate,
    PmDefaultTeam,
}

impl SettingKey {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "pm.no_delegate" => Ok(Self::PmNoDelegate),
            "pm.default_team" => Ok(Self::PmDefaultTeam),
            _ => Err(anyhow!(
                "unsupported setting '{value}'; expected pm.no_delegate or pm.default_team"
            )),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::PmNoDelegate => "pm.no_delegate",
            Self::PmDefaultTeam => "pm.default_team",
        }
    }
}

pub fn run(args: &ConfigArgs, profile_name: &str) -> Result<()> {
    match &args.command {
        ConfigCommand::Set { key, value } => set(profile_name, SettingKey::parse(key)?, value),
        ConfigCommand::Get { key } => get(
            profile_name,
            key.as_deref().map(SettingKey::parse).transpose()?,
        ),
        ConfigCommand::Unset { key } => unset(profile_name, SettingKey::parse(key)?),
    }
}

fn set(profile_name: &str, key: SettingKey, value: &str) -> Result<()> {
    let mut settings = settings::load()?;
    let profile = settings::profile_mut(&mut settings, profile_name)?;

    match key {
        SettingKey::PmNoDelegate => {
            profile.pm.no_delegate = Some(parse_bool(value)?);
        }
        SettingKey::PmDefaultTeam => {
            let value = value.trim();
            if value.is_empty() {
                return Err(anyhow!("pm.default_team must not be empty"));
            }
            profile.pm.default_team = Some(value.to_string());
        }
    }

    settings::save(&settings)?;
    println!("Set {} for profile '{profile_name}'.", key.name());
    Ok(())
}

fn get(profile_name: &str, key: Option<SettingKey>) -> Result<()> {
    let settings = settings::load()?;
    let profile = settings::profile(&settings, profile_name)?;

    match key {
        None => println!("{}", serde_json::to_string_pretty(&profile)?),
        Some(SettingKey::PmNoDelegate) => match profile.pm.no_delegate {
            Some(value) => println!("{value}"),
            None => return Err(not_set(profile_name, SettingKey::PmNoDelegate)),
        },
        Some(SettingKey::PmDefaultTeam) => match profile.pm.default_team {
            Some(value) => println!("{value}"),
            None => return Err(not_set(profile_name, SettingKey::PmDefaultTeam)),
        },
    }
    Ok(())
}

fn unset(profile_name: &str, key: SettingKey) -> Result<()> {
    let mut settings = settings::load()?;
    let mut profile = settings::profile(&settings, profile_name)?;
    let removed = match key {
        SettingKey::PmNoDelegate => profile.pm.no_delegate.take().is_some(),
        SettingKey::PmDefaultTeam => profile.pm.default_team.take().is_some(),
    };

    if removed {
        settings.profiles.insert(profile_name.to_string(), profile);
        settings::remove_empty_profile(&mut settings, profile_name);
        settings::save(&settings)?;
        println!("Unset {} for profile '{profile_name}'.", key.name());
    } else {
        println!("{} is not set for profile '{profile_name}'.", key.name());
    }
    Ok(())
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(anyhow!("pm.no_delegate must be 'true' or 'false'")),
    }
}

fn not_set(profile_name: &str, key: SettingKey) -> anyhow::Error {
    anyhow!("{} is not set for profile '{profile_name}'", key.name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_keys() {
        assert_eq!(
            SettingKey::parse("pm.no_delegate").unwrap(),
            SettingKey::PmNoDelegate
        );
        assert_eq!(
            SettingKey::parse("pm.default_team").unwrap(),
            SettingKey::PmDefaultTeam
        );
        assert!(SettingKey::parse("pm.unknown").is_err());
    }

    #[test]
    fn parses_boolean_values_strictly() {
        assert!(parse_bool("true").unwrap());
        assert!(!parse_bool("FALSE").unwrap());
        assert!(parse_bool("1").is_err());
    }

    #[test]
    fn empty_profile_serializes_as_an_object() {
        assert_eq!(
            serde_json::to_value(settings::ProfileSettings::default()).unwrap(),
            serde_json::json!({})
        );
    }
}
