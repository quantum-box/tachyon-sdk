//! MCP server: tool registration and `generate_image` handler.
//!
//! `generate_image` is registered only when `OPENAI_API_KEY` is set at construction
//! time. Otherwise the constructor logs a warning and the server runs with no tools.

use anyhow::Result;
use base64::Engine as _;
use reqwest::Client;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use serde::Deserialize;

use super::openai::{GenerateRequest, OpenAiClient, DEFAULT_MODEL};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GenerateImageParams {
    /// Text prompt describing the image to generate.
    pub prompt: String,
    /// OpenAI model identifier. Defaults to the latest GPT Image model.
    #[serde(default)]
    pub model: Option<String>,
    /// Image size hint (e.g. "1024x1024", "1024x1536", "auto"). Forwarded to OpenAI.
    #[serde(default)]
    pub size: Option<String>,
    /// Quality hint (e.g. "low", "medium", "high"). Forwarded to OpenAI.
    #[serde(default)]
    pub quality: Option<String>,
    /// Number of images to generate (1-10).
    #[serde(default)]
    pub n: Option<u8>,
    /// Response format: "b64_json" (default, returned as MCP image content) or "url"
    /// (returned as MCP text content).
    #[serde(default)]
    pub response_format: Option<String>,
}

#[derive(Clone)]
pub struct TachyonMcpServer {
    openai: Option<OpenAiClient>,
    http: Client,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl TachyonMcpServer {
    pub fn new(openai: Option<OpenAiClient>) -> Self {
        let mut router = Self::tool_router();
        if openai.is_none() {
            // dispatch requirement: skip tool registration when key missing
            router.remove_route("generate_image");
        }
        Self {
            openai,
            http: Client::new(),
            tool_router: router,
        }
    }

    #[tool(
        description = "Generate an image from a text prompt using OpenAI's GPT Image \
                       model. Returns a base64-encoded image (default) or a URL."
    )]
    async fn generate_image(
        &self,
        Parameters(params): Parameters<GenerateImageParams>,
    ) -> Result<CallToolResult, McpError> {
        let Some(openai) = &self.openai else {
            return Err(McpError::invalid_request(
                "generate_image is unavailable: OPENAI_API_KEY is not configured on the server",
                None,
            ));
        };

        let n = params.n.unwrap_or(1).clamp(1, 10);
        let response_format = params
            .response_format
            .clone()
            .unwrap_or_else(|| "b64_json".to_string());
        let model = params
            .model
            .clone()
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        let req = GenerateRequest {
            prompt: params.prompt.clone(),
            model: model.clone(),
            size: params.size.clone(),
            quality: params.quality.clone(),
            n,
            response_format: Some(response_format.clone()),
        };

        let resp = openai
            .generate(&req)
            .await
            .map_err(|e| McpError::internal_error(format!("OpenAI error: {e}"), None))?;

        let mut content = Vec::with_capacity(resp.data.len() * 2);
        for img in resp.data {
            if let Some(rp) = img.revised_prompt {
                content.push(Content::text(format!("revised_prompt: {rp}")));
            }
            if let Some(b64) = img.b64_json {
                content.push(Content::image(b64, "image/png"));
            } else if let Some(url) = img.url {
                if response_format == "b64_json" {
                    // Asked for base64 but provider returned URL — download and encode.
                    match self.fetch_and_encode(&url).await {
                        Ok((data, mime)) => content.push(Content::image(data, mime)),
                        Err(e) => content
                            .push(Content::text(format!("(failed to fetch image: {e}) {url}"))),
                    }
                } else {
                    content.push(Content::text(url));
                }
            }
        }

        if content.is_empty() {
            return Err(McpError::internal_error(
                "OpenAI returned no image data",
                None,
            ));
        }

        Ok(CallToolResult::success(content))
    }

    async fn fetch_and_encode(&self, url: &str) -> anyhow::Result<(String, &'static str)> {
        let resp = self.http.get(url).send().await?.error_for_status()?;
        let mime = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let bytes = resp.bytes().await?;
        let mime_str = match mime.as_deref() {
            Some(m) if m.contains("webp") => "image/webp",
            Some(m) if m.contains("jpeg") || m.contains("jpg") => "image/jpeg",
            _ => "image/png",
        };
        Ok((
            base64::engine::general_purpose::STANDARD.encode(&bytes),
            mime_str,
        ))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for TachyonMcpServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.instructions = Some(
            "Tachyon MCP server. Provides AI image generation via OpenAI's GPT Image model."
                .to_string(),
        );
        info
    }
}
