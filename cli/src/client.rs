use anyhow::{anyhow, Context, Result};
use reqwest::{header, Client};
use serde::de::DeserializeOwned;
use tachyon_sdk::apis::configuration::Configuration;

use crate::auth;

/// Non-secret context used to explain authentication failures.
#[derive(Debug, Clone)]
pub struct AuthDiagnostics {
    pub profile: Option<String>,
    pub token_kind: Option<String>,
    pub oauth_client_configured: bool,
}

/// Structured HTTP error returned by [`ApiClient`] requests.
///
/// Carries the HTTP status as a typed value so callers can branch on it
/// (e.g. retry on 404) without parsing the human-readable message. The
/// `Display` output is kept byte-compatible with the previous inline
/// `anyhow!` message so existing user-facing logs are unchanged.
#[derive(Debug)]
pub struct HttpError {
    pub method: String,
    pub path: String,
    pub status: reqwest::StatusCode,
    pub body: String,
    /// Extra authentication diagnostics appended for 401 responses.
    pub diagnostics: Option<String>,
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} failed: status={}, body={}",
            self.method, self.path, self.status, self.body
        )?;
        if let Some(diagnostics) = &self.diagnostics {
            write!(f, "\n{diagnostics}")?;
        }
        Ok(())
    }
}

impl std::error::Error for HttpError {}

/// Returns the HTTP status of `err` when it originates from an [`ApiClient`]
/// request, without relying on string matching. Returns `None` for errors that
/// are not [`HttpError`] (e.g. transport/parse failures).
pub fn http_error_status(err: &anyhow::Error) -> Option<reqwest::StatusCode> {
    err.downcast_ref::<HttpError>().map(|e| e.status)
}

/// Shared API client that carries Tachyon auth headers.
pub struct ApiClient {
    pub client: Client,
    pub base_url: String,
    auth_diagnostics: Option<AuthDiagnostics>,
}

impl ApiClient {
    /// Build from SDK configuration and tenant ID.
    pub fn new(config: &Configuration, tenant_id: &str) -> Result<Self> {
        Self::new_with_auth_diagnostics(config, tenant_id, None)
    }

