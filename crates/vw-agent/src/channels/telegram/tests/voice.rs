//! Telegram 频道语音消息处理测试模块
//!
//! 本模块包含针对 Telegram 频道语音消息功能的单元测试和集成测试。
//! 主要测试以下功能：
//!
//! - **语音元数据解析**：从 Telegram 消息中提取语音/音频文件的元数据
//! - **文件名推断**：根据元数据和 MIME 类型推断合适的文件名
//! - **转录配置**：验证语音转录功能的启用/禁用逻辑
//! - **语音消息解析**：测试完整的语音消息处理流程
//! - **端到端转录**：使用真实 API 进行语音转文字测试（需要 API 密钥）

use super::*;

/// 测试从包含 voice 字段的消息中提取语音元数据
///
/// 验证点：
/// - file_id 正确提取
/// - duration 正确提取
/// - voice_note 标志正确设置为 true（表示这是语音消息而非普通音频）
#[test]
fn parse_voice_metadata_extracts_voice() {
    let msg = serde_json::json!({
        "voice": {
            "file_id": "abc123",
            "duration": 5
        }
    });
    let meta = TelegramChannel::parse_voice_metadata(&msg).unwrap();
    assert_eq!(meta.file_id, "abc123");
    assert_eq!(meta.duration_secs, 5);
    assert!(meta.voice_note);
}

/// 测试从包含 audio 字段的消息中提取音频元数据
///
/// 验证点：
/// - file_id 正确提取
/// - duration 正确提取
/// - voice_note 标志正确设置为 false（表示这是普通音频文件）
#[test]
fn parse_voice_metadata_extracts_audio() {
    let msg = serde_json::json!({
        "audio": {
            "file_id": "audio456",
            "duration": 30
        }
    });
    let meta = TelegramChannel::parse_voice_metadata(&msg).unwrap();
    assert_eq!(meta.file_id, "audio456");
    assert_eq!(meta.duration_secs, 30);
    assert!(!meta.voice_note);
}

/// 测试纯文本消息不返回语音元数据
///
/// 当消息只包含文本字段时，应该返回 None，
/// 表示这不是语音/音频消息
#[test]
fn parse_voice_metadata_returns_none_for_text() {
    let msg = serde_json::json!({
        "text": "hello"
    });
    assert!(TelegramChannel::parse_voice_metadata(&msg).is_none());
}

/// 测试缺失 duration 字段时默认为零
///
/// Telegram API 可能返回没有 duration 字段的消息，
/// 此时应该使用 0 作为默认值
#[test]
fn parse_voice_metadata_defaults_duration_to_zero() {
    let msg = serde_json::json!({
        "voice": {
            "file_id": "no_dur"
        }
    });
    let meta = TelegramChannel::parse_voice_metadata(&msg).unwrap();
    assert_eq!(meta.duration_secs, 0);
}

/// 测试文件名推断优先使用带有扩展名的提示名称
///
/// 当元数据中包含 file_name_hint 且该提示已带有扩展名时，
/// 应直接使用该名称，无需从路径或 MIME 类型推断
#[test]
fn infer_voice_filename_prefers_hint_with_extension() {
    let meta = VoiceMetadata {
        file_id: "f".into(),
        duration_secs: 0,
        file_name_hint: Some("telegram_voice.m4a".into()),
        mime_type_hint: Some("audio/mp4".into()),
        voice_note: false,
    };
    assert_eq!(
        TelegramChannel::infer_voice_filename("voice/file_without_ext", &meta),
        "telegram_voice.m4a"
    );
}

/// 测试当路径无扩展名且无文件名提示时，使用 MIME 类型推断扩展名
///
/// 推断优先级：
/// 1. file_name_hint（如果存在且带扩展名）
/// 2. 根据 mime_type_hint 推断扩展名
/// 3. 默认扩展名（.mp3 或 .ogg）
#[test]
fn infer_voice_filename_uses_mime_extension_when_path_has_none() {
    let meta = VoiceMetadata {
        file_id: "f".into(),
        duration_secs: 0,
        file_name_hint: None,
        mime_type_hint: Some("audio/ogg".into()),
        voice_note: true,
    };
    assert_eq!(
        TelegramChannel::infer_voice_filename("voice/file_without_ext", &meta),
        "file_without_ext.ogg"
    );
}

/// 测试当没有任何提示信息时，使用默认扩展名
///
/// 当缺少 file_name_hint 和 mime_type_hint 时，
/// 对于非语音消息，应该使用 .mp3 作为默认扩展名
#[test]
fn infer_voice_filename_falls_back_for_audio_without_hints() {
    let meta = VoiceMetadata {
        file_id: "f".into(),
        duration_secs: 0,
        file_name_hint: None,
        mime_type_hint: None,
        voice_note: false,
    };
    assert_eq!(
        TelegramChannel::infer_voice_filename("voice/file_without_ext", &meta),
        "file_without_ext.mp3"
    );
}

/// 测试 with_transcription 方法在启用时正确设置配置
///
/// 当 TranscriptionConfig.enabled 为 true 时，
/// TelegramChannel 应该存储转录配置
#[test]
fn with_transcription_sets_config_when_enabled() {
    let mut tc = crate::app::agent::config::TranscriptionConfig::default();
    tc.enabled = true;

    let ch = TelegramChannel::new("token".into(), vec!["*".into()], false).with_transcription(tc);
    assert!(ch.transcription.is_some());
}

/// 测试 with_transcription 方法在禁用时不存储配置
///
/// 当 TranscriptionConfig.enabled 为 false（默认值）时，
/// TelegramChannel 不应该存储转录配置
#[test]
fn with_transcription_skips_when_disabled() {
    let tc = crate::app::agent::config::TranscriptionConfig::default();
    let ch = TelegramChannel::new("token".into(), vec!["*".into()], false).with_transcription(tc);
    assert!(ch.transcription.is_none());
}

