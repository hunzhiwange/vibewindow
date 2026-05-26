//! 设计画布形状渲染模块。
//!
//! 该模块封装填充、描边、阴影和形状树遍历等绘制细节，让上层渲染流程可以按节点语义组合图形输出。

use iced::{
    Color, Point, Size,
    widget::canvas::{Frame, LineCap, LineDash, Path, Stroke},
};

use crate::app::views::design::{
    canvas::parse::{parse_color, parse_thickness, resolve_variable},
    models::VariableDef,
};

/// 模块内部可见的 StrokeSides 结构体，承载该模块边界内传递的结构化状态。
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct StrokeSides {
    pub(super) top: f32,
    pub(super) right: f32,
    pub(super) bottom: f32,
    pub(super) left: f32,
}

impl StrokeSides {
    /// 模块内部可见的 uniform 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub(super) fn uniform(v: f32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    /// 模块内部可见的 any_positive 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub(super) fn any_positive(self) -> bool {
        self.top > 0.0 || self.right > 0.0 || self.bottom > 0.0 || self.left > 0.0
    }

    /// 模块内部可见的 is_uniform 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub(super) fn is_uniform(self) -> Option<f32> {
        let v = self.top;
        if (self.right - v).abs() <= f32::EPSILON
            && (self.bottom - v).abs() <= f32::EPSILON
            && (self.left - v).abs() <= f32::EPSILON
        {
            Some(v)
        } else {
            None
        }
    }

    /// 模块内部可见的 max 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub(super) fn max(self) -> f32 {
        self.top.max(self.right).max(self.bottom).max(self.left)
    }
}

/// 模块内部可见的 parse_stroke_sides 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn parse_stroke_sides(
    v: Option<&serde_json::Value>,
    variables: &std::collections::HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> StrokeSides {
    let parse_one = |val: &serde_json::Value| -> Option<f32> {
        match val {
            serde_json::Value::Number(n) => n.as_f64().map(|f| f as f32),
            serde_json::Value::String(s) => {
                let mut resolved = s.trim().to_string();
                while resolved.starts_with("$-") {
                    let var_name = resolved.strip_prefix("$").unwrap_or(&resolved);
                    if let Some(val_str) = resolve_variable(var_name, variables, theme_mode) {
                        resolved = val_str.clone();
                    } else {
                        break;
                    }
                }
                resolved.parse::<f32>().ok()
            }
            _ => None,
        }
    };

    match v {
        None => StrokeSides::uniform(1.0),
        Some(serde_json::Value::Number(n)) => {
            StrokeSides::uniform(n.as_f64().unwrap_or(1.0) as f32)
        }
        Some(serde_json::Value::String(_)) => {
            let fallback = parse_thickness(&v, variables, theme_mode);
            StrokeSides::uniform(fallback)
        }
        Some(serde_json::Value::Object(map)) => {
            let top = map.get("top").and_then(parse_one).unwrap_or(0.0);
            let right = map.get("right").and_then(parse_one).unwrap_or(0.0);
            let bottom = map.get("bottom").and_then(parse_one).unwrap_or(0.0);
            let left = map.get("left").and_then(parse_one).unwrap_or(0.0);
            StrokeSides {
                top,
                right,
                bottom,
                left,
            }
        }
        _ => StrokeSides::uniform(1.0),
    }
}

/// 模块内部可见的 StrokeAlign 枚举，描述该模块支持的一组离散状态或事件。
#[derive(Debug, Clone, Copy, Default)]
pub(super) enum StrokeAlign {
    Inside,
    Outside,
    #[default]
    Center,
}

impl StrokeAlign {
    /// 模块内部可见的 from_str 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub(super) fn from_str(v: Option<&str>) -> Self {
        match v {
            Some("inside") => StrokeAlign::Inside,
            Some("outside") => StrokeAlign::Outside,
            _ => StrokeAlign::Center,
        }
    }
}

/// 模块内部可见的 DeferredStroke 枚举，描述该模块支持的一组离散状态或事件。
pub(super) enum DeferredStroke {
    Path {
        path: Path,
        color: Color,
        width: f32,
        dash_segments: Option<Vec<f32>>,
        round_cap: bool,
    },
    Sides {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        sides_px: StrokeSides,
        align: StrokeAlign,
        color: Color,
    },
}

