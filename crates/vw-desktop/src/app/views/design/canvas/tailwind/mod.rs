//! Tailwind CSS 集成模块
//!
//! 本模块提供 Tailwind CSS 样式系统的完整集成能力，包括：
//! - HTML 解析与 DOM 树构建
//! - Tailwind 类名解析与样式提取
//! - 样式渲染与输出
//! - Tailwind 颜色系统支持
//!
//! ## 模块结构
//!
//! - [`classes`] - Tailwind 类名获取与处理
//! - [`colors`] - Tailwind 颜色系统定义
//! - [`dom`] - HTML DOM 解析与节点表示
//! - [`parser`] - Tailwind 样式解析器
//! - [`renderer`] - 样式渲染输出
//!
//! ## 使用示例
//!
//! ```ignore
//! use app::views::design::canvas::tailwind::{
//!     TailwindParser, TailwindNode, render, TailwindColors
//! };
//!
//! // 解析 HTML 并提取 Tailwind 样式
//! let node = parse_html(html_string)?;
//! let parser = TailwindParser::new();
//! let styles = parser.parse(&node)?;
//!
//! // 渲染样式
//! let output = render(&styles);
//! ```

pub mod classes;
pub mod colors;
pub mod dom;
pub mod parser;
pub mod renderer;

/// 获取 Tailwind 类名集合
///
/// 返回当前支持的 Tailwind CSS 类名列表，用于类名验证和自动补全。
///
/// # 返回值
///
/// 返回包含所有支持的 Tailwind 类名的集合
///
/// # 示例
///
/// ```ignore
/// use app::views::design::canvas::tailwind::get_tailwind_classes;
///
/// let classes = get_tailwind_classes();
/// assert!(classes.contains("flex"));
/// assert!(classes.contains("text-center"));
/// ```
pub use classes::get_tailwind_classes;

/// Tailwind 颜色系统
///
/// 提供 Tailwind CSS 中定义的所有颜色常量和颜色处理工具。
/// 包括调色板定义、颜色值转换等功能。
pub use colors::TailwindColors;

/// Tailwind DOM 节点类型
///
/// 表示解析后的 HTML DOM 节点，包含节点类型、属性和子节点信息。
/// 用于在 Tailwind 样式处理流程中表示文档结构。
pub use dom::TailwindNode;

/// HTML 解析函数
///
/// 将 HTML 字符串解析为 Tailwind DOM 节点树。
///
/// # 参数
///
/// * `html` - 要解析的 HTML 字符串
///
/// # 返回值
///
/// 返回解析后的根节点，或解析错误
///
/// # 示例
///
/// ```ignore
/// use app::views::design::canvas::tailwind::parse_html;
///
/// let html = r#"<div class="flex items-center"><p>Hello</p></div>"#;
/// let root = parse_html(html)?;
/// ```
pub use dom::parse_html;

/// 解析后的样式结构
///
/// 表示从 Tailwind 类名解析出的单个样式属性，包含属性名和值。
pub use parser::{ParsedStyle, TailwindParseAnalysis, TailwindTokenIssue, TailwindTokenSupport};

/// Tailwind 样式解析器
///
/// 用于解析 HTML 元素上的 Tailwind 类名，并将其转换为具体的 CSS 样式。
///
/// # 示例
///
/// ```ignore
/// use app::views::design::canvas::tailwind::{TailwindParser, TailwindNode};
///
/// let parser = TailwindParser::new();
/// let styles = parser.parse_from_classes("flex text-center bg-blue-500");
/// ```
pub use parser::TailwindParser;

/// 样式渲染函数
///
/// 将解析后的样式集合渲染为最终的 CSS 输出格式。
///
/// # 参数
///
/// * `styles` - 要渲染的样式集合
///
/// # 返回值
///
/// 返回渲染后的 CSS 字符串
pub use renderer::render;
