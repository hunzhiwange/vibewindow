//! 转录功能测试模块
//!
//! 本模块包含音频转录功能的单元测试和集成测试，主要验证以下方面：
//! - 音频文件大小限制
//! - API 密钥验证
//! - 音频格式 MIME 类型映射
//! - 音频文件名规范化
//! - 不支持的音频格式拒绝
//!
//! # 测试策略
//!
//! 测试分为同步测试和异步测试两类：
//! - 同步测试：主要验证纯函数逻辑，如 MIME 类型映射和文件名规范化
//! - 异步测试：验证与外部服务的交互逻辑，如 API 调用错误处理

use super::*;

/// 测试模块内部定义
#[allow(dead_code)]
mod tests {
    use super::*;

    /// 测试超大音频文件被正确拒绝
    ///
    /// # 测试场景
    ///
    /// 当音频文件大小超过 `MAX_AUDIO_BYTES` 限制时，转录函数应返回错误。
    ///
    /// # 验证点
    ///
    /// - 错误消息中应包含 "too large" 关键字
    #[tokio::test]
    async fn rejects_oversized_audio() {
        // 创建一个超出大小限制的音频数据（MAX_AUDIO_BYTES + 1 字节）
        let big = vec![0u8; MAX_AUDIO_BYTES + 1];
        let config = TranscriptionConfig::default();

        // 调用转录函数，预期返回错误
        let err = transcribe_audio(big, "test.ogg", &config).await.unwrap_err();
        // 验证错误消息包含大小限制相关提示
        assert!(err.to_string().contains("too large"), "expected size error, got: {err}");
    }

    /// 测试缺少 API 密钥时被正确拒绝
    ///
    /// # 测试场景
    ///
    /// 当环境变量中没有 `GROQ_API_KEY` 时，转录函数应返回错误。
    ///
    /// # 安全性说明
    ///
    /// 使用 `unsafe` 块操作环境变量仅用于测试目的，不应在生产代码中使用。
    ///
    /// # 验证点
    ///
    /// - 错误消息中应包含 "GROQ_API_KEY" 关键字
    #[tokio::test]
    async fn rejects_missing_api_key() {
        // 确保测试环境中不存在 GROQ_API_KEY 环境变量
        unsafe { std::env::remove_var("GROQ_API_KEY") };

        // 创建一个小型测试音频数据
        let data = vec![0u8; 100];
        let config = TranscriptionConfig::default();

        // 调用转录函数，预期返回错误
        let err = transcribe_audio(data, "test.ogg", &config).await.unwrap_err();
        // 验证错误消息包含 API 密钥相关提示
        assert!(err.to_string().contains("GROQ_API_KEY"), "expected missing-key error, got: {err}");
    }

    /// 测试音频格式到 MIME 类型的正确映射
    ///
    /// # 测试场景
    ///
    /// 验证 `mime_for_audio` 函数能正确将支持的音频文件扩展名映射到对应的 MIME 类型。
    ///
    /// # 支持的格式
    ///
    /// - FLAC (audio/flac)
    /// - MP3/MPEG/MPGA (audio/mpeg)
    /// - MP4/M4A (audio/mp4)
    /// - OGG/OGA (audio/ogg)
    /// - Opus (audio/opus)
    /// - WAV (audio/wav)
    /// - WebM (audio/webm)
    #[test]
    fn mime_for_audio_maps_accepted_formats() {
        // 定义测试用例：文件扩展名 -> 期望的 MIME 类型
        let cases = [
            ("flac", "audio/flac"),
            ("mp3", "audio/mpeg"),
            ("mpeg", "audio/mpeg"),
            ("mpga", "audio/mpeg"),
            ("mp4", "audio/mp4"),
            ("m4a", "audio/mp4"),
            ("ogg", "audio/ogg"),
            ("oga", "audio/ogg"),
            ("opus", "audio/opus"),
            ("wav", "audio/wav"),
            ("webm", "audio/webm"),
        ];
        // 逐一验证每个扩展名的 MIME 类型映射
        for (ext, expected) in cases {
            assert_eq!(mime_for_audio(ext), Some(expected), "failed for extension: {ext}");
        }
    }

