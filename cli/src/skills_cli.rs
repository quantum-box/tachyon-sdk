use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct SkillsArgs {
    #[command(subcommand)]
    pub command: SkillsCommand,
}

#[derive(Subcommand, Debug)]
pub enum SkillsCommand {
    /// List bundled agent skills
    List,
    /// Install bundled agent skills into a local agent skill directory
    Install(InstallArgs),
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Skill name to install. Defaults to all bundled skills.
    pub skill: Option<String>,
    /// Install to ~/.agents/skills
    #[arg(long)]
    pub agents: bool,
    /// Install to ~/.codex/skills
    #[arg(long)]
    pub codex: bool,
    /// Install to ~/.claude/skills
    #[arg(long)]
    pub claude: bool,
    /// Install to a custom skill directory
    #[arg(long)]
    pub target_dir: Option<PathBuf>,
}

#[derive(Debug)]
struct SkillAsset {
    path: &'static str,
    contents: &'static str,
}

#[derive(Debug)]
struct BundledSkill {
    name: &'static str,
    description: &'static str,
    files: &'static [SkillAsset],
}

const CLOUD_APP_DEPLOY_FILES: &[SkillAsset] = &[
    SkillAsset {
        path: "SKILL.md",
        contents: include_str!("../../.agents/skills/cloud-app-deploy/SKILL.md"),
    },
    SkillAsset {
        path: "agents/openai.yaml",
        contents: include_str!("../../.agents/skills/cloud-app-deploy/agents/openai.yaml"),
    },
    SkillAsset {
        path: "references/cloud-app-operations.md",
        contents: include_str!(
            "../../.agents/skills/cloud-app-deploy/references/cloud-app-operations.md"
        ),
    },
];

const BUNDLED_SKILLS: &[BundledSkill] = &[BundledSkill {
    name: "cloud-app-deploy",
    description: "Operate Tachyon Cloud Apps with tachyon compute, tachyon.yml, env vars, build logs, deployments, and feedback reports",
    files: CLOUD_APP_DEPLOY_FILES,
}];

pub fn run(args: &SkillsArgs) -> Result<()> {
    match &args.command {
        SkillsCommand::List => list_skills(),
        SkillsCommand::Install(install_args) => install_skills(install_args),
    }
}

fn list_skills() -> Result<()> {
    for skill in BUNDLED_SKILLS {
        println!("{}\t{}", skill.name, skill.description);
    }
    Ok(())
}

fn install_skills(args: &InstallArgs) -> Result<()> {
    let target_dir = resolve_target_dir(args)?;
    let selected = select_skills(args.skill.as_deref())?;

    fs::create_dir_all(&target_dir)
        .with_context(|| format!("failed to create {}", target_dir.display()))?;

    for skill in selected {
        let skill_dir = target_dir.join(skill.name);
        fs::create_dir_all(&skill_dir)
            .with_context(|| format!("failed to create {}", skill_dir.display()))?;

        for asset in skill.files {
            let output_path = skill_dir.join(asset.path);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::write(&output_path, asset.contents)
                .with_context(|| format!("failed to write {}", output_path.display()))?;
        }

        println!("Installed {} -> {}", skill.name, skill_dir.display());
    }

    println!("Agent skills installed in {}", target_dir.display());
    Ok(())
}

fn resolve_target_dir(args: &InstallArgs) -> Result<PathBuf> {
    let selected_count = [
        args.agents,
        args.codex,
        args.claude,
        args.target_dir.is_some(),
    ]
    .iter()
    .filter(|selected| **selected)
    .count();

    if selected_count > 1 {
        bail!("choose only one of --agents, --codex, --claude, or --target-dir");
    }

    if let Some(path) = &args.target_dir {
        return Ok(path.clone());
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow!("home directory is not available"))?;
    if args.codex {
        Ok(home.join(".codex/skills"))
    } else if args.claude {
        Ok(home.join(".claude/skills"))
    } else {
        Ok(home.join(".agents/skills"))
    }
}

fn select_skills(name: Option<&str>) -> Result<Vec<&'static BundledSkill>> {
    match name {
        Some(name) => BUNDLED_SKILLS
            .iter()
            .find(|skill| skill.name == name)
            .map(|skill| vec![skill])
            .ok_or_else(|| anyhow!("unknown skill `{name}`")),
        None => Ok(BUNDLED_SKILLS.iter().collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_defaults_to_agents() {
        let args = InstallArgs {
            skill: None,
            agents: false,
            codex: false,
            claude: false,
            target_dir: None,
        };

        let target = resolve_target_dir(&args).unwrap();
        assert!(target.ends_with(".agents/skills"));
    }

    #[test]
    fn rejects_multiple_targets() {
        let args = InstallArgs {
            skill: None,
            agents: true,
            codex: true,
            claude: false,
            target_dir: None,
        };

        let err = resolve_target_dir(&args).unwrap_err();
        assert!(err.to_string().contains("choose only one"));
    }

    #[test]
    fn selects_known_skill() {
        let selected = select_skills(Some("cloud-app-deploy")).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, "cloud-app-deploy");
    }

    #[test]
    fn rejects_unknown_skill() {
        let err = select_skills(Some("missing")).unwrap_err();
        assert!(err.to_string().contains("unknown skill"));
    }
}
