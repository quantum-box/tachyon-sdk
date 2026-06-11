use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use dialoguer::{theme::ColorfulTheme, Select};
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
    /// Install scope. User scope writes under ~/.<agent>/skills; workspace scope writes under the current workspace.
    #[arg(long, value_enum)]
    pub scope: Option<SkillScope>,
    /// Disable prompts and use flags/defaults instead
    #[arg(long)]
    pub non_interactive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SkillScope {
    User,
    Workspace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SkillHost {
    Agents,
    Codex,
    Claude,
}

impl SkillHost {
    fn dirname(self) -> &'static str {
        match self {
            SkillHost::Agents => ".agents",
            SkillHost::Codex => ".codex",
            SkillHost::Claude => ".claude",
        }
    }

    fn label(self) -> &'static str {
        match self {
            SkillHost::Agents => "Agents (~/.agents/skills)",
            SkillHost::Codex => "Codex (~/.codex/skills)",
            SkillHost::Claude => "Claude Code (~/.claude/skills)",
        }
    }
}

#[derive(Debug)]
struct SkillAsset {
    path: &'static str,
    contents: &'static str,
}

#[derive(Debug)]
struct BundledSkill {
    name: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
    files: &'static [SkillAsset],
}

const TACHYON_CLOUD_FILES: &[SkillAsset] = &[
    SkillAsset {
        path: "SKILL.md",
        contents: include_str!("../../.agents/skills/tachyon-cloud/SKILL.md"),
    },
    SkillAsset {
        path: "agents/openai.yaml",
        contents: include_str!("../../.agents/skills/tachyon-cloud/agents/openai.yaml"),
    },
    SkillAsset {
        path: "references/cloud-app-operations.md",
        contents: include_str!(
            "../../.agents/skills/tachyon-cloud/references/cloud-app-operations.md"
        ),
    },
];

const BUNDLED_SKILLS: &[BundledSkill] = &[BundledSkill {
    name: "tachyon-cloud",
    aliases: &["cloud-app-deploy"],
    description: "Operate Tachyon Cloud Apps with tachyon compute, tachyon.yml, env vars, build logs, deployments, and feedback reports",
    files: TACHYON_CLOUD_FILES,
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
    let install_plan = resolve_install_plan(args)?;

    fs::create_dir_all(&install_plan.target_dir)
        .with_context(|| format!("failed to create {}", install_plan.target_dir.display()))?;

    for skill in install_plan.skills {
        let skill_dir = install_plan.target_dir.join(skill.name);
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

    println!(
        "Agent skills installed in {}",
        install_plan.target_dir.display()
    );
    Ok(())
}

struct InstallPlan {
    target_dir: PathBuf,
    skills: Vec<&'static BundledSkill>,
}

fn resolve_install_plan(args: &InstallArgs) -> Result<InstallPlan> {
    let target_dir = resolve_target_dir(args)?;
    let skills = resolve_selected_skills(args)?;

    Ok(InstallPlan { target_dir, skills })
}

fn resolve_selected_skills(args: &InstallArgs) -> Result<Vec<&'static BundledSkill>> {
    if args.non_interactive || args.skill.is_some() {
        return select_skills(args.skill.as_deref());
    }

    prompt_for_skills()
}

fn resolve_target_dir(args: &InstallArgs) -> Result<PathBuf> {
    let host = resolve_host(args)?;
    let selected_count = [args.target_dir.is_some(), host.is_some()]
        .iter()
        .filter(|selected| **selected)
        .count();

    if selected_count > 1 {
        bail!("choose only one of --agents, --codex, --claude, or --target-dir");
    }

    if let Some(path) = &args.target_dir {
        return Ok(path.clone());
    }

    let host = if args.non_interactive {
        host.unwrap_or(SkillHost::Agents)
    } else {
        match host {
            Some(host) => host,
            None => prompt_for_host()?,
        }
    };
    let scope = if args.non_interactive {
        args.scope.unwrap_or(SkillScope::User)
    } else {
        match args.scope {
            Some(scope) => scope,
            None => prompt_for_scope()?,
        }
    };

    target_dir_for(host, scope)
}

fn resolve_host(args: &InstallArgs) -> Result<Option<SkillHost>> {
    let selected = [
        (args.agents, SkillHost::Agents),
        (args.codex, SkillHost::Codex),
        (args.claude, SkillHost::Claude),
    ]
    .into_iter()
    .filter_map(|(enabled, host)| enabled.then_some(host))
    .collect::<Vec<_>>();

    if selected.len() > 1 {
        bail!("choose only one of --agents, --codex, or --claude");
    }

    Ok(selected.first().copied())
}

fn target_dir_for(host: SkillHost, scope: SkillScope) -> Result<PathBuf> {
    let base = match scope {
        SkillScope::User => {
            dirs::home_dir().ok_or_else(|| anyhow!("home directory is not available"))?
        }
        SkillScope::Workspace => std::env::current_dir().context("failed to read current dir")?,
    };

    Ok(base.join(host.dirname()).join("skills"))
}

fn prompt_for_skills() -> Result<Vec<&'static BundledSkill>> {
    let mut labels = vec!["All bundled skills".to_string()];
    labels.extend(
        BUNDLED_SKILLS
            .iter()
            .map(|skill| format!("{} - {}", skill.name, skill.description)),
    );

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Install which skill?")
        .items(&labels)
        .default(0)
        .interact()
        .context("skill selection was cancelled")?;

    if selection == 0 {
        Ok(BUNDLED_SKILLS.iter().collect())
    } else {
        Ok(vec![&BUNDLED_SKILLS[selection - 1]])
    }
}

