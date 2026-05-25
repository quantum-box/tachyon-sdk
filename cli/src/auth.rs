use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::Duration;

/// Polling interval when waiting for the callback relay.
const POLL_INTERVAL: Duration = Duration::from_secs(2);
/// Maximum time to wait for the user to complete login.
const POLL_TIMEOUT: Duration = Duration::from_secs(300);

/// Default profile name when none is configured.
pub const DEFAULT_PROFILE: &str = "default";
pub const DEFAULT_OAUTH_SCOPES: &[&str] = &[
    "openid",
    "profile",
    "email",
    "aws.cognito.signin.user.admin",
];

/// Cognito OAuth configuration.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub cognito_domain: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    pub fn authorize_url(&self, state: &str, code_challenge: &str) -> String {
        let scopes = self.scopes.join("+");
        format!(
            "{}/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            self.cognito_domain,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            scopes,
            urlencoding::encode(state),
            urlencoding::encode(code_challenge),
        )
    }

    pub fn token_url(&self) -> String {
        format!("{}/oauth2/token", self.cognito_domain)
    }
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            cognito_domain: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: String::new(),
            scopes: DEFAULT_OAUTH_SCOPES
                .iter()
                .map(|scope| (*scope).to_string())
                .collect(),
        }
    }
}

/// Stored credentials on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_at: Option<i64>,
    pub token_type: String,
    /// Default operator (tenant) ID saved after login.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_id: Option<String>,
}

/// Token kind selected for Tachyon API Bearer authentication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiTokenKind {
    Access,
    Id,
    ApiKey,
}

impl ApiTokenKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Access => "access_token",
            Self::Id => "id_token",
            Self::ApiKey => "api_key",
        }
    }
}

/// Bearer token selected for Tachyon API calls.
#[derive(Debug, Clone)]
pub struct ApiBearerToken {
    pub value: String,
    pub kind: ApiTokenKind,
}

/// Select the token used as `Authorization: Bearer ...` for Tachyon API calls.
///
/// Cognito and Tachyon OAuth access tokens are the API credential. ID tokens
/// are only a fallback for legacy profile files that do not contain an access
/// token.
pub fn select_api_bearer_token(creds: &StoredCredentials) -> Option<ApiBearerToken> {
    if !creds.access_token.trim().is_empty() {
        return Some(ApiBearerToken {
            value: creds.access_token.clone(),
            kind: ApiTokenKind::Access,
        });
    }
    creds
        .id_token
        .as_ref()
        .filter(|token| !token.trim().is_empty())
        .map(|token| ApiBearerToken {
            value: token.clone(),
            kind: ApiTokenKind::Id,
        })
}

/// Cognito token endpoint response.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    expires_in: Option<i64>,
    token_type: String,
}

/// Response from the poll endpoint.
#[derive(Debug, Deserialize)]
struct PollResponse {
    code: String,
}

/// Generate a random string for PKCE code_verifier and state.
fn random_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..len).map(|_| rng.gen()).collect();
    URL_SAFE_NO_PAD.encode(&bytes)
}

/// Generate PKCE code_verifier and code_challenge (S256).
fn pkce_pair() -> (String, String) {
    let verifier = random_string(32);
    let digest = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(digest);
    (verifier, challenge)
}

// -------------------------------------------------------------------------
// Profile-aware storage layer (PLT-724 Phase 1).
// -------------------------------------------------------------------------
//
// Storage layout (under `dirs::config_dir()/tachyon/`):
//
//   credentials.json          legacy single-account file (auto-migrated to
//                             profiles/default.json on first access; kept on
//                             disk so users can downgrade if needed).
//   active_profile            plain text containing the active profile name.
//   profiles/<name>.json      one StoredCredentials JSON per profile.
//
// Profile name resolution priority (handled by callers):
//   1. `--profile <name>` global CLI flag
//   2. `TACHYON_PROFILE` env var (collapsed into the flag by clap)
//   3. `active_profile` file
//   4. `DEFAULT_PROFILE` ("default")
//
// TODO(PLT-724-phase2): keychain / secret-service / credential-at-rest
// encryption. For now profiles/<name>.json is plaintext with 0o600 perms,
// matching the prior credentials.json behaviour.

/// Base config directory, e.g. `~/.config/tachyon/` on Linux.
pub fn config_dir() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .ok_or_else(|| anyhow!("could not determine config directory"))?
        .join("tachyon"))
}

