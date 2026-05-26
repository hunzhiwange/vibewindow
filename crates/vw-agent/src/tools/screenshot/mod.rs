//! 截图工具
//!
//! 使用平台原生命令捕获屏幕截图。支持 macOS (screencapture) 和
//! Linux (gnome-screenshot/scrot/ImageMagick)。
//!
//! # 功能特性
//!
//! - 跨平台支持：macOS 和 Linux
//! - 安全的文件名处理：防止路径遍历和 shell 注入攻击
//! - Base64 编码输出：支持内嵌图像数据
//! - 超时保护：防止截图命令无限期挂起
//! - 工作区限制：确保截图保存在允许的目录内

use super::traits::{Tool, ToolResult};
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::shell::tokio_command;
use async_trait::async_trait;
use serde_json::json;
use std::fmt::Write;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// 截图命令执行的超时时间（秒）
///
/// 防止截图命令因用户交互或其他原因无限期挂起。
/// 在 macOS 上，"selection" 模式需要用户手动选择区域，
/// 因此超时时间设置为相对宽松的 15 秒。
const SCREENSHOT_TIMEOUT_SECS: u64 = 15;

/// 原始截图允许内联编码的最大字节数。
///
/// 与默认多模态 5 MiB 单图上限保持同一量级，避免截图工具比普通
/// 图片附件更早触发“过大”分支。
const MAX_RAW_BYTES: u64 = 5 * 1024 * 1024;

/// Base64 编码数据的最大允许大小（字节）
///
/// 限制为 7 MiB，可覆盖 5 MiB 原始图像数据经 Base64 编码后的膨胀。
/// 超过此限制的 base64 输出将被截断，以防止：
/// - 内存溢出
/// - 响应体过大
/// - 传输性能问题
const MAX_BASE64_BYTES: usize = 7 * 1024 * 1024;

/// 屏幕截图工具
///
/// 使用平台原生命令捕获屏幕截图。支持以下平台：
///
/// # 平台支持
///
/// - **macOS**: 使用 `screencapture` 命令
///   - 支持全屏、交互式选择区域、前台窗口模式
/// - **Linux**: 按优先级尝试以下工具
///   1. `gnome-screenshot` (GNOME 桌面环境)
///   2. `scrot` (轻量级截图工具)
///   3. `import` (ImageMagick 套件)
///
/// # 安全性
///
/// - 所有输出路径必须位于配置的工作区内
/// - 拒绝通过符号链接写入（防止符号链接攻击）
/// - 文件名经过净化处理，移除危险字符
/// - 需要安全策略授权才能执行
///
/// # 示例
///
/// ```ignore
/// use std::sync::Arc;
/// use vibe_agent::tools::screenshot::ScreenshotTool;
/// use vibe_agent::security::SecurityPolicy;
///
/// let security = Arc::new(SecurityPolicy::default());
/// let tool = ScreenshotTool::new(security);
///
/// // 执行全屏截图
/// let result = tool.execute(json!({})).await?;
/// ```
pub struct ScreenshotTool {
    /// 安全策略引用，用于工作区限制和执行权限检查
    security: Arc<SecurityPolicy>,
}

