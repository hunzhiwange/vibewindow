//! Discord 消息附件处理模块
//!
//! 本模块负责处理 Discord 消息中的各类附件，根据附件类型执行不同的处理逻辑：
//!
//! - **图片附件**：生成 `[IMAGE:<url>]` 标记，供代理识别图片内容
//! - **音频附件**：当启用转录功能时，下载音频并转录为文本
//! - **文本附件**：直接获取文本内容并内联到消息中
//! - **其他类型**：跳过处理并记录调试日志
//!
//! # 主要功能
//!
//! - 附件类型识别（基于 MIME 类型和文件扩展名）
//! - 音频时长解析与限制检查
//! - 音频文件名推断
//! - 支持的媒体格式验证
//!
//! # 安全考虑
//!
//! - 音频转录有最大时长限制，防止资源过度消耗
//! - 网络请求失败时优雅降级，不影响主流程

use crate::app::agent::config::TranscriptionConfig;
use std::path::Path;

/// 处理 Discord 消息附件，返回要追加到代理消息上下文的字符串
///
/// 根据附件的 MIME 类型执行不同的处理策略：
///
/// - `image/*` 附件：转换为 `[IMAGE:<url>]` 标记
/// - `application/octet-stream` 或缺失 MIME 类型：通过文件名/URL 扩展名判断是否为图片
/// - `audio/*` 附件：当配置了转录时进行音频转文字
/// - `text/*` 附件：获取内容并内联
/// - 其他类型：跳过处理
///
/// # 参数
///
/// * `attachments` - Discord 附件 JSON 数组，每个附件包含 `content_type`、`filename`、`url` 等字段
/// * `client` - HTTP 客户端，用于获取附件内容
/// * `transcription` - 转录配置，`None` 表示禁用音频转录功能
///
/// # 返回值
///
/// 返回处理后的字符串，各部分用 `\n---\n` 分隔。若无有效附件则返回空字符串。
///
/// # 示例
///
/// ```ignore
/// let attachments = vec![
///     json!({"content_type": "image/png", "filename": "photo.png", "url": "https://cdn.discord.app/..."}),
/// ];
/// let result = process_attachments(&attachments, &client, None).await;
/// // result: "[IMAGE:https://cdn.discord.app/...]"
/// ```
///
/// # 错误处理
///
/// - 获取附件失败时记录警告日志并跳过该附件
/// - 音频转录失败时记录警告日志并跳过
/// - 不会因单个附件处理失败而中断整体流程
pub(super) async fn process_attachments(
    attachments: &[serde_json::Value],
    client: &reqwest::Client,
    transcription: Option<&TranscriptionConfig>,
) -> String {
    // 收集所有处理后的附件内容片段
    let mut parts: Vec<String> = Vec::new();

    for att in attachments {
        // 提取附件的基本信息：MIME 类型、文件名、URL
        let ct = att.get("content_type").and_then(|v| v.as_str()).unwrap_or("");
        let name = att.get("filename").and_then(|v| v.as_str()).unwrap_or("file");

        // URL 是必需的，无 URL 则跳过
        let Some(url) = att.get("url").and_then(|v| v.as_str()) else {
            tracing::warn!(name, "discord: attachment has no url, skipping");
            continue;
        };

        // 根据附件类型分发处理逻辑
        if is_image_attachment(ct, name, url) {
            // 图片类型：生成标记供代理识别
            parts.push(format!("[IMAGE:{url}]"));
        } else if is_audio_attachment(ct, name, url) {
            // 音频类型：需要转录配置
            let Some(config) = transcription else {
                tracing::debug!(
                    name,
                    content_type = ct,
                    "discord: skipping audio attachment because transcription is disabled"
                );
                continue;
            };

            // 检查音频时长是否超过配置的限制
            if let Some(duration_secs) = parse_attachment_duration_secs(att) {
                if duration_secs > config.max_duration_secs {
                    tracing::warn!(
                        name,
                        duration_secs,
                        max_duration_secs = config.max_duration_secs,
                        "discord: skipping audio attachment that exceeds transcription duration limit"
                    );
                    continue;
                }
            }

            // 下载音频数据
            let audio_data = match client.get(url).send().await {
                // 请求成功且响应状态码为 2xx
                Ok(resp) if resp.status().is_success() => match resp.bytes().await {
                    Ok(bytes) => bytes.to_vec(),
                    Err(error) => {
                        tracing::warn!(
                            name,
                            error = %error,
                            "discord: failed to read audio attachment body"
                        );
                        continue;
                    }
                },
                // 请求成功但状态码非 2xx
                Ok(resp) => {
                    tracing::warn!(
                        name,
                        status = %resp.status(),
                        "discord audio attachment fetch failed"
                    );
                    continue;
                }
                // 网络请求失败
                Err(error) => {
                    tracing::warn!(name, error = %error, "discord audio attachment fetch error");
                    continue;
                }
            };

            // 推断合适的音频文件名
            let file_name = infer_audio_filename(name, url, ct);

            // 执行音频转录
            match crate::app::agent::channels::transcription::transcribe_audio(
                audio_data, &file_name, config,
            )
            .await
            {
                Ok(transcript) => {
                    let transcript = transcript.trim();
                    if transcript.is_empty() {
                        tracing::info!(name, "discord: transcription returned empty text");
                    } else {
                        // 转录成功，添加到结果中
                        parts.push(format!("[Voice:{file_name}] {transcript}"));
                    }
                }
                Err(error) => {
                    tracing::warn!(name, error = %error, "discord: audio transcription failed");
                }
            }
        } else if ct.starts_with("text/") {
            // 文本类型：直接获取内容并内联
            match client.get(url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(text) = resp.text().await {
                        parts.push(format!("[{name}]\n{text}"));
                    }
                }
                Ok(resp) => {
                    tracing::warn!(name, status = %resp.status(), "discord attachment fetch failed");
                }
                Err(e) => {
                    tracing::warn!(name, error = %e, "discord attachment fetch error");
                }
            }
        } else {
            // 不支持的附件类型，记录调试日志
            tracing::debug!(
                name,
                content_type = ct,
                "discord: skipping unsupported attachment type"
            );
        }
    }

    // 用分隔符连接所有片段
    parts.join("\n---\n")
}