    /// 测试 MIME 类型映射的大小写不敏感性
    ///
    /// # 测试场景
    ///
    /// 验证 `mime_for_audio` 函数对文件扩展名的大小写处理。
    /// 无论扩展名是大写、小写还是混合大小写，都应返回正确的 MIME 类型。
    #[test]
    fn mime_for_audio_case_insensitive() {
        // 测试全大写扩展名
        assert_eq!(mime_for_audio("OGG"), Some("audio/ogg"));
        assert_eq!(mime_for_audio("MP3"), Some("audio/mpeg"));
        // 测试混合大小写扩展名
        assert_eq!(mime_for_audio("Opus"), Some("audio/opus"));
    }

    /// 测试未知音频格式返回 None
    ///
    /// # 测试场景
    ///
    /// 验证 `mime_for_audio` 函数对不支持或不存在的文件扩展名返回 `None`。
    ///
    /// # 测试用例
    ///
    /// - "txt"：文本文件，不是音频格式
    /// - "pdf"：PDF 文件，不是音频格式
    /// - "aac"：虽然 AAC 是音频格式，但不在支持列表中
    /// - ""：空字符串，无扩展名
    #[test]
    fn mime_for_audio_rejects_unknown() {
        assert_eq!(mime_for_audio("txt"), None);
        assert_eq!(mime_for_audio("pdf"), None);
        assert_eq!(mime_for_audio("aac"), None);
        assert_eq!(mime_for_audio(""), None);
    }

    /// 测试 .oga 文件扩展名规范化为 .ogg
    ///
    /// # 测试场景
    ///
    /// 验证 `normalize_audio_filename` 函数将 .oga 扩展名规范化为 .ogg。
    /// .oga 是 Ogg 音频的另一种扩展名，但在某些系统中可能不被识别。
    ///
    /// # 验证点
    ///
    /// - 小写 .oga 应规范化为 .ogg
    /// - 大写 .OGA 应规范化为小写 .ogg
    #[test]
    fn normalize_audio_filename_rewrites_oga() {
        assert_eq!(normalize_audio_filename("voice.oga"), "voice.ogg");
        assert_eq!(normalize_audio_filename("file.OGA"), "file.ogg");
    }

    /// 测试已支持的音频扩展名保持不变
    ///
    /// # 测试场景
    ///
    /// 验证 `normalize_audio_filename` 函数对已支持的音频格式不进行修改。
    ///
    /// # 验证点
    ///
    /// - .ogg、.mp3、.opus 等已支持格式的文件名应原样返回
    #[test]
    fn normalize_audio_filename_preserves_accepted() {
        assert_eq!(normalize_audio_filename("voice.ogg"), "voice.ogg");
        assert_eq!(normalize_audio_filename("track.mp3"), "track.mp3");
        assert_eq!(normalize_audio_filename("clip.opus"), "clip.opus");
    }

    /// 测试无扩展名的文件名保持不变
    ///
    /// # 测试场景
    ///
    /// 验证 `normalize_audio_filename` 函数对没有扩展名的文件名的处理。
    /// 无扩展名的文件名应原样返回，不进行任何修改。
    #[test]
    fn normalize_audio_filename_no_extension() {
        assert_eq!(normalize_audio_filename("voice"), "voice");
    }

    /// 测试不支持的音频格式被正确拒绝
    ///
    /// # 测试场景
    ///
    /// 当音频文件使用不支持的格式（如 AAC）时，转录函数应返回明确的错误信息。
    ///
    /// # 验证点
    ///
    /// - 错误消息中应包含 "Unsupported audio format" 关键字
    /// - 错误消息中应包含被拒绝的文件扩展名（如 ".aac"）
    #[tokio::test]
    async fn rejects_unsupported_audio_format() {
        // 创建一个小型测试音频数据
        let data = vec![0u8; 100];
        let config = TranscriptionConfig::default();

        // 使用不支持的 AAC 格式调用转录函数，预期返回错误
        let err = transcribe_audio(data, "recording.aac", &config).await.unwrap_err();
        let msg = err.to_string();
        // 验证错误消息包含不支持的格式提示
        assert!(
            msg.contains("Unsupported audio format"),
            "expected unsupported-format error, got: {msg}"
        );
        // 验证错误消息包含被拒绝的扩展名
        assert!(msg.contains(".aac"), "error should mention the rejected extension, got: {msg}");
    }
}