    /// Build from SDK configuration and tenant ID with auth diagnostics.
    pub fn new_with_auth_diagnostics(
        config: &Configuration,
        tenant_id: &str,
        auth_diagnostics: Option<AuthDiagnostics>,
    ) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert("x-operator-id", header::HeaderValue::from_str(tenant_id)?);
        if let Some(token) = &config.bearer_access_token {
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("Bearer {token}"))?,
            );
        }
        let client = Client::builder().default_headers(headers).build()?;
        let base_url = config.base_path.trim_end_matches('/').to_string();
        Ok(Self {
            client,
            base_url,
            auth_diagnostics,
        })
    }

    /// GET a JSON endpoint and deserialize the response.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        let status = resp.status();
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                if let Some(token) = self.refresh_bearer_after_401().await {
                    let retry = self
                        .client
                        .get(&url)
                        .bearer_auth(token)
                        .send()
                        .await
                        .with_context(|| format!("GET {url}"))?;
                    return self.json_or_error("GET", path, retry).await;
                }
            }
            let body = resp.text().await.unwrap_or_default();
            return Err(self.http_error("GET", path, status, &body));
        }
        resp.json()
            .await
            .with_context(|| format!("parse GET {path}"))
    }

    /// GET with query parameters.
    pub async fn get_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .query(query)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        let status = resp.status();
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                if let Some(token) = self.refresh_bearer_after_401().await {
                    let retry = self
                        .client
                        .get(&url)
                        .query(query)
                        .bearer_auth(token)
                        .send()
                        .await
                        .with_context(|| format!("GET {url}"))?;
                    return self.json_or_error("GET", path, retry).await;
                }
            }
            let body = resp.text().await.unwrap_or_default();
            return Err(self.http_error("GET", path, status, &body));
        }
        resp.json()
            .await
            .with_context(|| format!("parse GET {path}"))
    }

    /// POST a JSON body and deserialize the response.
    pub async fn post<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {url}"))?;
        let status = resp.status();
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                if let Some(token) = self.refresh_bearer_after_401().await {
                    let retry = self
                        .client
                        .post(&url)
                        .json(body)
                        .bearer_auth(token)
                        .send()
                        .await
                        .with_context(|| format!("POST {url}"))?;
                    return self.json_or_error("POST", path, retry).await;
                }
            }
            let body_text = resp.text().await.unwrap_or_default();
            return Err(self.http_error("POST", path, status, &body_text));
        }
        resp.json()
            .await
            .with_context(|| format!("parse POST {path}"))
    }

    /// POST with no response body expected (returns status text).
    pub async fn post_no_body(&self, path: &str) -> Result<String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .send()
            .await
            .with_context(|| format!("POST {url}"))?;
        let status = resp.status();
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                if let Some(token) = self.refresh_bearer_after_401().await {
                    let retry = self
                        .client
                        .post(&url)
                        .bearer_auth(token)
                        .send()
                        .await
                        .with_context(|| format!("POST {url}"))?;
                    let retry_status = retry.status();
                    if retry_status.is_success() {
                        return Ok(retry_status.to_string());
                    }
                    let body = retry.text().await.unwrap_or_default();
                    return Err(self.http_error("POST", path, retry_status, &body));
                }
            }
            let body = resp.text().await.unwrap_or_default();
            return Err(self.http_error("POST", path, status, &body));
        }
        Ok(status.to_string())
    }

    /// POST a JSON body where no response body is needed.
    pub async fn post_no_response<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {url}"))?;
        let status = resp.status();
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                if let Some(token) = self.refresh_bearer_after_401().await {
                    let retry = self
                        .client
                        .post(&url)
                        .json(body)
                        .bearer_auth(token)
                        .send()
                        .await
                        .with_context(|| format!("POST {url}"))?;
                    let retry_status = retry.status();
                    if retry_status.is_success() {
                        return Ok(());
                    }
                    let body_text = retry.text().await.unwrap_or_default();
                    return Err(self.http_error("POST", path, retry_status, &body_text));
                }
            }
            let body_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "POST {path} failed: status={status}, body={body_text}"
            ));
        }
        Ok(())
    }

    /// PATCH a JSON body.
    pub async fn patch<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .patch(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("PATCH {url}"))?;
        let status = resp.status();
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                if let Some(token) = self.refresh_bearer_after_401().await {
                    let retry = self
                        .client
                        .patch(&url)
                        .json(body)
                        .bearer_auth(token)
                        .send()
                        .await
                        .with_context(|| format!("PATCH {url}"))?;
                    return self.json_or_error("PATCH", path, retry).await;
                }
            }
            let body_text = resp.text().await.unwrap_or_default();
            return Err(self.http_error("PATCH", path, status, &body_text));
        }
        resp.json()
            .await
            .with_context(|| format!("parse PATCH {path}"))
    }

    /// PUT a JSON body.
    pub async fn put<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .put(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("PUT {url}"))?;
        let status = resp.status();
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                if let Some(token) = self.refresh_bearer_after_401().await {
                    let retry = self
                        .client
                        .put(&url)
                        .json(body)
                        .bearer_auth(token)
                        .send()
                        .await
                        .with_context(|| format!("PUT {url}"))?;
                    return self.json_or_error("PUT", path, retry).await;
                }
            }
            let body_text = resp.text().await.unwrap_or_default();
            return Err(self.http_error("PUT", path, status, &body_text));
        }
        resp.json()
            .await
            .with_context(|| format!("parse PUT {path}"))
    }

    /// DELETE an endpoint.
    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .with_context(|| format!("DELETE {url}"))?;
        let status = resp.status();
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                if let Some(token) = self.refresh_bearer_after_401().await {
                    let retry = self
                        .client
                        .delete(&url)
                        .bearer_auth(token)
                        .send()
                        .await
                        .with_context(|| format!("DELETE {url}"))?;
                    let retry_status = retry.status();
                    if retry_status.is_success() {
                        return Ok(());
                    }
                    let body = retry.text().await.unwrap_or_default();
                    return Err(self.http_error("DELETE", path, retry_status, &body));
                }
            }
            let body = resp.text().await.unwrap_or_default();
            return Err(self.http_error("DELETE", path, status, &body));
        }
        Ok(())
    }

    async fn json_or_error<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        resp: reqwest::Response,
    ) -> Result<T> {
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(self.http_error(method, path, status, &body));
        }
        resp.json()
            .await
            .with_context(|| format!("parse {method} {path}"))
    }

    async fn refresh_bearer_after_401(&self) -> Option<String> {
        let context = auth::runtime_auth_context()?;
        let creds = match auth::load_profile(&context.profile) {
            Ok(Some(creds)) => creds,
            Ok(None) => return None,
            Err(err) => {
                eprintln!(
                    "Warning: failed to load profile '{}' for token refresh: {err}",
                    context.profile
                );
                return None;
            }
        };
        if creds
            .refresh_token
            .as_deref()
            .unwrap_or_default()
            .is_empty()
        {
            return None;
        }

        match auth::refresh_access_token(&context.oauth_config, &context.profile, &creds).await {
            Ok(new_creds) => {
                let selected = auth::select_api_bearer_token(&new_creds);
                let token_kind = selected
                    .as_ref()
                    .map(|token| token.kind.as_str())
                    .unwrap_or("none");
                eprintln!(
                    "Token refreshed after 401 (profile: {}, api_token={token_kind}).",
                    context.profile
                );
                selected.map(|token| token.value)
            }
            Err(err) => {
                eprintln!(
                    "Warning: token refresh after 401 failed for profile '{}': {err}.",
                    context.profile
                );
                None
            }
        }
    }

    fn http_error(
        &self,
        method: &str,
        path: &str,
        status: reqwest::StatusCode,
        body: &str,
    ) -> anyhow::Error {
        let make = |diagnostics: Option<String>| {
            anyhow::Error::new(HttpError {
                method: method.to_string(),
                path: path.to_string(),
                status,
                body: body.to_string(),
                diagnostics,
            })
        };

        if status != reqwest::StatusCode::UNAUTHORIZED {
            return make(None);
        }

        let Some(diagnostics) = &self.auth_diagnostics else {
            return make(None);
        };

        let profile = diagnostics.profile.as_deref().unwrap_or("-");
        let token_kind = diagnostics.token_kind.as_deref().unwrap_or("-");
        let oauth_client = if diagnostics.oauth_client_configured {
            "configured"
        } else {
            "not configured"
        };

        make(Some(format!(
            "Authentication diagnostics: profile='{profile}', token_kind='{token_kind}', \
             oauth_client={oauth_client}, api_base='{}'. \
             If this is `verify token failed`, check that the profile was created \
             by the intended Cognito OAuth client, the token issuer matches the \
             API Cognito user pool, and the API COGNITO_ALLOWED_CLIENT_IDS includes \
             the CLI OAuth client. For ID tokens inspect `aud`; for access tokens \
             inspect `client_id`. Re-run `tachyon auth login --profile {profile}` \
             after correcting the client/issuer/audience configuration.",
            self.base_url
        )))
    }
}

