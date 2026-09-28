use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

use base64::Engine as _;
use serde_json::Value;
use tempfile::TempDir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_tachyon")
}

fn isolated_command(home: &Path) -> Command {
    let mut cmd = Command::new(bin());
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("TACHYON_TENANT_ID", "tn_test1234567890")
        .env("TACHYON_API_KEY", "test-token")
        .env_remove("TACHYON_CONFIG")
        .env_remove("TACHYON_PROFILE")
        .current_dir(home);
    cmd
}

fn start_server(body: String) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0_u8; 16384];
        let n = stream.read(&mut buf).unwrap();
        tx.send(String::from_utf8_lossy(&buf[..n]).to_string())
            .unwrap();

        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, rx, handle)
}

fn request_json_body(request: &str) -> Value {
    let body = request.split("\r\n\r\n").nth(1).unwrap();
    serde_json::from_str(body).unwrap()
}

fn speech_response(audio: &[u8], mime_type: &str) -> String {
    speech_response_for_model(audio, mime_type, "gemini-2.5-flash-preview-tts")
}

fn speech_response_for_model(audio: &[u8], mime_type: &str, model: &str) -> String {
    let audio_b64 = base64::engine::general_purpose::STANDARD.encode(audio);
    serde_json::json!({
        "audio_b64": audio_b64,
        "mime_type": mime_type,
        "model": model,
        "cost_nanodollars": 0,
    })
    .to_string()
}

#[test]
fn tts_models_and_synthesize_help_list_the_supported_catalog() {
    let tmp = TempDir::new().unwrap();
    let output = isolated_command(tmp.path())
        .args(["tts", "models"])
        .output()
        .expect("run tachyon tts models");

    assert!(output.status.success());
    let models = String::from_utf8_lossy(&output.stdout);
    for model in [
        "gemini-3.8-flash-tts",
        "gemini-3.8-flash-lite-tts",
        "gemini-3.1-flash-tts-preview",
        "gemini-2.5-flash-preview-tts",
        "gemini-2.5-pro-preview-tts",
    ] {
        assert!(models.contains(model), "missing {model} in:\n{models}");
    }
    assert!(models.contains("legacy; migrate to Gemini 3.8"), "{models}");
    assert!(models.contains("(default)"), "{models}");

    let help = isolated_command(tmp.path())
        .args(["tts", "synthesize", "--help"])
        .output()
        .expect("run tachyon tts synthesize --help");

    assert!(help.status.success());
    let help = String::from_utf8_lossy(&help.stdout);
    for model in [
        "gemini-3.8-flash-tts",
        "gemini-3.8-flash-lite-tts",
        "gemini-3.1-flash-tts-preview",
        "gemini-2.5-flash-preview-tts",
        "gemini-2.5-pro-preview-tts",
    ] {
        assert!(help.contains(model), "missing {model} in:\n{help}");
    }
}

#[test]
fn tts_synthesize_rejects_models_outside_the_catalog() {
    let tmp = TempDir::new().unwrap();
    let output = isolated_command(tmp.path())
        .args([
            "tts",
            "synthesize",
            "--text",
            "hello",
            "--model",
            "gemini-9.9-fake-tts",
        ])
        .output()
        .expect("run tachyon tts synthesize with an unsupported model");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"), "{stderr}");
    assert!(stderr.contains("gemini-3.8-flash-tts"), "{stderr}");
    assert!(stderr.contains("gemini-3.8-flash-lite-tts"), "{stderr}");
}

