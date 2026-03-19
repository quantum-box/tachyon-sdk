use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const REPO: &str = "quantum-box/tachyon-sdk";
const BIN_NAME: &str = "tachyon";

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

/// Detect the OS string used in release artifact names.
fn detect_os() -> Result<&'static str> {
    match env::consts::OS {
        "linux" => Ok("linux"),
        "macos" => Ok("darwin"),
        _ => bail!("Unsupported OS: {}", env::consts::OS),
    }
}

/// Detect the architecture string used in release artifact names.
fn detect_arch() -> Result<&'static str> {
    match env::consts::ARCH {
        "x86_64" => Ok("x86_64"),
        "aarch64" => Ok("arm64"),
        _ => bail!("Unsupported architecture: {}", env::consts::ARCH),
    }
}

/// Determine the install directory (same logic as scripts/install.sh).
fn install_dir() -> PathBuf {
    let usr_local_bin = PathBuf::from("/usr/local/bin");
    if usr_local_bin.exists()
        && fs::metadata(&usr_local_bin)
            .map(|m| m.permissions().mode() & 0o200 != 0)
            .unwrap_or(false)
    {
        // Check if we can actually write there
        let test_path = usr_local_bin.join(".tachyon_write_test");
        if fs::write(&test_path, b"").is_ok() {
            let _ = fs::remove_file(&test_path);
            return usr_local_bin;
        }
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".local").join("bin")
}

/// Find the path to the currently running binary.
fn current_binary_path() -> Result<PathBuf> {
    env::current_exe().context("Failed to determine current binary path")
}

pub async fn run() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    let os = detect_os()?;
    let arch = detect_arch()?;

    // Fetch latest release from GitHub API
    eprintln!("Checking for latest release...");
    let client = reqwest::Client::new();
    let release: GitHubRelease = client
        .get(format!(
            "https://api.github.com/repos/{REPO}/releases/latest"
        ))
        .header("User-Agent", format!("tachyon-cli/{current_version}"))
        .send()
        .await
        .context("Failed to fetch latest release")?
        .error_for_status()
        .context("GitHub API returned an error")?
        .json()
        .await
        .context("Failed to parse release response")?;

    let tag = &release.tag_name;
    let latest_version = tag.strip_prefix('v').unwrap_or(tag);

    if latest_version == current_version {
        eprintln!("Already up to date (v{current_version}).");
        return Ok(());
    }

    eprintln!("Updating from v{current_version} to {tag}...");

    let artifact = format!("{BIN_NAME}-{os}-{arch}");
    let download_url =
        format!("https://github.com/{REPO}/releases/download/{tag}/{artifact}.tar.gz");

    // Download tarball
    eprintln!("Downloading {artifact}.tar.gz...");
    let response = client
        .get(&download_url)
        .header("User-Agent", format!("tachyon-cli/{current_version}"))
        .send()
        .await
        .context("Failed to download release")?
        .error_for_status()
        .context("Download failed")?;

    let bytes = response
        .bytes()
        .await
        .context("Failed to read download body")?;

    // Extract to temp directory
    let tmp_dir = env::temp_dir().join(format!("tachyon-install-{}", std::process::id()));
    fs::create_dir_all(&tmp_dir).context("Failed to create temp directory")?;

    let tarball_path = tmp_dir.join(format!("{artifact}.tar.gz"));
    fs::write(&tarball_path, &bytes).context("Failed to write tarball")?;

    let status = Command::new("tar")
        .args([
            "-xzf",
            &tarball_path.to_string_lossy(),
            "-C",
            &tmp_dir.to_string_lossy(),
        ])
        .status()
        .context("Failed to run tar")?;

    if !status.success() {
        let _ = fs::remove_dir_all(&tmp_dir);
        bail!("Failed to extract tarball");
    }

    let extracted_bin = tmp_dir.join(BIN_NAME);
    if !extracted_bin.exists() {
        let _ = fs::remove_dir_all(&tmp_dir);
        bail!("Extracted archive does not contain '{BIN_NAME}' binary");
    }

    // Determine install location
    let install_path = current_binary_path().unwrap_or_else(|_| install_dir().join(BIN_NAME));
    let install_directory = install_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(install_dir);

    fs::create_dir_all(&install_directory).context("Failed to create install directory")?;

    let target = install_directory.join(BIN_NAME);

    // Replace binary using install command for atomic replacement
    let status = Command::new("install")
        .args([
            "-m",
            "755",
            &extracted_bin.to_string_lossy(),
            &target.to_string_lossy(),
        ])
        .status()
        .context("Failed to install binary")?;

    // Cleanup
    let _ = fs::remove_dir_all(&tmp_dir);

    if !status.success() {
        bail!("Failed to install binary to {}", target.display());
    }

    eprintln!("Successfully updated to {tag}.");
    eprintln!("Installed to: {}", target.display());

    // Verify by running the new binary
    let output = Command::new(&target).arg("--version").output();
    if let Ok(out) = output {
        let version_str = String::from_utf8_lossy(&out.stdout);
        eprintln!("Verification: {}", version_str.trim());
    }

    Ok(())
}