/// Format helper: truncate a string with ellipsis if longer than max_len.
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{truncated}...")
    } else {
        s.to_string()
    }
}

/// Format helper: print a JSON value as pretty-printed JSON.
pub fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    fn http_err(status: StatusCode, diagnostics: Option<&str>) -> anyhow::Error {
        anyhow::Error::new(HttpError {
            method: "GET".to_string(),
            path: "/v1/compute/builds/b_123/logs".to_string(),
            status,
            body: status.canonical_reason().unwrap_or("error").to_string(),
            diagnostics: diagnostics.map(str::to_string),
        })
    }

    #[test]
    fn http_error_status_extracts_typed_status() {
        let err = http_err(StatusCode::NOT_FOUND, None);
        assert_eq!(http_error_status(&err), Some(StatusCode::NOT_FOUND));

        let err = http_err(StatusCode::INTERNAL_SERVER_ERROR, None);
        assert_eq!(
            http_error_status(&err),
            Some(StatusCode::INTERNAL_SERVER_ERROR)
        );
    }

    #[test]
    fn http_error_status_is_none_for_non_http_errors() {
        let err = anyhow::anyhow!("transport failure");
        assert_eq!(http_error_status(&err), None);
    }

    #[test]
    fn http_error_display_is_backward_compatible() {
        // The previous string-parsing call site matched this exact substring;
        // keep it stable so logs and any external scrapers are unaffected.
        let err = http_err(StatusCode::NOT_FOUND, None);
        assert!(err.to_string().contains("status=404 Not Found"));
    }

    #[test]
    fn http_error_display_includes_diagnostics_when_present() {
        let err = http_err(
            StatusCode::UNAUTHORIZED,
            Some("Authentication diagnostics: ..."),
        );
        let rendered = err.to_string();
        assert!(rendered.contains("status=401 Unauthorized"));
        assert!(rendered.contains("Authentication diagnostics: ..."));
    }
}