#[test]
fn tts_synthesize_selects_gemini_3_8_models_and_writes_wav() {
    let wav = b"RIFF\x28\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x01\x00\xc0\x5d\x00\x00\x80\xbb\x00\x00\x02\x00\x10\x00data\x04\x00\x00\x00\x01\x00\x02\x00";

    for model in ["gemini-3.8-flash-tts", "gemini-3.8-flash-lite-tts"] {
        let tmp = TempDir::new().unwrap();
        let (api_url, rx, handle) =
            start_server(speech_response_for_model(wav, "audio/wav", model));

        let output = isolated_command(tmp.path())
            .env("TACHYON_API_URL", api_url)
            .args([
                "tts",
                "synthesize",
                "--text",
                "hello",
                "--model",
                model,
                "--output",
                "out.wav",
            ])
            .output()
            .expect("run tachyon tts synthesize for a Gemini 3.8 model");

        assert!(
            output.status.success(),
            "tts synthesize failed for {model}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        handle.join().unwrap();
        let request = rx.recv().unwrap();
        let body = request_json_body(&request);
        assert_eq!(body["model"], model);
        assert_eq!(body["format"], "wav");
        assert_eq!(std::fs::read(tmp.path().join("out.wav")).unwrap(), wav);
    }
}

#[test]
fn tts_synthesize_requests_wav_and_writes_wav_file() {
    let tmp = TempDir::new().unwrap();
    let wav = b"RIFF\x28\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x01\x00\xc0\x5d\x00\x00\x80\xbb\x00\x00\x02\x00\x10\x00data\x04\x00\x00\x00\x01\x00\x02\x00";
    let (api_url, rx, handle) = start_server(speech_response(wav, "audio/wav"));

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "tts",
            "synthesize",
            "--text",
            "こんにちは",
            "--voice",
            "Kore",
            "--output",
            "out.wav",
        ])
        .output()
        .expect("run tachyon tts synthesize");

    assert!(
        output.status.success(),
        "tts synthesize failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    handle.join().unwrap();
    let req = rx.recv().unwrap();
    assert!(req.starts_with("POST /v1/audio/speech "), "{req}");
    assert!(req.contains("authorization: Bearer test-token"));
    assert!(req.contains("x-operator-id: tn_test1234567890"));
    let body = request_json_body(&req);
    assert_eq!(body["text"], "こんにちは");
    assert_eq!(body["model"], "gemini-2.5-flash-preview-tts");
    assert_eq!(body["voice"], "Kore");
    assert_eq!(body["format"], "wav");

    let saved = std::fs::read(tmp.path().join("out.wav")).unwrap();
    assert_eq!(saved, wav);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MIME type: audio/wav"), "{stdout}");
    assert!(stdout.contains("Saved to: out.wav"), "{stdout}");
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("warning:"),
        "unexpected warning: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn tts_synthesize_wraps_raw_pcm_from_older_api_and_defaults_output_path() {
    let tmp = TempDir::new().unwrap();
    let pcm: Vec<u8> = vec![1, 0, 2, 0, 3, 0, 4, 0];
    let (api_url, _rx, handle) =
        start_server(speech_response(&pcm, "audio/L16;codec=pcm;rate=24000"));

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args(["tts", "synthesize", "--text", "hello", "--json"])
        .output()
        .expect("run tachyon tts synthesize");

    assert!(
        output.status.success(),
        "tts synthesize failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    handle.join().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("json output");
    assert_eq!(json["path"], "speech.wav");
    assert_eq!(json["mime_type"], "audio/wav");
    assert_eq!(json["wrapped_pcm"], true);
    assert_eq!(json["bytes"], 44 + pcm.len());

    let saved = std::fs::read(tmp.path().join("speech.wav")).unwrap();
    assert_eq!(saved.len(), 44 + pcm.len());
    assert_eq!(&saved[0..4], b"RIFF");
    assert_eq!(&saved[8..12], b"WAVE");
    assert_eq!(
        u32::from_le_bytes(saved[24..28].try_into().unwrap()),
        24_000
    );
    assert_eq!(&saved[44..], &pcm[..]);
}

#[test]
fn tts_synthesize_warns_when_extension_mismatches_mime() {
    let tmp = TempDir::new().unwrap();
    let (api_url, _rx, handle) = start_server(speech_response(
        &[1, 0, 2, 0],
        "audio/L16;codec=pcm;rate=24000",
    ));

    let output = isolated_command(tmp.path())
        .env("TACHYON_API_URL", api_url)
        .args([
            "tts",
            "synthesize",
            "--text",
            "hello",
            "--output",
            "out.mp3",
        ])
        .output()
        .expect("run tachyon tts synthesize");

    assert!(output.status.success());
    handle.join().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("warning: API returned audio/wav but output path ends with .mp3"),
        "{stderr}"
    );
    assert!(tmp.path().join("out.mp3").exists());
}
