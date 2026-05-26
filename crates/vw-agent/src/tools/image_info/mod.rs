//! 图像信息工具
//!
//! 本模块提供图像元数据提取功能，用于读取图像文件的基本信息。
//!
//! # 主要功能
//!
//! - 读取图像文件的元数据（文件大小、格式、尺寸）
//! - 通过魔数检测图像格式（PNG、JPEG、GIF、WebP、BMP）
//! - 从图像头部字节提取尺寸信息
//! - 可选返回 Base64 编码的图像数据
//!
//! # 设计目的
//!
//! 由于当前 AI 提供方主要支持文本模式，此工具先提取可用信息，
//! 并提供 Base64 数据为未来的多模态提供方支持做准备。
//!
//! # 安全考虑
//!
//! - 所有路径访问都经过安全策略验证
//! - 文件大小限制为 5MB，防止内存溢出
//! - 仅允许访问工作区内的文件

use super::traits::{Tool, ToolResult};
use crate::app::agent::security::SecurityPolicy;
use async_trait::async_trait;
use serde_json::json;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 最大图像文件大小限制（5 MB）
///
/// 超过此大小的图像文件将被拒绝读取和 Base64 编码，
/// 以防止内存消耗过大和潜在的拒绝服务攻击。
const MAX_IMAGE_BYTES: u64 = 5_242_880;

/// 图像信息工具
///
/// 用于读取图像文件元数据并可选返回 Base64 编码数据的工具。
///
/// # 功能说明
///
/// 该工具从图像文件中提取以下信息：
/// - 文件大小（字节）
/// - 图像格式（通过魔数检测）
/// - 图像尺寸（从头部字节解析）
/// - 可选的 Base64 编码数据（用于多模态提供方）
///
/// # 支持的格式
///
/// - PNG（可提取尺寸）
/// - JPEG（可提取尺寸）
/// - GIF（可提取尺寸）
/// - WebP（格式检测）
/// - BMP（可提取尺寸）
///
/// # 示例
///
/// ```ignore
/// use std::sync::Arc;
/// use vibe_agent::app::agent::security::SecurityPolicy;
/// use vibe_agent::app::agent::tools::ImageInfoTool;
/// use vibe_agent::app::agent::tools::traits::Tool;
///
/// let security = Arc::new(SecurityPolicy::default());
/// let tool = ImageInfoTool::new(security);
///
/// // 获取工具名称
/// assert_eq!(tool.name(), "image_info");
/// ```
pub struct ImageInfoTool {
    /// 安全策略引用，用于路径访问验证
    security: Arc<SecurityPolicy>,
}

