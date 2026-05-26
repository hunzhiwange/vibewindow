//! 思维导图视图通用工具模块
//!
//! 本模块提供思维导图视图层的通用工具函数，用于颜色处理和样式配置。
//!
//! # 主要功能
//!
//! - **颜色转换**：将 RGBA 整数值转换为 iced Color 类型
//! - **优先级颜色映射**：根据优先级级别返回对应的颜色
//! - **文本颜色计算**：根据背景亮度自动选择最佳文本颜色
//! - **容器样式配置**：提供选择器和基础容器的默认样式

use iced::{Border, Color, Theme};

/// 将 RGBA 格式的 u32 整数转换为 iced Color
///
/// 该函数将 32 位无符号整数解析为 RGBA 颜色分量，并转换为 iced 框架的 Color 类型。
/// RGBA 格式在整数中的布局为：最高位字节是红色，依次为绿色、蓝色，最低位字节是透明度。
///
/// # 参数
///
/// - `rgba`：包含 RGBA 颜色信息的 32 位无符号整数
///   - 位 24-31：红色分量 (R)
///   - 位 16-23：绿色分量 (G)
///   - 位 8-15：蓝色分量 (B)
///   - 位 0-7：透明度分量 (A)
///
/// # 返回值
///
/// 返回转换后的 iced `Color` 实例
///
/// # 示例
///
/// ```ignore
/// // 红色（R=255, G=0, B=0, A=255）的不透明颜色
/// let color = rgba_u32_to_color(0xFF_00_00_FF);
/// ```
pub(super) fn rgba_u32_to_color(rgba: u32) -> Color {
    // 提取红色分量：右移 24 位，取低 8 位
    let r = ((rgba >> 24) & 0xFF) as u8;
    // 提取绿色分量：右移 16 位，取低 8 位
    let g = ((rgba >> 16) & 0xFF) as u8;
    // 提取蓝色分量：右移 8 位，取低 8 位
    let b = ((rgba >> 8) & 0xFF) as u8;
    // 提取透明度分量：直接取低 8 位
    let a = (rgba & 0xFF) as u8;
    // 将透明度从 0-255 范围归一化到 0.0-1.0 浮点范围
    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}

/// 根据优先级级别返回对应的颜色
///
/// 该函数将 1-10 的优先级级别映射到预定义的颜色方案中，
/// 颜色从红色（高优先级）逐渐过渡到绿色（低优先级）。
/// 未知级别默认返回灰色。
///
/// # 参数
///
/// - `level`：优先级级别（1-10），数值越小优先级越高
///
/// # 返回值
///
/// 返回对应优先级的 iced `Color` 实例
///
/// # 颜色映射
///
/// | 级别 | 颜色 | 说明 |
/// |------|------|------|
/// | 1 | 红色 | 最高优先级 |
/// | 2 | 橙色 | 非常高优先级 |
/// | 3 | 琥珀色 | 高优先级 |
/// | 4 | 黄色 | 中高优先级 |
/// | 5 | 绿色 | 中等优先级 |
/// | 6 | 青色 | 中低优先级 |
/// | 7 | 蓝色 | 低优先级 |
/// | 8 | 靛蓝色 | 较低优先级 |
/// | 9 | 紫色 | 非常低优先级 |
/// | 10 | 绿色 | 最低优先级 |
/// | 其他 | 灰色 | 未知级别 |
///
/// # 示例
///
/// ```ignore
/// let high_priority_color = priority_color(1);  // 返回红色
/// let low_priority_color = priority_color(7);   // 返回蓝色
/// ```
pub(super) fn priority_color(level: u8) -> Color {
    match level {
        // 最高优先级 - 红色（危险、紧急）
        1 => Color::from_rgba8(239, 68, 68, 1.0),
        // 非常高优先级 - 橙色
        2 => Color::from_rgba8(249, 115, 22, 1.0),
        // 高优先级 - 琥珀色
        3 => Color::from_rgba8(245, 158, 11, 1.0),
        // 中高优先级 - 黄色
        4 => Color::from_rgba8(234, 179, 8, 1.0),
        // 中等优先级 - 绿色
        5 => Color::from_rgba8(34, 197, 94, 1.0),
        // 中低优先级 - 青色
        6 => Color::from_rgba8(20, 184, 166, 1.0),
        // 低优先级 - 蓝色
        7 => Color::from_rgba8(59, 130, 246, 1.0),
        // 较低优先级 - 靛蓝色
        8 => Color::from_rgba8(99, 102, 241, 1.0),
        // 非常低优先级 - 紫色
        9 => Color::from_rgba8(168, 85, 247, 1.0),
        // 最低优先级 - 绿色（表示无压力）
        10 => Color::from_rgba8(34, 197, 94, 1.0),
        // 未知级别 - 灰色
        _ => Color::from_rgba8(107, 114, 128, 1.0),
    }
}

