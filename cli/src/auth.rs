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
pub fn save_credentials(creds: &StoredCredentials) -> Result<()> {
    let path = credentials_path()?;
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

/// Refresh the access token using a stored refresh token.
///
/// Returns updated credentials on success, or an error if the refresh fails
/// (e.g. the refresh token itself has expired).
pub async fn refresh_access_token(
    oauth_config: &OAuthConfig,
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
    };

    save_credentials(&new_creds)?;
    Ok(new_creds)
}

/// Run the full OAuth login flow.
///
/// 1. Display the authorization URL for the user to open in a browser
/// 2. Browser redirects to the API callback relay after login
/// 3. CLI polls the relay endpoint to retrieve the authorization code
/// 4. CLI exchanges the code for tokens and saves them
pub async fn login(oauth_config: &OAuthConfig, api_url: &str) -> Result<()> {
    let state = random_string(16);
    let (code_verifier, code_challenge) = pkce_pair();

    let auth_url = oauth_config.authorize_url(&state, &code_challenge);

    println!();
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
}
