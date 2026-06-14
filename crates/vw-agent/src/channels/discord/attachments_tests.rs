use super::attachments::{
    audio_extension_from_content_type, extension_from_media_path, has_audio_extension,
    infer_audio_filename, is_audio_attachment, is_image_attachment, is_supported_audio_extension,
    normalize_content_type, parse_attachment_duration_secs, process_attachments,
};
use crate::app::agent::config::TranscriptionConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[test]
fn content_type_and_extension_helpers_normalize_media_values() {
    assert_eq!(normalize_content_type("Audio/MPEG; charset=utf-8"), "audio/mpeg");
    assert_eq!(
        extension_from_media_path("https://cdn.test/voice.OGG?x=1"),
        Some("ogg".to_string())
    );
    assert!(is_supported_audio_extension("mp3"));
    assert!(has_audio_extension("clip.webm#frag"));
    assert_eq!(audio_extension_from_content_type("audio/x-wav"), Some("wav"));
}

#[test]
fn attachment_type_detection_trusts_explicit_content_type_before_extension() {
    assert!(is_image_attachment("image/png", "file.bin", "https://cdn.test/file.bin"));
    assert!(!is_image_attachment("text/plain", "image.png", "https://cdn.test/image.png"));
    assert!(is_audio_attachment("application/octet-stream", "voice.ogg", ""));
    assert!(!is_audio_attachment("text/plain", "voice.ogg", ""));
}

#[test]
fn duration_and_audio_filename_are_inferred_deterministically() {
    assert_eq!(parse_attachment_duration_secs(&serde_json::json!({"duration_secs": 1.2})), Some(2));
    assert_eq!(parse_attachment_duration_secs(&serde_json::json!({"duration_secs": -1})), None);
    assert_eq!(
        infer_audio_filename("", "https://cdn.test/path/audio.mp3?download=1", ""),
        "audio.mp3"
    );
    assert_eq!(infer_audio_filename("raw", "", "audio/ogg"), "audio.ogg");
}

#[test]
fn media_extension_helpers_cover_images_and_fallbacks() {
    assert_eq!(extension_from_media_path("no-extension"), None);
    assert_eq!(extension_from_media_path("photo.JPEG#preview"), Some("jpeg".to_string()));
    assert!(is_image_attachment("", "photo.avif", ""));
    assert!(is_image_attachment("application/octet-stream", "file.bin", "https://cdn.test/a.svg"));
    assert!(!is_supported_audio_extension("txt"));
    assert_eq!(audio_extension_from_content_type("application/json"), None);
    assert_eq!(infer_audio_filename("  voice.webm  ", "", ""), "voice.webm");
    assert_eq!(infer_audio_filename("raw", "https://cdn.test/no-extension", ""), "audio.ogg");
}

#[test]
fn duration_parser_accepts_integer_zero_and_rejects_non_numbers() {
    assert_eq!(parse_attachment_duration_secs(&serde_json::json!({"duration_secs": 0})), Some(0));
    assert_eq!(parse_attachment_duration_secs(&serde_json::json!({"duration_secs": "1"})), None);
    assert_eq!(parse_attachment_duration_secs(&serde_json::json!({})), None);
}

async fn serve_once(status: &str, body: &str) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let status = status.to_string();
    let body = body.to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0_u8; 1024];
        let _ = socket.read(&mut request).await;
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await.unwrap();
    });

    format!("http://{addr}/attachment")
}

#[tokio::test]
async fn process_attachments_inlines_images_and_text_files() {
    let text_url = serve_once("200 OK", "hello from attachment").await;
    let attachments = vec![
        serde_json::json!({
            "content_type": "image/png",
            "filename": "photo.png",
            "url": "https://cdn.example.test/photo.png"
        }),
        serde_json::json!({
            "content_type": "text/plain; charset=utf-8",
            "filename": "note.txt",
            "url": text_url
        }),
        serde_json::json!({
            "content_type": "image/png",
            "filename": "missing-url.png"
        }),
        serde_json::json!({
            "content_type": "application/pdf",
            "filename": "manual.pdf",
            "url": "https://cdn.example.test/manual.pdf"
        }),
    ];

    let output = process_attachments(&attachments, &reqwest::Client::new(), None).await;

    assert_eq!(
        output,
        "[IMAGE:https://cdn.example.test/photo.png]\n---\n[note.txt]\nhello from attachment"
    );
}

#[tokio::test]
async fn process_attachments_skips_failed_text_fetch_and_long_audio() {
    let text_url = serve_once("404 Not Found", "nope").await;
    let mut transcription = TranscriptionConfig {
        enabled: true,
        max_duration_secs: 1,
        ..TranscriptionConfig::default()
    };
    transcription.api_url = "http://127.0.0.1:9/transcribe".to_string();
    let attachments = vec![
        serde_json::json!({
            "content_type": "text/plain",
            "filename": "missing.txt",
            "url": text_url
        }),
        serde_json::json!({
            "content_type": "audio/ogg",
            "filename": "too-long.ogg",
            "url": "http://127.0.0.1:9/audio.ogg",
            "duration_secs": 2
        }),
        serde_json::json!({
            "content_type": "audio/ogg",
            "filename": "disabled.ogg",
            "url": "http://127.0.0.1:9/disabled.ogg"
        }),
    ];

    let output =
        process_attachments(&attachments[..2], &reqwest::Client::new(), Some(&transcription)).await;
    assert!(output.is_empty());

    let output = process_attachments(&attachments[2..], &reqwest::Client::new(), None).await;
    assert!(output.is_empty());
}