impl ImageInfoTool {
    /// 创建新的图像信息工具实例
    ///
    /// # 参数
    ///
    /// - `security`: 安全策略的 Arc 引用，用于验证文件路径访问权限
    ///
    /// # 返回值
    ///
    /// 返回配置好安全策略的 `ImageInfoTool` 实例
    ///
    /// # 示例
    ///
    /// ```ignore
    /// use std::sync::Arc;
    /// use vibe_agent::app::agent::security::SecurityPolicy;
    /// use vibe_agent::app::agent::tools::ImageInfoTool;
    ///
    /// let security = Arc::new(SecurityPolicy::default());
    /// let tool = ImageInfoTool::new(security);
    /// ```
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self { security }
    }

    /// 从文件头部字节检测图像格式（魔数检测）
    ///
    /// 通过检查文件的前几个字节（魔数）来确定图像格式。
    /// 这是一种快速且可靠的格式检测方法，不依赖文件扩展名。
    ///
    /// # 参数
    ///
    /// - `bytes`: 图像文件的字节数据（至少需要 4 字节）
    ///
    /// # 返回值
    ///
    /// 返回图像格式的静态字符串标识：
    /// - `"png"` - PNG 图像（魔数：`\x89PNG`）
    /// - `"jpeg"` - JPEG 图像（魔数：`\xFF\xD8\xFF`）
    /// - `"gif"` - GIF 图像（魔数：`GIF8`）
    /// - `"webp"` - WebP 图像（魔数：`RIFF...WEBP`）
    /// - `"bmp"` - BMP 图像（魔数：`BM`）
    /// - `"unknown"` - 无法识别的格式或数据不足
    ///
    /// # 检测逻辑
    ///
    /// ```text
    /// PNG:  89 50 4E 47 (魔数 + ASCII "PNG")
    /// JPEG: FF D8 FF
    /// GIF:  47 49 46 38 (ASCII "GIF8")
    /// WebP: 52 49 46 46 xx xx xx xx 57 45 42 50 (RIFF...WEBP)
    /// BMP:  42 4D (ASCII "BM")
    /// ```
    fn detect_format(bytes: &[u8]) -> &'static str {
        if bytes.len() < 4 {
            return "unknown";
        }
        if bytes.starts_with(b"\x89PNG") {
            "png"
        } else if bytes.starts_with(b"\xFF\xD8\xFF") {
            "jpeg"
        } else if bytes.starts_with(b"GIF8") {
            "gif"
        } else if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WEBP" {
            "webp"
        } else if bytes.starts_with(b"BM") {
            "bmp"
        } else {
            "unknown"
        }
    }

    /// 尝试从图像头部字节提取尺寸信息
    ///
    /// 根据图像格式解析相应的头部结构，提取宽度和高度信息。
    /// 对于无法直接从头部提取尺寸的格式（如 WebP），返回 `None`。
    ///
    /// # 参数
    ///
    /// - `bytes`: 图像文件的字节数据
    /// - `format`: 图像格式（由 `detect_format` 检测得到）
    ///
    /// # 返回值
    ///
    /// - `Some((width, height))` - 成功提取的尺寸信息（宽度和高度均为像素值）
    /// - `None` - 无法提取尺寸（格式不支持或数据不足）
    ///
    /// # 支持的格式
    ///
    /// - **PNG**: 从 IHDR 块读取（字节 16-23，大端序）
    /// - **GIF**: 从逻辑屏幕描述符读取（字节 6-9，小端序）
    /// - **BMP**: 从 DIB 头读取（字节 18-25，小端序，高度可能有符号）
    /// - **JPEG**: 通过解析 SOF 标记读取（见 `jpeg_dimensions` 方法）
    fn extract_dimensions(bytes: &[u8], format: &str) -> Option<(u32, u32)> {
        match format {
            "png" => {
                if bytes.len() >= 24 {
                    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
                    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
                    Some((w, h))
                } else {
                    None
                }
            }
            "gif" => {
                if bytes.len() >= 10 {
                    let w = u32::from(u16::from_le_bytes([bytes[6], bytes[7]]));
                    let h = u32::from(u16::from_le_bytes([bytes[8], bytes[9]]));
                    Some((w, h))
                } else {
                    None
                }
            }
            "bmp" => {
                if bytes.len() >= 26 {
                    let w = u32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]);
                    let h_raw = i32::from_le_bytes([bytes[22], bytes[23], bytes[24], bytes[25]]);
                    let h = h_raw.unsigned_abs();
                    Some((w, h))
                } else {
                    None
                }
            }
            "jpeg" => Self::jpeg_dimensions(bytes),
            _ => None,
        }
    }

    /// 解析 JPEG SOF 标记以提取尺寸信息
    ///
    /// JPEG 文件格式使用 SOF（Start of Frame）标记来存储图像尺寸信息。
    /// 此方法遍历 JPEG 标记段，查找 SOF0-SOF3 标记并提取尺寸。
    ///
    /// # 参数
    ///
    /// - `bytes`: JPEG 文件的字节数据
    ///
    /// # 返回值
    ///
    /// - `Some((width, height))` - 成功找到 SOF 标记并提取的尺寸
    /// - `None` - 未找到有效的 SOF 标记或数据格式错误
    ///
    /// # JPEG 结构说明
    ///
    /// ```text
    /// 标记格式: 0xFF + 标记类型
    /// SOF 标记: 0xFFC0 - 0xFFC3 (帧开始标记)
    /// SOF 段结构:
    ///   - 2 字节: 段长度
    ///   - 1 字节: 精度
    ///   - 2 字节: 高度（大端序）
    ///   - 2 字节: 宽度（大端序）
    ///   - 1 字节: 组件数量
    /// ```
    fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
        let mut i = 2;
        while i + 1 < bytes.len() {
            if bytes[i] != 0xFF {
                return None;
            }
            let marker = bytes[i + 1];
            i += 2;

            if (0xC0..=0xC3).contains(&marker) {
                if i + 7 <= bytes.len() {
                    let h = u32::from(u16::from_be_bytes([bytes[i + 3], bytes[i + 4]]));
                    let w = u32::from(u16::from_be_bytes([bytes[i + 5], bytes[i + 6]]));
                    return Some((w, h));
                }
                return None;
            }

            if i + 1 < bytes.len() {
                let seg_len = u16::from_be_bytes([bytes[i], bytes[i + 1]]) as usize;
                if seg_len < 2 {
                    return None;
                }
                i += seg_len;
            } else {
                return None;
            }
        }
        None
    }

    /// 解析并验证图像文件路径
    ///
    /// 将用户提供的路径字符串解析为绝对路径，并验证其符合安全策略。
    /// 支持绝对路径和相对于工作区的相对路径。
    ///
    /// # 参数
    ///
    /// - `path_str`: 用户提供的路径字符串（绝对路径或相对路径）
    ///
    /// # 返回值
    ///
    /// - `Ok(PathBuf)`: 解析后的规范化绝对路径
    /// - `Err(String)`: 路径验证失败的错误信息
    ///
    /// # 安全验证步骤
    ///
    /// 1. 语法级别检查：验证路径是否符合安全策略的允许规则
    /// 2. 路径规范化：将相对路径转换为基于工作区的绝对路径
    /// 3. 解析绝对路径：调用 `canonicalize` 获取真实的文件系统路径
    /// 4. 解析后验证：确保规范化后的路径仍在允许范围内
    ///
    /// # 错误情况
    ///
    /// - 路径不在允许的工作区范围内
    /// - 文件不存在（canonicalize 失败）
    /// - 路径解析后违反安全策略
    fn resolve_image_path(&self, path_str: &str) -> Result<PathBuf, String> {
        if !self.security.is_path_allowed(path_str) {
            return Err(format!("Path not allowed: {path_str} (must be within workspace)"));
        }

        let raw_path = Path::new(path_str);
        let candidate = if raw_path.is_absolute() {
            raw_path.to_path_buf()
        } else {
            self.security.workspace_dir.join(raw_path)
        };

        let resolved =
            candidate.canonicalize().map_err(|_| format!("File not found: {path_str}"))?;

        if !self.security.is_resolved_path_allowed(&resolved) {
            return Err(self.security.resolved_path_violation_message(&resolved));
        }

        Ok(resolved)
    }
}

