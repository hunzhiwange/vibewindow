use iced::Color;

/// 解析后的样式结构体
///
/// 包含从 Tailwind CSS 类名解析出的所有样式属性。
/// 每个字段都是 `Option` 类型，表示该属性可能未被设置。
///
/// # 字段说明
///
/// - 布局相关：`display`、`flex_direction`、`align_items`、`justify_content`、`position`
/// - 间距相关：`padding`、`margin`、`gap_x`、`gap_y` 及其方向变体
/// - 尺寸相关：`width`、`height`
/// - 颜色相关：`text_color`、`background_color`、`border_color`
/// - 排版相关：`font_size`、`font_weight`、`text_align`、`line_height` 等
/// - 边框相关：`border_width`、`border_radius`、`border_style` 等
#[derive(Debug, Clone, Default)]
pub struct ParsedStyle {
    /// 元素宽度（像素），-1.0 表示 "full"（100%）
    pub width: Option<f32>,
    /// 元素最大宽度约束（像素）
    pub max_width: Option<f32>,
    /// 元素高度（像素），-1.0 表示 "full"（100%）
    pub height: Option<f32>,
    /// 四周内边距（像素）
    pub padding: Option<f32>,
    /// 顶部内边距
    pub padding_top: Option<f32>,
    /// 底部内边距
    pub padding_bottom: Option<f32>,
    /// 左侧内边距
    pub padding_left: Option<f32>,
    /// 右侧内边距
    pub padding_right: Option<f32>,
    /// 四周外边距（像素）
    pub margin: Option<f32>,
    /// 顶部外边距
    pub margin_top: Option<f32>,
    /// 底部外边距
    pub margin_bottom: Option<f32>,
    /// 左侧外边距
    pub margin_left: Option<f32>,
    /// 右侧外边距
    pub margin_right: Option<f32>,
    /// 水平间隙（用于 flex/grid 布局）
    pub gap_x: Option<f32>,
    /// 垂直间隙（用于 flex/grid 布局）
    pub gap_y: Option<f32>,
    /// 边框圆角半径
    pub border_radius: Option<f32>,
    /// 文本颜色
    pub text_color: Option<Color>,
    /// 背景颜色
    pub background_color: Option<Color>,
    /// 字体大小（像素）
    pub font_size: Option<f32>,
    /// 字体粗细（100-900）
    pub font_weight: Option<u16>,
    /// 文本对齐方式：left、center、right、justify
    pub text_align: Option<String>,
    /// 文本方向：ltr、rtl
    pub text_direction: Option<String>,
    /// 弹性布局方向："row" 或 "column"
    pub flex_direction: Option<String>,
    /// 交叉轴对齐方式：flex-start、center、flex-end 等
    pub align_items: Option<String>,
    /// 主轴对齐方式：flex-start、center、space-between 等
    pub justify_content: Option<String>,
    /// flex-grow 系数
    pub flex_grow: Option<f32>,
    /// flex-shrink 系数
    pub flex_shrink: Option<f32>,
    /// flex-basis 首选主轴尺寸（像素）
    pub flex_basis: Option<f32>,
    /// 显示类型：flex、block、inline、none 等
    pub display: Option<String>,
    /// 网格列数
    pub grid_cols: Option<usize>,
    /// 定位方式：relative、absolute 等
    pub position: Option<String>,
    /// 顶部偏移值
    pub top: Option<f32>,
    /// 左侧偏移值
    pub left: Option<f32>,
    /// 右侧偏移值
    pub right: Option<f32>,
    /// 底部偏移值
    pub bottom: Option<f32>,
    /// 文本装饰：underline、line-through、none
    pub text_decoration: Option<String>,
    /// 字体样式：italic、normal
    pub font_style: Option<String>,
    /// 字母间距（像素）
    pub letter_spacing: Option<f32>,
    /// 行高（倍数）
    pub line_height: Option<f32>,
    /// 不透明度（0.0-1.0）
    pub opacity: Option<f32>,
    /// 水平位移（像素）
    pub translate_x: Option<f32>,
    /// 垂直位移（像素）
    pub translate_y: Option<f32>,
    /// 文本转换：uppercase、lowercase、capitalize
    pub text_transform: Option<String>,
    /// 四周边框宽度
    pub border_width: Option<f32>,
    /// 顶部边框宽度
    pub border_top_width: Option<f32>,
    /// 右侧边框宽度
    pub border_right_width: Option<f32>,
    /// 底部边框宽度
    pub border_bottom_width: Option<f32>,
    /// 左侧边框宽度
    pub border_left_width: Option<f32>,
    /// 行首边框宽度（逻辑属性，依赖书写方向）
    pub border_inline_start_width: Option<f32>,
    /// 行尾边框宽度（逻辑属性，依赖书写方向）
    pub border_inline_end_width: Option<f32>,
    /// 边框颜色
    pub border_color: Option<Color>,
    /// 水平分割线宽度
    pub divide_x_width: Option<f32>,
    /// 垂直分割线宽度
    pub divide_y_width: Option<f32>,
    /// 水平分割线是否反转
    pub divide_x_reverse: bool,
    /// 垂直分割线是否反转
    pub divide_y_reverse: bool,
    /// 边框样式：solid、dashed、dotted、double、hidden、none
    pub border_style: Option<String>,
    /// 阴影颜色（包含透明度）
    pub shadow_color: Option<Color>,
    /// 阴影水平偏移
    pub shadow_offset_x: Option<f32>,
    /// 阴影垂直偏移
    pub shadow_offset_y: Option<f32>,
    /// 阴影扩张半径
    pub shadow_spread: Option<f32>,
    /// outline 宽度
    pub outline_width: Option<f32>,
    /// outline 颜色
    pub outline_color: Option<Color>,
    /// outline 样式：solid、dashed、dotted、double、none
    pub outline_style: Option<String>,
    /// outline 偏移
    pub outline_offset: Option<f32>,
}

/// Tailwind token 在静态画布上的支持分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TailwindTokenSupport {
    /// 变体前缀被压平成静态快照后参与了画布预览。
    FlattenedVariant,
    /// token 会在导出时保留，但不会进入静态画布预览。
    ExportOnly,
    /// token 当前既不参与画布预览，也未进入已知的显式降级分支。
    Unsupported,
}

/// 单个 Tailwind token 的静态画布分析结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TailwindTokenIssue {
    pub original_class: String,
    pub normalized_class: Option<String>,
    pub support: TailwindTokenSupport,
    pub reason: &'static str,
}

/// Tailwind 类名字符串的完整分析结果。
#[derive(Debug, Clone, Default)]
pub struct TailwindParseAnalysis {
    pub style: ParsedStyle,
    pub issues: Vec<TailwindTokenIssue>,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
