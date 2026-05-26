//! 自定义主题模块
//!
//! 本模块提供思维导图的自定义主题配置功能，允许用户定义完全个性化的配色方案。
//! 与预设主题不同，自定义主题提供了对每个视觉元素的精确颜色控制。
//!
//! # 主要功能
//!
//! - 定义思维导图中各个层级节点的颜色配置
//! - 支持亮色/暗色模式切换
//! - 提供多种预设的自定义主题供选择
//!
//! # 颜色格式
//!
//! 所有颜色值使用 `u32` 类型表示，采用 RGBA 格式（每8位表示一个通道）：
//! - 高8位：红色通道 (R)
//! - 次8位：绿色通道 (G)
//! - 再次8位：蓝色通道 (B)
//! - 低8位：透明度通道 (A)
//!
//! # 示例
//!
//! ```ignore
//! use crate::apps::mindmap::canvas::theme::custom::{MindMapCustomTheme, default_custom_themes};
//!
//! // 获取预设主题列表
//! let themes = default_custom_themes();
//!
//! // 创建自定义主题
//! let my_theme = MindMapCustomTheme {
//!     background_color: 0xFFFFFFFF,
//!     root_fill: 0x1E3A5FFF,
//!     root_text: 0xFFFFFFFF,
//!     branch_fills: vec![0x93C5FDFF],
//!     branch_text: 0x111827FF,
//!     leaf_fill: 0xFFFFFFFF,
//!     leaf_text: 0x111827FF,
//!     line_color: Some(0x1E3A5FFF),
//!     is_dark: false,
//! };
//! ```

/// 思维导图自定义主题配置结构体
///
/// 该结构体定义了思维导图中所有可配置的视觉元素的颜色方案，
/// 包括背景、根节点、分支节点、叶子节点以及连接线的颜色。
///
/// # 字段说明
///
/// 所有颜色值均为 `u32` 类型的 RGBA 颜色值，格式为 `0xRRGGBBAA`。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MindMapCustomTheme {
    /// 画布背景颜色
    ///
    /// 整个思维导图画布的背景填充色。
    /// 通常使用浅色（亮色模式）或深色（暗色模式）。
    pub background_color: u32,

    /// 根节点填充颜色
    ///
    /// 思维导图中心（根）节点的背景填充色。
    /// 通常使用最突出的颜色以强调其作为核心主题的地位。
    pub root_fill: u32,

    /// 根节点文本颜色
    ///
    /// 根节点中文字的颜色。
    /// 应与 `root_fill` 形成良好的对比度以确保可读性。
    pub root_text: u32,

    /// 分支节点填充颜色列表
    ///
    /// 一级子节点（直接连接到根节点的分支）的背景填充色。
    /// 使用向量存储多个颜色值，不同分支可以呈现不同颜色，
    /// 从而在视觉上区分不同的主题分支。
    ///
    /// # 示例
    ///
    /// 如果有三个一级分支，它们将依次使用此列表中的颜色。
    /// 如果分支数量超过颜色数量，颜色可能会循环使用。
    pub branch_fills: Vec<u32>,

    /// 分支节点文本颜色
    ///
    /// 一级子节点中文字的颜色。
    /// 应与所有 `branch_fills` 中的颜色都保持良好的对比度。
    pub branch_text: u32,

    /// 叶子节点填充颜色
    ///
    /// 所有末级节点（没有子节点的节点）的背景填充色。
    /// 通常使用较淡或中性的颜色，以避免视觉干扰。
    pub leaf_fill: u32,

    /// 叶子节点文本颜色
    ///
    /// 末级节点中文字的颜色。
    /// 应与 `leaf_fill` 形成良好的对比度。
    pub leaf_text: u32,

    /// 连接线颜色（可选）
    ///
    /// 节点之间连接线的颜色。
    /// 设置为 `None` 时，连接线可能使用默认颜色或与父节点颜色相同。
    /// 设置为 `Some(color)` 时，所有连接线将使用指定的统一颜色。
    pub line_color: Option<u32>,

    /// 是否为暗色主题
    ///
    /// 标识此主题是否为暗色模式。
    /// - `true`：暗色主题，适用于低光环境
    /// - `false`：亮色主题，适用于正常光环境
    ///
    /// 此标志可能影响 UI 的其他方面，如阴影、边框等。
    pub is_dark: bool,
}

