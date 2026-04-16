//! `tachyon image` subcommand — AI image generation.
//!
//! Usage:
//!   tachyon image generate --prompt "..." [options]

use anyhow::{Context, Result};
use base64::Engine as _;
use clap::{Args, Subcommand};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::ApiClient;

#[derive(Debug, Clone, Args)]
pub struct ImageArgs {
    #[command(subcommand)]
    pub command: ImageCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ImageCommand {
    /// Generate an image from a text prompt
    Generate {
        /// Text prompt describing the image to generate
        #[arg(long, short = 'p')]
        prompt: String,

        /// Model to use for generation
        /// (e.g., gpt-image-1.5, gpt-image-1, gpt-image-1-mini,
        ///  grok-2-image, gemini-2.0-flash-exp-image-generation)
        #[arg(long, short = 'm', default_value = "gpt-image-1.5")]
        model: String,

        /// Image size: 1024x1024, 1024x1536, 1536x1024, or auto
        #[arg(long)]
        size: Option<String>,

        /// Quality: low, medium, or high
        #[arg(long)]
        quality: Option<String>,

        /// Number of images to generate (1-10)
        #[arg(long, default_value_t = 1)]
        n: u8,

        /// Response format: url or b64_json
        #[arg(long, default_value = "url")]
        response_format: String,

        /// Save generated image to this local file path
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Upload generated image to Tachyon Storage
        #[arg(long)]
        storage: bool,
    },
}

#[derive(Debug, Serialize)]
struct GenerateImageRequest {
    prompt: String,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<String>,
    n: u8,
    response_format: String,
}

#[derive(Debug, Deserialize)]
struct GenerateImageResponse {
    data: Vec<ImageData>,
    model: String,
    cost_nanodollars: i64,
}

#[derive(Debug, Deserialize)]
struct ImageData {
    url: Option<String>,
    b64_json: Option<String>,
    revised_prompt: Option<String>,
}

#[derive(Debug, Serialize)]
struct UploadUrlRequest {
    content_type: Option<String>,
    extension: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UploadUrlResponse {
    url: String,
    storage_key: String,
    expires_in_secs: u64,
}

#[derive(Debug, Serialize)]
struct ConfirmRequest {
    storage_key: String,
}

#[derive(Debug, Deserialize)]
struct ConfirmResponse {
    storage_key: String,
    url: String,
    content_length: u64,
}

/// Determine the default image file extension from a model name.
fn model_to_extension(model: &str) -> &'static str {
    if model.contains("grok") {
        "webp"
    } else {
        "png"
    }
}

/// Fetch raw image bytes — either decode b64_json or download from URL.
async fn fetch_image_bytes(
    client: &Client,
    img: &ImageData,
) -> Result<(Vec<u8>, &'static str)> {
    if let Some(b64) = &img.b64_json {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .context("Failed to decode base64 image data")?;
        return Ok((bytes, "png"));
    }
    if let Some(url) = &img.url {
        let response = client
            .get(url)
            .send()
            .await
            .context("Failed to download image from URL")?;
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/png")
            .to_string();
        let ext = if content_type.contains("webp") {
            "webp"
        } else if content_type.contains("jpeg")
            || content_type.contains("jpg")
        {
            "jpg"
        } else {
            "png"
        };
        let bytes = response
            .bytes()
            .await
            .context("Failed to read image bytes from URL")?;
        return Ok((bytes.to_vec(), ext));
    }
    anyhow::bail!("Image has neither url nor b64_json data");
}

/// Save image bytes to a local file.
async fn save_to_file(path: &str, bytes: &[u8]) -> Result<()> {
    tokio::fs::write(path, bytes)
        .await
        .with_context(|| format!("Failed to write image to {path}"))?;
    println!("  Saved to: {path}");
    Ok(())
}

/// Upload image bytes to Tachyon Storage via presigned URL.
async fn upload_to_storage(
    api: &ApiClient,
    bytes: Vec<u8>,
    ext: &str,
) -> Result<()> {
    let content_type = match ext {
        "webp" => "image/webp",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "image/png",
    };

    // 1. Get presigned upload URL from Tachyon API
    let upload: UploadUrlResponse = api
        .post(
            "/v1/storage/upload-url",
            &UploadUrlRequest {
                content_type: Some(content_type.to_string()),
                extension: Some(ext.to_string()),
            },
        )
        .await?;

    println!(
        "  Upload URL valid for {}s — uploading...",
        upload.expires_in_secs
    );

    // 2. PUT image bytes directly to the presigned S3 URL (no Tachyon auth headers)
    let plain_client = Client::new();
    let put_status = plain_client
        .put(&upload.url)
        .header(header::CONTENT_TYPE, content_type)
        .body(bytes)
        .send()
        .await
        .context("Failed to upload image to storage")?
        .status();

    if !put_status.is_success() {
        anyhow::bail!("Storage upload failed with status: {put_status}");
    }

    // 3. Confirm the upload through Tachyon API
    let confirm: ConfirmResponse = api
        .post(
            "/v1/storage/confirm",
            &ConfirmRequest {
                storage_key: upload.storage_key,
            },
        )
        .await?;

    println!("  Storage key: {}", confirm.storage_key);
    println!(
        "  Stored: {} ({} bytes)",
        confirm.url, confirm.content_length
    );
    Ok(())
}

pub async fn run(
    args: &ImageArgs,
    config: &Configuration,
    tenant_id: &str,
) -> Result<()> {
    match &args.command {
        ImageCommand::Generate {
            prompt,
            model,
            size,
            quality,
            n,
            response_format,
            output,
            storage,
        } => {
            generate(
                config,
                tenant_id,
                prompt,
                model,
                size.as_deref(),
                quality.as_deref(),
                *n,
                response_format,
                output.as_deref(),
                *storage,
            )
            .await
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn generate(
    config: &Configuration,
    tenant_id: &str,
    prompt: &str,
    model: &str,
    size: Option<&str>,
    quality: Option<&str>,
    n: u8,
    response_format: &str,
    output: Option<&str>,
    storage: bool,
) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    // When saving locally or to storage we need the raw bytes.
    // Prefer b64_json to avoid URL expiry issues.
    let effective_format =
        if (output.is_some() || storage) && response_format == "url" {
            "b64_json"
        } else {
            response_format
        };

    println!("Generating image with model: {model}");
    println!("Prompt: {prompt}");

    let result: GenerateImageResponse = api
        .post(
            "/v1/images/generations",
            &GenerateImageRequest {
                prompt: prompt.to_string(),
                model: model.to_string(),
                size: size.map(String::from),
                quality: quality.map(String::from),
                n,
                response_format: effective_format.to_string(),
            },
        )
        .await?;

    println!(
        "\nGenerated {} image(s) using {}",
        result.data.len(),
        result.model
    );
    let cost_usd = result.cost_nanodollars as f64 / 1_000_000_000.0;
    println!("Cost: ${cost_usd:.6}");

    let ext = model_to_extension(model);

    // Reuse the auth-aware reqwest client for URL downloads
    let dl_client = api.client.clone();

    for (i, img) in result.data.iter().enumerate() {
        println!("\n--- Image {} ---", i + 1);

        if let Some(rp) = &img.revised_prompt {
            println!("Revised prompt: {rp}");
        }

        if let Some(img_url) = &img.url {
            println!("URL: {img_url}");
        }

        if let Some(b64) = &img.b64_json {
            let preview = if b64.len() > 80 {
                format!("{}... ({} bytes total)", &b64[..80], b64.len())
            } else {
                b64.clone()
            };
            println!("Base64 data: {preview}");
        }

        // Save locally
        if let Some(path) = output {
            let save_path = if result.data.len() > 1 {
                let stem = path.trim_end_matches(&format!(".{ext}"));
                format!("{stem}_{}.{ext}", i + 1)
            } else {
                path.to_string()
            };
            let (bytes, _detected_ext) =
                fetch_image_bytes(&dl_client, img).await?;
            save_to_file(&save_path, &bytes).await?;
        }

        // Upload to Tachyon Storage
        if storage {
            let (bytes, detected_ext) =
                fetch_image_bytes(&dl_client, img).await?;
            upload_to_storage(&api, bytes, detected_ext).await?;
        }
    }

    Ok(())
}
