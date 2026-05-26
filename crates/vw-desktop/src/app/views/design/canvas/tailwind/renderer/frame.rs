//! Tailwind 渲染器模块，负责把解析后的节点样式转换为画布中的布局、命中区域和绘制数据。

use iced::Rectangle;

use super::super::parser::ParsedStyle;
use super::style::clamp_explicit_size_to_bounds;

#[derive(Debug, Clone, Copy)]
/// ResolvedNodeFrame 状态结构，保存当前 UI 或导入流程需要跨消息传递的数据。
pub(super) struct ResolvedNodeFrame {
    pub(super) draw_bounds: Rectangle,
    pub(super) width: f32,
    pub(super) height_fixed: Option<f32>,
    pub(super) pt: f32,
    pub(super) pb: f32,
    pub(super) pl: f32,
    pub(super) pr: f32,
    pub(super) mt: f32,
    pub(super) mb: f32,
    pub(super) ml: f32,
    pub(super) mr: f32,
    pub(super) gap_x: f32,
    pub(super) gap_y: f32,
}

impl ResolvedNodeFrame {
    /// 执行 content_bounds 对应的设计辅助逻辑。
    ///
    /// 返回值直接交给调用方继续渲染、导入或属性更新。
    pub(super) fn content_bounds(&self) -> Rectangle {
        Rectangle {
            x: self.draw_bounds.x + self.pl,
            y: self.draw_bounds.y + self.pt,
            width: (self.draw_bounds.width - self.pl - self.pr).max(0.0),
            height: (self.draw_bounds.height - self.pt - self.pb).max(0.0),
        }
    }
}

/// 执行 resolve_node_frame 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn resolve_node_frame(
    style: &ParsedStyle,
    bounds: Rectangle,
    zoom: f32,
) -> ResolvedNodeFrame {
    let mut width = if let Some(w) = style.width {
        if w < 0.0 { bounds.width } else { clamp_explicit_size_to_bounds(w * zoom, bounds.width) }
    } else {
        bounds.width
    };
    if let Some(max_width) = style.max_width {
        width = clamp_explicit_size_to_bounds(width, max_width * zoom);
    }
    let height_fixed = style.height.map(|h| {
        if h < 0.0 { bounds.height } else { clamp_explicit_size_to_bounds(h * zoom, bounds.height) }
    });

    let pt = style.padding_top.or(style.padding).unwrap_or(0.0) * zoom;
    let pb = style.padding_bottom.or(style.padding).unwrap_or(0.0) * zoom;
    let pl = style.padding_left.or(style.padding).unwrap_or(0.0) * zoom;
    let pr = style.padding_right.or(style.padding).unwrap_or(0.0) * zoom;

    let mt = style.margin_top.or(style.margin).unwrap_or(0.0) * zoom;
    let mb = style.margin_bottom.or(style.margin).unwrap_or(0.0) * zoom;
    let ml = style.margin_left.or(style.margin).unwrap_or(0.0) * zoom;
    let mr = style.margin_right.or(style.margin).unwrap_or(0.0) * zoom;

    let gap_x = style.gap_x.unwrap_or(0.0) * zoom;
    let gap_y = style.gap_y.unwrap_or(0.0) * zoom;

    let mut draw_bounds = Rectangle {
        x: bounds.x + ml,
        y: bounds.y + mt,
        width: (width - ml - mr).max(0.0),
        height: height_fixed.unwrap_or(0.0),
    };

    if let (Some(m_l), Some(m_r)) = (style.margin_left, style.margin_right)
        && m_l < 0.0
        && m_r < 0.0
        && width > 0.0
        && width < bounds.width
    {
        let remaining = bounds.width - width;
        draw_bounds.x = bounds.x + remaining / 2.0;
        draw_bounds.width = width;
    }

    if let (Some(m_t), Some(m_b)) = (style.margin_top, style.margin_bottom)
        && m_t < 0.0
        && m_b < 0.0
        && let Some(h) = style.height
        && h > 0.0
    {
        let h_scaled = h * zoom;
        let remaining = bounds.height - h_scaled;
        draw_bounds.y = bounds.y + remaining / 2.0;
        draw_bounds.height = h_scaled;
    }

    ResolvedNodeFrame {
        draw_bounds,
        width,
        height_fixed,
        pt,
        pb,
        pl,
        pr,
        mt,
        mb,
        ml,
        mr,
        gap_x,
        gap_y,
    }
}

#[cfg(test)]
#[path = "frame_tests.rs"]
mod frame_tests;
