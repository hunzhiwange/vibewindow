//! 思维导图主题类型定义模块
//!
//! 本模块定义了思维导图主题系统的核心数据结构，包括：
//! - 主题配置（`MindMapTheme`）：定义完整的主题样式
//! - 主题视图（`MindMapThemeView`）：用于运行时主题数据的轻量级视图
//! - 主题分组（`MindMapThemeGroup`）：将相关主题组织在一起的分组结构
//!
//! 所有颜色值使用 32 位无符号整数表示，遵循 0xRRGGBBAA 格式。

/// 思维导图主题配置
///
/// 定义了思维导图的完整视觉样式，包括背景色、各级节点颜色、连线颜色等。
/// 该结构体设计为静态配置，所有字符串和颜色数组均使用静态生命周期引用，
/// 以确保在编译时确定所有主题数据，提高性能和内存效率。
///
/// # 字段说明
///
/// - `id`: 主题的唯一标识符，用于程序化引用
/// - `name`: 主题的显示名称，用于用户界面展示
/// - `background_color`: 画布背景颜色
/// - `root_fill`: 根节点填充颜色
/// - `root_text`: 根节点文本颜色
/// - `branch_fills`: 分支节点填充颜色数组（支持多种颜色循环使用）
/// - `branch_text`: 分支节点文本颜色
/// - `leaf_fill`: 叶子节点填充颜色
/// - `leaf_text`: 叶子节点文本颜色
/// - `line_color`: 连线颜色（可选，若为 None 则使用默认逻辑）
/// - `is_dark`: 标识是否为深色主题（用于 UI 适配）
///
/// # 示例
///
/// ```ignore
/// use crate::apps::mindmap::canvas::theme::types::MindMapTheme;
///
/// let theme = MindMapTheme {
///     id: "classic",
///     name: "经典主题",
///     background_color: 0xFFFFFFFF,
///     root_fill: 0x4A90E2FF,
///     root_text: 0xFFFFFFFF,
///     branch_fills: &[0x5B9BD5FF, 0x70AD47FF],
///     branch_text: 0x000000FF,
///     leaf_fill: 0xE7E6E6FF,
///     leaf_text: 0x000000FF,
///     line_color: Some(0x808080FF),
///     is_dark: false,
/// };
///
/// // 获取第一个分支的颜色
/// let color = theme.palette(0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct MindMapTheme {
    /// 主题唯一标识符（静态字符串）
    pub id: &'static str,

    /// 主题显示名称（静态字符串）
    pub name: &'static str,

    /// 画布背景颜色（0xRRGGBBAA 格式）
    pub background_color: u32,

    /// 根节点填充颜色
    pub root_fill: u32,

    /// 根节点文本颜色
    pub root_text: u32,

    /// 分支节点填充颜色数组（用于多色循环）
    pub branch_fills: &'static [u32],

    /// 分支节点文本颜色
    pub branch_text: u32,

    /// 叶子节点填充颜色
    pub leaf_fill: u32,

    /// 叶子节点文本颜色
    pub leaf_text: u32,

    /// 连线颜色（可选）
    pub line_color: Option<u32>,

    /// 是否为深色主题标志
    pub is_dark: bool,
}

/// 思维导图主题视图
///
/// 提供主题数据的轻量级运行时视图，与 `MindMapTheme` 不同，
/// 该结构体中的数组使用泛型生命周期引用，允许从不同数据源构建主题视图。
/// 适用于动态主题切换或从配置文件加载主题数据的场景。
///
/// # 生命周期
///
/// `'a` - 引用的颜色数组的生命周期，必须与使用该视图的上下文匹配
///
/// # 字段说明
///
/// 所有字段与 `MindMapTheme` 中对应字段含义相同，但不包含 `id` 和 `name`，
/// 因为视图仅关注视觉样式数据。
#[derive(Debug, Clone, Copy)]
pub struct MindMapThemeView<'a> {
    /// 画布背景颜色（0xRRGGBBAA 格式）
    pub background_color: u32,

    /// 根节点填充颜色
    pub root_fill: u32,

    /// 根节点文本颜色
    pub root_text: u32,

    /// 分支节点填充颜色数组引用
    pub branch_fills: &'a [u32],

    /// 分支节点文本颜色
    pub branch_text: u32,

    /// 叶子节点填充颜色
    pub leaf_fill: u32,

    /// 叶子节点文本颜色
    pub leaf_text: u32,

    /// 连线颜色（可选）
    pub line_color: Option<u32>,

    /// 是否为深色主题标志
    pub is_dark: bool,
}

