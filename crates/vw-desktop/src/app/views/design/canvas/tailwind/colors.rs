//! Tailwind CSS 颜色与设计系统常量
//!
//! 本模块提供了基于 Tailwind CSS 设计系统的颜色、间距、圆角和字体大小常量。
//! 这些常量用于在 Iced UI 框架中保持一致的视觉风格和设计规范。
//!
//! # 主要功能
//!
//! - **颜色常量**：提供 Tailwind 调色板中的常用颜色，包括天蓝、蓝、红、灰、靛蓝等色系
//! - **间距常量**：基于 Tailwind 的 4px 基准间距系统
//! - **圆角常量**：从超小（xs）到完整圆角（full）的圆角尺寸
//! - **字体大小常量**：常用的文本尺寸规格
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::app::views::design::canvas::tailwind::colors::TailwindColors;
//! use iced::Color;
//!
//! // 使用颜色常量
//! let primary_color = TailwindColors::INDIGO_600;
//! let background_color = TailwindColors::WHITE;
//!
//! // 使用间距常量
//! let padding = TailwindColors::SPACING_8;
//!
//! // 使用圆角常量
//! let border_radius = TailwindColors::ROUNDED_LG;
//! ```

use iced::Color;

/// Tailwind CSS 设计系统常量集合
///
/// 该结构体包含所有与 Tailwind CSS 设计系统相关的常量，
/// 包括颜色、间距、圆角和字体大小等。所有成员都是常量，
/// 可以直接通过结构体访问，无需实例化。
///
/// # 设计原则
///
/// - 颜色值遵循 Tailwind CSS 默认调色板
/// - 间距基于 4px 基准单位（0.25rem）
/// - 圆角值与 Tailwind 的圆角类对应
/// - 字体大小使用像素单位
pub struct TailwindColors;

impl TailwindColors {
    /// 有限 spacing scale token，供 parser 与 classes 共同复用。
    pub const SPACING_SCALE_TOKENS: &[&str] = &[
        "0", "px", "1", "2", "3", "4", "5", "6", "7", "8", "10", "12", "16", "20", "24", "32",
        "40", "48", "56", "64",
    ];

    /// 当前 parser 支持的 grid 列数集合。
    pub const GRID_COLUMN_VALUES: &[usize] = &[1, 2, 3, 4, 5, 6, 12];

    /// 当前 `bg-*` 真正支持的颜色 token。
    pub const BACKGROUND_COLOR_TOKENS: &[&str] = &[
        "white",
        "black",
        "transparent",
        "gray-100",
        "gray-500",
        "gray-600",
        "gray-800",
        "red-500",
        "blue-100",
        "blue-500",
        "green-500",
        "yellow-500",
        "indigo-500",
        "indigo-600",
        "sky-500",
    ];

    /// 当前 `text-*` 真正支持的颜色 token。
    pub const TEXT_COLOR_TOKENS: &[&str] = Self::BACKGROUND_COLOR_TOKENS;

    /// 当前 `border-*` 真正支持的颜色 token。
    pub const BORDER_COLOR_TOKENS: &[&str] = Self::BACKGROUND_COLOR_TOKENS;

    // ================================
    // 颜色常量
    // ================================

    /// 天蓝色 500 - 明亮的天蓝色
    ///
    /// RGB: (15, 165, 233) / 十六进制: #0ea5e9
    ///
    /// 适用于强调元素、链接或次要操作按钮
    pub const SKY_500: Color = Color::from_rgb(0.06, 0.73, 0.98);

    /// 蓝色 500 - 标准蓝色
    ///
    /// RGB: (59, 130, 246) / 十六进制: #3b82f6
    ///
    /// 适用于主要操作按钮、链接或信息提示
    pub const BLUE_500: Color = Color::from_rgb(0.23, 0.51, 0.96);

    /// 红色 500 - 标准红色
    ///
    /// RGB: (239, 68, 68) / 十六进制: #ef4444
    ///
    /// 适用于错误提示、危险操作按钮或警告信息
    pub const RED_500: Color = Color::from_rgb(0.93, 0.26, 0.26);

    /// 绿色 500 - 标准绿色
    pub const GREEN_500: Color = Color::from_rgb8(34, 197, 94);

    /// 黄色 500 - 标准黄色
    pub const YELLOW_500: Color = Color::from_rgb8(234, 179, 8);

