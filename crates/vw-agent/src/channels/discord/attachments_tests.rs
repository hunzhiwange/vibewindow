use super::attachments::{
    audio_extension_from_content_type, extension_from_media_path, has_audio_extension,
    infer_audio_filename, is_audio_attachment, is_image_attachment, is_supported_audio_extension,
    normalize_content_type, parse_attachment_duration_secs,
};

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