/// 根据背景颜色计算最佳文本颜色
///
/// 该函数使用亮度公式计算背景颜色的感知亮度，并根据亮度值自动选择
/// 深色或浅色文本，以确保文本在背景上具有良好的可读性。
///
/// # 算法说明
///
/// 使用 ITU-R BT.601 标准的亮度计算公式：
/// `luma = 0.299 * R + 0.587 * G + 0.114 * B`
///
/// 该公式考虑了人眼对不同颜色的敏感度差异（绿色最敏感，蓝色最不敏感）。
///
/// # 参数
///
/// - `bg`：背景颜色的 iced `Color` 实例
///
/// # 返回值
///
/// - 如果背景亮度 > 0.72：返回深灰色文本（RGB: 17, 24, 39）
/// - 如果背景亮度 ≤ 0.72：返回白色文本
///
/// # 示例
///
/// ```ignore
/// let white_bg = Color::from_rgba8(255, 255, 255, 1.0);
/// let text_color = ideal_text_color(white_bg);  // 返回深灰色文本
///
/// let black_bg = Color::from_rgba8(0, 0, 0, 1.0);
/// let text_color = ideal_text_color(black_bg);  // 返回白色文本
/// ```
pub(super) fn ideal_text_color(bg: Color) -> Color {
    // 使用亮度公式计算感知亮度（绿色权重最大，蓝色最小）
    let luma = 0.299 * bg.r + 0.587 * bg.g + 0.114 * bg.b;
    // 如果背景较亮，使用深色文本；否则使用白色文本
    if luma > 0.72 { Color::from_rgba8(17, 24, 39, 1.0) } else { Color::WHITE }
}

/// 生成选择器容器样式
///
/// 该函数为颜色选择器、下拉菜单等弹出式组件提供统一的容器样式，
/// 包含背景色和圆角边框。
///
/// # 参数
///
/// - `theme`：iced 主题引用，用于获取调色板颜色
///
/// # 返回值
///
/// 返回配置好的容器样式，包含：
/// - 背景：使用主题的基础背景色
/// - 边框：1px 宽度，使用主题的弱背景色，10px 圆角
///
/// # 示例
///
/// ```ignore
/// let style = picker_style(&theme);
/// // 应用到容器组件
/// container(content).style(picker_style);
/// ```
#[allow(dead_code)]
pub(super) fn picker_style(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        // 使用主题的基础背景色作为容器背景
        background: Some(theme.extended_palette().background.base.color.into()),
        // 配置圆角边框
        border: Border {
            width: 1.0,
            color: theme.extended_palette().background.weak.color,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

/// 生成基础容器样式
///
/// 该函数提供简单的基础容器样式，仅包含背景色，无边框装饰。
/// 适用于需要透明或简洁外观的容器组件。
///
/// # 参数
///
/// - `theme`：iced 主题引用，用于获取调色板颜色
///
/// # 返回值
///
/// 返回配置好的容器样式，包含：
/// - 背景：使用主题的基础背景色
/// - 边框：0px 宽度（无边框），透明色，0px 圆角
///
/// # 示例
///
/// ```ignore
/// let style = base_style(&theme);
/// // 应用到容器组件
/// container(content).style(base_style);
/// ```
pub(super) fn base_style(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        // 使用主题的基础背景色作为容器背景
        background: Some(theme.extended_palette().background.base.color.into()),
        // 无边框配置（完全透明）
        border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 0.0.into() },
        ..Default::default()
    }
}
