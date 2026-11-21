//! Workload Identity Federation (WIF) client.
//!
//! Exchanges an OIDC token issued to a workload (for example a GitHub
//! Actions job) for a short-lived Tachyon access token, so the workload does
//! not need a long-lived API key.
//!
//! ```no_run
//! # async fn demo() -> Result<(), tachyon_sdk::workload_identity::WorkloadIdentityError> {
//! use tachyon_sdk::workload_identity::{
//!     SubjectTokenSource, WorkloadIdentityConfig, WorkloadIdentityCredentials,
//! };
//!
//! let config = WorkloadIdentityConfig {
//!     api_url: "https://api.n1.tachy.one".into(),
//!     app_id: "app_01...".into(),
//!     operator_id: "tn_01...".into(),
//!     environment: "production".into(),
//!     action: "compute:CreateDeployment".into(),
//!     subject_token: SubjectTokenSource::github_actions_from_env(None)
//!         .expect("running inside GitHub Actions with id-token: write"),
//!     ttl_seconds: None,
//! };
//! let credentials = WorkloadIdentityCredentials::new(reqwest::Client::new(), config);
//! let token = credentials.access_token().await?;
//! // Send `Authorization: Bearer {token}` and `x-operator-id: tn_01...`.
//! # let _ = token;
//! # Ok(())
//! # }
//! ```

use std::fmt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Path of the unauthenticated token exchange endpoint.
pub const TOKEN_EXCHANGE_PATH: &str = "/v1/auth/workload-identities/token/exchange";

/// Audience Tachyon federation providers expect by default.
pub const DEFAULT_AUDIENCE: &str = "tachyon-api/workload-identity";

/// GitHub Actions variable holding the OIDC token request URL.
pub const GITHUB_ACTIONS_ID_TOKEN_REQUEST_URL: &str = "ACTIONS_ID_TOKEN_REQUEST_URL";

/// GitHub Actions variable holding the bearer token for the OIDC request.
pub const GITHUB_ACTIONS_ID_TOKEN_REQUEST_TOKEN: &str = "ACTIONS_ID_TOKEN_REQUEST_TOKEN";

/// Refresh a cached access token this long before it expires.
pub const REFRESH_SKEW: Duration = Duration::from_secs(60);

/// Errors from fetching a subject token or exchanging it.
///
/// Messages never contain token values.
#[derive(Debug)]
pub enum WorkloadIdentityError {
    /// No subject token source could be used.
    SubjectToken(String),
    /// The HTTP request could not be sent or its body could not be read.
    Transport(String),
    /// The server answered with a non-success status.
    Status {
        /// Which call failed (`token exchange`, `GitHub Actions OIDC`).
        call: &'static str,
        /// HTTP status code.
        status: u16,
        /// Response body (server error envelope; never echoes the token).
        body: String,
    },
    /// The response body was not the expected JSON.
    Decode(String),
}

impl fmt::Display for WorkloadIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SubjectToken(message) => write!(f, "subject token: {message}"),
            Self::Transport(message) => write!(f, "request failed: {message}"),
            Self::Status { call, status, body } => {
                write!(f, "{call} failed: status={status}, body={body}")
            }
            Self::Decode(message) => write!(f, "unexpected response: {message}"),
        }
    }
}

impl std::error::Error for WorkloadIdentityError {}

/// Body of `POST /v1/auth/workload-identities/token/exchange`.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenExchangeRequest {
    /// Cloud App ID the binding belongs to.
    pub app_id: String,
    /// Tenant (operator) ID the binding belongs to.
    pub operator_id: String,
    /// Binding environment, e.g. `production`.
    pub environment: String,
    /// Action to authorize; must be one of the binding's allowed actions.
    pub action: String,
    /// OIDC token issued to the workload.
    pub subject_token: String,
    /// Requested lifetime. The server clamps it to 60-900 seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<i64>,
}

impl fmt::Debug for TokenExchangeRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenExchangeRequest")
            .field("app_id", &self.app_id)
            .field("operator_id", &self.operator_id)
            .field("environment", &self.environment)
            .field("action", &self.action)
            .field("subject_token", &"<redacted>")
            .field("ttl_seconds", &self.ttl_seconds)
            .finish()
    }
}

/// Response of the token exchange endpoint.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenExchangeResponse {
    /// Always `Bearer`.
    pub token_type: String,
    /// Tachyon access token (JWT).
    pub access_token: String,
    /// Lifetime in seconds.
    pub expires_in: i64,
}

impl fmt::Debug for TokenExchangeResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenExchangeResponse")
            .field("token_type", &self.token_type)
            .field("access_token", &"<redacted>")
            .field("expires_in", &self.expires_in)
            .finish()
    }
}

fn endpoint(api_url: &str, path: &str) -> String {
    format!("{}{path}", api_url.trim_end_matches('/'))
}