/// 返回预设的自定义主题列表
///
/// 该函数提供一组精心设计的自定义主题配置，用户可以直接使用这些主题，
/// 也可以将它们作为创建自己主题的参考模板。
///
/// # 返回值
///
/// 返回一个包含多个 `MindMapCustomTheme` 实例的向量，每个实例代表一个完整的主题方案。
///
/// # 主题列表
///
/// 当前包含以下预设主题：
///
/// 1. **海洋蓝主题**：以深蓝色为主色调，搭配蓝色、绿色、黄色分支
/// 2. **紫罗兰主题**：以紫色为主色调，搭配粉色、黄色分支
/// 3. **青绿主题**：以青绿色为主色调，搭配绿色、蓝色、黄色分支
/// 4. **阳光主题**：以明黄色为主色调，搭配紫色、蓝色分支
///
/// # 示例
///
/// ```ignore
/// let themes = default_custom_themes();
/// assert!(!themes.is_empty());
///
/// // 使用第一个主题
/// let first_theme = &themes[0];
/// println!("背景颜色: #{:08X}", first_theme.background_color);
/// ```
pub fn default_custom_themes() -> Vec<MindMapCustomTheme> {
    vec![
        // 海洋蓝主题
        // 特点：专业、沉稳，适合商务和技术类思维导图
        MindMapCustomTheme {
            background_color: 0xFFFFFFFF, // 纯白背景
            root_fill: 0x1E3A5FFF,        // 深蓝色根节点
            root_text: 0xFFFFFFFF,        // 白色根节点文字
            branch_fills: vec![
                0x93C5FDFF, // 浅蓝色分支
                0xA7F3D0FF, // 浅绿色分支
                0xFDE047FF, // 黄色分支
            ],
            branch_text: 0x111827FF,      // 深灰色分支文字
            leaf_fill: 0xFFFFFFFF,        // 白色叶子节点
            leaf_text: 0x111827FF,        // 深灰色叶子节点文字
            line_color: Some(0x1E3A5FFF), // 深蓝色连接线
            is_dark: false,               // 亮色模式
        },
        // 紫罗兰主题
        // 特点：优雅、创意，适合艺术和设计类思维导图
        MindMapCustomTheme {
            background_color: 0xFFF1F2FF, // 淡粉色背景
            root_fill: 0x7C3AEDFF,        // 紫色根节点
            root_text: 0xFFFFFFFF,        // 白色根节点文字
            branch_fills: vec![
                0xF9A8D4FF, // 粉色分支
                0xFDE047FF, // 黄色分支
            ],
            branch_text: 0x374151FF,      // 深灰色分支文字
            leaf_fill: 0xFFFFFFFF,        // 白色叶子节点
            leaf_text: 0x111827FF,        // 深灰色叶子节点文字
            line_color: Some(0xF9A8D4FF), // 粉色连接线
            is_dark: false,               // 亮色模式
        },
        // 青绿主题
        // 特点：清新、自然，适合环境和生态类思维导图
        MindMapCustomTheme {
            background_color: 0xECFDF5FF, // 淡绿色背景
            root_fill: 0x14B8A6FF,        // 青绿色根节点
            root_text: 0xFFFFFFFF,        // 白色根节点文字
            branch_fills: vec![
                0x86EFACFF, // 浅绿色分支
                0x93C5FDFF, // 浅蓝色分支
                0xFDE047FF, // 黄色分支
            ],
            branch_text: 0x111827FF,      // 深灰色分支文字
            leaf_fill: 0xFFFFFFFF,        // 白色叶子节点
            leaf_text: 0x111827FF,        // 深灰色叶子节点文字
            line_color: Some(0x14B8A6FF), // 青绿色连接线
            is_dark: false,               // 亮色模式
        },
        // 阳光主题
        // 特点：活泼、明快，适合教育和娱乐类思维导图
        MindMapCustomTheme {
            background_color: 0xFFFFFFFF, // 纯白背景
            root_fill: 0xFDE047FF,        // 明黄色根节点
            root_text: 0x111827FF,        // 深灰色根节点文字
            branch_fills: vec![
                0x7C3AEDFF, // 紫色分支
                0x93C5FDFF, // 浅蓝色分支
            ],
            branch_text: 0xFFFFFFFF,      // 白色分支文字
            leaf_fill: 0x7C3AEDFF,        // 紫色叶子节点
            leaf_text: 0xFFFFFFFF,        // 白色叶子节点文字
            line_color: Some(0xFEF9C3FF), // 淡黄色连接线
            is_dark: false,               // 亮色模式
        },
    ]
}