/// Path to the legacy single-account credentials file.
pub fn credentials_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("credentials.json"))
}

/// Directory containing per-profile credentials files.
pub fn profiles_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("profiles"))
}

/// Path to the file that records the currently-active profile name.
pub fn active_profile_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("active_profile"))
}

/// Reject names that would escape the profiles dir or contain shell-hostile
/// characters. Allowed: ASCII letters / digits / `_` / `.` / `-`, length 1..=64,
/// not equal to `.` or `..`.
pub fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("profile name must not be empty"));
    }
    if name.len() > 64 {
        return Err(anyhow!("profile name must be 64 chars or fewer"));
    }
    if name == "." || name == ".." {
        return Err(anyhow!("profile name '{name}' is reserved"));
    }
    for c in name.chars() {
        let ok = c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.';
        if !ok {
            return Err(anyhow!(
                "profile name '{name}' contains invalid character '{c}' \
                 (allowed: a-z, A-Z, 0-9, _, -, .)"
            ));
        }
    }
    Ok(())
}

/// Path to a profile's credentials file. Validates the name first.
pub fn profile_path(name: &str) -> Result<PathBuf> {
    validate_profile_name(name)?;
    Ok(profiles_dir()?.join(format!("{name}.json")))
}

/// Read the active profile name.
///
/// Returns `DEFAULT_PROFILE` if the pointer file is missing, empty, or invalid.
pub fn read_active_profile() -> Result<String> {
    let path = active_profile_path()?;
    if !path.exists() {
        return Ok(DEFAULT_PROFILE.to_string());
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(DEFAULT_PROFILE.to_string());
    }
    // Defensive: refuse to honour a corrupt value rather than treat it as a
    // valid profile (which would silently look up a non-existent file).
    validate_profile_name(trimmed).with_context(|| {
        format!(
            "{} contains an invalid profile name; \
             run `tachyon auth use <name>` to fix",
            path.display()
        )
    })?;
    Ok(trimmed.to_string())
}

/// Write the active profile pointer atomically (write tmp + rename).
pub fn write_active_profile(name: &str) -> Result<()> {
    validate_profile_name(name)?;
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let path = active_profile_path()?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, format!("{name}\n"))
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    std::fs::rename(&tmp, &path)
        .with_context(|| format!("failed to rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

/// List profile names currently on disk, sorted.
pub fn list_profiles() -> Result<Vec<String>> {
    let dir = profiles_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in
        std::fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if validate_profile_name(stem).is_err() {
            continue;
        }
        names.push(stem.to_string());
    }
    names.sort();
    Ok(names)
}

/// Load credentials for a specific profile. Returns `Ok(None)` when the
/// profile file does not exist (i.e. user has not logged in yet).
pub fn load_profile(name: &str) -> Result<Option<StoredCredentials>> {
    let path = profile_path(name)?;
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let creds: StoredCredentials = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(Some(creds))
}

/// Save credentials for a specific profile (creates dirs as needed, 0o600).
pub fn save_profile(name: &str, creds: &StoredCredentials) -> Result<()> {
    let path = profile_path(name)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(creds)?;
    std::fs::write(&path, data).with_context(|| format!("failed to write {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Delete a profile's credentials file. Returns `true` if the file existed.
pub fn delete_profile(name: &str) -> Result<bool> {
    let path = profile_path(name)?;
    if !path.exists() {
        return Ok(false);
    }
    std::fs::remove_file(&path).with_context(|| format!("failed to remove {}", path.display()))?;
    Ok(true)
}

/// Copy any legacy `credentials.json` into `profiles/default.json` if the
/// latter does not yet exist. The legacy file is intentionally left in place
/// so users can roll back to an older CLI build if they need to.
///
/// Idempotent: a no-op once `profiles/default.json` exists.
///
/// Tolerant of malformed legacy files: emits a warning and returns Ok so the
/// CLI can still run when a user has a stale/partial credentials.json from
/// an older format.
pub fn migrate_legacy_if_needed() -> Result<()> {
    let legacy = credentials_path()?;
    if !legacy.exists() {
        return Ok(());
    }
    let default_path = profile_path(DEFAULT_PROFILE)?;
    if default_path.exists() {
        return Ok(());
    }
    let data = match std::fs::read_to_string(&legacy) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Warning: legacy credentials file {} unreadable ({e}); skipping migration.",
                legacy.display()
            );
            return Ok(());
        }
    };
    let creds: StoredCredentials = match serde_json::from_str(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "Warning: legacy credentials file {} could not be parsed ({e}); \
                 skipping migration. Run `tachyon auth login` to create a fresh profile.",
                legacy.display()
            );
            return Ok(());
        }
    };
    if let Err(e) = save_profile(DEFAULT_PROFILE, &creds) {
        eprintln!(
            "Warning: failed to write migrated profile {}: {e}",
            default_path.display()
        );
        return Ok(());
    }
    eprintln!(
        "Migrated legacy credentials to profile '{}': {}",
        DEFAULT_PROFILE,
        default_path.display()
    );
    Ok(())
}