/// 模块内部可见的 parse_stroke_paint 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn parse_stroke_paint(
    stroke_fill: Option<&str>,
    variables: &std::collections::HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> (Color, Option<Vec<f32>>) {
    let Some(raw) = stroke_fill else {
        return (Color::TRANSPARENT, None);
    };
    let raw = raw.trim();
    if raw.is_empty() {
        return (Color::TRANSPARENT, None);
    }

    let parsed_json = if raw.starts_with('[') || raw.starts_with('{') {
        serde_json::from_str::<serde_json::Value>(raw).ok()
    } else {
        None
    };

    if let Some(json) = parsed_json {
        let fill_obj = match json {
            serde_json::Value::Array(arr) => {
                arr.first().cloned().unwrap_or(serde_json::Value::Null)
            }
            serde_json::Value::Object(_) => json,
            _ => serde_json::Value::Null,
        };

        if let serde_json::Value::Object(map) = fill_obj {
            let color = map
                .get("color")
                .and_then(|v| v.as_str())
                .map(|s| parse_color(s, variables, theme_mode))
                .unwrap_or(Color::TRANSPARENT);
            let opacity = map
                .get("opacity")
                .and_then(|v| v.as_f64())
                .map(|v| v.clamp(0.0, 1.0) as f32)
                .unwrap_or(1.0);
            let color = Color {
                a: color.a * opacity,
                ..color
            };
            let dash_segments = map
                .get("dashArray")
                .and_then(|v| v.as_array())
                .and_then(|arr| {
                    let segments: Vec<f32> = arr
                        .iter()
                        .filter_map(|v| v.as_f64().map(|n| n as f32))
                        .filter(|n| *n > 0.0)
                        .collect();
                    if segments.is_empty() {
                        None
                    } else {
                        Some(segments)
                    }
                });
            return (color, dash_segments);
        }
    }

    (parse_color(raw, variables, theme_mode), None)
}

/// 模块内部可见的 draw_deferred_stroke 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn draw_deferred_stroke(frame: &mut Frame, deferred: DeferredStroke) {
    match deferred {
        DeferredStroke::Path {
            path,
            color,
            width,
            dash_segments,
            round_cap,
        } => {
            let mut stroke = Stroke::default().with_color(color).with_width(width);
            if let Some(dash_segments) = dash_segments.as_ref() {
                stroke.line_dash = LineDash {
                    segments: dash_segments.as_slice(),
                    offset: 0,
                };
            }
            if round_cap {
                stroke.line_cap = LineCap::Round;
            }
            frame.stroke(&path, stroke);
        }
        DeferredStroke::Sides {
            x,
            y,
            w,
            h,
            sides_px,
            align,
            color,
        } => {
            let top_w = sides_px.top.max(0.0);
            let right_w = sides_px.right.max(0.0);
            let bottom_w = sides_px.bottom.max(0.0);
            let left_w = sides_px.left.max(0.0);

            match align {
                StrokeAlign::Inside => {
                    if top_w > 0.0 && w > 0.0 {
                        frame.fill_rectangle(Point::new(x, y), Size::new(w, top_w), color);
                    }
                    if bottom_w > 0.0 && w > 0.0 {
                        frame.fill_rectangle(
                            Point::new(x, y + h - bottom_w),
                            Size::new(w, bottom_w),
                            color,
                        );
                    }
                    if left_w > 0.0 && h > 0.0 {
                        frame.fill_rectangle(Point::new(x, y), Size::new(left_w, h), color);
                    }
                    if right_w > 0.0 && h > 0.0 {
                        frame.fill_rectangle(
                            Point::new(x + w - right_w, y),
                            Size::new(right_w, h),
                            color,
                        );
                    }
                }
                StrokeAlign::Outside => {
                    let outer_x = x - left_w;
                    let outer_y = y - top_w;
                    let outer_w = w + left_w + right_w;
                    let outer_h = h + top_w + bottom_w;

                    if top_w > 0.0 && outer_w > 0.0 {
                        frame.fill_rectangle(
                            Point::new(outer_x, outer_y),
                            Size::new(outer_w, top_w),
                            color,
                        );
                    }
                    if bottom_w > 0.0 && outer_w > 0.0 {
                        frame.fill_rectangle(
                            Point::new(outer_x, y + h),
                            Size::new(outer_w, bottom_w),
                            color,
                        );
                    }
                    if left_w > 0.0 && outer_h > 0.0 {
                        frame.fill_rectangle(
                            Point::new(outer_x, outer_y),
                            Size::new(left_w, outer_h),
                            color,
                        );
                    }
                    if right_w > 0.0 && outer_h > 0.0 {
                        frame.fill_rectangle(
                            Point::new(x + w, outer_y),
                            Size::new(right_w, outer_h),
                            color,
                        );
                    }
                }
                StrokeAlign::Center => {
                    let outer_x = x - left_w / 2.0;
                    let outer_y = y - top_w / 2.0;
                    let outer_w = w + left_w / 2.0 + right_w / 2.0;
                    let outer_h = h + top_w / 2.0 + bottom_w / 2.0;

                    if top_w > 0.0 && outer_w > 0.0 {
                        frame.fill_rectangle(
                            Point::new(outer_x, outer_y),
                            Size::new(outer_w, top_w),
                            color,
                        );
                    }
                    if bottom_w > 0.0 && outer_w > 0.0 {
                        frame.fill_rectangle(
                            Point::new(outer_x, y + h - bottom_w / 2.0),
                            Size::new(outer_w, bottom_w),
                            color,
                        );
                    }
                    if left_w > 0.0 && outer_h > 0.0 {
                        frame.fill_rectangle(
                            Point::new(outer_x, outer_y),
                            Size::new(left_w, outer_h),
                            color,
                        );
                    }
                    if right_w > 0.0 && outer_h > 0.0 {
                        frame.fill_rectangle(
                            Point::new(x + w - right_w / 2.0, outer_y),
                            Size::new(right_w, outer_h),
                            color,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "stroke_tests.rs"]
mod stroke_tests;