/// Tool trait 实现
///
/// 为 ImageInfoTool 实现 Tool trait，使其可作为代理工具使用。
/// 该实现根据目标平台提供不同的执行能力：
/// - 非 WASM 平台：完整的图像信息提取功能
/// - WASM 平台：返回不支持的错误信息
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for ImageInfoTool {
    /// 返回工具名称
    ///
    /// # 返回值
    ///
    /// 返回固定字符串 `"image_info"`，用于工具注册和调用识别
    fn name(&self) -> &str {
        "image_info"
    }

    /// 返回工具描述
    ///
    /// # 返回值
    ///
    /// 返回中文描述，说明工具的功能和用途
    fn description(&self) -> &str {
        "Read image file metadata: format, dimensions, size, and optional Base64 data."
    }

    /// 返回工具参数的 JSON Schema
    ///
    /// 定义工具接受的参数结构，用于参数验证和自动生成调用文档。
    ///
    /// # 参数说明
    ///
    /// - `path` (必需): 图像文件路径，支持绝对路径或相对于工作区的路径
    /// - `include_base64` (可选): 是否在输出中包含 Base64 编码的图像数据，默认为 false
    ///
    /// # 返回值
    ///
    /// 返回符合 JSON Schema 规范的对象定义
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "图像文件路径（绝对路径或相对于工作区的路径）"
                },
                "include_base64": {
                    "type": "boolean",
                    "description": "在输出中包含 Base64 编码的图像数据（默认：false）"
                }
            },
            "required": ["path"]
        })
    }

    /// 执行图像信息提取（非 WASM 平台）
    ///
    /// 读取指定图像文件，提取元数据并可选返回 Base64 编码数据。
    ///
    /// # 参数
    ///
    /// - `args`: JSON 格式的参数对象，包含：
    ///   - `path`: 图像文件路径（必需）
    ///   - `include_base64`: 是否包含 Base64 数据（可选，默认 false）
    ///
    /// # 返回值
    ///
    /// 返回 `ToolResult` 结构：
    /// - 成功时：`success=true`，`output` 包含格式化的图像信息
    /// - 失败时：`success=false`，`error` 包含错误描述
    ///
    /// # 输出格式
    ///
    /// ```text
    /// File: /path/to/image.png
    /// Format: png
    /// Size: 12345 bytes
    /// Dimensions: 800x600
    /// data:image/png;base64,iVBORw0KGgo...  (如果 include_base64=true)
    /// ```
    ///
    /// # 错误情况
    ///
    /// - 缺少 `path` 参数
    /// - 路径不在允许的工作区范围内
    /// - 文件不存在或不是常规文件
    /// - 文件大小超过 5MB 限制
    /// - 读取文件失败
    #[cfg(not(target_arch = "wasm32"))]
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let args: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(args).map_err(|e| anyhow::anyhow!("Invalid arguments: {e}"))?;

        let path_str = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        let include_base64 =
            args.get("include_base64").and_then(serde_json::Value::as_bool).unwrap_or(false);

        let resolved_path = match self.resolve_image_path(path_str) {
            Ok(path) => path,
            Err(error) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(error),
                });
            }
        };

        if !resolved_path.is_file() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Not a file: {}", resolved_path.display())),
            });
        }

        let metadata = tokio::fs::metadata(&resolved_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read file metadata: {e}"))?;

        let file_size = metadata.len();

        if file_size > MAX_IMAGE_BYTES {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Image too large: {file_size} bytes (max {MAX_IMAGE_BYTES} bytes)"
                )),
            });
        }

        let bytes = tokio::fs::read(&resolved_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read image file: {e}"))?;

        let format = Self::detect_format(&bytes);
        let dimensions = Self::extract_dimensions(&bytes, format);

        let mut output = format!("File: {path_str}\nFormat: {format}\nSize: {file_size} bytes");

        if let Some((w, h)) = dimensions {
            let _ = write!(output, "\nDimensions: {w}x{h}");
        }

        if include_base64 {
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
            let mime = match format {
                "png" => "image/png",
                "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "webp" => "image/webp",
                "bmp" => "image/bmp",
                _ => "application/octet-stream",
            };
            let _ = write!(output, "\ndata:{mime};base64,{encoded}");
        }

        Ok(ToolResult { success: true, output, error: None })
    }

    /// 执行图像信息提取（WASM 平台）
    ///
    /// 在 WebAssembly 环境中，图像信息工具不被支持。
    /// 此方法直接返回错误信息，指示该功能在 Web 平台上不可用。
    ///
    /// # 参数
    ///
    /// - `_args`: 忽略的参数（WASM 平台不处理）
    ///
    /// # 返回值
    ///
    /// 始终返回 `ToolResult`，其中：
    /// - `success=false`
    /// - `error=Some("Image info tool is not supported on Web")`
    #[cfg(target_arch = "wasm32")]
    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some("Image info tool is not supported on Web".to_string()),
        })
    }
}

/// 单元测试模块
///
/// 测试代码位于 `tests/image_info.rs` 文件中，
/// 包含图像格式检测、尺寸提取和工具执行的测试用例。
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
