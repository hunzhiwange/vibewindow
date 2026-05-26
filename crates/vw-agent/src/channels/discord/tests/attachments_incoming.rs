use super::*;

use axum::{Json, Router, routing::get, routing::post};
use serde_json::json as json_value;

/// 测试空附件列表返回空字符串
/// 没有附件时应该返回空字符串
#[tokio::test]
async fn process_attachments_empty_list_returns_empty() {
    let client = reqwest::Client::new();
    let result = attachments::process_attachments(&[], &client, None).await;
    assert!(result.is_empty());
}

/// 测试跳过不支持的附件类型
/// 非图片/音频类型应该被跳过
#[tokio::test]
async fn process_attachments_skips_unsupported_types() {
    let client = reqwest::Client::new();
    let attachments = vec![serde_json::json!({
        "url": "https://cdn.discordapp.com/attachments/123/456/doc.pdf",
        "filename": "doc.pdf",
        "content_type": "application/pdf"
    })];
    let result = attachments::process_attachments(&attachments, &client, None).await;
    assert!(result.is_empty());
}

/// 测试图片内容类型产生图片标记
/// 图片附件应该生成 [IMAGE:url] 标记
#[tokio::test]
async fn process_attachments_emits_image_marker_for_image_content_type() {
    let client = reqwest::Client::new();
    let attachments = vec![serde_json::json!({
        "url": "https://cdn.discordapp.com/attachments/123/456/photo.png",
        "filename": "photo.png",
        "content_type": "image/png"
    })];
    let result = attachments::process_attachments(&attachments, &client, None).await;
    assert_eq!(result, "[IMAGE:https://cdn.discordapp.com/attachments/123/456/photo.png]");
}

/// 测试多个图片标记的生成
/// 多个图片应该用分隔符连接
#[tokio::test]
async fn process_attachments_emits_multiple_image_markers() {
    let client = reqwest::Client::new();
    let attachments = vec![
        serde_json::json!({
            "url": "https://cdn.discordapp.com/attachments/123/456/one.jpg",
            "filename": "one.jpg",
            "content_type": "image/jpeg"
        }),
        serde_json::json!({
            "url": "https://cdn.discordapp.com/attachments/123/456/two.webp",
            "filename": "two.webp",
            "content_type": "image/webp"
        }),
    ];
    let result = attachments::process_attachments(&attachments, &client, None).await;
    assert_eq!(
        result,
        "[IMAGE:https://cdn.discordapp.com/attachments/123/456/one.jpg]\n---\n[IMAGE:https://cdn.discordapp.com/attachments/123/456/two.webp]"
    );
}

/// 测试没有内容类型时从文件名推断图片
/// 当缺少 content_type 时，应该根据文件扩展名判断
#[tokio::test]
async fn process_attachments_emits_image_marker_from_filename_without_content_type() {
    let client = reqwest::Client::new();
    let attachments = vec![serde_json::json!({
        "url": "https://cdn.discordapp.com/attachments/123/456/photo.jpeg?size=1024",
        "filename": "photo.jpeg"
    })];
    let result = attachments::process_attachments(&attachments, &client, None).await;
    assert_eq!(
        result,
        "[IMAGE:https://cdn.discordapp.com/attachments/123/456/photo.jpeg?size=1024]"
    );
}

/// 测试音频附件在启用时进行转录
/// 启用转录功能时，音频应该被转录为文本
#[tokio::test]
#[ignore = "需要本地回环 TCP 绑定"]
async fn process_attachments_transcribes_audio_when_enabled() {
    async fn audio_handler() -> ([(String, String); 1], Vec<u8>) {
        (
            [("content-type".to_string(), "audio/ogg; codecs=opus".to_string())],
            vec![1_u8, 2, 3, 4, 5, 6],
        )
    }

    async fn transcribe_handler() -> Json<serde_json::Value> {
        Json(json_value!({ "text": "hello from discord audio" }))
    }

    let app = Router::new()
        .route("/audio.ogg", get(audio_handler))
        .route("/transcribe", post(transcribe_handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("绑定测试服务器");
    let addr = listener.local_addr().expect("本地地址");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let mut transcription = TranscriptionConfig::default();
    transcription.enabled = true;
    transcription.api_url = format!("http://{addr}/transcribe");
    transcription.model = "whisper-test".to_string();

    let client = reqwest::Client::new();
    let attachments = vec![serde_json::json!({
        "url": format!("http://{addr}/audio.ogg"),
        "filename": "voice.ogg",
        "content_type": "audio/ogg",
        "duration_secs": 4
    })];

    let result = attachments::process_attachments(&attachments, &client, Some(&transcription)).await;
    assert_eq!(result, "[Voice:voice.ogg] hello from discord audio");
}

/// 测试音频时长超过限制时跳过转录
/// 超过最大时长的音频应该被跳过
#[tokio::test]
async fn process_attachments_skips_audio_when_duration_exceeds_limit() {
    let mut transcription = TranscriptionConfig::default();
    transcription.enabled = true;
    transcription.api_url = "http://127.0.0.1:1/transcribe".to_string();
    transcription.max_duration_secs = 5;

    let client = reqwest::Client::new();
    let attachments = vec![serde_json::json!({
        "url": "http://127.0.0.1:1/audio.ogg",
        "filename": "voice.ogg",
        "content_type": "audio/ogg",
        "duration_secs": 120
    })];

    let result = attachments::process_attachments(&attachments, &client, Some(&transcription)).await;
    assert!(result.is_empty());
}

/// 测试图片附件检测优先使用内容类型而非扩展名
/// content_type 非 image 时，即使扩展名是图片也不应识别为图片
#[test]
fn is_image_attachment_prefers_non_image_content_type_over_extension() {
    assert!(!attachments::is_image_attachment(
        "text/plain",
        "photo.png",
        "https://cdn.discordapp.com/attachments/123/456/photo.png"
    ));
}

/// 测试图片附件允许 octet-stream 时使用扩展名回退
/// content_type 为 application/octet-stream 时，应该使用扩展名判断
#[test]
fn is_image_attachment_allows_octet_stream_extension_fallback() {
    assert!(attachments::is_image_attachment(
        "application/octet-stream",
        "photo.png",
        "https://cdn.discordapp.com/attachments/123/456/photo.png"
    ));
}

/// 测试音频附件检测优先使用内容类型而非扩展名
/// content_type 非 audio 时，即使扩展名是音频也不应识别为音频
#[test]
fn is_audio_attachment_prefers_non_audio_content_type_over_extension() {
    assert!(!attachments::is_audio_attachment(
        "text/plain",
        "voice.ogg",
        "https://cdn.discordapp.com/attachments/123/456/voice.ogg"
    ));
}

/// 测试音频附件允许 octet-stream 时使用扩展名回退
/// content_type 为 application/octet-stream 时，应该使用扩展名判断
#[test]
fn is_audio_attachment_allows_octet_stream_extension_fallback() {
    assert!(attachments::is_audio_attachment(
        "application/octet-stream",
        "voice.ogg",
        "https://cdn.discordapp.com/attachments/123/456/voice.ogg"
    ));
}

/// 测试当文件名缺少扩展名时使用内容类型推断音频文件名
/// 文件名无扩展名时，应该从 content_type 推断
#[test]
fn infer_audio_filename_uses_content_type_when_name_lacks_extension() {
    let file_name = attachments::infer_audio_filename(
        "voice_upload",
        "https://cdn.discordapp.com/attachments/123/456/blob",
        "audio/ogg; codecs=opus",
    );
    assert_eq!(file_name, "audio.ogg");
}