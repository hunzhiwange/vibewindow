//! 渐变色工具函数模块
//!
//! 本模块提供处理渐变色相关的辅助工具函数，包括：
//! - 十六进制颜色输入的规范化处理
//! - 百分比值的格式化
//! - 渐变停止点的命中测试
//! - 指定位置的渐变颜色计算
//!
//! 这些函数主要用于渐变色编辑器的交互与渲染逻辑。

use iced::{Color, Point, Rectangle};
use std::cmp::Ordering;

use crate::app::views::design::properties::fill::types::GradientStop;

/// 规范化十六进制颜色输入字符串
///
/// 该函数将用户输入的颜色字符串转换为标准格式的十六进制颜色表示。
/// 支持以下格式的输入：
/// - 3位十六进制（如 `#RGB` 或 `RGB`）→ 转换为 6 位
/// - 4位十六进制（如 `#RGBA` 或 `RGBA`）→ 转换为 8 位
/// - 6位十六进制（如 `#RRGGBB` 或 `RRGGBB`）→ 保持原样
/// - 8位十六进制（如 `#RRGGBBAA` 或 `RRGGBBAA`）→ 保持原样
///
/// # 参数
///
/// * `raw` - 原始颜色字符串输入
///
/// # 返回值
///
/// 返回规范化后的颜色字符串，格式为 `#rrggbb` 或 `#rrggbbaa`。
/// 如果输入不是有效的十六进制颜色，则返回原始输入的 trim 结果。
///
/// # 示例
///
/// ```ignore
/// normalize_hex_color_input("#fff")    // 返回 "#ffffff"
/// normalize_hex_color_input("fff")     // 返回 "#ffffff"
/// normalize_hex_color_input("#abcd")   // 返回 "#aabbccdd"
/// normalize_hex_color_input("red")     // 返回 "red"（非十六进制）
/// ```
pub(super) fn normalize_hex_color_input(raw: &str) -> String {
    // 去除首尾空白字符
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // 分离井号前缀和颜色主体
    let (has_hash, body) = if let Some(stripped) = trimmed.strip_prefix('#') {
        (true, stripped)
    } else {
        (false, trimmed)
    };

    // 转换为小写以便统一处理
    let lower = body.to_ascii_lowercase();

    // 验证是否为有效的十六进制字符
    let is_hex = lower.chars().all(|c| c.is_ascii_hexdigit());
    if !is_hex {
        return trimmed.to_string();
    }

    // 根据长度进行规范化处理
    let normalized = match lower.len() {
        // 3位格式（RGB）→ 扩展为6位（RRGGBB）
        3 => {
            let mut chars = lower.chars();
            let r = chars.next().unwrap();
            let g = chars.next().unwrap();
            let b = chars.next().unwrap();
            format!("{r}{r}{g}{g}{b}{b}")
        }
        // 4位格式（RGBA）→ 扩展为8位（RRGGBBAA）
        4 => {
            let mut chars = lower.chars();
            let r = chars.next().unwrap();
            let g = chars.next().unwrap();
            let b = chars.next().unwrap();
            let a = chars.next().unwrap();
            format!("{r}{r}{g}{g}{b}{b}{a}{a}")
        }
        // 6位或8位格式 → 保持原样
        6 | 8 => lower,
        // 其他长度 → 不进行转换
        _ => {
            if has_hash {
                return format!("#{lower}");
            }
            return trimmed.to_string();
        }
    };

    // 确保输出格式带有井号前缀
    format!("#{normalized}")
}

/// 格式化百分比值字符串
///
/// 将浮点数值格式化为百分比字符串，自动去除不必要的尾随零和小数点。
/// 保留最多两位小数精度。
///
/// # 参数
///
/// * `value` - 要格式化的浮点数值（通常为 0.0 到 100.0 之间）
///
/// # 返回值
///
/// 返回格式化后的字符串，不包含不必要的尾随零。
///
/// # 示例
///
/// ```ignore
/// format_percent(50.0)     // 返回 "50"
/// format_percent(33.33)    // 返回 "33.33"
/// format_percent(25.50)    // 返回 "25.5"
/// format_percent(100.00)   // 返回 "100"
/// ```
pub(super) fn format_percent(value: f64) -> String {
    // 首先格式化为保留两位小数
    let mut s = format!("{:.2}", value);

    // 移除尾随的零
    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }

    // 如果小数点后没有数字，也移除小数点
    if s.ends_with('.') {
        s.pop();
    }
    s
}

/// 对渐变停止点进行命中测试
///
/// 检测鼠标光标位置是否与某个渐变停止点的可交互区域重叠。
/// 每个停止点的交互区域是以停止点位置为中心、半径为9像素的正方形区域。
///
/// # 参数
///
/// * `stops` - 渐变停止点数组引用
/// * `bounds` - 渐变轨道的边界矩形
/// * `cursor` - 鼠标光标的当前位置
///
/// # 返回值
///
/// 如果光标命中某个停止点，返回该停止点在数组中的索引（`Some(index)`）；
/// 否则返回 `None`。
///
/// # 示例
///
/// ```ignore
/// let stops = vec![
///     GradientStop { position: 0.0, color: "#ff0000".to_string() },
///     GradientStop { position: 1.0, color: "#0000ff".to_string() },
/// ];
/// let bounds = Rectangle { x: 0.0, y: 0.0, width: 100.0, height: 20.0 };
/// let cursor = Point { x: 5.0, y: 10.0 };  // 靠近第一个停止点
/// let hit = hit_test_stop(&stops, bounds, cursor);  // 返回 Some(0)
/// ```
pub(super) fn hit_test_stop(
    stops: &[GradientStop],
    bounds: Rectangle,
    cursor: Point,
) -> Option<usize> {
    // 计算轨道中心的 Y 坐标
    let center_y = bounds.height / 2.0;

    // 定义命中检测的半径（9像素的正方形区域）
    let radius = 9.0;

    // 遍历所有停止点进行命中检测
    for (i, stop) in stops.iter().enumerate() {
        // 根据停止点位置比例计算其在轨道上的 X 坐标
        let x = stop.position as f32 * bounds.width;

        // 检测光标是否在停止点的交互区域内
        if (cursor.x - x).abs() <= radius && (cursor.y - center_y).abs() <= radius {
            return Some(i);
        }
    }
    None
}

