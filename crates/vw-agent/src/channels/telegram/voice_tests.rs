use super::{voice::VoiceMetadata, TelegramChannel};

#[test]
fn voice_metadata_parser_reads_required_file_id() {
    let value = serde_json::json!({
        "voice": {
            "file_id": "voice-file",
            "file_unique_id": "unique",
            "mime_type": "audio/ogg",
            "duration": 9
        }
    });

    let metadata = TelegramChannel::parse_voice_metadata(&value).unwrap();

    assert_eq!(metadata.file_id, "voice-file");
    assert_eq!(metadata.mime_type_hint.as_deref(), Some("audio/ogg"));
    assert_eq!(metadata.duration_secs, 9);
    assert!(metadata.voice_note);
}

#[test]
fn voice_metadata_struct_preserves_optional_fields() {
    let metadata = VoiceMetadata {
        file_id: "id".to_string(),
        duration_secs: 0,
        file_name_hint: Some("clip.ogg".to_string()),
        mime_type_hint: None,
        voice_note: true,
    };

    assert_eq!(metadata.file_name_hint.as_deref(), Some("clip.ogg"));
}
