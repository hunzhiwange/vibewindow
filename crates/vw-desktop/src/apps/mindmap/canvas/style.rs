//! 思维导图画布样式模块
//!
//! 本模块提供思维导图画布渲染相关的样式计算功能，包括：
//! - 节点边框宽度计算
//! - 颜色格式转换（RGBA 到 iced::Color）
//! - 优先级颜色映射
//! - 背景文本颜色对比度计算
//! - 边线条纹样式与颜色生成
//!
//! 所有样式相关的计算逻辑都集中在此模块中，便于统一管理和维护。

use crate::apps::mindmap::state::EdgeStyle;
use iced::Color;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 虚线样式的线段长度（像素）
const DASH_SEGMENT_LENGTH: f32 = 12.0;

/// 点线样式的线段长度（像素）
const DOT_SEGMENT_LENGTH: f32 = 3.5;

/// 默认边框描边色的 RGBA 值（浅灰色，完全不透明）
pub(super) const DEFAULT_STROKE_RGBA: u32 = 0xD0D7DEFF;

/// 虚线样式的线段模式：[线段长度, 间隔长度]
const DASH_PATTERN: [f32; 2] = [DASH_SEGMENT_LENGTH, DASH_SEGMENT_LENGTH];

/// 点线样式的线段模式：[点长度, 间隔长度]
const DOT_PATTERN: [f32; 2] = [DOT_SEGMENT_LENGTH, DOT_SEGMENT_LENGTH * 3.0];

/// 背景面板块的默认宽度（像素）
#[allow(dead_code)]
pub(super) const BG_PANEL_BLOCK_W: f32 = 360.0;

/// 背景面板块的默认高度（像素）
#[allow(dead_code)]
pub(super) const BG_PANEL_BLOCK_H: f32 = 250.0;

/// 背景面板块的默认外边距（像素）
#[allow(dead_code)]
pub(super) const BG_PANEL_BLOCK_MARGIN: f32 = 14.0;

/// 根据缩放级别计算节点边框宽度
///
/// 边框宽度随缩放级别线性缩放，但被限制在 [1.0, 2.0] 范围内，
/// 然后乘以 2 得到最终像素宽度。这确保了在不同缩放级别下
/// 边框既不会太细（难以看清）也不会太粗（影响美观）。
///
/// # 参数
///
/// * `zoom` - 当前画布的缩放级别（1.0 = 100%）
///
/// # 返回值
///
/// 返回计算后的边框像素宽度
///
/// # 示例
///
/// ```ignore
/// let width_100 = node_border_width_px(1.0);   // 2.0
/// let width_50 = node_border_width_px(0.5);    // 2.0 (被 clamp 到 1.0 * 2.0)
/// let width_200 = node_border_width_px(2.0);   // 4.0
/// ```
pub(super) fn node_border_width_px(zoom: f32) -> f32 {
    (1.0 * zoom).clamp(1.0, 2.0) * 2.0
}

/// 将 32 位无符号整数形式的 RGBA 颜色转换为 iced::Color
///
/// 输入格式为 0xRRGGBBAA，其中：
/// - RR: 红色通道（0-255）
/// - GG: 绿色通道（0-255）
/// - BB: 蓝色通道（0-255）
/// - AA: Alpha 通道（0-255，0 为完全透明，255 为完全不透明）
///
/// # 参数
///
/// * `rgba` - 32 位 RGBA 颜色值，格式为 0xRRGGBBAA
///
/// # 返回值
///
/// 返回对应的 iced::Color 实例，其中 RGB 分量范围为 [0.0, 1.0]
///
/// # 示例
///
/// ```ignore
/// // 红色，不透明
/// let red = rgba_u32_to_color(0xFF0000FF);
/// // 半透明白色
/// let semi_white = rgba_u32_to_color(0xFFFFFF80);
/// ```
pub(super) fn rgba_u32_to_color(rgba: u32) -> Color {
    // 提取红色通道（最高 8 位）
    let r = ((rgba >> 24) & 0xFF) as u8;
    // 提取绿色通道（第 16-23 位）
    let g = ((rgba >> 16) & 0xFF) as u8;
    // 提取蓝色通道（第 8-15 位）
    let b = ((rgba >> 8) & 0xFF) as u8;
    // 提取 Alpha 通道（最低 8 位）
    let a = (rgba & 0xFF) as u8;
    // 将 Alpha 从 [0, 255] 转换为 [0.0, 1.0] 范围
    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}