impl ScreenshotTool {
    /// 创建新的截图工具实例
    ///
    /// # 参数
    ///
    /// - `security`: 安全策略的原子引用，用于：
    ///   - 验证输出路径是否在允许的工作区内
    ///   - 检查是否允许执行写操作
    ///
    /// # 返回
    ///
    /// 返回配置好的 `ScreenshotTool` 实例
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self { security }
    }

    /// 净化输出文件名，防止路径遍历攻击
    ///
    /// 从用户提供的文件名中提取安全的基名（basename），
    /// 移除任何目录组件，防止路径遍历攻击。
    ///
    /// # 参数
    ///
    /// - `filename`: 用户提供的原始文件名，可能包含目录路径
    /// - `fallback`: 当文件名无效时使用的后备文件名
    ///
    /// # 返回
    ///
    /// 返回净化后的安全文件名字符串
    ///
    /// # 安全检查
    ///
    /// 函数会拒绝以下情况并返回 `fallback`：
    /// - 空字符串
    /// - 当前目录标记 "."
    /// - 父目录标记 ".."
    /// - 包含空字符的字符串
    /// - 无法解析为有效 UTF-8 的文件名
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let safe = ScreenshotTool::sanitize_output_filename("../../../etc/passwd", "safe.png");
    /// assert_eq!(safe, "passwd");  // 只保留基名
    /// ```
    fn sanitize_output_filename(filename: &str, fallback: &str) -> String {
        // 提取文件名的基名部分（去除目录路径）
        let Some(basename) = Path::new(filename).file_name().and_then(|name| name.to_str()) else {
            return fallback.to_string();
        };

        // 去除首尾空白字符
        let trimmed = basename.trim();

        // 检查危险值：空字符串、目录标记、空字符
        if trimmed.is_empty() || trimmed == "." || trimmed == ".." || trimmed.contains('\0') {
            return fallback.to_string();
        }

        trimmed.to_string()
    }

    /// 解析并验证输出路径，确保写入操作安全
    ///
    /// 此方法执行多层安全检查：
    /// 1. 确保工作区目录存在
    /// 2. 解析路径并验证其在工作区内
    /// 3. 检查并拒绝符号链接（防止符号链接攻击）
    /// 4. 验证目标路径是常规文件
    ///
    /// # 参数
    ///
    /// - `filename`: 净化后的文件名（不含目录路径）
    ///
    /// # 返回
    ///
    /// - `Ok(PathBuf)`: 验证通过的完整输出路径
    /// - `Err`: 如果路径不安全或验证失败
    ///
    /// # 错误
    ///
    /// 函数在以下情况返回错误：
    /// - 无法创建工作区目录
    /// - 路径解析后位于工作区外
    /// - 目标路径是符号链接
    /// - 目标路径不是常规文件
    ///
    /// # 安全性
    ///
    /// 此方法是防止路径遍历攻击的关键防线：
    /// - 使用 `canonicalize` 解析所有符号链接
    /// - 比较解析后的路径与工作区边界
    /// - 显式拒绝通过符号链接写入
    async fn resolve_output_path_for_write(&self, filename: &str) -> anyhow::Result<PathBuf> {
        // 确保工作区目录存在，不存在则创建
        tokio::fs::create_dir_all(&self.security.workspace_dir).await?;

        // 获取工作区的规范化路径（解析符号链接）
        // 如果规范化失败（如目录刚创建），使用原始路径作为后备
        let workspace_root = tokio::fs::canonicalize(&self.security.workspace_dir)
            .await
            .unwrap_or_else(|_| self.security.workspace_dir.clone());

        // 构建输出路径：工作区根目录 + 文件名
        let output_path = workspace_root.join(filename);

        // 安全检查：确保父目录解析后仍在工作区内
        // 这防止了通过父目录符号链接逃逸工作区
        let parent = output_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid screenshot output path"))?;
        let resolved_parent = tokio::fs::canonicalize(parent).await?;
        if !self.security.is_resolved_path_allowed(&resolved_parent) {
            anyhow::bail!("{}", self.security.resolved_path_violation_message(&resolved_parent));
        }

        // 检查输出路径的元数据（不跟踪符号链接）
        match tokio::fs::symlink_metadata(&output_path).await {
            Ok(meta) => {
                // 安全检查：拒绝符号链接
                // 攻击者可能创建指向系统文件的符号链接
                if meta.file_type().is_symlink() {
                    anyhow::bail!(
                        "Refusing to write screenshot through symlink: {}",
                        output_path.display()
                    );
                }
                // 类型检查：确保目标是常规文件
                if !meta.is_file() {
                    anyhow::bail!(
                        "Screenshot output path is not a regular file: {}",
                        output_path.display()
                    );
                }
            }
            // 文件不存在是正常情况，允许继续
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            // 其他错误（如权限问题）需要上报
            Err(e) => return Err(e.into()),
        }

        Ok(output_path)
    }

    /// 根据当前平台确定可用的截图命令列表
    ///
    /// 返回一组候选命令，按优先级排序。执行时会依次尝试，
    /// 直到找到可用的工具。
    ///
    /// # 参数
    ///
    /// - `output_path`: 截图保存的完整路径
    ///
    /// # 返回
    ///
    /// 返回命令参数向量，每个元素是完整的命令行参数列表
    ///
    /// # 平台差异
    ///
    /// - **macOS**: 使用内置的 `screencapture` 命令
    ///   - `-x` 参数：静默执行（不播放快门声音）
    /// - **Linux**: 尝试多个工具（按优先级）
    ///   1. `gnome-screenshot -f`: GNOME 桌面截图工具
    ///   2. `scrot`: 轻量级命令行截图工具
    ///   3. `import -window root`: ImageMagick 的截图工具
    /// - **其他平台**: 返回空列表（不支持）
    fn screenshot_commands(output_path: &str) -> Vec<Vec<String>> {
        if cfg!(target_os = "macos") {
            vec![vec![
                "screencapture".into(),
                "-x".into(), // 静默模式：不播放快门声音
                output_path.into(),
            ]]
        } else if cfg!(target_os = "linux") {
            vec![
                // 优先尝试 GNOME 截图工具
                vec!["gnome-screenshot".into(), "-f".into(), output_path.into()],
                // 其次尝试 scrot
                vec!["scrot".into(), output_path.into()],
                // 最后尝试 ImageMagick 的 import 命令
                vec!["import".into(), "-window".into(), "root".into(), output_path.into()],
            ]
        } else {
            Vec::new()
        }
    }

    /// 执行屏幕截图并返回结果
    ///
    /// 这是截图工具的核心方法，处理完整的截图流程：
    /// 1. 解析和验证参数
    /// 2. 净化文件名
    /// 3. 验证输出路径安全性
    /// 4. 执行平台原生的截图命令
    /// 5. 读取并编码截图文件
    ///
    /// # 参数
    ///
    /// - `args`: JSON 格式的参数对象，支持以下字段：
    ///   - `filename`: 可选的自定义文件名（默认：screenshot_<时间戳>.png）
    ///   - `region`: 可选的区域模式（仅 macOS）
    ///     - `"selection"`: 交互式选择区域
    ///     - `"window"`: 截取前台窗口
    ///
    /// # 返回
    ///
    /// 返回 `ToolResult`，其中：
    /// - `output`: 包含文件路径、文件大小和 Base64 编码的图像数据
    /// - `error`: 如果失败，包含错误描述
    ///
    /// # 错误处理
    ///
    /// 函数在以下情况返回错误结果：
    /// - 文件名包含危险的 shell 字符
    /// - 输出路径不在允许的工作区内
    /// - 没有可用的截图工具
    /// - 截图命令执行超时
    /// - 截图命令执行失败
    ///
    /// # 超时保护
    ///
    /// 所有截图命令都有 `SCREENSHOT_TIMEOUT_SECS` 秒的超时限制，
    /// 防止命令因用户交互（如选择区域）或其他原因无限期挂起。
    async fn capture(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 生成时间戳用于默认文件名
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

        // 解析文件名参数，如未提供则使用默认值
        let filename = args
            .get("filename")
            .and_then(|v| v.as_str())
            .map_or_else(|| format!("screenshot_{timestamp}.png"), String::from);

        // 准备后备文件名（当用户提供的文件名无效时使用）
        let fallback_name = format!("screenshot_{timestamp}.png");

        // 净化文件名：只保留安全的基名，拒绝路径遍历尝试
        let safe_name = Self::sanitize_output_filename(&filename, &fallback_name);

        // 定义对 shell 有危险的字符集合
        // 这些字符可能导致命令注入或其他安全问题
        const SHELL_UNSAFE: &[char] =
            &['\'', '"', '`', '$', '\\', ';', '|', '&', '\n', '\0', '(', ')'];

        // 检查文件名是否包含危险字符
        if safe_name.contains(SHELL_UNSAFE) {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Filename contains characters unsafe for shell execution".into()),
            });
        }

        // 解析并验证输出路径的安全性
        let output_path = match self.resolve_output_path_for_write(&safe_name).await {
            Ok(path) => path,
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Invalid screenshot output path: {e}")),
                });
            }
        };
        let output_str = output_path.to_string_lossy().to_string();

        // 获取当前平台的候选截图命令
        let mut commands = Self::screenshot_commands(&output_str);
        if commands.is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Screenshot not supported on this platform".into()),
            });
        }

        // macOS 特有功能：区域选择模式
        // 这些选项在 Linux 上会被忽略
        if cfg!(target_os = "macos") {
            if let Some(region) = args.get("region").and_then(|v| v.as_str()) {
                match region {
                    // -s: 交互式选择区域模式
                    "selection" => commands[0].insert(1, "-s".into()),
                    // -w: 截取前台窗口模式
                    "window" => commands[0].insert(1, "-w".into()),
                    // 未知区域选项：忽略
                    _ => {}
                }
            }
        }

        // 跟踪是否找到可执行的命令
        let mut saw_spawnable_command = false;
        // 记录最后一次失败的原因
        let mut last_failure: Option<String> = None;

        // 依次尝试候选命令
        for mut cmd_args in commands {
            if cmd_args.is_empty() {
                continue;
            }
            // 提取程序名称（第一个参数）
            let program = cmd_args.remove(0);

            // 执行命令，带超时保护
            let result = tokio::time::timeout(
                Duration::from_secs(SCREENSHOT_TIMEOUT_SECS),
                tokio_command(&program).args(&cmd_args).output(),
            )
            .await;

            match result {
                // 命令执行成功（无论退出码如何）
                Ok(Ok(output)) => {
                    saw_spawnable_command = true;
                    // 检查命令是否成功完成
                    if output.status.success() {
                        // 成功：读取并编码截图文件
                        return Self::read_and_encode(&output_path).await;
                    }
                    // 失败：记录错误信息
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if stderr.is_empty() {
                        last_failure =
                            Some(format!("{} exited with status {}", program, output.status));
                    } else {
                        last_failure = Some(stderr);
                    }
                }
                // 命令未找到：继续尝试下一个候选
                Ok(Err(e)) if e.kind() == ErrorKind::NotFound => {
                    // 静默跳过，尝试下一个工具
                }
                // 其他执行错误（权限、资源等）
                Ok(Err(e)) => {
                    saw_spawnable_command = true;
                    last_failure = Some(format!("Failed to execute screenshot command: {e}"));
                }
                // 超时错误
                Err(_) => {
                    saw_spawnable_command = true;
                    last_failure =
                        Some(format!("Screenshot timed out after {SCREENSHOT_TIMEOUT_SECS}s"));
                }
            }
        }

        // 没有找到任何可执行的截图工具
        if !saw_spawnable_command {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(
                    "No screenshot tool found. Install gnome-screenshot, scrot, or ImageMagick."
                        .into(),
                ),
            });
        }

        // 所有尝试都失败，返回最后一次错误
        Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(
                last_failure
                    .unwrap_or_else(|| "Screenshot command failed for unknown reasons".into()),
            ),
        })
    }

    /// 读取截图文件并返回 Base64 编码的结果
    ///
    /// 读取截图文件，进行大小检查，然后将图像数据编码为
    /// Base64 格式，嵌入到输出消息中。
    ///
    /// # 参数
    ///
    /// - `output_path`: 截图文件的路径
    ///
    /// # 返回
    ///
    /// 返回 `ToolResult`，其中：
    /// - `output`: 包含文件路径、大小信息和 data URI 格式的图像数据
    /// - `error`: 如果读取失败，包含错误描述
    ///
    /// # 大小限制
    ///
    /// - 原始文件大小限制：5 MiB（`MAX_RAW_BYTES`）
    /// - Base64 编码后限制：7 MiB（`MAX_BASE64_BYTES`）
    ///
    /// 超过原始文件大小限制的截图将不会进行 Base64 编码，
    /// 但仍会报告成功和文件路径。超过 Base64 限制的输出
    /// 会被截断以确保 UTF-8 边界正确。
    ///
    /// # 输出格式
    ///
    /// 成功时的输出格式：
    /// ```text
    /// Screenshot saved to: /path/to/screenshot.png
    /// Size: 12345 bytes
    /// Base64 length: 16460
    /// data:image/png;base64,iVBORw0KGgo...
    /// ```
    #[allow(clippy::incompatible_msrv)]
    async fn read_and_encode(output_path: &std::path::Path) -> anyhow::Result<ToolResult> {
        // 在读取前检查文件大小，防止大文件导致内存溢出
        if let Ok(meta) = tokio::fs::metadata(output_path).await {
            if meta.len() > MAX_RAW_BYTES {
                // 文件过大，不进行 Base64 编码，但报告成功
                return Ok(ToolResult {
                    success: true,
                    output: format!(
                        "Screenshot saved to: {}\nSize: {} bytes (too large to base64-encode inline)",
                        output_path.display(),
                        meta.len(),
                    ),
                    error: None,
                });
            }
        }

        // 读取文件内容
        match tokio::fs::read(output_path).await {
            Ok(bytes) => {
                use base64::Engine;
                let size = bytes.len();

                // 进行 Base64 编码
                let mut encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);

                // 检查并截断过长的 Base64 输出
                // 使用 floor_utf8_char_boundary 确保 UTF-8 边界正确
                let truncated = if encoded.len() > MAX_BASE64_BYTES {
                    encoded.truncate(crate::app::agent::util::floor_utf8_char_boundary(
                        &encoded,
                        MAX_BASE64_BYTES,
                    ));
                    true
                } else {
                    false
                };

                // 构建输出消息
                let mut output_msg = format!(
                    "Screenshot saved to: {}\nSize: {size} bytes\nBase64 length: {}",
                    output_path.display(),
                    encoded.len(),
                );

                // 如果被截断，添加标记
                if truncated {
                    output_msg.push_str(" (truncated)");
                }

                // 根据文件扩展名确定 MIME 类型
                let mime = match output_path.extension().and_then(|e| e.to_str()) {
                    Some("jpg" | "jpeg") => "image/jpeg",
                    Some("bmp") => "image/bmp",
                    Some("gif") => "image/gif",
                    Some("webp") => "image/webp",
                    _ => "image/png", // 默认 PNG
                };

                // 附加 data URI（RFC 2397 格式）
                let _ = write!(output_msg, "\ndata:{mime};base64,{encoded}");

                Ok(ToolResult { success: true, output: output_msg, error: None })
            }
            // 读取失败：文件可能已被删除或权限问题
            Err(e) => Ok(ToolResult {
                success: false,
                output: format!("Screenshot saved to: {}", output_path.display()),
                error: Some(format!("Failed to read screenshot file: {e}")),
            }),
        }
    }
}

