use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;

/// Default redirect URI — Cognito redirects here after login.
/// The page won't load (no server), but the authorization code is visible
/// in the browser address bar for the user to copy.
const DEFAULT_REDIRECT_URI: &str = "http://localhost/callback";

/// Cognito OAuth configuration.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub cognito_domain: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    pub fn authorize_url(&self, state: &str, code_challenge: &str) -> String {
        let scopes = self.scopes.join("+");
        format!(
            "{}/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            self.cognito_domain,
            self.client_id,
            urlencoding::encode(&self.redirect_uri),
            scopes,
            state,
            code_challenge,
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
            redirect_uri: DEFAULT_REDIRECT_URI.to_string(),
            scopes: vec!["openid".into(), "profile".into(), "email".into()],
        }
    }
}

/// Stored credentials on disk.
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_at: Option<i64>,
    pub token_type: String,
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

/// Return the path to the credentials file.
pub fn credentials_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("could not determine config directory"))?
        .join("tachyon");
    Ok(config_dir.join("credentials.json"))
}

/// Load stored credentials from disk, if they exist.
pub fn load_credentials() -> Result<Option<StoredCredentials>> {
    let path = credentials_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let creds: StoredCredentials =
        serde_json::from_str(&data).context("failed to parse credentials file")?;
    Ok(Some(creds))
}

/// Save credentials to disk.
fn save_credentials(creds: &StoredCredentials) -> Result<()> {
    let path = credentials_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(creds)?;
    std::fs::write(&path, data)
        .with_context(|| format!("failed to write {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Extract the authorization code from user input.
///
/// Accepts either:
/// - A full redirect URL containing `?code=...&state=...`
/// - Just the raw authorization code string
fn extract_code_from_input(input: &str, expected_state: &str) -> Result<String> {
    let trimmed = input.trim();

    // If the input looks like a URL, extract `code` and `state` from query params
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let query = trimmed
            .split_once('?')
            .map(|(_, q)| q)
            .ok_or_else(|| anyhow!("URL has no query parameters — expected ?code=..."))?;

        let params: Vec<(&str, &str)> = query
            .split('&')
            .filter_map(|p| p.split_once('='))
            .collect();

        // Check for OAuth error in redirect
        if let Some((_, error)) = params.iter().find(|(k, _)| *k == "error") {
            let desc = params
                .iter()
                .find(|(k, _)| *k == "error_description")
                .map(|(_, v)| urlencoding::decode(v).unwrap_or_default().into_owned())
                .unwrap_or_default();
            return Err(anyhow!("OAuth error: {error} — {desc}"));
        }

        let code = params
            .iter()
            .find(|(k, _)| *k == "code")
            .map(|(_, v)| v.to_string())
            .ok_or_else(|| anyhow!("no 'code' parameter found in the URL"))?;

        // Validate state if present
        if let Some((_, state)) = params.iter().find(|(k, _)| *k == "state") {
            if *state != expected_state {
                return Err(anyhow!(
                    "state mismatch: expected {expected_state}, got {state}"
                ));
            }
        }

        Ok(code)
    } else {
        // Treat as a raw authorization code
        if trimmed.is_empty() {
            return Err(anyhow!("no input provided"));
        }
        Ok(trimmed.to_string())
    }
}

/// Prompt the user to paste input and read a line from stdin.
fn prompt_input(prompt: &str) -> Result<String> {
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("failed to read from stdin")?;
    Ok(line)
}

/// Exchange authorization code for tokens at the Cognito token endpoint.
async fn exchange_code(
    oauth_config: &OAuthConfig,
    code: &str,
    code_verifier: &str,
) -> Result<TokenResponse> {
    let client = Client::new();
    let resp = client
        .post(&oauth_config.token_url())
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", &oauth_config.client_id),
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

/// Run the full OAuth login flow (no local server — SSH-friendly).
///
/// 1. Display the authorization URL for the user to open in a browser
/// 2. After login, the browser redirects to the redirect URI with the code in the URL
/// 3. User pastes the redirect URL (or just the code) back into the CLI
/// 4. CLI exchanges the code for tokens and saves them
pub async fn login(oauth_config: &OAuthConfig) -> Result<()> {
    let state = random_string(16);
    let (code_verifier, code_challenge) = pkce_pair();

    let auth_url = oauth_config.authorize_url(&state, &code_challenge);

    println!();
    println!("Open the following URL in your browser to log in:");
    println!();
    println!("  {auth_url}");
    println!();
    println!("After logging in, your browser will redirect to a URL starting with:");
    println!("  {}", oauth_config.redirect_uri);
    println!();
    println!("The page may not load — that is expected.");
    println!("Copy the full URL from your browser's address bar and paste it below.");
    println!();

    let input = tokio::task::spawn_blocking(|| {
        prompt_input("Paste the redirect URL (or authorization code): ")
    })
    .await
    .context("prompt task panicked")??;

    let code = extract_code_from_input(&input, &state)?;

    println!("Exchanging authorization code for tokens...");

    let token_resp = exchange_code(oauth_config, &code, &code_verifier).await?;

    let now = chrono::Utc::now().timestamp();
    let expires_at = token_resp.expires_in.map(|e| now + e);

    let creds = StoredCredentials {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token,
        id_token: token_resp.id_token,
        expires_at,
        token_type: token_resp.token_type,
    };

    save_credentials(&creds)?;

    let path = credentials_path()?;
    println!("Login successful! Credentials saved to {}", path.display());

    Ok(())
}

/// Remove stored credentials (logout).
pub fn logout() -> Result<()> {
    let path = credentials_path()?;
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("failed to remove {}", path.display()))?;
        println!("Logged out. Credentials removed.");
    } else {
        println!("No stored credentials found.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_from_full_url() {
        let state = "abc123";
        let url = "http://localhost/callback?code=AUTH_CODE_XYZ&state=abc123";
        let code = extract_code_from_input(url, state).unwrap();
        assert_eq!(code, "AUTH_CODE_XYZ");
    }

    #[test]
    fn test_extract_code_from_url_state_mismatch() {
        let url = "http://localhost/callback?code=AUTH_CODE_XYZ&state=wrong";
        let result = extract_code_from_input(url, "expected");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("state mismatch"));
    }

    #[test]
    fn test_extract_code_from_raw_code() {
        let code = extract_code_from_input("  AUTH_CODE_XYZ  ", "any_state").unwrap();
        assert_eq!(code, "AUTH_CODE_XYZ");
    }

    #[test]
    fn test_extract_code_from_empty_input() {
        let result = extract_code_from_input("", "state");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_code_from_url_with_error() {
        let url = "http://localhost/callback?error=access_denied&error_description=User+cancelled";
        let result = extract_code_from_input(url, "state");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("access_denied"));
    }

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
            redirect_uri: "http://localhost/callback".to_string(),
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
}