/// 根据优先级级别返回对应的颜色
///
/// 优先级从 1 到 10，颜色从暖色（红/橙）逐渐过渡到冷色（蓝/紫）：
/// - 1-2: 红/橙色（最高优先级，紧急）
/// - 3-4: 黄/金色（高优先级）
/// - 5-6: 绿/青色（中等优先级）
/// - 7-8: 蓝/靛色（低优先级）
/// - 9-10: 紫/绿色（最低优先级）
///
/// # 参数
///
/// * `level` - 优先级级别（1-10），超出范围返回灰色
///
/// # 返回值
///
/// 返回对应优先级的完全不透明颜色
///
/// # 示例
///
/// ```ignore
/// let urgent = priority_color(1);  // 红色
/// let normal = priority_color(5);  // 绿色
/// let low = priority_color(8);     // 靛蓝色
/// ```
pub(super) fn priority_color(level: u8) -> Color {
    match level {
        1 => Color::from_rgba8(239, 68, 68, 1.0), // 红色 - 最高优先级
        2 => Color::from_rgba8(249, 115, 22, 1.0), // 橙色
        3 => Color::from_rgba8(245, 158, 11, 1.0), // 琥珀色
        4 => Color::from_rgba8(234, 179, 8, 1.0), // 黄色
        5 => Color::from_rgba8(34, 197, 94, 1.0), // 绿色
        6 => Color::from_rgba8(20, 184, 166, 1.0), // 青色
        7 => Color::from_rgba8(59, 130, 246, 1.0), // 蓝色
        8 => Color::from_rgba8(99, 102, 241, 1.0), // 靛蓝色
        9 => Color::from_rgba8(168, 85, 247, 1.0), // 紫色
        10 => Color::from_rgba8(34, 197, 94, 1.0), // 绿色 - 最低优先级
        _ => Color::from_rgba8(107, 114, 128, 1.0), // 灰色 - 无效级别
    }
}

/// 根据背景色计算最佳的文本颜色（黑或白）
///
/// 使用 ITU-R BT.601 标准的亮度公式计算背景色的感知亮度：
/// `Luma = 0.299 * R + 0.587 * G + 0.114 * B`
///
/// 如果背景亮度大于 0.72（较亮），返回深色文本；
/// 否则返回白色文本。这确保了文本与背景之间有足够的对比度。
///
/// # 参数
///
/// * `bg` - 背景颜色
///
/// # 返回值
///
/// 返回深灰色（#111827）或白色，取决于背景亮度
///
/// # 示例
///
/// ```ignore
/// let text_on_white = ideal_text_color(Color::WHITE);  // 深灰色
/// let text_on_black = ideal_text_color(Color::BLACK);  // 白色
/// ```
pub(super) fn ideal_text_color(bg: Color) -> Color {
    // 使用加权亮度公式计算感知亮度
    // 绿色权重最高（人眼对绿色最敏感）
    let luma = 0.299 * bg.r + 0.587 * bg.g + 0.114 * bg.b;
    // 亮度阈值 0.72：高于此值用深色文本，低于此值用白色文本
    if luma > 0.72 { Color::from_rgba8(17, 24, 39, 1.0) } else { Color::WHITE }
}

/// 根据边的路径生成默认的边线颜色
///
/// 使用路径的哈希值生成一个 0-359 范围内的色相值，
/// 然后转换为具有固定饱和度（0.78）和明度（0.92）的 RGB 颜色。
/// 这确保了相同路径始终获得相同的颜色，同时不同路径获得不同颜色。
///
/// # 参数
///
/// * `path` - 边的路径，包含从源节点到目标节点的节点 ID 序列
///
/// # 返回值
///
/// 返回基于路径哈希的鲜艳颜色
///
/// # 示例
///
/// ```ignore
/// let color1 = default_edge_stroke_color(&[1, 2, 3]);
/// let color2 = default_edge_stroke_color(&[1, 2, 3]); // 与 color1 相同
/// let color3 = default_edge_stroke_color(&[4, 5]);    // 不同的颜色
/// ```
#[allow(dead_code)]
pub(super) fn default_edge_stroke_color(path: &[usize]) -> Color {
    // 使用默认哈希器计算路径的哈希值
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    // 将哈希值映射到色相环（0-359 度）
    let h = (hasher.finish() % 360) as f32;
    // 转换为具有固定饱和度和明度的 RGB 颜色
    hsv_to_rgb(h, 0.78, 0.92)
}