/// 规范化 MIME 类型字符串
///
/// 移除 MIME 类型中的参数部分（如 `; charset=utf-8`）并转换为小写。
///
/// # 参数
///
/// * `content_type` - 原始 Content-Type 字符串
///
/// # 返回值
///
/// 规范化后的 MIME 类型字符串（小写，无参数）
///
/// # 示例
///
/// ```
/// # use crate::app::agent::channels::discord::attachments::normalize_content_type;
/// assert_eq!(normalize_content_type("Image/PNG"), "image/png");
/// assert_eq!(normalize_content_type("audio/mpeg; charset=utf-8"), "audio/mpeg");
/// assert_eq!(normalize_content_type(""), "");
/// ```
pub(super) fn normalize_content_type(content_type: &str) -> String {
    // MIME 类型格式为 "type/subtype; params"，取分号前的部分
    content_type.split(';').next().unwrap_or("").trim().to_ascii_lowercase()
}

/// 判断附件是否为图片类型
///
/// 判断逻辑：
/// 1. 如果 MIME 类型以 `image/` 开头，返回 `true`
/// 2. 如果 MIME 类型非空且非 `application/octet-stream`，信任 MIME 类型并返回 `false`
/// 3. 否则，通过文件名或 URL 的扩展名判断
///
/// # 参数
///
/// * `content_type` - MIME 类型字符串
/// * `filename` - 附件文件名
/// * `url` - 附件 URL
///
/// # 返回值
///
/// 如果是图片类型返回 `true`，否则返回 `false`
///
/// # 设计说明
///
/// 对于 `application/octet-stream` 或缺失 MIME 类型的情况，回退到扩展名判断。
/// 这是为了兼容某些服务器不正确设置 MIME 类型的场景。
pub(super) fn is_image_attachment(content_type: &str, filename: &str, url: &str) -> bool {
    let normalized_content_type = normalize_content_type(content_type);

    // MIME 类型非空时的处理
    if !normalized_content_type.is_empty() {
        // 明确的图片类型
        if normalized_content_type.starts_with("image/") {
            return true;
        }
        // 非 octet-stream 的明确非图片类型，信任 MIME 类型避免误判
        if normalized_content_type != "application/octet-stream" {
            return false;
        }
    }

    // MIME 类型为空或为 octet-stream 时，通过扩展名判断
    has_image_extension(filename) || has_image_extension(url)
}