async fn error_body(response: reqwest::Response) -> String {
    response.text().await.unwrap_or_default()
}

/// Exchange `request.subject_token` for a Tachyon access token.
pub async fn exchange_token(
    client: &reqwest::Client,
    api_url: &str,
    request: &TokenExchangeRequest,
) -> Result<TokenExchangeResponse, WorkloadIdentityError> {
    let response = client
        .post(endpoint(api_url, TOKEN_EXCHANGE_PATH))
        .json(request)
        .send()
        .await
        .map_err(|err| WorkloadIdentityError::Transport(err.without_url().to_string()))?;
    let status = response.status();
    if !status.is_success() {
        return Err(WorkloadIdentityError::Status {
            call: "token exchange",
            status: status.as_u16(),
            body: error_body(response).await,
        });
    }
    response
        .json::<TokenExchangeResponse>()
        .await
        .map_err(|err| WorkloadIdentityError::Decode(err.without_url().to_string()))
}

#[derive(Deserialize)]
struct GithubIdTokenResponse {
    value: String,
}

/// Build the GitHub Actions OIDC request URL for `audience`.
///
/// `ACTIONS_ID_TOKEN_REQUEST_URL` already carries a query string, so the
/// audience is appended as another query pair.
pub fn github_actions_id_token_url(
    request_url: &str,
    audience: &str,
) -> Result<url::Url, WorkloadIdentityError> {
    let mut url = url::Url::parse(request_url).map_err(|err| {
        WorkloadIdentityError::SubjectToken(format!(
            "{GITHUB_ACTIONS_ID_TOKEN_REQUEST_URL} is not a valid URL: {err}"
        ))
    })?;
    url.query_pairs_mut().append_pair("audience", audience);
    Ok(url)
}

/// Request an OIDC token from the GitHub Actions token service.
pub async fn fetch_github_actions_id_token(
    client: &reqwest::Client,
    request_url: &str,
    request_token: &str,
    audience: &str,
) -> Result<String, WorkloadIdentityError> {
    let url = github_actions_id_token_url(request_url, audience)?;
    let response = client
        .get(url)
        .bearer_auth(request_token)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|err| WorkloadIdentityError::Transport(err.without_url().to_string()))?;
    let status = response.status();
    if !status.is_success() {
        return Err(WorkloadIdentityError::Status {
            call: "GitHub Actions OIDC",
            status: status.as_u16(),
            body: error_body(response).await,
        });
    }
    let body: GithubIdTokenResponse = response
        .json()
        .await
        .map_err(|err| WorkloadIdentityError::Decode(err.without_url().to_string()))?;
    if body.value.trim().is_empty() {
        return Err(WorkloadIdentityError::Decode(
            "GitHub Actions OIDC returned an empty token".into(),
        ));
    }
    Ok(body.value)
}

/// Where the workload's OIDC (subject) token comes from.
///
/// Every exchange reads the source again, so a refresh uses a fresh token
/// (GitHub Actions tokens and projected Kubernetes tokens are short-lived).
#[derive(Clone)]
pub enum SubjectTokenSource {
    /// A fixed token value.
    Literal(String),
    /// A file holding the token, e.g. a projected service-account token.
    File(PathBuf),
    /// An environment variable holding the token.
    Env(String),
    /// The GitHub Actions OIDC token service.
    GithubActions {
        /// Value of `ACTIONS_ID_TOKEN_REQUEST_URL`.
        request_url: String,
        /// Value of `ACTIONS_ID_TOKEN_REQUEST_TOKEN`.
        request_token: String,
        /// Audience to request.
        audience: String,
    },
}

impl fmt::Debug for SubjectTokenSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(_) => f.write_str("Literal(<redacted>)"),
            Self::File(path) => f.debug_tuple("File").field(path).finish(),
            Self::Env(name) => f.debug_tuple("Env").field(name).finish(),
            Self::GithubActions { audience, .. } => f
                .debug_struct("GithubActions")
                .field("audience", audience)
                .finish_non_exhaustive(),
        }
    }
}

impl SubjectTokenSource {
    /// GitHub Actions source built from the job's environment, or `None`
    /// when the OIDC variables are absent (the workflow lacks
    /// `permissions: id-token: write`, or this is not GitHub Actions).
    pub fn github_actions_from_env(audience: Option<&str>) -> Option<Self> {
        Self::github_actions_from_lookup(audience, |name| std::env::var(name).ok())
    }

    /// Same as [`Self::github_actions_from_env`] with an injectable lookup.
    pub fn github_actions_from_lookup(
        audience: Option<&str>,
        lookup: impl Fn(&str) -> Option<String>,
    ) -> Option<Self> {
        let non_empty = |name| lookup(name).filter(|value| !value.trim().is_empty());
        let request_url = non_empty(GITHUB_ACTIONS_ID_TOKEN_REQUEST_URL)?;
        let request_token = non_empty(GITHUB_ACTIONS_ID_TOKEN_REQUEST_TOKEN)?;
        Some(Self::GithubActions {
            request_url,
            request_token,
            audience: audience.unwrap_or(DEFAULT_AUDIENCE).to_string(),
        })
    }

