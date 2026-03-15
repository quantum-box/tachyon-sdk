use anyhow::{anyhow, Context, Result};
use reqwest::{header, Client};
use serde::de::DeserializeOwned;
use tachyon_sdk::apis::configuration::Configuration;

/// Shared API client that carries Tachyon auth headers.
pub struct ApiClient {
    pub client: Client,
    pub base_url: String,
}

impl ApiClient {
    /// Build from SDK configuration and tenant ID.
    pub fn new(config: &Configuration, tenant_id: &str) -> Result<Self> {
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
        Ok(Self { client, base_url })
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
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("GET {path} failed: status={status}, body={body}"));
        }
        resp.json().await.with_context(|| format!("parse GET {path}"))
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
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("GET {path} failed: status={status}, body={body}"));
        }
        resp.json().await.with_context(|| format!("parse GET {path}"))
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
            let body_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "POST {path} failed: status={status}, body={body_text}"
            ));
        }
        resp.json().await.with_context(|| format!("parse POST {path}"))
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
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "POST {path} failed: status={status}, body={body}"
            ));
        }
        Ok(status.to_string())
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
            let body_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "PATCH {path} failed: status={status}, body={body_text}"
            ));
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
            let body_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "PUT {path} failed: status={status}, body={body_text}"
            ));
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
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "DELETE {path} failed: status={status}, body={body}"
            ));
        }
        Ok(())
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