/// Resolve the active profile name given an optional CLI/env override.
///
/// Priority: explicit override (CLI flag or `TACHYON_PROFILE` env, both
/// collapsed into one `Option<String>` by clap) > `active_profile` file >
/// `DEFAULT_PROFILE`.
///
/// Always runs `migrate_legacy_if_needed` first so legacy single-account
/// installs see their data under the `default` profile transparently.
pub fn resolve_active_profile(explicit: Option<&str>) -> Result<String> {
    migrate_legacy_if_needed()?;
    if let Some(name) = explicit {
        validate_profile_name(name)?;
        return Ok(name.to_string());
    }
    read_active_profile()
}

// -------------------------------------------------------------------------
// OAuth flow (per-profile).
// -------------------------------------------------------------------------

/// Exchange authorization code for tokens at the Cognito token endpoint.
async fn exchange_code(
    oauth_config: &OAuthConfig,
    code: &str,
    code_verifier: &str,
) -> Result<TokenResponse> {
    let client = Client::new();
    let resp = client
        .post(oauth_config.token_url())
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", &oauth_config.client_id),
            ("client_secret", &oauth_config.client_secret),
            ("code", code),
            ("redirect_uri", &oauth_config.redirect_uri),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await
        .context("failed to request token")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "token exchange failed: status={status}, body={body}"
        ));
    }

    resp.json::<TokenResponse>()
        .await
        .context("failed to parse token response")
}

/// Poll the callback relay endpoint until the authorization code is available.
async fn poll_for_code(api_url: &str, state: &str) -> Result<String> {
    let client = Client::new();
    let poll_url = format!(
        "{}/v1/auth/cli/poll?state={}",
        api_url.trim_end_matches('/'),
        state
    );
    let deadline = tokio::time::Instant::now() + POLL_TIMEOUT;

    loop {
        if tokio::time::Instant::now() > deadline {
            return Err(anyhow!("login timed out — no response within 5 minutes"));
        }

        tokio::time::sleep(POLL_INTERVAL).await;

        let resp = match client.get(&poll_url).send().await {
            Ok(r) => r,
            Err(_) => continue, // Network error, retry
        };

        if resp.status() == reqwest::StatusCode::OK {
            let poll_resp: PollResponse =
                resp.json().await.context("failed to parse poll response")?;
            return Ok(poll_resp.code);
        }
        // 204 No Content or other = not ready yet, keep polling
    }
}

/// Refresh the access token using a stored refresh token, persisting the
/// result back to the named profile.
pub async fn refresh_access_token(
    oauth_config: &OAuthConfig,
    profile: &str,
    creds: &StoredCredentials,
) -> Result<StoredCredentials> {
    let refresh_token = creds
        .refresh_token
        .as_deref()
        .ok_or_else(|| anyhow!("no refresh token available"))?;

    let client = Client::new();
    let resp = client
        .post(oauth_config.token_url())
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", &oauth_config.client_id),
            ("client_secret", &oauth_config.client_secret),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await
        .context("failed to request token refresh")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "token refresh failed: status={status}, body={body}"
        ));
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .context("failed to parse token refresh response")?;

    let now = chrono::Utc::now().timestamp();
    let expires_at = token_resp.expires_in.map(|e| now + e);

    let new_creds = StoredCredentials {
        access_token: token_resp.access_token,
        // Cognito may not return a new refresh token on refresh; keep the old one.
        refresh_token: token_resp
            .refresh_token
            .or_else(|| creds.refresh_token.clone()),
        id_token: token_resp.id_token.or_else(|| creds.id_token.clone()),
        expires_at,
        token_type: token_resp.token_type,
        operator_id: creds.operator_id.clone(),
    };

    save_profile(profile, &new_creds)?;
    Ok(new_creds)
}