/// 将 HSV 颜色空间转换为 RGB 颜色空间
///
/// HSV 到 RGB 的转换算法：
/// 1. 根据色相确定颜色所在的 60 度扇区
/// 2. 计算中间值 X（基于色相在扇区内的位置）
/// 3. 根据扇区选择 (R1, G1, B1) 的组合
/// 4. 加上明度偏移量 M 得到最终 RGB 值
///
/// # 参数
///
/// * `h` - 色相（0-360 度），超出范围会被取模
/// * `s` - 饱和度（0.0-1.0），超出范围会被 clamp
/// * `v` - 明度/亮度（0.0-1.0），超出范围会被 clamp
///
/// # 返回值
///
/// 返回对应的 RGB 颜色（Alpha = 1.0）
///
/// # 示例
///
/// ```ignore
/// let red = hsv_to_rgb(0.0, 1.0, 1.0);    // 红色
/// let green = hsv_to_rgb(120.0, 1.0, 1.0); // 绿色
/// let blue = hsv_to_rgb(240.0, 1.0, 1.0);  // 蓝色
/// ```
#[allow(dead_code)]
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    // 规范化输入：色相取模，饱和度和明度限制在有效范围内
    let h = h.rem_euclid(360.0);
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);

    // 色度（Chroma）：颜色的纯度
    let c = v * s;
    // 中间值：用于计算扇区内的颜色变化
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    // 明度偏移量：用于调整最终亮度
    let m = v - c;

    // 根据色相所在的 60 度扇区选择 RGB 分量
    // 每个扇区对应色相环上的一种主色过渡
    let (r1, g1, b1) = match h {
        h if h < 60.0 => (c, x, 0.0),  // 红 -> 黄
        h if h < 120.0 => (x, c, 0.0), // 黄 -> 绿
        h if h < 180.0 => (0.0, c, x), // 绿 -> 青
        h if h < 240.0 => (0.0, x, c), // 青 -> 蓝
        h if h < 300.0 => (x, 0.0, c), // 蓝 -> 品红
        _ => (c, 0.0, x),              // 品红 -> 红
    };

    // 加上明度偏移量得到最终 RGB 值
    Color::from_rgb(r1 + m, g1 + m, b1 + m)
}

/// 根据边样式返回虚线线段模式
///
/// 返回一个线段长度数组，用于指定虚线或点线的绘制模式。
/// 数组中的值交替表示：[线段长度, 间隔长度, 线段长度, 间隔长度, ...]
///
/// # 参数
///
/// * `style` - 边的样式（Solid、Dashed 或 Dotted）
/// * `_zoom` - 缩放级别（当前未使用，保留用于未来扩展）
///
/// # 返回值
///
/// - `Some(&[f32])` - 虚线或点线的线段模式
/// - `None` - 实线样式，无需虚线模式
///
/// # 示例
///
/// ```ignore
/// let solid = dash_segments_px(EdgeStyle::Solid, 1.0);   // None
/// let dashed = dash_segments_px(EdgeStyle::Dashed, 1.0); // Some(&[12.0, 12.0])
/// let dotted = dash_segments_px(EdgeStyle::Dotted, 1.0); // Some(&[3.5, 10.5])
/// ```
pub(crate) fn dash_segments_px(style: EdgeStyle, _zoom: f32) -> Option<&'static [f32]> {
    match style {
        EdgeStyle::Dashed => Some(&DASH_PATTERN), // 虚线：[12.0, 12.0]
        EdgeStyle::Dotted => Some(&DOT_PATTERN),  // 点线：[3.5, 10.5]
        EdgeStyle::Solid => None,                 // 实线：无虚线模式
    }
}
