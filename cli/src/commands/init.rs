use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};
use dialoguer::{Input, Select};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Framework {
    Nextjs,
    Vite,
    Static,
    None,
}

impl Framework {
    fn as_str(self) -> &'static str {
        match self {
            Framework::Nextjs => "nextjs",
            Framework::Vite => "vite",
            Framework::Static => "static",
            Framework::None => "none",
        }
    }

    fn choices() -> [&'static str; 4] {
        ["nextjs", "vite", "static", "none"]
    }
}

#[derive(Debug, Clone, Args)]
pub struct InitArgs {
    /// Cloud app name (defaults to the repository directory name)
    #[arg(long)]
    pub name: Option<String>,
    /// Web framework for the app
    #[arg(long, value_enum)]
    pub framework: Option<Framework>,
    /// Tenant ID to use for this app config
    #[arg(long)]
    pub tenant_id: Option<String>,
    /// Skip prompts and use provided flags/defaults
    #[arg(long)]
    pub non_interactive: bool,
    /// Overwrite an existing tachyon.yml
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InitConfig {
    name: String,
    framework: Framework,
    tenant_id: String,
}

pub fn run(args: &InitArgs) -> Result<()> {
    run_in_dir(args, &std::env::current_dir()?)
}

fn run_in_dir(args: &InitArgs, dir: &Path) -> Result<()> {
    let config = resolve_config(args, dir)?;
    let path = dir.join("tachyon.yml");
    if path.exists() && !args.force {
        return Err(anyhow!(
            "{} already exists. Re-run with --force to overwrite it.",
            path.display()
        ));
    }

    fs::write(&path, render_template(&config))?;
    println!("Created {}", path.display());
    Ok(())
}

fn resolve_config(args: &InitArgs, dir: &Path) -> Result<InitConfig> {
    let default_name = dir
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("tachyon-app")
        .to_string();

    let name = match (&args.name, args.non_interactive) {
        (Some(name), _) => name.clone(),
        (None, true) => default_name,
        (None, false) => Input::<String>::new()
            .with_prompt("App name")
            .default(default_name)
            .interact_text()?,
    };

    let framework = match (args.framework, args.non_interactive) {
        (Some(framework), _) => framework,
        (None, true) => Framework::None,
        (None, false) => {
            let selected = Select::new()
                .with_prompt("Framework")
                .items(Framework::choices())
                .default(3)
                .interact()?;
            match selected {
                0 => Framework::Nextjs,
                1 => Framework::Vite,
                2 => Framework::Static,
                _ => Framework::None,
            }
        }
    };

    let tenant_id = match (&args.tenant_id, args.non_interactive) {
        (Some(tenant_id), _) if !tenant_id.trim().is_empty() => tenant_id.clone(),
        (_, true) => return Err(anyhow!("--tenant-id is required")),
        _ => Input::<String>::new()
            .with_prompt("Tenant ID")
            .allow_empty(false)
            .interact_text()?,
    };

    Ok(InitConfig {
        name,
        framework,
        tenant_id,
    })
}

fn render_template(config: &InitConfig) -> String {
    format!(
        "apiVersion: tachyon/v1\nkind: CloudApp\nmetadata:\n  name: {}\n  tenant_id: {}\nspec:\n  framework: {}\n  # build_command: yarn build\n  # output_directory: dist\n",
        config.name,
        config.tenant_id,
        config.framework.as_str()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn non_interactive_requires_tenant_id() {
        let tmp = TempDir::new().unwrap();
        let args = InitArgs {
            name: None,
            framework: None,
            tenant_id: None,
            non_interactive: true,
            force: false,
        };

        let err = run_in_dir(&args, tmp.path()).unwrap_err();
        assert!(err.to_string().contains("--tenant-id is required"));
    }

    #[test]
    fn non_interactive_writes_template() {
        let tmp = TempDir::new().unwrap();
        let args = InitArgs {
            name: Some("plt1098".to_string()),
            framework: Some(Framework::Vite),
            tenant_id: Some("tn_test".to_string()),
            non_interactive: true,
            force: false,
        };

        run_in_dir(&args, tmp.path()).unwrap();
        let yaml = fs::read_to_string(tmp.path().join("tachyon.yml")).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed["metadata"]["name"], "plt1098");
        assert_eq!(parsed["metadata"]["tenant_id"], "tn_test");
        assert!(yaml.contains("name: plt1098"));
        assert!(yaml.contains("tenant_id: tn_test"));
        assert!(yaml.contains("framework: vite"));
        assert!(yaml.contains("# build_command: yarn build"));
    }

    #[test]
    fn refuses_to_overwrite_without_force() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("tachyon.yml"), "existing").unwrap();
        let args = InitArgs {
            name: Some("plt1098".to_string()),
            framework: Some(Framework::Static),
            tenant_id: Some("tn_test".to_string()),
            non_interactive: true,
            force: false,
        };

        let err = run_in_dir(&args, tmp.path()).unwrap_err();
        assert!(err.to_string().contains("already exists"));
        assert_eq!(
            fs::read_to_string(tmp.path().join("tachyon.yml")).unwrap(),
            "existing"
        );
    }

    #[test]
    fn force_overwrites_existing_file() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("tachyon.yml"), "existing").unwrap();
        let args = InitArgs {
            name: Some("plt1098".to_string()),
            framework: Some(Framework::Nextjs),
            tenant_id: Some("tn_test".to_string()),
            non_interactive: true,
            force: true,
        };

        run_in_dir(&args, tmp.path()).unwrap();
        let yaml = fs::read_to_string(tmp.path().join("tachyon.yml")).unwrap();
        assert!(yaml.contains("framework: nextjs"));
    }
}