impl MindMapThemeView<'_> {
    /// 获取指定索引位置的分支颜色
    ///
    /// 根据给定的索引从 `branch_fills` 数组中获取对应的颜色值。
    /// 如果索引超出数组范围，将使用取模运算循环使用数组中的颜色。
    /// 如果 `branch_fills` 数组为空，将返回默认的蓝色（0xFF0000FF）。
    ///
    /// # 参数
    ///
    /// - `index`: 分支索引，用于确定使用哪个颜色
    ///
    /// # 返回值
    ///
    /// 返回对应索引位置的颜色值（u32），格式为 0xRRGGBBAA
    ///
    /// # 示例
    ///
    /// ```ignore
    /// use crate::apps::mindmap::canvas::theme::types::MindMapThemeView;
    ///
    /// let view = MindMapThemeView {
    ///     background_color: 0xFFFFFFFF,
    ///     root_fill: 0x4A90E2FF,
    ///     root_text: 0xFFFFFFFF,
    ///     branch_fills: &[0xFF0000FF, 0x00FF00FF, 0x0000FFFF],
    ///     branch_text: 0x000000FF,
    ///     leaf_fill: 0xE7E6E6FF,
    ///     leaf_text: 0x000000FF,
    ///     line_color: None,
    ///     is_dark: false,
    /// };
    ///
    /// // 获取第一个分支颜色
    /// assert_eq!(view.palette(0), 0xFF0000FF);
    ///
    /// // 索引超出范围时循环使用
    /// assert_eq!(view.palette(3), 0xFF0000FF);
    /// ```
    pub fn palette(&self, index: usize) -> u32 {
        // 检查颜色数组是否为空，为空时返回默认蓝色
        if self.branch_fills.is_empty() {
            return 0xFF0000FF;
        }

        // 使用取模运算实现颜色循环
        self.branch_fills[index % self.branch_fills.len()]
    }
}

/// 思维导图主题分组
///
/// 将多个相关的主题组织在一起的容器结构，用于主题分类管理。
/// 例如，可以将"亮色主题"、"暗色主题"或"节日主题"等作为分组。
///
/// # 字段说明
///
/// - `id`: 分组的唯一标识符
/// - `name`: 分组的显示名称
/// - `variants`: 该分组下的所有主题变体数组
///
/// # 示例
///
/// ```ignore
/// use crate::apps::mindmap::canvas::theme::types::{MindMapTheme, MindMapThemeGroup};
///
/// let light_theme = MindMapTheme {
///     id: "light",
///     name: "明亮",
///     background_color: 0xFFFFFFFF,
///     root_fill: 0x4A90E2FF,
///     root_text: 0xFFFFFFFF,
///     branch_fills: &[0x5B9BD5FF],
///     branch_text: 0x000000FF,
///     leaf_fill: 0xE7E6E6FF,
///     leaf_text: 0x000000FF,
///     line_color: None,
///     is_dark: false,
/// };
///
/// let group = MindMapThemeGroup {
///     id: "basic",
///     name: "基础主题",
///     variants: &[light_theme],
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MindMapThemeGroup {
    /// 分组唯一标识符
    pub id: &'static str,

    /// 分组显示名称
    pub name: &'static str,

    /// 该分组下的主题变体数组
    pub variants: &'static [MindMapTheme],
}

impl MindMapTheme {
    /// 获取指定索引位置的分支颜色
    ///
    /// 根据给定的索引从 `branch_fills` 数组中获取对应的颜色值。
    /// 如果索引超出数组范围，将使用取模运算循环使用数组中的颜色。
    /// 如果 `branch_fills` 数组为空，将返回默认的蓝色（0xFF0000FF）。
    ///
    /// # 参数
    ///
    /// - `index`: 分支索引，用于确定使用哪个颜色
    ///
    /// # 返回值
    ///
    /// 返回对应索引位置的颜色值（u32），格式为 0xRRGGBBAA
    ///
    /// # 示例
    ///
    /// ```ignore
    /// use crate::apps::mindmap::canvas::theme::types::MindMapTheme;
    ///
    /// let theme = MindMapTheme {
    ///     id: "classic",
    ///     name: "经典主题",
    ///     background_color: 0xFFFFFFFF,
    ///     root_fill: 0x4A90E2FF,
    ///     root_text: 0xFFFFFFFF,
    ///     branch_fills: &[0x5B9BD5FF, 0x70AD47FF, 0xFFC000FF],
    ///     branch_text: 0x000000FF,
    ///     leaf_fill: 0xE7E6E6FF,
    ///     leaf_text: 0x000000FF,
    ///     line_color: None,
    ///     is_dark: false,
    /// };
    ///
    /// // 获取第二个分支颜色
    /// assert_eq!(theme.palette(1), 0x70AD47FF);
    ///
    /// // 索引超出范围时循环使用
    /// assert_eq!(theme.palette(5), 0x70AD47FF); // 5 % 3 = 2
    /// ```
    pub fn palette(&self, index: usize) -> u32 {
        // 检查颜色数组是否为空，为空时返回默认蓝色
        if self.branch_fills.is_empty() {
            return 0xFF0000FF;
        }

        // 使用取模运算实现颜色循环
        self.branch_fills[index % self.branch_fills.len()]
    }
}