fn prompt_for_host() -> Result<SkillHost> {
    let hosts = [SkillHost::Agents, SkillHost::Codex, SkillHost::Claude];
    let labels = hosts.iter().map(|host| host.label()).collect::<Vec<_>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Install for which agent?")
        .items(&labels)
        .default(0)
        .interact()
        .context("agent selection was cancelled")?;

    Ok(hosts[selection])
}

fn prompt_for_scope() -> Result<SkillScope> {
    let scopes = [SkillScope::User, SkillScope::Workspace];
    let labels = ["User scope", "Workspace scope"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Install scope?")
        .items(labels)
        .default(0)
        .interact()
        .context("scope selection was cancelled")?;

    Ok(scopes[selection])
}

fn select_skills(name: Option<&str>) -> Result<Vec<&'static BundledSkill>> {
    match name {
        Some(name) => BUNDLED_SKILLS
            .iter()
            .find(|skill| skill.name == name || skill.aliases.contains(&name))
            .map(|skill| vec![skill])
            .ok_or_else(|| anyhow!("unknown skill `{name}`")),
        None => Ok(BUNDLED_SKILLS.iter().collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_install_args() -> InstallArgs {
        InstallArgs {
            skill: None,
            agents: false,
            codex: false,
            claude: false,
            target_dir: None,
            scope: None,
            non_interactive: true,
        }
    }

    #[test]
    fn target_defaults_to_agents() {
        let args = default_install_args();

        let target = resolve_target_dir(&args).unwrap();
        assert!(target.ends_with(".agents/skills"));
    }

    #[test]
    fn codex_workspace_scope_targets_current_workspace() {
        let mut args = default_install_args();
        args.codex = true;
        args.scope = Some(SkillScope::Workspace);

        let target = resolve_target_dir(&args).unwrap();
        assert!(target.ends_with(".codex/skills"));
        assert!(target.starts_with(std::env::current_dir().unwrap()));
    }

    #[test]
    fn rejects_multiple_targets() {
        let mut args = default_install_args();
        args.agents = true;
        args.codex = true;

        let err = resolve_target_dir(&args).unwrap_err();
        assert!(err.to_string().contains("choose only one"));
    }

    #[test]
    fn selects_known_skill() {
        let selected = select_skills(Some("tachyon-cloud")).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, "tachyon-cloud");
    }

    #[test]
    fn accepts_legacy_skill_alias() {
        let selected = select_skills(Some("cloud-app-deploy")).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, "tachyon-cloud");
    }

    #[test]
    fn rejects_unknown_skill() {
        let err = select_skills(Some("missing")).unwrap_err();
        assert!(err.to_string().contains("unknown skill"));
    }
}