    /// Read the current subject token.
    pub async fn fetch(&self, client: &reqwest::Client) -> Result<String, WorkloadIdentityError> {
        let token = match self {
            Self::Literal(value) => value.clone(),
            Self::File(path) => std::fs::read_to_string(path).map_err(|err| {
                WorkloadIdentityError::SubjectToken(format!(
                    "failed to read {}: {err}",
                    path.display()
                ))
            })?,
            Self::Env(name) => std::env::var(name).map_err(|_| {
                WorkloadIdentityError::SubjectToken(format!(
                    "environment variable {name} is not set"
                ))
            })?,
            Self::GithubActions {
                request_url,
                request_token,
                audience,
            } => {
                fetch_github_actions_id_token(client, request_url, request_token, audience).await?
            }
        };
        let token = token.trim().to_string();
        if token.is_empty() {
            return Err(WorkloadIdentityError::SubjectToken(format!(
                "{self:?} yielded an empty token"
            )));
        }
        Ok(token)
    }
}

/// Everything needed to obtain Tachyon access tokens for one binding.
#[derive(Debug, Clone)]
pub struct WorkloadIdentityConfig {
    /// Tachyon API base URL.
    pub api_url: String,
    /// Cloud App ID of the binding.
    pub app_id: String,
    /// Tenant (operator) ID of the binding.
    pub operator_id: String,
    /// Binding environment.
    pub environment: String,
    /// Action to authorize.
    pub action: String,
    /// Where the subject token comes from.
    pub subject_token: SubjectTokenSource,
    /// Requested lifetime (server clamps to 60-900 seconds).
    pub ttl_seconds: Option<i64>,
}

impl WorkloadIdentityConfig {
    /// Fetch a subject token and exchange it once, without caching.
    pub async fn exchange(
        &self,
        client: &reqwest::Client,
    ) -> Result<TokenExchangeResponse, WorkloadIdentityError> {
        let subject_token = self.subject_token.fetch(client).await?;
        let request = TokenExchangeRequest {
            app_id: self.app_id.clone(),
            operator_id: self.operator_id.clone(),
            environment: self.environment.clone(),
            action: self.action.clone(),
            subject_token,
            ttl_seconds: self.ttl_seconds,
        };
        exchange_token(client, &self.api_url, &request).await
    }
}

struct CachedToken {
    access_token: String,
    refresh_at: Instant,
}

/// Caching access-token provider: exchanges on first use and again shortly
/// before the token expires (see [`REFRESH_SKEW`]).
pub struct WorkloadIdentityCredentials {
    client: reqwest::Client,
    config: WorkloadIdentityConfig,
    cached: tokio::sync::Mutex<Option<CachedToken>>,
}

impl fmt::Debug for WorkloadIdentityCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkloadIdentityCredentials")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl WorkloadIdentityCredentials {
    /// Create a provider; no request is sent until a token is needed.
    pub fn new(client: reqwest::Client, config: WorkloadIdentityConfig) -> Self {
        Self {
            client,
            config,
            cached: tokio::sync::Mutex::new(None),
        }
    }

    /// The binding this provider exchanges for.
    pub fn config(&self) -> &WorkloadIdentityConfig {
        &self.config
    }

    /// A valid access token, exchanging a new one when none is cached or the
    /// cached one is about to expire.
    pub async fn access_token(&self) -> Result<String, WorkloadIdentityError> {
        let mut cached = self.cached.lock().await;
        if let Some(token) = cached.as_ref() {
            if Instant::now() < token.refresh_at {
                return Ok(token.access_token.clone());
            }
        }
        self.exchange_into(&mut cached).await
    }

    /// Exchange a new access token even if the cached one looks valid, e.g.
    /// after the API rejected it with 401.
    pub async fn refresh(&self) -> Result<String, WorkloadIdentityError> {
        let mut cached = self.cached.lock().await;
        self.exchange_into(&mut cached).await
    }

    async fn exchange_into(
        &self,
        cached: &mut Option<CachedToken>,
    ) -> Result<String, WorkloadIdentityError> {
        let response = self.config.exchange(&self.client).await?;
        let lifetime = Duration::from_secs(response.expires_in.max(0) as u64);
        // Short-lived tokens refresh at half their lifetime instead of
        // immediately when the lifetime is shorter than twice the skew.
        let lead = REFRESH_SKEW.min(lifetime / 2);
        *cached = Some(CachedToken {
            access_token: response.access_token.clone(),
            refresh_at: Instant::now() + (lifetime - lead),
        });
        Ok(response.access_token)
    }
}