    /// 蓝色 100 - 浅蓝色
    ///
    /// RGB: (219, 234, 254) / 十六进制: #dbeafe
    ///
    /// 适用于背景色、悬停状态或信息区域背景
    pub const BLUE_100: Color = Color::from_rgb(0.859, 0.918, 0.996);

    /// 灰色 100 - 极浅灰色
    ///
    /// RGB: (243, 244, 246) / 十六进制: #f3f4f6
    ///
    /// 适用于页面背景、分隔区域或禁用状态背景
    pub const GRAY_100: Color = Color::from_rgb(0.957, 0.965, 0.973);

    /// 灰色 500 - 中等灰色
    ///
    /// RGB: (107, 114, 128) / 十六进制: #6b7280
    ///
    /// 适用于次要文本、图标或边框
    pub const GRAY_500: Color = Color::from_rgb(0.42, 0.447, 0.49);

    /// 灰色 600 - 深灰色
    ///
    /// RGB: (112, 128, 143) / 十六进制: #70808f
    ///
    /// 适用于普通文本、次要标题或辅助信息
    pub const GRAY_600: Color = Color::from_rgb(0.44, 0.50, 0.56);

    /// 灰色 800 - 深色灰色
    ///
    /// RGB: (31, 41, 55) / 十六进制: #1f2937
    ///
    /// 适用于深色背景、标题文本或高对比度元素
    pub const GRAY_800: Color = Color::from_rgb(0.122, 0.141, 0.169);

    /// 靛蓝色 500 - 标准靛蓝色
    ///
    /// RGB: (99, 102, 241) / 十六进制: #6366f1
    ///
    /// 适用于品牌色、主要操作或特殊强调
    pub const INDIGO_500: Color = Color::from_rgb(0.392, 0.38, 0.925);

    /// 靛蓝色 600 - 深靛蓝色
    ///
    /// RGB: (79, 70, 229) / 十六进制: #4f46e5
    ///
    /// 适用于主要品牌色、活跃状态或重要交互元素
    pub const INDIGO_600: Color = Color::from_rgb8(79, 70, 229);

    /// 纯白色
    ///
    /// RGB: (255, 255, 255) / 十六进制: #ffffff
    ///
    /// 适用于背景、文本（在深色背景上）或高亮元素
    pub const WHITE: Color = Color::from_rgb(1.0, 1.0, 1.0);

    /// 纯黑色
    ///
    /// RGB: (0, 0, 0) / 十六进制: #000000
    ///
    /// 适用于文本（在浅色背景上）、边框或阴影
    pub const BLACK: Color = Color::from_rgb(0.0, 0.0, 0.0);

    /// 透明色
    pub const TRANSPARENT: Color = Color::TRANSPARENT;

    // ================================
    // 间距常量（基于 Tailwind 的 0.25rem = 4px 比例）
    // ================================

    /// 间距 6 - 24 像素（1.5rem）
    ///
    /// 适用于中等间距的内边距或外边距
    pub const SPACING_6: f32 = 24.0;

    /// 间距 7 - 28 像素（1.75rem）
    ///
    /// 适用于较大间距的内边距或外边距
    pub const SPACING_7: f32 = 28.0;

    /// 间距 8 - 32 像素（2rem）
    ///
    /// 适用于大间距的内边距或外边距，常用于章节分隔
    pub const SPACING_8: f32 = 32.0;

    /// 间距 10 - 40 像素（2.5rem）
    ///
    /// 适用于较大区域的内边距或外边距
    pub const SPACING_10: f32 = 40.0;

    /// 间距 12 - 48 像素（3rem）
    ///
    /// 适用于大区域分隔的内边距或外边距
    pub const SPACING_12: f32 = 48.0;

    /// 间距 16 - 64 像素（4rem）
    ///
    /// 适用于页面级别的大间距分隔
    pub const SPACING_16: f32 = 64.0;

    // ================================
    // 圆角常量
    // ================================

    /// 超小圆角 - 2 像素（0.125rem）
    ///
    /// 适用于细微的视觉变化，保持几乎直角的边角
    pub const ROUNDED_XS: f32 = 2.0;

    /// 小圆角 - 4 像素（0.25rem）
    ///
    /// 适用于按钮、输入框等小元素的轻微圆角
    pub const ROUNDED_SM: f32 = 4.0;

