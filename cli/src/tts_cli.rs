//! `tachyon tts` subcommand — AI text-to-speech.
//!
//! Usage:
//!   tachyon tts synthesize --text "..." [options]

use anyhow::{Context, Result};
use base64::Engine as _;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::ApiClient;

#[derive(Debug, Clone, Args)]
pub struct TtsArgs {
    #[command(subcommand)]
    pub command: TtsCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum TtsCommand {
    /// Convert text to speech audio
    Synthesize {
        /// Text to convert to speech
        #[arg(long, short = 't')]
        text: String,

        /// Model to use for synthesis
        /// (e.g., gemini-2.5-flash-preview-tts, gemini-3.1-flash-tts)
        #[arg(long, short = 'm', default_value = "gemini-2.5-flash-preview-tts")]
        model: String,

        /// Voice name (e.g., Aoede, Charon, Fenrir, Kore, Puck, Orbit,
        /// Zephyr)
        #[arg(long, short = 'v')]
        voice: Option<String>,

        /// Audio format: mp3, wav, or ogg
        #[arg(long, short = 'f', default_value = "mp3")]
        format: String,

        /// Save audio to this local file path
        #[arg(long, short = 'o')]
        output: Option<String>,
    },

    /// List available TTS models
    Models,
}

#[derive(Debug, Serialize)]
struct SynthesizeSpeechRequest {
    text: String,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    voice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SynthesizeSpeechResponse {
    audio_b64: String,
    mime_type: String,
    model: String,
    cost_nanodollars: i64,
}

pub async fn run(args: &TtsArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    match &args.command {
        TtsCommand::Synthesize {
            text,
            model,
            voice,
            format,
            output,
        } => {
            synthesize(
                config,
                tenant_id,
                text,
                model,
                voice.as_deref(),
                format,
                output.as_deref(),
            )
            .await
        }
        TtsCommand::Models => list_models(config, tenant_id).await,
    }
}

async fn synthesize(
    config: &Configuration,
    tenant_id: &str,
    text: &str,
    model: &str,
    voice: Option<&str>,
    format: &str,
    output: Option<&str>,
) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    println!("Synthesizing speech with model: {model}");
    if let Some(v) = voice {
        println!("Voice: {v}");
    }
    println!("Text: {text}");

    let result: SynthesizeSpeechResponse = api
        .post(
            "/v1/audio/speech",
            &SynthesizeSpeechRequest {
                text: text.to_string(),
                model: model.to_string(),
                voice: voice.map(String::from),
                format: Some(format.to_string()),
            },
        )
        .await?;

    println!("\nSynthesized audio using {}", result.model);
    println!("MIME type: {}", result.mime_type);
    let cost_usd = result.cost_nanodollars as f64 / 1_000_000_000.0;
    println!("Cost: ${cost_usd:.6}");

    // Decode and save audio
    let audio_bytes = base64::engine::general_purpose::STANDARD
        .decode(&result.audio_b64)
        .context("Failed to decode base64 audio data")?;

    println!("Audio size: {} bytes", audio_bytes.len());

    if let Some(path) = output {
        tokio::fs::write(path, &audio_bytes)
            .await
            .with_context(|| format!("Failed to write audio to {path}"))?;
        println!("Saved to: {path}");
    } else {
        println!(
            "Base64 audio: {}...",
            &result.audio_b64[..80.min(result.audio_b64.len())]
        );
        println!("Use --output <file.mp3> to save the audio file.");
    }

    Ok(())
}

async fn list_models(_config: &Configuration, _tenant_id: &str) -> Result<()> {
    println!("Available TTS models:");
    println!();
    println!(
        "  gemini-2.5-flash-preview-tts  \
         Google Gemini 2.5 Flash Preview TTS"
    );
    println!(
        "  gemini-3.1-flash-tts          \
         Google Gemini 3.1 Flash TTS"
    );
    println!();
    println!("Available voices (all models):");
    let voices = [
        "Aoede",
        "Charon",
        "Fenrir",
        "Kore",
        "Puck",
        "Orbit",
        "Zephyr",
        "Autonoe",
        "Enceladus",
        "Iapetus",
        "Umbriel",
        "Algieba",
        "Despina",
        "Erinome",
        "Laomedeia",
        "Rasalgethi",
        "Achernar",
        "Achird",
        "Algenib",
        "Schedar",
        "Sulafat",
    ];
    for voice in &voices {
        println!("  {voice}");
    }
    Ok(())
}