/// 为 ScreenshotTool 实现 Tool trait
///
/// 提供截图工具的标准接口实现，包括名称、描述、
/// 参数 schema 和执行方法。
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for ScreenshotTool {
    /// 返回工具名称
    ///
    /// 工具名称用于在系统中标识此工具，以及日志和调试。
    fn name(&self) -> &str {
        "screenshot"
    }

    /// 返回工具的简短描述
    ///
    /// 此描述会显示给用户或 LLM，帮助理解工具的用途。
    fn description(&self) -> &str {
        "Capture a screenshot of the current screen and return the file path plus Base64 PNG data."
    }

    /// 返回工具参数的 JSON Schema
    ///
    /// 定义工具接受的参数结构，包括：
    /// - `filename`: 可选的自定义文件名
    /// - `region`: 可选的截图区域模式（仅 macOS）
    ///
    /// 此 schema 用于：
    /// - 验证输入参数
    /// - 生成工具文档
    /// - 帮助 LLM 理解如何调用工具
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "filename": {
                    "type": "string",
                    "description": "可选文件名（默认：screenshot_<时间戳>.png）。保存在工作区中。"
                },
                "region": {
                    "type": "string",
                    "description": "macOS 可选区域：'selection' 用于交互式裁剪，'window' 用于前台窗口。Linux 上忽略此参数。"
                }
            }
        })
    }

    /// 执行截图操作
    ///
    /// 这是工具的主入口点。首先检查安全策略是否允许执行，
    /// 然后调用内部的 capture 方法完成实际截图。
    ///
    /// # 参数
    ///
    /// - `args`: JSON 格式的参数对象
    ///
    /// # 返回
    ///
    /// - `Ok(ToolResult)`: 截图结果，包含文件信息和图像数据
    /// - `Err`: 系统级错误（如配置问题）
    ///
    /// # 安全检查
    ///
    /// 在执行截图前，会检查 `security.can_act()`，
    /// 确保当前运行模式允许执行写操作。
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 安全策略检查：确保允许执行操作
        if !self.security.can_act() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Action blocked: autonomy is read-only".into()),
            });
        }
        // 执行实际的截图操作
        self.capture(args).await
    }
}

/// 测试模块
///
/// 测试代码位于 `tests/screenshot.rs` 文件中，
/// 使用 `#[path]` 属性指定外部测试文件路径。
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
