//! `tachyon tts` subcommand — AI text-to-speech.
//!
//! Usage:
//!   tachyon tts synthesize --text "..." [--voice Kore] [--output speech.wav]
//!   tachyon tts models
//!
//! The API returns audio as base64. Gemini TTS models produce 16-bit PCM,
//! which the API wraps in a WAV container (`audio/wav`). Older API
//! versions returned the raw PCM (`audio/L16;codec=pcm;rate=24000`); this
//! command wraps such payloads in a WAV header locally so the saved file
//! is always playable.

use std::path::Path;

use anyhow::{Context, Result};
use base64::Engine as _;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tachyon_sdk::apis::configuration::Configuration;

use crate::client::ApiClient;

/// Default audio format requested from the API.
pub const DEFAULT_FORMAT: &str = "wav";
/// Default TTS model.
pub const DEFAULT_MODEL: &str = "gemini-2.5-flash-preview-tts";
/// Sample rate assumed for raw PCM responses that omit `rate=`.
const DEFAULT_PCM_SAMPLE_RATE: u32 = 24_000;

#[derive(Debug, Clone, Args)]
pub struct TtsArgs {
    #[command(subcommand)]
    pub command: TtsCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum TtsCommand {
    /// Convert text to speech audio and save it to a file
    Synthesize {
        /// Text to convert to speech
        #[arg(long, short = 't')]
        text: String,

        /// Model to use for synthesis (gemini-2.5-flash-preview-tts,
        /// gemini-2.5-pro-preview-tts, gemini-3.1-flash-tts-preview)
        #[arg(long, short = 'm', default_value = DEFAULT_MODEL)]
        model: String,

        /// Voice name (e.g., Aoede, Charon, Fenrir, Kore, Puck, Orbit,
        /// Zephyr). Run `tachyon tts models` for the full list.
        #[arg(long, short = 'v')]
        voice: Option<String>,

        /// Audio format to request. Gemini TTS models only produce wav.
        #[arg(long, short = 'f', default_value = DEFAULT_FORMAT)]
        format: String,

        /// Save audio to this local file path. Defaults to
        /// `speech.<ext>` in the current directory, where <ext> follows
        /// the returned MIME type (e.g. speech.wav).
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Print the result as JSON instead of human-readable text
        #[arg(long)]
        json: bool,
    },

    /// List available TTS models and voices
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

#[derive(Debug, Serialize)]
struct SynthesizeSpeechOutput {
    path: String,
    mime_type: String,
    model: String,
    bytes: usize,
    cost_nanodollars: i64,
    /// True when the API returned raw PCM and the CLI wrapped it in WAV.
    wrapped_pcm: bool,
}

pub async fn run(args: &TtsArgs, config: &Configuration, tenant_id: &str) -> Result<()> {
    match &args.command {
        TtsCommand::Synthesize {
            text,
            model,
            voice,
            format,
            output,
            json,
        } => {
            synthesize(
                config,
                tenant_id,
                text,
                model,
                voice.as_deref(),
                format,
                output.as_deref(),
                *json,
            )
            .await
        }
        TtsCommand::Models => list_models(config, tenant_id).await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn synthesize(
    config: &Configuration,
    tenant_id: &str,
    text: &str,
    model: &str,
    voice: Option<&str>,
    format: &str,
    output: Option<&str>,
    json: bool,
) -> Result<()> {
    let api = ApiClient::new(config, tenant_id)?;

    if !json {
        println!("Synthesizing speech with model: {model}");
        if let Some(v) = voice {
            println!("Voice: {v}");
        }
        println!("Text: {text}");
    }

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

    let raw = base64::engine::general_purpose::STANDARD
        .decode(&result.audio_b64)
        .context("Failed to decode base64 audio data")?;

    // Older API versions return the raw PCM Gemini produces. Wrap it in a
    // WAV header locally so the saved file is playable everywhere.
    let (audio_bytes, mime_type, wrapped_pcm) = match pcm_sample_rate_from_mime(&result.mime_type) {
        Some(rate) => (
            wrap_pcm_in_wav(&raw, rate, 1, 16),
            "audio/wav".to_string(),
            true,
        ),
        None => (raw, result.mime_type.clone(), false),
    };

    let ext = extension_for_mime(&mime_type);
    let path = output
        .map(str::to_string)
        .unwrap_or_else(|| format!("speech.{ext}"));

    if let Some(actual) = Path::new(&path).extension().and_then(|e| e.to_str()) {
        if !actual.eq_ignore_ascii_case(ext) && ext != "bin" {
            eprintln!(
                "warning: API returned {mime_type} but output path ends with .{actual}; \
                 the file content is {ext}"
            );
        }
    }

    tokio::fs::write(&path, &audio_bytes)
        .await
        .with_context(|| format!("Failed to write audio to {path}"))?;

    if json {
        let out = SynthesizeSpeechOutput {
            path,
            mime_type,
            model: result.model,
            bytes: audio_bytes.len(),
            cost_nanodollars: result.cost_nanodollars,
            wrapped_pcm,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!();
    println!("Synthesized audio using {}", result.model);
    println!("MIME type: {mime_type}");
    if wrapped_pcm {
        println!(
            "(API returned raw PCM {}; wrapped in a WAV header)",
            result.mime_type
        );
    }
    let cost_usd = result.cost_nanodollars as f64 / 1_000_000_000.0;
    println!("Cost: ${cost_usd:.6}");
    println!("Audio size: {} bytes", audio_bytes.len());
    println!("Saved to: {path}");

    Ok(())
}

/// Returns the PCM sample rate when `mime_type` denotes raw PCM
/// (`audio/L16` / `audio/pcm`), or `None` for container formats.
pub fn pcm_sample_rate_from_mime(mime_type: &str) -> Option<u32> {
    let mut parts = mime_type.split(';').map(str::trim);
    let media_type = parts.next()?.to_ascii_lowercase();
    if media_type != "audio/l16" && media_type != "audio/pcm" {
        return None;
    }
    let rate = parts
        .filter_map(|param| param.split_once('='))
        .find(|(key, _)| key.trim().eq_ignore_ascii_case("rate"))
        .and_then(|(_, value)| value.trim().parse::<u32>().ok())
        .filter(|rate| *rate > 0)
        .unwrap_or(DEFAULT_PCM_SAMPLE_RATE);
    Some(rate)
}

/// Wraps raw little-endian PCM samples in a canonical 44-byte RIFF/WAVE
/// header.
pub fn wrap_pcm_in_wav(
    pcm: &[u8],
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
) -> Vec<u8> {
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * u32::from(block_align);
    let data_len = u32::try_from(pcm.len()).unwrap_or(u32::MAX);

    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36u32.saturating_add(data_len)).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(pcm);
    wav
}

/// Maps an audio MIME type to a file extension.
pub fn extension_for_mime(mime_type: &str) -> &'static str {
    let media_type = mime_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    match media_type.as_str() {
        "audio/wav" | "audio/x-wav" | "audio/wave" | "audio/vnd.wave" => "wav",
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/ogg" => "ogg",
        "audio/flac" => "flac",
        "audio/aac" => "aac",
        _ => "bin",
    }
}

async fn list_models(_config: &Configuration, _tenant_id: &str) -> Result<()> {
    println!("Available TTS models (output: wav, 16-bit PCM 24 kHz mono):");
    println!();
    println!(
        "  gemini-2.5-flash-preview-tts  \
         Google Gemini 2.5 Flash Preview TTS (default)"
    );
    println!(
        "  gemini-2.5-pro-preview-tts    \
         Google Gemini 2.5 Pro Preview TTS"
    );
    println!(
        "  gemini-3.1-flash-tts-preview  \
         Google Gemini 3.1 Flash TTS Preview"
    );
    println!();
    println!("Available voices (all models):");
    let voices = [
        "Zephyr",
        "Puck",
        "Charon",
        "Kore",
        "Fenrir",
        "Leda",
        "Orus",
        "Aoede",
        "Callirrhoe",
        "Autonoe",
        "Enceladus",
        "Iapetus",
        "Umbriel",
        "Algieba",
        "Despina",
        "Erinome",
        "Algenib",
        "Rasalgethi",
        "Laomedeia",
        "Achernar",
        "Alnilam",
        "Schedar",
        "Gacrux",
        "Pulcherrima",
        "Achird",
        "Zubenelgenubi",
        "Vindemiatrix",
        "Sadachbia",
        "Sadaltager",
        "Sulafat",
    ];
    for voice in &voices {
        println!("  {voice}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcm_mime_detection() {
        assert_eq!(
            pcm_sample_rate_from_mime("audio/L16;codec=pcm;rate=24000"),
            Some(24_000)
        );
        assert_eq!(
            pcm_sample_rate_from_mime("audio/L16"),
            Some(DEFAULT_PCM_SAMPLE_RATE)
        );
        assert_eq!(pcm_sample_rate_from_mime("audio/wav"), None);
        assert_eq!(pcm_sample_rate_from_mime("audio/mpeg"), None);
    }

    #[test]
    fn wav_header_layout() {
        let pcm = [0u8, 1, 2, 3];
        let wav = wrap_pcm_in_wav(&pcm, 24_000, 1, 16);
        assert_eq!(wav.len(), 48);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(wav[4..8].try_into().unwrap()), 40);
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 24_000);
        assert_eq!(u32::from_le_bytes(wav[28..32].try_into().unwrap()), 48_000);
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 4);
        assert_eq!(&wav[44..], &pcm);
    }

    #[test]
    fn extension_mapping() {
        assert_eq!(extension_for_mime("audio/wav"), "wav");
        assert_eq!(extension_for_mime("audio/x-wav; charset=binary"), "wav");
        assert_eq!(extension_for_mime("audio/mpeg"), "mp3");
        assert_eq!(extension_for_mime("audio/ogg"), "ogg");
        assert_eq!(extension_for_mime("application/octet-stream"), "bin");
    }
}