    /// 基础圆角 - 4 像素
    ///
    /// Tailwind 默认圆角值，适用于大多数普通元素
    pub const ROUNDED_BASE: f32 = 4.0;

    /// 中等圆角 - 6 像素
    ///
    /// 适用于卡片、面板等中等大小元素
    pub const ROUNDED_MD: f32 = 6.0;

    /// 大圆角 - 8 像素
    ///
    /// 适用于较大的卡片、对话框或突出显示的元素
    pub const ROUNDED_LG: f32 = 8.0;

    /// 超大圆角 - 12 像素
    ///
    /// 适用于大型容器、特殊强调的 UI 元素
    pub const ROUNDED_XL: f32 = 12.0;

    /// 2倍超大圆角 - 16 像素
    ///
    /// 适用于弹出层、模态框或需要显著视觉区分的元素
    pub const ROUNDED_2XL: f32 = 16.0;

    /// 3倍超大圆角 - 24 像素
    ///
    /// 适用于特殊设计效果或大型展示元素
    pub const ROUNDED_3XL: f32 = 24.0;

    /// 4倍超大圆角 - 32 像素
    ///
    /// 适用于极端设计需求或特殊视觉效果
    pub const ROUNDED_4XL: f32 = 32.0;

    /// 完整圆角 - 9999 像素
    ///
    /// 创建完全圆形或胶囊形的元素，适用于头像、徽章等
    pub const ROUNDED_FULL: f32 = 9999.0;

    // ================================
    // 字体大小常量
    // ================================

    /// 小号文本 - 14 像素
    ///
    /// 适用于辅助文本、说明文字或次要信息
    pub const TEXT_SM: f32 = 14.0;

    /// 基础文本 - 16 像素
    ///
    /// 适用于正文内容、默认文本大小
    pub const TEXT_BASE: f32 = 16.0;

    /// 大号文本 - 18 像素
    ///
    /// 适用于小标题、重要段落或需要强调的文本
    pub const TEXT_LG: f32 = 18.0;

    /// 超大文本 - 20 像素
    ///
    /// 适用于副标题、重要标题或突出显示的文本
    pub const TEXT_XL: f32 = 20.0;

    /// 2倍超大文本 - 24 像素
    ///
    /// 适用于页面标题、主要标题或重要标识
    pub const TEXT_2XL: f32 = 24.0;

    /// 3倍超大文本 - 30 像素
    ///
    /// 适用于大型标题、营销文案或需要极大视觉冲击的文本
    pub const TEXT_3XL: f32 = 30.0;

    pub fn resolve_color_token(token: &str) -> Option<Color> {
        match token {
            "white" => Some(Self::WHITE),
            "black" => Some(Self::BLACK),
            "transparent" => Some(Self::TRANSPARENT),
            "gray-100" => Some(Self::GRAY_100),
            "gray-500" => Some(Self::GRAY_500),
            "gray-600" => Some(Self::GRAY_600),
            "gray-800" => Some(Self::GRAY_800),
            "red-500" => Some(Self::RED_500),
            "blue-100" => Some(Self::BLUE_100),
            "blue-500" => Some(Self::BLUE_500),
            "green-500" => Some(Self::GREEN_500),
            "yellow-500" => Some(Self::YELLOW_500),
            "indigo-500" => Some(Self::INDIGO_500),
            "indigo-600" => Some(Self::INDIGO_600),
            "sky-500" => Some(Self::SKY_500),
            _ => None,
        }
    }

    pub fn resolve_background_color(token: &str) -> Option<Color> {
        Self::resolve_scoped_color(token, Self::BACKGROUND_COLOR_TOKENS)
    }

    pub fn resolve_text_color(token: &str) -> Option<Color> {
        Self::resolve_scoped_color(token, Self::TEXT_COLOR_TOKENS)
    }

    pub fn resolve_border_color(token: &str) -> Option<Color> {
        Self::resolve_scoped_color(token, Self::BORDER_COLOR_TOKENS)
    }

    fn resolve_scoped_color(token: &str, supported_tokens: &[&str]) -> Option<Color> {
        if supported_tokens.contains(&token) { Self::resolve_color_token(token) } else { None }
    }
}

#[cfg(test)]
#[path = "colors_tests.rs"]
mod colors_tests;
