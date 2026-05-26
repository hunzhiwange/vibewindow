use anyhow::{Context, Result, bail};
use reqwest::multipart::{Form, Part};

use crate::app::agent::config::TranscriptionConfig;

/// Maximum upload size accepted by the Groq Whisper API (25 MB).
const MAX_AUDIO_BYTES: usize = 25 * 1024 * 1024;

/// Map file extension to MIME type for Whisper-compatible transcription APIs.
fn mime_for_audio(extension: &str) -> Option<&'static str> {
    match extension.to_ascii_lowercase().as_str() {
        "flac" => Some("audio/flac"),
        "mp3" | "mpeg" | "mpga" => Some("audio/mpeg"),
        "mp4" | "m4a" => Some("audio/mp4"),
        "ogg" | "oga" => Some("audio/ogg"),
        "opus" => Some("audio/opus"),
        "wav" => Some("audio/wav"),
        "webm" => Some("audio/webm"),
        _ => None,
    }
}

/// Normalize audio filename for Whisper-compatible APIs.
///
/// Groq validates the filename extension — `.oga` (Opus-in-Ogg) is not in
/// its accepted list, so we rewrite it to `.ogg`.
fn normalize_audio_filename(file_name: &str) -> String {
    match file_name.rsplit_once('.') {
        Some((stem, ext)) if ext.eq_ignore_ascii_case("oga") => format!("{stem}.ogg"),
        _ => file_name.to_string(),
    }
}

/// Transcribe audio bytes via a Whisper-compatible transcription API.
///
/// Returns the transcribed text on success.  Requires `GROQ_API_KEY` in the
/// environment.  The caller is responsible for enforcing duration limits
/// *before* downloading the file; this function enforces the byte-size cap.
pub async fn transcribe_audio(
    audio_data: Vec<u8>,
    file_name: &str,
    config: &TranscriptionConfig,
) -> Result<String> {
    if audio_data.len() > MAX_AUDIO_BYTES {
        bail!("Audio file too large ({} bytes, max {MAX_AUDIO_BYTES})", audio_data.len());
    }

    let normalized_name = normalize_audio_filename(file_name);
    let extension = normalized_name.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
    let mime = mime_for_audio(extension).ok_or_else(|| {
        anyhow::anyhow!(
            "Unsupported audio format '.{extension}' — accepted: flac, mp3, mp4, mpeg, mpga, m4a, ogg, opus, wav, webm"
        )
    })?;

    let api_key = std::env::var("GROQ_API_KEY").context(
        "GROQ_API_KEY environment variable is not set — required for voice transcription",
    )?;

    let client = crate::app::agent::config::build_runtime_proxy_client("transcription.groq");

    let file_part = Part::bytes(audio_data).file_name(normalized_name).mime_str(mime)?;

    let mut form = Form::new()
        .part("file", file_part)
        .text("model", config.model.clone())
        .text("response_format", "json");

    if let Some(ref lang) = config.language {
        form = form.text("language", lang.clone());
    }

    let resp = client
        .post(&config.api_url)
        .bearer_auth(&api_key)
        .multipart(form)
        .send()
        .await
        .context("Failed to send transcription request")?;

    let status = resp.status();
    let body: serde_json::Value =
        resp.json().await.context("Failed to parse transcription response")?;

    if !status.is_success() {
        let error_msg = body["error"]["message"].as_str().unwrap_or("unknown error");
        bail!("Transcription API error ({}): {}", status, error_msg);
    }

    let text =
        body["text"].as_str().context("Transcription response missing 'text' field")?.to_string();

    Ok(text)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
