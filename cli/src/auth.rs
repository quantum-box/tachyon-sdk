use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;

/// Default local port for the OAuth callback server.
const CALLBACK_PORT: u16 = 8765;
const CALLBACK_PATH: &str = "/callback";

/// Cognito OAuth configuration.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub cognito_domain: String,
    pub client_id: String,
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    pub fn redirect_uri(&self) -> String {
        format!("http://localhost:{CALLBACK_PORT}{CALLBACK_PATH}")
    }

    pub fn authorize_url(&self, state: &str, code_challenge: &str) -> String {
        let scopes = self.scopes.join("+");
        format!(
            "{}/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            self.cognito_domain,
            self.client_id,
            urlencoding::encode(&self.redirect_uri()),
            scopes,
            state,
            code_challenge,
        )
    }

    pub fn token_url(&self) -> String {
        format!("{}/oauth2/token", self.cognito_domain)
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
    // Restrict permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Start a local HTTP server, wait for the OAuth callback, and return the
/// authorization code.
fn wait_for_callback(expected_state: &str) -> Result<String> {
    let listener = TcpListener::bind(format!("127.0.0.1:{CALLBACK_PORT}"))
        .with_context(|| format!("failed to bind to port {CALLBACK_PORT}"))?;

    println!("Waiting for authorization callback on port {CALLBACK_PORT}...");

    let (stream, _) = listener.accept().context("failed to accept connection")?;
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .context("failed to read request")?;

    // Parse query parameters from "GET /callback?code=...&state=... HTTP/1.1"
    let path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow!("malformed HTTP request"))?;

    let query = path
        .split_once('?')
        .map(|(_, q)| q)
        .unwrap_or("");

    let params: Vec<(&str, &str)> = query
        .split('&')
        .filter_map(|p| p.split_once('='))
        .collect();

    // Check for error response
    if let Some((_, error)) = params.iter().find(|(k, _)| *k == "error") {
        let desc = params
            .iter()
            .find(|(k, _)| *k == "error_description")
            .map(|(_, v)| urlencoding::decode(v).unwrap_or_default().into_owned())
            .unwrap_or_default();
        send_response(&stream, 400, "Login failed. You can close this tab.");
        return Err(anyhow!("OAuth error: {error} - {desc}"));
    }

    let code = params
        .iter()
        .find(|(k, _)| *k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| anyhow!("no authorization code in callback"))?;

    let state = params
        .iter()
        .find(|(k, _)| *k == "state")
        .map(|(_, v)| v.to_string())
        .unwrap_or_default();

    if state != expected_state {
        send_response(&stream, 400, "Login failed: state mismatch. You can close this tab.");
        return Err(anyhow!("state mismatch in OAuth callback"));
    }

    send_response(&stream, 200, "Login successful! You can close this tab and return to your terminal.");

    Ok(code)
}

fn send_response(stream: &std::net::TcpStream, status: u16, body: &str) {
    let status_text = if status == 200 { "OK" } else { "Bad Request" };
    let html = format!(
        "<html><body><h2>{body}</h2></body></html>"
    );
    let response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}",
        html.len()
    );
    let _ = (&*stream).write_all(response.as_bytes());
    let _ = (&*stream).flush();
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
            ("redirect_uri", &oauth_config.redirect_uri()),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await
        .context("failed to request token")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("token exchange failed: status={status}, body={body}"));
    }

    resp.json::<TokenResponse>()
        .await
        .context("failed to parse token response")
}

/// Run the full OAuth login flow.
pub async fn login(oauth_config: &OAuthConfig) -> Result<()> {
    let state = random_string(16);
    let (code_verifier, code_challenge) = pkce_pair();

    let auth_url = oauth_config.authorize_url(&state, &code_challenge);

    println!("Opening browser for login...");
    if open::that(&auth_url).is_err() {
        println!("Could not open browser automatically.");
        println!("Please open the following URL in your browser:\n");
        println!("  {auth_url}\n");
    }

    // Wait for the callback in a blocking thread so we don't block the tokio runtime
    let expected_state = state.clone();
    let code = tokio::task::spawn_blocking(move || wait_for_callback(&expected_state))
        .await
        .context("callback task panicked")??;

    println!("Authorization code received. Exchanging for tokens...");

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
