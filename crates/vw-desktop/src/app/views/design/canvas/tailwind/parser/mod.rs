//! Tailwind CSS 类名解析器模块
//!
//! 本模块提供将 Tailwind CSS 类名字符串解析为结构化样式对象的能力。
//! 主要用于将设计画布中的 Tailwind 类名转换为 Iced 框架可使用的样式属性。
//!
//! # 主要功能
//!
//! - 解析 Tailwind CSS 类名字符串（如 `"flex items-center p-4"`）
//! - 支持布局、间距、颜色、排版、边框等常用样式属性
//! - 提供解析结果缓存以提升性能
//! - 支持从 HTML 标签中提取类名和文本内容
//!
//! # 示例
//!
//! ```ignore
//! use crate::app::views::design::canvas::tailwind::parser::TailwindParser;
//!
//! let style = TailwindParser::parse("flex items-center p-4 bg-white text-gray-800");
//! // style.display == Some("flex")
//! // style.align_items == Some("center")
//! // style.padding == Some(16.0)
//! ```

mod analysis;
mod types;
mod utilities;

use analysis::analyze_classes;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub use types::{ParsedStyle, TailwindParseAnalysis, TailwindTokenIssue, TailwindTokenSupport};

/// 解析结果缓存
///
/// 使用线程安全的哈希表缓存已解析的样式结果，避免重复解析相同的类名字符串。
/// 当缓存条目超过 4096 时会自动清空，防止内存无限增长。
static PARSED_ANALYSIS_CACHE: Lazy<Mutex<HashMap<String, TailwindParseAnalysis>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Tailwind CSS 类名解析器
///
/// 提供将 Tailwind CSS 类名解析为 `ParsedStyle` 结构体的能力。
/// 该结构体不包含状态，所有方法都是关联函数。
///
/// # 功能
///
/// - 解析类名字符串并返回结构化样式
/// - 自动缓存解析结果以提升性能
/// - 从 HTML 内容中提取类名和文本
pub struct TailwindParser;

impl TailwindParser {
    /// 解析 Tailwind CSS 类名字符串
    ///
    /// 将类名字符串（如 `"flex items-center p-4"`）解析为 `ParsedStyle` 结构体。
    /// 支持同时解析多个类名，用空格分隔。
    ///
    /// # 参数
    ///
    /// - `class_string`: Tailwind CSS 类名字符串，多个类名用空格分隔
    ///
    /// # 返回值
    ///
    /// 返回 `ParsedStyle` 结构体，包含所有解析出的样式属性。
    /// 如果输入为空字符串，返回默认的 `ParsedStyle`。
    ///
    /// # 缓存机制
    ///
    /// - 首次解析时会缓存结果
    /// - 相同的类名字符串会直接返回缓存结果
    /// - 缓存条目超过 4096 时会自动清空
    ///
    /// # 类名预处理
    ///
    /// - 可压平的响应式/交互态前缀会被记录为静态降级，再尝试解析基础 utility
    /// - `dark:` 和明确不支持的高级效果会被记录为 export-only，而不是静默忽略
    /// - 支持透明度修饰符（如 `bg-blue-500/50`），会提取斜杠前的类名
    ///
    /// # 示例
    ///
    /// ```ignore
    /// // 解析布局类
    /// let style = TailwindParser::parse("flex flex-col items-center");
    /// assert_eq!(style.display, Some("flex".to_string()));
    /// assert_eq!(style.flex_direction, Some("column".to_string()));
    ///
    /// // 解析间距类
    /// let style = TailwindParser::parse("p-4 m-2 gap-4");
    /// assert_eq!(style.padding, Some(16.0));
    /// assert_eq!(style.margin, Some(8.0));
    /// ```
    pub fn parse(class_string: &str) -> ParsedStyle {
        Self::analyze(class_string).style
    }

    /// 解析 Tailwind 类名并返回静态画布支持分类。
    pub fn analyze(class_string: &str) -> TailwindParseAnalysis {
        let key = class_string.trim();
        if key.is_empty() {
            return TailwindParseAnalysis::default();
        }

        if let Ok(cache) = PARSED_ANALYSIS_CACHE.lock()
            && let Some(v) = cache.get(key)
        {
            return v.clone();
        }

        let analysis = analyze_classes(key);

        if let Ok(mut cache) = PARSED_ANALYSIS_CACHE.lock() {
            if cache.len() > 4096 {
                cache.clear();
            }
            cache.insert(key.to_string(), analysis.clone());
        }
        analysis
    }

    /// 从 HTML 内容中提取类名和文本内容
    ///
    /// 这是一个简单的 HTML 解析器，用于从类似 `<div class="...">text</div>`
    /// 的 HTML 标签中提取 class 属性值和内部文本内容。
    ///
    /// # 参数
    ///
    /// - `html`: HTML 字符串片段
    ///
    /// # 返回值
    ///
    /// 返回元组 `(Option<String>, Option<String>)`：
    /// - 第一个元素：class 属性值（如果存在）
    /// - 第二个元素：标签内的文本内容（如果存在）
    ///
    /// # 限制
    ///
    /// - 仅支持简单的单层标签解析
    /// - 不处理嵌套标签
    /// - 不处理转义字符
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let html = r#"<div class="flex items-center">Hello</div>"#;
    /// let (class, text) = TailwindParser::parse_html_content(html);
    /// assert_eq!(class, Some("flex items-center".to_string()));
    /// assert_eq!(text, Some("Hello".to_string()));
    /// ```
    pub fn parse_html_content(html: &str) -> (Option<String>, Option<String>) {
        let class_start_marker = "class=\"";
        let class_start = html.find(class_start_marker).map(|i| i + class_start_marker.len());

        let class_str = if let Some(start) = class_start {
            html[start..].find('"').map(|end| html[start..start + end].to_string())
        } else {
            None
        };

        let content_start = html.find('>').map(|i| i + 1);
        let content_end = html.rfind('<');

        let text_content = if let (Some(start), Some(end)) = (content_start, content_end) {
            if end > start { Some(html[start..end].trim().to_string()) } else { None }
        } else {
            None
        };

        (class_str, text_content)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
