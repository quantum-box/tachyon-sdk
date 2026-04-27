use anyhow::{bail, Context, Result};
use reqwest::header::LOCATION;
use reqwest::redirect::Policy;
use reqwest::Url;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const REPO: &str = "quantum-box/tachyon-sdk";
const BIN_NAME: &str = "tachyon";
const TAG_PREFIX: &str = "tachyon-cli-v";

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

fn extract_release_tag(download_url: &str) -> Result<String> {
    let parsed = Url::parse(download_url)
        .or_else(|_| Url::parse(&format!("https://github.com{download_url}")))
        .with_context(|| format!("Failed to parse GitHub redirect URL: {download_url}"))?;
    let segments = parsed
        .path_segments()
        .context("Failed to parse GitHub redirect path")?
        .collect::<Vec<_>>();

    for window in segments.windows(2) {
        if window[0] == "download" || window[0] == "tag" {
            return Ok(window[1].to_string());
        }
    }

    bail!("Could not parse release tag from redirect URL: {download_url}");
}

async fn resolve_latest_tag(resolver: &reqwest::Client, url: &str) -> Result<String> {
    let response = resolver
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to resolve latest release from {url}"))?
        .error_for_status()
        .with_context(|| format!("GitHub release resolution returned an error: {url}"))?;

    let location = response
        .headers()
        .get(LOCATION)
        .context("GitHub did not return redirect location for latest release")?
        .to_str()
        .context("Invalid redirect location value from GitHub")?;

    extract_release_tag(location)
}

pub async fn run() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    let os = detect_os()?;
    let arch = detect_arch()?;

    // Resolve latest release from GitHub without hitting GitHub API.
    eprintln!("Checking for latest release...");
    let user_agent = format!("tachyon-cli/{current_version}");
    let client = reqwest::Client::builder()
        .user_agent(&user_agent)
        .build()
        .context("Failed to build GitHub client")?;
    let resolver = reqwest::Client::builder()
        .user_agent(&user_agent)
        .redirect(Policy::none())
        .build()
        .context("Failed to build GitHub redirect resolver")?;

    let artifact = format!("{BIN_NAME}-{os}-{arch}");
    let latest_url =
        format!("https://github.com/{REPO}/releases/latest/download/{artifact}.tar.gz");
    let fallback_url = format!("https://github.com/{REPO}/releases/latest");

    let tag = match resolve_latest_tag(&resolver, &latest_url).await {
        Ok(tag) => tag,
        Err(_) => resolve_latest_tag(&resolver, &fallback_url)
            .await
            .context("Failed to resolve latest release from GitHub")?,
    };
    // Tag prefix changed from `v$VERSION` to `tachyon-cli-v$VERSION` after PLT-923
    // when the CLI moved to its own release lane. Strip either prefix.
    let latest_version = tag
        .strip_prefix(TAG_PREFIX)
        .or_else(|| tag.strip_prefix('v'))
        .unwrap_or(&tag);

    if latest_version == current_version {
        eprintln!("Already up to date (v{current_version}).");
        return Ok(());
    }

    eprintln!("Updating from v{current_version} to v{latest_version}...");

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

    eprintln!("Successfully updated to v{latest_version} (release tag: {tag}).");
    eprintln!("Installed to: {}", target.display());

    // Verify by running the new binary
    let output = Command::new(&target).arg("--version").output();
    if let Ok(out) = output {
        let version_str = String::from_utf8_lossy(&out.stdout);
        eprintln!("Verification: {}", version_str.trim());
    }

    Ok(())
}