/// 计算渐变在指定位置的颜色
///
/// 根据渐变停止点数组，通过线性插值计算指定位置 `t` 处的颜色值。
/// `t` 的范围通常为 0.0 到 1.0，表示从渐变起点到终点的位置比例。
///
/// # 参数
///
/// * `stops` - 渐变停止点数组引用，每个停止点包含位置和颜色
/// * `t` - 目标位置比例（0.0 = 起点，1.0 = 终点）
///
/// # 返回值
///
/// 返回位置 `t` 处计算得到的颜色值。
/// - 如果停止点数组为空，返回默认的灰色 `Color::from_rgb(0.8, 0.8, 0.8)`
/// - 如果 `t` 在第一个停止点之前，返回第一个停止点的颜色
/// - 如果 `t` 在最后一个停止点之后，返回最后一个停止点的颜色
/// - 否则在相邻两个停止点之间进行线性插值
///
/// # 示例
///
/// ```ignore
/// let stops = vec![
///     GradientStop { position: 0.0, color: "#ff0000".to_string() },
///     GradientStop { position: 1.0, color: "#0000ff".to_string() },
/// ];
/// let color = gradient_color_at(&stops, 0.5);  // 返回红色和蓝色的混合色
/// ```
pub(super) fn gradient_color_at(stops: &[GradientStop], t: f32) -> Color {
    // 如果没有停止点，返回默认灰色
    if stops.is_empty() {
        return Color::from_rgb(0.8, 0.8, 0.8);
    }

    // 复制停止点并按位置排序
    let mut sorted = stops.to_vec();
    sorted.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap_or(Ordering::Equal));

    let t64 = t as f64;
    let first = &sorted[0];

    // 如果 t 在第一个停止点之前，返回第一个颜色
    if t64 <= first.position {
        return parse_color(&first.color).unwrap_or(Color::BLACK);
    }

    let last = sorted.last().unwrap();

    // 如果 t 在最后一个停止点之后，返回最后一个颜色
    if t64 >= last.position {
        return parse_color(&last.color).unwrap_or(Color::BLACK);
    }

    // 在相邻停止点之间查找并插值
    for pair in sorted.windows(2) {
        let a = &pair[0];
        let b = &pair[1];

        // 找到包含 t 的停止点区间
        if t64 >= a.position && t64 <= b.position {
            // 计算区间的跨度，避免除零
            let span = (b.position - a.position).max(0.0001);

            // 计算 t 在该区间内的局部位置（0.0 到 1.0）
            let local = ((t64 - a.position) / span) as f32;

            // 解析两个端点的颜色
            let c1 = parse_color(&a.color).unwrap_or(Color::BLACK);
            let c2 = parse_color(&b.color).unwrap_or(Color::BLACK);

            // 线性插值计算结果颜色
            return lerp_color(c1, c2, local);
        }
    }

    // 兜底返回最后一个颜色
    parse_color(&last.color).unwrap_or(Color::BLACK)
}

/// 解析十六进制颜色字符串为 Color 对象
///
/// 将十六进制颜色字符串（如 `#ff0000` 或 `#ff000080`）转换为 iced 的 Color 对象。
/// 该函数委托给 solid 模块的 `parse_hex_to_rgba` 函数进行实际解析。
///
/// # 参数
///
/// * `s` - 十六进制颜色字符串
///
/// # 返回值
///
/// 如果解析成功，返回 `Some(Color)`；
/// 如果解析失败，返回 `None`。
fn parse_color(s: &str) -> Option<Color> {
    let (r, g, b, a) = super::super::solid::parse_hex_to_rgba(s);
    Some(Color::from_rgba(r, g, b, a))
}

/// 在两个颜色之间进行线性插值
///
/// 根据插值因子 `t` 在颜色 `a` 和颜色 `b` 之间进行线性混合。
/// 该函数对 RGBA 所有通道分别进行插值计算。
///
/// # 参数
///
/// * `a` - 起始颜色
/// * `b` - 目标颜色
/// * `t` - 插值因子（0.0 = 完全使用 a，1.0 = 完全使用 b）
///
/// # 返回值
///
/// 返回插值后的颜色。`t` 会被自动钳制到 [0.0, 1.0] 范围内。
///
/// # 示例
///
/// ```ignore
/// let red = Color::from_rgb(1.0, 0.0, 0.0);
/// let blue = Color::from_rgb(0.0, 0.0, 1.0);
/// let purple = lerp_color(red, blue, 0.5);  // 返回紫色
/// ```
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    // 将插值因子限制在有效范围内
    let t = t.clamp(0.0, 1.0);

    // 对 RGBA 四个通道分别进行线性插值
    Color::from_rgba(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

#[cfg(test)]
#[path = "utils_tests.rs"]
mod utils_tests;
