//! `tachyon auth token` — print the logged-in user's API bearer token so a
//! script can read it from a pipe, without the user copying the token by hand.
//!
//! The token goes to stdout only, and only when stdout is not a terminal.
//! Errors and progress go to stderr and never contain the token.

use std::io::{IsTerminal, Write};

use anyhow::{anyhow, bail, Result};
use clap::Args;

use crate::auth;

/// Refresh when the stored token expires within this many seconds, so a
/// script that starts now still has a usable token for its run.
pub const DEFAULT_MIN_VALIDITY_SECS: i64 = 600;

#[derive(Args, Debug)]
pub struct TokenAuthArgs {
    /// Refresh first when the stored token expires within this many seconds.
    #[arg(long, default_value_t = DEFAULT_MIN_VALIDITY_SECS)]
    pub min_validity: i64,
}

/// Whether the stored token must be refreshed before it is handed out.
/// Profiles without `expires_at` keep the token as it is, matching how other
/// commands treat them.
pub fn needs_refresh(expires_at: Option<i64>, now: i64, min_validity: i64) -> bool {
    expires_at.is_some_and(|exp| exp - now < min_validity)
}

/// Refuse the cases where the token would land on a screen or where the
/// output would not be the user's own login.
pub fn check_output_allowed(stdout_is_terminal: bool, api_key_set: bool) -> Result<()> {
    if api_key_set {
        bail!(
            "`auth token` prints the token of the logged-in user, but --api-key / \
             TACHYON_API_KEY is set. Unset it and run again."
        );
    }
    if stdout_is_terminal {
        bail!(
            "refusing to print the token to a terminal. Pipe it into the command \
             that reads it, e.g. `tachyon auth token | <command> --token-stdin`."
        );
    }
    Ok(())
}

pub async fn run(
    args: &TokenAuthArgs,
    oauth_config: &auth::OAuthConfig,
    profile: &str,
    api_key_set: bool,
) -> Result<()> {
    check_output_allowed(std::io::stdout().is_terminal(), api_key_set)?;
    if args.min_validity < 0 {
        bail!("--min-validity must be zero or more seconds");
    }

    let creds = auth::load_profile(profile)?.ok_or_else(|| {
        anyhow!(
            "not logged in (profile '{profile}'). Run `tachyon auth login --profile {profile}`."
        )
    })?;

    let now = chrono::Utc::now().timestamp();
    let creds = if needs_refresh(creds.expires_at, now, args.min_validity) {
        let refreshed = auth::refresh_access_token(oauth_config, profile, &creds)
            .await
            .map_err(|err| {
                anyhow!(
                    "token refresh failed for profile '{profile}': {err}. \
                     Run `tachyon auth login --profile {profile}`."
                )
            })?;
        eprintln!("Token refreshed (profile: {profile}).");
        refreshed
    } else {
        creds
    };

    let token = auth::select_api_bearer_token(&creds).ok_or_else(|| {
        anyhow!("profile '{profile}' has no token. Run `tachyon auth login --profile {profile}`.")
    })?;

    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{}", token.value)?;
    stdout.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refreshes_only_when_expiry_is_within_min_validity() {
        assert!(!needs_refresh(None, 1_000, 600));
        assert!(!needs_refresh(Some(1_600), 1_000, 600));
        assert!(needs_refresh(Some(1_599), 1_000, 600));
        assert!(needs_refresh(Some(900), 1_000, 600));
        assert!(!needs_refresh(Some(1_000), 1_000, 0));
        assert!(needs_refresh(Some(999), 1_000, 0));
    }

    #[test]
    fn refuses_terminal_stdout_and_api_key() {
        assert!(check_output_allowed(false, false).is_ok());
        let tty = check_output_allowed(true, false).unwrap_err().to_string();
        assert!(tty.contains("terminal"));
        let key = check_output_allowed(false, true).unwrap_err().to_string();
        assert!(key.contains("TACHYON_API_KEY"));
    }
}
