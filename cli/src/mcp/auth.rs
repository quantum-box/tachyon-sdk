//! HTTP transport authentication middleware.
//!
//! Two modes (HTTP transport only — stdio uses env per MCP spec 2025-06-18):
//!
//! 1. `Authorization: Bearer <token>` — MCP-spec compliant (RFC 6750, OAuth 2.1 §5).
//! 2. `?apikey=<token>` URL query — **custom non-spec extension** (spec explicitly
//!    forbids tokens in URI: "Access tokens MUST NOT be included in the URI
//!    query string"). Disabled by default; opt-in via `--custom-query-auth`.
//!
//! Both modes log the masked token only. Responses include `Referrer-Policy:
//! no-referrer` to mitigate URL-based token leakage when query mode is enabled.

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct AuthConfig {
    /// Accepted bearer tokens. Empty = no auth required (local dev).
    pub tokens: Vec<String>,
    /// Allow `?apikey=<token>` URL query authentication. **Non-spec extension.**
    pub allow_query_auth: bool,
}

impl AuthConfig {
    pub fn is_enforced(&self) -> bool {
        !self.tokens.is_empty()
    }
}

pub async fn auth_layer(
    State(cfg): State<Arc<AuthConfig>>,
    req: Request,
    next: Next,
) -> Response {
    if !cfg.is_enforced() {
        return apply_security_headers(next.run(req).await);
    }

    let provided = extract_token(&req, cfg.allow_query_auth);
    let masked = provided.as_deref().map(mask_token).unwrap_or_default();

    match provided {
        Some(token) if cfg.tokens.iter().any(|t| constant_time_eq(t, &token)) => {
            tracing::debug!(token = %masked, "mcp auth ok");
            apply_security_headers(next.run(req).await)
        }
        Some(_) => {
            tracing::warn!(token = %masked, "mcp auth rejected: token mismatch");
            unauthorized()
        }
        None => {
            tracing::warn!("mcp auth rejected: no credentials");
            unauthorized()
        }
    }
}

fn extract_token(req: &Request<Body>, allow_query: bool) -> Option<String> {
    if let Some(v) = req.headers().get(axum::http::header::AUTHORIZATION) {
        if let Ok(s) = v.to_str() {
            if let Some(rest) = s.strip_prefix("Bearer ") {
                return Some(rest.trim().to_string());
            }
        }
    }
    if allow_query {
        if let Some(q) = req.uri().query() {
            for pair in q.split('&') {
                if let Some(v) = pair.strip_prefix("apikey=") {
                    return Some(urlencoding::decode(v).ok()?.to_string());
                }
            }
        }
    }
    None
}

fn unauthorized() -> Response {
    let mut resp = (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    resp.headers_mut().insert(
        axum::http::header::WWW_AUTHENTICATE,
        HeaderValue::from_static("Bearer realm=\"tachyon-mcp\""),
    );
    apply_security_headers(resp)
}

fn apply_security_headers(mut resp: Response) -> Response {
    let h: &mut HeaderMap = resp.headers_mut();
    h.insert(
        axum::http::header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    h.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
    resp
}

/// Mask a token for safe logging: keep first 4 + last 2 chars.
pub fn mask_token(token: &str) -> String {
    let n = token.chars().count();
    if n <= 8 {
        return "*".repeat(n);
    }
    let head: String = token.chars().take(4).collect();
    let tail: String = token.chars().rev().take(2).collect::<String>().chars().rev().collect();
    format!("{head}***{tail}")
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}
