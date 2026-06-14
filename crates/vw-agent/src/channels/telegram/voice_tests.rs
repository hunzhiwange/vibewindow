use super::{TelegramChannel, voice::VoiceMetadata};

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

#[test]
fn voice_metadata_prefers_voice_over_audio_and_filters_blank_hints() {
    let value = serde_json::json!({
        "voice": {
            "file_id": "voice-id",
            "file_name": "   ",
            "mime_type": "",
            "duration": 3
        },
        "audio": {
            "file_id": "audio-id",
            "file_name": "song.mp3",
            "mime_type": "audio/mpeg",
            "duration": 9
        }
    });

    let metadata = TelegramChannel::parse_voice_metadata(&value).unwrap();

    assert_eq!(metadata.file_id, "voice-id");
    assert_eq!(metadata.file_name_hint, None);
    assert_eq!(metadata.mime_type_hint, None);
    assert!(metadata.voice_note);
}

#[test]
fn voice_metadata_reads_audio_file_name_and_defaults_duration() {
    let value = serde_json::json!({
        "audio": {
            "file_id": "audio-id",
            "file_name": "track.mp3",
            "mime_type": "audio/mpeg"
        }
    });

    let metadata = TelegramChannel::parse_voice_metadata(&value).unwrap();

    assert_eq!(metadata.file_id, "audio-id");
    assert_eq!(metadata.duration_secs, 0);
    assert_eq!(metadata.file_name_hint.as_deref(), Some("track.mp3"));
    assert_eq!(metadata.mime_type_hint.as_deref(), Some("audio/mpeg"));
    assert!(!metadata.voice_note);
}

#[test]
fn voice_metadata_requires_string_file_id() {
    assert!(TelegramChannel::parse_voice_metadata(&serde_json::json!({"voice": {}})).is_none());
    assert!(
        TelegramChannel::parse_voice_metadata(&serde_json::json!({"voice": {"file_id": 123}}))
            .is_none()
    );
    assert!(TelegramChannel::parse_voice_metadata(&serde_json::json!({"text": "hello"})).is_none());
}

#[test]
fn extension_from_audio_mime_type_accepts_supported_aliases() {
    let cases = [
        (" audio/flac ", Some("flac")),
        ("AUDIO/X-FLAC", Some("flac")),
        ("audio/mpeg", Some("mp3")),
        ("audio/mp4", Some("mp4")),
        ("audio/x-m4a", Some("m4a")),
        ("application/ogg", Some("ogg")),
        ("audio/opus", Some("opus")),
        ("audio/x-wav", Some("wav")),
        ("audio/wave", Some("wav")),
        ("audio/webm", Some("webm")),
        ("audio/aac", None),
    ];

    for (mime, expected) in cases {
        assert_eq!(TelegramChannel::extension_from_audio_mime_type(mime), expected);
    }
}

#[test]
fn has_file_extension_requires_non_empty_extension() {
    assert!(TelegramChannel::has_file_extension("clip.ogg"));
    assert!(TelegramChannel::has_file_extension("/tmp/archive.tar.gz"));
    assert!(!TelegramChannel::has_file_extension("clip"));
    assert!(!TelegramChannel::has_file_extension("clip."));
}

#[test]
fn infer_voice_filename_prefers_path_basename_with_extension() {
    let metadata = VoiceMetadata {
        file_id: "id".to_string(),
        duration_secs: 0,
        file_name_hint: Some("hint.mp3".to_string()),
        mime_type_hint: Some("audio/ogg".to_string()),
        voice_note: true,
    };

    assert_eq!(
        TelegramChannel::infer_voice_filename("voice/path/server-name.opus", &metadata),
        "server-name.opus"
    );
}

#[test]
fn infer_voice_filename_uses_hint_extension_before_mime() {
    let metadata = VoiceMetadata {
        file_id: "id".to_string(),
        duration_secs: 0,
        file_name_hint: Some("hint.m4a".to_string()),
        mime_type_hint: Some("audio/ogg".to_string()),
        voice_note: false,
    };

    assert_eq!(TelegramChannel::infer_voice_filename("voice/no_extension", &metadata), "hint.m4a");
}

#[test]
fn infer_voice_filename_uses_stem_hint_and_defaults_for_missing_path() {
    let voice = VoiceMetadata {
        file_id: "id".to_string(),
        duration_secs: 0,
        file_name_hint: Some("spoken-note".to_string()),
        mime_type_hint: None,
        voice_note: true,
    };
    assert_eq!(TelegramChannel::infer_voice_filename("", &voice), "spoken-note.ogg");

    let audio = VoiceMetadata { voice_note: false, ..voice };
    assert_eq!(TelegramChannel::infer_voice_filename("downloads/audio.", &audio), "audio.mp3");
}

#[tokio::test]
async fn try_parse_voice_message_returns_none_for_missing_update_message_or_metadata() {
    let mut config = crate::app::agent::config::TranscriptionConfig::default();
    config.enabled = true;
    let channel =
        TelegramChannel::new("token".into(), vec!["*".into()], false).with_transcription(config);

    assert!(channel.try_parse_voice_message(&serde_json::json!({})).await.is_none());
    assert!(
        channel
            .try_parse_voice_message(&serde_json::json!({
                "message": {
                    "message_id": 1,
                    "text": "not voice",
                    "from": {"id": 1},
                    "chat": {"id": 10}
                }
            }))
            .await
            .is_none()
    );
}

#[tokio::test]
async fn try_parse_voice_message_rejects_group_mention_only_before_download() {
    let mut config = crate::app::agent::config::TranscriptionConfig::default();
    config.enabled = true;
    config.max_duration_secs = 60;
    let channel =
        TelegramChannel::new("token".into(), vec!["alice".into()], true).with_transcription(config);

    let update = serde_json::json!({
        "message": {
            "message_id": 1,
            "voice": {"file_id": "voice-file", "duration": 1},
            "from": {"id": 11, "username": "alice"},
            "chat": {"id": -100, "type": "group"}
        }
    });

    assert!(channel.try_parse_voice_message(&update).await.is_none());
    assert!(channel.voice_transcriptions.lock().is_empty());
}