/// Run the full OAuth login flow and persist tokens to the named profile.
///
/// Also writes `active_profile` to `profile` if no active profile is set, so
/// a fresh user logging in for the first time has a working setup.
pub async fn login(oauth_config: &OAuthConfig, api_url: &str, profile: &str) -> Result<()> {
    validate_profile_name(profile)?;

    let state = random_string(16);
    let (code_verifier, code_challenge) = pkce_pair();

    let auth_url = oauth_config.authorize_url(&state, &code_challenge);

    println!();
    println!("Logging in profile '{profile}'.");
    println!("Open the following URL in your browser to log in:");
    println!();
    println!("  {auth_url}");
    println!();
    println!("Waiting for login to complete...");

    let code = poll_for_code(api_url, &state).await?;

    println!("Exchanging authorization code for tokens...");

    let token_resp = exchange_code(oauth_config, &code, &code_verifier).await?;

    let now = chrono::Utc::now().timestamp();
    let expires_at = token_resp.expires_in.map(|e| now + e);

    let mut creds = StoredCredentials {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token,
        id_token: token_resp.id_token,
        expires_at,
        token_type: token_resp.token_type,
        operator_id: None,
    };

    // Fetch operators and save the default one. If this fails, keep the login
    // result but make the auth/API mismatch visible immediately.
    match fetch_default_operator(api_url, &creds.access_token).await {
        Ok(operator_id) => {
            creds.operator_id = Some(operator_id);
        }
        Err(e) => {
            eprintln!("Warning: login completed but Tachyon API token verification failed: {e}");
            eprintln!(
                "Check the selected profile, OAuth client ID, Cognito issuer, \
                 token audience/client_id, and API COGNITO_ALLOWED_CLIENT_IDS."
            );
        }
    }

    save_profile(profile, &creds)?;

    // First-time setup: if no active_profile pointer exists yet, point at the
    // freshly-logged-in profile so subsequent commands work without an extra
    // `auth use` step.
    if !active_profile_path()?.exists() {
        write_active_profile(profile)?;
    }

    let path = profile_path(profile)?;
    println!(
        "Login successful! Profile '{profile}' saved to {}",
        path.display()
    );
    if let Some(ref op_id) = creds.operator_id {
        println!("Default tenant: {op_id}");
    }

    Ok(())
}

/// Fetch the default operator ID for the logged-in user.
/// Automatically selects if only one exists.
async fn fetch_default_operator(api_url: &str, access_token: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct OperatorEntry {
        id: String,
        #[serde(default)]
        alias: Option<String>,
    }

    let client = Client::new();
    let url = format!(
        "{}/v1/auth/operators/by-user",
        api_url.trim_end_matches('/')
    );
    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .context("failed to fetch operators")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "operators fetch failed: status={status}, body={body}"
        ));
    }

    let operators: Vec<OperatorEntry> = resp.json().await.context("failed to parse operators")?;

    match operators.len() {
        0 => Err(anyhow!("no operators found for this user")),
        1 => {
            let op = &operators[0];
            let label = op.alias.as_deref().unwrap_or(&op.id);
            println!("Tenant: {label} ({})", op.id);
            Ok(op.id.clone())
        }
        _ => {
            // Multiple operators — pick the first and show a hint.
            let op = &operators[0];
            let label = op.alias.as_deref().unwrap_or(&op.id);
            println!("Multiple tenants found. Using: {label} ({})", op.id);
            println!("Use --tenant-id or TACHYON_TENANT_ID to override.");
            Ok(op.id.clone())
        }
    }
}