/// 判断附件是否为音频类型
///
/// 判断逻辑：
/// 1. 如果 MIME 类型以 `audio/` 开头，返回 `true`
/// 2. 如果 MIME 类型非空且非 `application/octet-stream`，信任 MIME 类型并返回 `false`
/// 3. 否则，通过文件名或 URL 的扩展名判断
///
/// # 参数
///
/// * `content_type` - MIME 类型字符串
/// * `filename` - 附件文件名
/// * `url` - 附件 URL
///
/// # 返回值
///
/// 如果是音频类型返回 `true`，否则返回 `false`
///
/// # 设计说明
///
/// 对于 `application/octet-stream` 或缺失 MIME 类型的情况，回退到扩展名判断。
/// 这是为了兼容某些服务器不正确设置 MIME 类型的场景。
pub(super) fn is_audio_attachment(content_type: &str, filename: &str, url: &str) -> bool {
    let normalized_content_type = normalize_content_type(content_type);

    // MIME 类型非空时的处理
    if !normalized_content_type.is_empty() {
        // 明确的音频类型
        if normalized_content_type.starts_with("audio/") {
            return true;
        }
        // 非 octet-stream 的明确非音频类型，信任 MIME 类型避免误判
        if normalized_content_type != "application/octet-stream" {
            return false;
        }
    }

    // MIME 类型为空或为 octet-stream 时，通过扩展名判断
    has_audio_extension(filename) || has_audio_extension(url)
}

/// 从附件 JSON 中解析音频时长（秒）
///
/// 支持整数和浮点数格式的时长值。浮点数会向上取整。
///
/// # 参数
///
/// * `attachment` - Discord 附件 JSON 对象
///
/// # 返回值
///
/// 成功返回 `Some(秒数)`，失败返回 `None`
///
/// # 无效值处理
///
/// 以下情况返回 `None`：
/// - 附件中没有 `duration_secs` 字段
/// - 值为非数字类型
/// - 浮点数为无穷大或 NaN
/// - 浮点数为负数
/// - 数值超过 `u64::MAX`
pub(super) fn parse_attachment_duration_secs(attachment: &serde_json::Value) -> Option<u64> {
    let value = attachment.get("duration_secs")?;

    // 尝试作为整数解析
    if let Some(seconds) = value.as_u64() {
        return Some(seconds);
    }

    // 尝试作为浮点数解析
    let raw = value.as_f64()?;

    // 检查浮点数有效性
    if !raw.is_finite() || raw.is_sign_negative() {
        return None;
    }

    // 向上取整并检查是否溢出
    let rounded = raw.ceil();
    if rounded > u64::MAX as f64 {
        return None;
    }

    // 安全转换：通过格式化字符串避免浮点精度问题
    format!("{rounded:.0}").parse().ok()
}

/// 从媒体路径中提取文件扩展名
///
/// 处理 URL 中的查询参数和片段标识符，只取路径部分的扩展名。
///
/// # 参数
///
/// * `value` - 文件路径或 URL
///
/// # 返回值
///
/// 成功返回 `Some(小写扩展名)`，无扩展名返回 `None`
///
/// # 示例
///
/// ```ignore
/// assert_eq!(extension_from_media_path("audio.mp3"), Some("mp3".to_string()));
/// assert_eq!(extension_from_media_path("https://example.com/audio.ogg?v=1"), Some("ogg".to_string()));
/// assert_eq!(extension_from_media_path("noextension"), None);
/// ```
pub(super) fn extension_from_media_path(value: &str) -> Option<String> {
    // 移除 URL 查询参数（? 后的内容）
    let base = value.split('?').next().unwrap_or(value);
    // 移除片段标识符（# 后的内容）
    let base = base.split('#').next().unwrap_or(base);
    // 使用标准库提取扩展名并转为小写
    Path::new(base).extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase())
}

/// 检查扩展名是否为支持的音频格式
///
/// 支持的格式与 OpenAI Whisper API 兼容。
///
/// # 参数
///
/// * `extension` - 文件扩展名（小写）
///
/// # 返回值
///
/// 支持的格式返回 `true`，否则返回 `false`
///
/// # 支持的格式
///
/// - `flac` - FLAC 无损音频
/// - `mp3` / `mpeg` / `mpga` - MPEG 音频
/// - `mp4` / `m4a` - MPEG-4 音频
/// - `ogg` / `oga` - Ogg Vorbis/Opus
/// - `opus` - Opus 编码
/// - `wav` - WAV 波形音频
/// - `webm` - WebM 音频
pub(super) fn is_supported_audio_extension(extension: &str) -> bool {
    matches!(
        extension,
        "flac" | "mp3" | "mpeg" | "mpga" | "mp4" | "m4a" | "ogg" | "oga" | "opus" | "wav" | "webm"
    )
}

