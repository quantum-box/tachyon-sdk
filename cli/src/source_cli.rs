use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Args)]
pub struct SourceArgs {
    #[command(subcommand)]
    pub command: SourceCommand,
}
#[derive(Debug, Subcommand)]
pub enum SourceCommand {
    /// Clone a GitHub repository using a token from GITHUB_TOKEN.
    Checkout {
        #[arg(long)]
        repository: String,
        #[arg(long)]
        branch: Option<String>,
        #[arg(long)]
        destination: PathBuf,
        #[arg(long, default_value_t = 1)]
        depth: u32,
    },
}

pub fn run(args: &SourceArgs) -> Result<()> {
    match &args.command {
        SourceCommand::Checkout {
            repository,
            branch,
            destination,
            depth,
        } => checkout(repository, branch.as_deref(), destination, *depth),
    }
}

fn checkout(
    repository: &str,
    branch: Option<&str>,
    destination: &PathBuf,
    depth: u32,
) -> Result<()> {
    validate_github_repository_url(repository)?;
    if depth == 0 {
        bail!("--depth must be greater than zero");
    }
    let token =
        std::env::var("GITHUB_TOKEN").context("GITHUB_TOKEN is required for source checkout")?;
    if token.trim().is_empty() {
        bail!("GITHUB_TOKEN must not be empty");
    }

    // Pass auth through git's process environment so it is absent from argv
    // and remove the original variable from the child process.
    let mut command = Command::new("git");
    command
        .env_remove("GITHUB_TOKEN")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "http.https://github.com/.extraheader")
        .env(
            "GIT_CONFIG_VALUE_0",
            format!("AUTHORIZATION: bearer {token}"),
        )
        .arg("clone")
        .arg("--depth")
        .arg(depth.to_string());
    if let Some(branch) = branch.filter(|branch| !branch.is_empty()) {
        command.arg("--branch").arg(branch);
    }
    let status = command
        .arg("--")
        .arg(repository)
        .arg(destination)
        .status()
        .context("failed to start git clone")?;
    if !status.success() {
        bail!("git clone failed with {status}");
    }
    Ok(())
}

fn validate_github_repository_url(repository: &str) -> Result<()> {
    let prefix = "https://github.com/";
    let path = repository
        .strip_prefix(prefix)
        .ok_or_else(|| anyhow::anyhow!("repository must be a GitHub HTTPS URL"))?;
    if repository.contains(['?', '#', '@']) {
        bail!("repository must not contain credentials, query, or fragment");
    }
    let parts = path.trim_end_matches(".git").split('/').collect::<Vec<_>>();
    if parts.len() != 2 || parts.iter().any(|part| part.is_empty()) {
        bail!("repository must identify one GitHub repository");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_github_https_urls() {
        assert!(
            validate_github_repository_url("https://github.com/quantum-box/private-repo.git")
                .is_ok()
        );
    }

    #[test]
    fn rejects_non_github_and_embedded_credentials() {
        assert!(validate_github_repository_url("https://example.com/a/b").is_err());
        assert!(validate_github_repository_url("https://token@github.com/a/b").is_err());
    }
}
