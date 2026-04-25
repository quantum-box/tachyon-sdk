//! OpenAI image generation client.
//!
//! Calls `POST /v1/images/generations` directly with the configured API key.
//! Default model is `gpt-image-2` per OpenAI docs (2026-04-21 snapshot).

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const OPENAI_API_BASE: &str = "https://api.openai.com";
pub const DEFAULT_MODEL: &str = "gpt-image-2";

#[derive(Debug, Serialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    pub n: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneratedImage {
    pub url: Option<String>,
    pub b64_json: Option<String>,
    pub revised_prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    pub data: Vec<GeneratedImage>,
}

#[derive(Clone)]
pub struct OpenAiClient {
    http: Client,
    api_key: String,
    base_url: String,
}

impl OpenAiClient {
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").ok()?;
        if api_key.trim().is_empty() {
            return None;
        }
        let base_url =
            std::env::var("OPENAI_API_BASE").unwrap_or_else(|_| OPENAI_API_BASE.to_string());
        let http = Client::builder().build().ok()?;
        Some(Self {
            http,
            api_key,
            base_url,
        })
    }

    pub async fn generate(&self, req: &GenerateRequest) -> Result<GenerateResponse> {
        let url = format!(
            "{}/v1/images/generations",
            self.base_url.trim_end_matches('/')
        );
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(req)
            .send()
            .await
            .context("OpenAI request failed")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("OpenAI returned {status}: {body}");
        }
        resp.json::<GenerateResponse>()
            .await
            .context("Failed to decode OpenAI response")
    }
}