/// 检查路径或 URL 是否具有音频扩展名
///
/// # 参数
///
/// * `value` - 文件路径或 URL
///
/// # 返回值
///
/// 具有支持的音频扩展名返回 `true`，否则返回 `false`
pub(super) fn has_audio_extension(value: &str) -> bool {
    matches!(
        extension_from_media_path(value).as_deref(),
        Some(ext) if is_supported_audio_extension(ext)
    )
}

/// 从 MIME 类型推断音频文件扩展名
///
/// # 参数
///
/// * `content_type` - MIME 类型字符串
///
/// # 返回值
///
/// 成功返回 `Some(扩展名)`，不支持的格式返回 `None`
///
/// # 支持的 MIME 类型映射
///
/// | MIME 类型 | 扩展名 |
/// |-----------|--------|
/// | audio/flac, audio/x-flac | flac |
/// | audio/mpeg | mp3 |
/// | audio/mpga | mpga |
/// | audio/mp4, audio/x-m4a, audio/m4a | m4a |
/// | audio/ogg, application/ogg | ogg |
/// | audio/opus | opus |
/// | audio/wav, audio/x-wav, audio/wave | wav |
/// | audio/webm | webm |
pub(super) fn audio_extension_from_content_type(content_type: &str) -> Option<&'static str> {
    match normalize_content_type(content_type).as_str() {
        "audio/flac" | "audio/x-flac" => Some("flac"),
        "audio/mpeg" => Some("mp3"),
        "audio/mpga" => Some("mpga"),
        "audio/mp4" | "audio/x-m4a" | "audio/m4a" => Some("m4a"),
        "audio/ogg" | "application/ogg" => Some("ogg"),
        "audio/opus" => Some("opus"),
        "audio/wav" | "audio/x-wav" | "audio/wave" => Some("wav"),
        "audio/webm" => Some("webm"),
        _ => None,
    }
}

/// 推断音频文件的完整文件名
///
/// 按优先级依次尝试：
/// 1. 使用原始文件名（如果有有效的音频扩展名）
/// 2. 从 URL 提取扩展名，生成 `audio.<ext>`
/// 3. 从 MIME 类型推断扩展名，生成 `audio.<ext>`
/// 4. 默认返回 `audio.ogg`
///
/// # 参数
///
/// * `filename` - 原始文件名
/// * `url` - 附件 URL
/// * `content_type` - MIME 类型
///
/// # 返回值
///
/// 推断的文件名字符串
///
/// # 设计说明
///
/// 确保返回的文件名具有有效的音频扩展名，这是转录服务所需要的。
/// `audio.ogg` 作为默认值，因为 Ogg 是广泛支持的容器格式。
pub(super) fn infer_audio_filename(filename: &str, url: &str, content_type: &str) -> String {
    let trimmed_name = filename.trim();
    // 优先使用原始文件名（如果有有效扩展名）
    if !trimmed_name.is_empty() && has_audio_extension(trimmed_name) {
        return trimmed_name.to_string();
    }

    // 尝试从 URL 提取扩展名
    if let Some(ext) =
        extension_from_media_path(url).filter(|ext| is_supported_audio_extension(ext))
    {
        return format!("audio.{ext}");
    }

    // 尝试从 MIME 类型推断扩展名
    if let Some(ext) = audio_extension_from_content_type(content_type) {
        return format!("audio.{ext}");
    }

    // 无法推断时使用默认值
    "audio.ogg".to_string()
}

/// 检查路径或 URL 是否具有图片扩展名
///
/// # 参数
///
/// * `value` - 文件路径或 URL
///
/// # 返回值
///
/// 具有支持的图片扩展名返回 `true`，否则返回 `false`
///
/// # 支持的图片格式
///
/// - `png` - PNG 格式
/// - `jpg` / `jpeg` - JPEG 格式
/// - `gif` - GIF 格式
/// - `webp` - WebP 格式
/// - `bmp` - BMP 格式
/// - `tif` / `tiff` - TIFF 格式
/// - `svg` - SVG 矢量图
/// - `avif` - AVIF 格式
/// - `heic` / `heif` - HEIF 格式（Apple 设备常用）
pub(super) fn has_image_extension(value: &str) -> bool {
    matches!(
        extension_from_media_path(value).as_deref(),
        Some(
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "webp"
                | "bmp"
                | "tif"
                | "tiff"
                | "svg"
                | "avif"
                | "heic"
                | "heif"
        )
    )
}
