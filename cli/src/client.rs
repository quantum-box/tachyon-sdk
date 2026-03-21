use anyhow::{anyhow, Context, Result};
use reqwest::{header, Client};
use serde::Deserialize;
use tachyon_sdk::apis::configuration::Configuration;

/// Build a reqwest client with Tachyon auth headers.
pub fn build_client(config: &Configuration, tenant_id: &str) -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert("x-operator-id", header::HeaderValue::from_str(tenant_id)?);
    if let Some(token) = &config.bearer_access_token {
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {token}"))?,
        );
    }
    Ok(Client::builder().default_headers(headers).build()?)
}

/// Helper to GET a URL and return the parsed JSON.
pub async fn get_json<T: for<'de> Deserialize<'de>>(client: &Client, url: &str) -> Result<T> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to GET {url}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("request failed: status={status}, body={body}"));
    }
    response
        .json()
        .await
        .with_context(|| format!("failed to parse response from {url}"))
}

/// Helper to GET a URL with query parameters.
pub async fn get_json_with_query<T, Q>(client: &Client, url: &str, query: &Q) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
    Q: serde::Serialize + ?Sized,
{
    let response = client
        .get(url)
        .query(query)
        .send()
        .await
        .with_context(|| format!("failed to GET {url}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("request failed: status={status}, body={body}"));
    }
    response
        .json()
        .await
        .with_context(|| format!("failed to parse response from {url}"))
}

/// Format a base URL by joining the base path and the given path.
pub fn api_url(config: &Configuration, path: &str) -> String {
    format!("{}{path}", config.base_path.trim_end_matches('/'))
}