/// Remove a single profile's stored credentials.
///
/// If the removed profile was the active one and other profiles remain, the
/// active pointer is rewritten to the alphabetically-first surviving profile
/// to avoid leaving the CLI in a broken state. If no profiles remain, the
/// pointer is deleted.
pub fn logout(profile: &str) -> Result<()> {
    validate_profile_name(profile)?;
    let removed = delete_profile(profile)?;
    if !removed {
        println!("No stored credentials found for profile '{profile}'.");
        return Ok(());
    }
    println!("Logged out profile '{profile}'.");

    // Repair the active_profile pointer if we just removed the active one.
    let active = read_active_profile().unwrap_or_else(|_| DEFAULT_PROFILE.to_string());
    if active == profile {
        let remaining = list_profiles()?;
        if let Some(next) = remaining.first() {
            write_active_profile(next)?;
            println!("Active profile is now '{next}'.");
        } else {
            // No profiles left — drop the pointer so a fresh login starts clean.
            let path = active_profile_path()?;
            if path.exists() {
                std::fs::remove_file(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
            }
        }
    }
    Ok(())
}

/// Switch the active profile to `name`. Errors if the profile does not exist.
pub fn use_profile(name: &str) -> Result<()> {
    validate_profile_name(name)?;
    migrate_legacy_if_needed()?;
    let path = profile_path(name)?;
    if !path.exists() {
        return Err(anyhow!(
            "profile '{name}' does not exist. \
             Run `tachyon auth login --profile {name}` first, \
             or `tachyon auth list` to see available profiles."
        ));
    }
    write_active_profile(name)?;
    println!("Switched active profile to '{name}'.");
    Ok(())
}

/// Print the list of registered profiles, marking the active one with `*`.
pub fn list_profiles_command() -> Result<()> {
    migrate_legacy_if_needed()?;
    let profiles = list_profiles()?;
    if profiles.is_empty() {
        println!("No profiles registered. Run `tachyon auth login --profile <name>` to start.");
        return Ok(());
    }
    let active = read_active_profile().unwrap_or_else(|_| DEFAULT_PROFILE.to_string());
    println!("  {:<24} TENANT", "PROFILE");
    for name in &profiles {
        let marker = if name == &active { "*" } else { " " };
        let tenant = load_profile(name)
            .ok()
            .flatten()
            .and_then(|c| c.operator_id)
            .unwrap_or_else(|| "-".to_string());
        println!("{marker} {name:<24} {tenant}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_pair_produces_valid_output() {
        let (verifier, challenge) = pkce_pair();
        assert!(!verifier.is_empty());
        assert!(!challenge.is_empty());
        // Verify challenge is SHA-256 of verifier
        let digest = Sha256::digest(verifier.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(digest);
        assert_eq!(challenge, expected);
    }

    #[test]
    fn test_authorize_url_format() {
        let config = OAuthConfig {
            cognito_domain: "https://auth.example.com".to_string(),
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            redirect_uri: "https://api.example.com/v1/auth/cli/callback".to_string(),
            scopes: vec!["openid".into(), "profile".into()],
        };
        let url = config.authorize_url("STATE", "CHALLENGE");
        assert!(url.starts_with("https://auth.example.com/oauth2/authorize?"));
        assert!(url.contains("client_id=test-client"));
        assert!(url.contains("state=STATE"));
        assert!(url.contains("code_challenge=CHALLENGE"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("scope=openid+profile"));
    }

    #[test]
    fn api_bearer_prefers_access_token_over_id_token() {
        let creds = StoredCredentials {
            access_token: "access-token".to_string(),
            refresh_token: Some("refresh-token".to_string()),
            id_token: Some("id-token".to_string()),
            expires_at: None,
            token_type: "Bearer".to_string(),
            operator_id: None,
        };

        let selected = select_api_bearer_token(&creds).expect("token selected");

        assert_eq!(selected.kind, ApiTokenKind::Access);
        assert_eq!(selected.value, "access-token");
    }

    #[test]
    fn api_bearer_falls_back_to_id_token_for_legacy_profiles() {
        let creds = StoredCredentials {
            access_token: String::new(),
            refresh_token: None,
            id_token: Some("id-token".to_string()),
            expires_at: None,
            token_type: "Bearer".to_string(),
            operator_id: None,
        };

        let selected = select_api_bearer_token(&creds).expect("token selected");

        assert_eq!(selected.kind, ApiTokenKind::Id);
        assert_eq!(selected.value, "id-token");
    }

    #[test]
    fn validate_profile_name_accepts_typical_names() {
        for name in ["default", "work", "personal", "ACME-Corp", "user_1", "v1.2"] {
            validate_profile_name(name).unwrap_or_else(|e| panic!("{name} rejected: {e}"));
        }
    }

    #[test]
    fn validate_profile_name_rejects_dangerous_inputs() {
        for bad in ["", ".", "..", "a/b", "a\\b", "a b", "a\0b", "name\n"] {
            assert!(
                validate_profile_name(bad).is_err(),
                "expected '{bad}' to be rejected"
            );
        }
        // 65 chars
        let too_long = "a".repeat(65);
        assert!(validate_profile_name(&too_long).is_err());
    }

    #[test]
    fn profile_path_uses_validated_name() {
        // We can't easily check the prefix without env mangling, but we can at
        // least confirm validation runs.
        assert!(profile_path("..").is_err());
        assert!(profile_path("a/b").is_err());
    }
}