/// 测试转录禁用时，try_parse_voice_message 返回 None
///
/// 当频道未配置语音转录功能时，
/// 即使收到语音消息也应该直接跳过处理
#[tokio::test]
async fn try_parse_voice_message_returns_none_when_transcription_disabled() {
    let ch = TelegramChannel::new("token".into(), vec!["*".into()], false);
    let update = serde_json::json!({
        "message": {
            "message_id": 1,
            "voice": { "file_id": "voice_file", "duration": 4 },
            "from": { "id": 123, "username": "alice" },
            "chat": { "id": 456, "type": "private" }
        }
    });

    let parsed = ch.try_parse_voice_message(&update).await;
    assert!(parsed.is_none());
}

/// 测试语音时长超过限制时跳过处理
///
/// 安全措施：防止处理过长的语音消息，
/// 避免资源消耗过大或转录服务超时
#[tokio::test]
async fn try_parse_voice_message_skips_when_duration_exceeds_limit() {
    let mut tc = crate::app::agent::config::TranscriptionConfig::default();
    tc.enabled = true;
    tc.max_duration_secs = 5;

    let ch = TelegramChannel::new("token".into(), vec!["*".into()], false).with_transcription(tc);
    let update = serde_json::json!({
        "message": {
            "message_id": 2,
            "voice": { "file_id": "voice_file", "duration": 30 },
            "from": { "id": 123, "username": "alice" },
            "chat": { "id": 456, "type": "private" }
        }
    });

    let parsed = ch.try_parse_voice_message(&update).await;
    assert!(parsed.is_none());
}

/// 测试未授权发送者在下载前被拒绝
///
/// 安全措施：在尝试下载语音文件之前，
/// 先验证发送者是否在允许列表中，避免不必要的网络请求
#[tokio::test]
async fn try_parse_voice_message_rejects_unauthorized_sender_before_download() {
    let mut tc = crate::app::agent::config::TranscriptionConfig::default();
    tc.enabled = true;
    tc.max_duration_secs = 120;

    let ch =
        TelegramChannel::new("token".into(), vec!["alice".into()], false).with_transcription(tc);
    let update = serde_json::json!({
        "message": {
            "message_id": 3,
            "voice": { "file_id": "voice_file", "duration": 4 },
            "from": { "id": 999, "username": "bob" },
            "chat": { "id": 456, "type": "private" }
        }
    });

    let parsed = ch.try_parse_voice_message(&update).await;
    assert!(parsed.is_none());
    // 验证转录缓存为空，确认未进行任何下载或转录操作
    assert!(ch.voice_transcriptions.lock().is_empty());
}

/// 端到端测试：语音转录和回复上下文缓存
///
/// 此测试验证完整的语音处理流程：
/// 1. 使用真实转录 API（GROQ）转录音频文件
/// 2. 将转录结果缓存到 TelegramChannel
/// 3. 验证回复上下文能正确提取缓存的转录文本
///
/// 注意：此测试需要设置 GROQ_API_KEY 环境变量，
/// 默认标记为 ignore 以避免在 CI 中失败
#[tokio::test]
#[ignore = "requires GROQ_API_KEY"]
async fn e2e_live_voice_transcription_and_reply_cache() {
    // 检查 API 密钥是否存在，不存在则跳过测试
    if std::env::var("GROQ_API_KEY").is_err() {
        eprintln!("GROQ_API_KEY not set — skipping live voice transcription test");
        return;
    }

    // 读取测试音频文件
    let fixture_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hello.mp3");
    let audio_data = std::fs::read(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {e}", fixture_path.display()));

    // 验证音频文件大小合理（至少 1000 字节），确保文件未损坏
    assert!(
        audio_data.len() > 1000,
        "fixture too small ({} bytes), likely corrupt",
        audio_data.len()
    );

    // 执行语音转录
    let config =
        crate::app::agent::config::TranscriptionConfig { enabled: true, ..Default::default() };
    let transcript: String = crate::app::agent::channels::transcription::transcribe_audio(
        audio_data,
        "hello.mp3",
        &config,
    )
    .await
    .expect("transcribe_audio should succeed with valid GROQ_API_KEY");

    // 验证转录结果包含预期的关键词
    assert!(
        transcript.to_lowercase().contains("hello"),
        "expected transcription to contain 'hello', got: '{transcript}'"
    );

    // 创建 TelegramChannel 并缓存转录结果
    let ch = TelegramChannel::new("test_token".into(), vec!["*".into()], false);
    let chat_id: i64 = 12345;
    let message_id: i64 = 67;
    let cache_key = format!("{chat_id}:{message_id}");
    ch.voice_transcriptions.lock().insert(cache_key, transcript.clone());

    // 构造回复消息的 JSON 结构
    let msg = serde_json::json!({
        "chat": { "id": chat_id },
        "reply_to_message": {
            "message_id": message_id,
            "from": { "username": "vibewindow_user" },
            "voice": { "file_id": "test_file", "duration": 1 }
        }
    });

    // 提取回复上下文
    let ctx = ch
        .extract_reply_context(&msg)
        .expect("extract_reply_context should return Some for voice reply");

    // 验证回复上下文包含缓存的转录文本
    assert!(
        ctx.contains(&format!("[Voice] {transcript}")),
        "expected cached transcription in reply context, got: {ctx}"
    );

    // 验证不使用回退占位符（应使用实际转录结果）
    assert!(
        !ctx.contains("[Voice message]"),
        "context should use cached transcription, not fallback placeholder, got: {ctx}"
    );
}
