#[test]
fn task_633_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("transcription_tests.rs"));
}

#[test]
fn transcription_config_defaults_and_deserialize_match_expected_values() {
    let config = super::TranscriptionConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.api_url, "https://api.groq.com/openai/v1/audio/transcriptions");
    assert_eq!(config.model, "whisper-large-v3-turbo");
    assert_eq!(config.language, None);
    assert_eq!(config.max_duration_secs, 300);

    let parsed: super::TranscriptionConfig = serde_json::from_str("{}").unwrap();
    assert!(!parsed.enabled);
    assert_eq!(parsed.api_url, "https://api.groq.com/openai/v1/audio/transcriptions");
    assert_eq!(parsed.model, "whisper-large-v3-turbo");
    assert_eq!(parsed.language, None);
    assert_eq!(parsed.max_duration_secs, 300);
}

#[test]
fn transcription_config_deserializes_custom_overrides() {
    let parsed: super::TranscriptionConfig = serde_json::from_value(serde_json::json!({
        "enabled": true,
        "api_url": "https://transcribe.example.com/v1/audio/transcriptions",
        "model": "whisper-custom",
        "language": "zh",
        "max_duration_secs": 45
    }))
    .unwrap();

    assert!(parsed.enabled);
    assert_eq!(parsed.api_url, "https://transcribe.example.com/v1/audio/transcriptions");
    assert_eq!(parsed.model, "whisper-custom");
    assert_eq!(parsed.language.as_deref(), Some("zh"));
    assert_eq!(parsed.max_duration_secs, 45);
}
