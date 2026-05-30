//! 设计画布渲染工具模块。
//!
//! 该模块提供路径、图片、文本和 Tailwind 样式转换等底层辅助函数，减少渲染主流程中的重复样板逻辑。

use iced::{Point, Size, border::Radius, widget::canvas::Path};

const SQUARE_EPS_PX: f32 = 0.5;

/// 公开的 element_path 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn element_path(kind: &str, x: f32, y: f32, w: f32, h: f32, radius: f32) -> Path {
    match kind {
        "line" | "Line" => Path::line(Point::new(x, y), Point::new(x + w, y + h)),
        "arrow_left" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y + h / 2.0));
            builder.line_to(Point::new(x + w * 0.35, y));
            builder.line_to(Point::new(x + w * 0.35, y + h * 0.3));
            builder.line_to(Point::new(x + w, y + h * 0.3));
            builder.line_to(Point::new(x + w, y + h * 0.7));
            builder.line_to(Point::new(x + w * 0.35, y + h * 0.7));
            builder.line_to(Point::new(x + w * 0.35, y + h));
            builder.close();
            builder.build()
        }
        "arrow_right" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + w, y + h / 2.0));
            builder.line_to(Point::new(x + w * 0.65, y));
            builder.line_to(Point::new(x + w * 0.65, y + h * 0.3));
            builder.line_to(Point::new(x, y + h * 0.3));
            builder.line_to(Point::new(x, y + h * 0.7));
            builder.line_to(Point::new(x + w * 0.65, y + h * 0.7));
            builder.line_to(Point::new(x + w * 0.65, y + h));
            builder.close();
            builder.build()
        }
        "ellipse" | "circle" => {
            let center = Point::new(x + w / 2.0, y + h / 2.0);
            let r = w.min(h) / 2.0;
            Path::circle(center, r)
        }
        "rounded" => {
            let radius = w.min(h) * 0.24;
            Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), radius.into())
        }
        "triangle" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + w / 2.0, y));
            builder.line_to(Point::new(x + w, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "triangle_down" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w / 2.0, y + h));
            builder.close();
            builder.build()
        }
        "star" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            let center_x = x + w / 2.0;
            let center_y = y + h / 2.0;
            let outer_radius = w.min(h) / 2.0;
            let inner_radius = outer_radius * 0.382;
            let num_points = 5;
            for i in 0..(num_points * 2) {
                let radius = if i % 2 == 0 { outer_radius } else { inner_radius };
                let angle = (i as f32 * std::f32::consts::PI / num_points as f32)
                    - std::f32::consts::PI / 2.0;
                let px = center_x + angle.cos() * radius;
                let py = center_y + angle.sin() * radius;
                if i == 0 {
                    builder.move_to(Point::new(px, py));
                } else {
                    builder.line_to(Point::new(px, py));
                }
            }
            builder.close();
            builder.build()
        }
        "hexagon" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            let center_x = x + w / 2.0;
            let center_y = y + h / 2.0;
            let radius = w.min(h) / 2.0;
            for i in 0..6 {
                let angle = (i as f32 * std::f32::consts::PI / 3.0) - std::f32::consts::PI / 2.0;
                let px = center_x + angle.cos() * radius;
                let py = center_y + angle.sin() * radius;
                if i == 0 {
                    builder.move_to(Point::new(px, py));
                } else {
                    builder.line_to(Point::new(px, py));
                }
            }
            builder.close();
            builder.build()
        }
        "octagon" => {
            let inset = w.min(h) * 0.24;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + inset, y));
            builder.line_to(Point::new(x + w - inset, y));
            builder.line_to(Point::new(x + w, y + inset));
            builder.line_to(Point::new(x + w, y + h - inset));
            builder.line_to(Point::new(x + w - inset, y + h));
            builder.line_to(Point::new(x + inset, y + h));
            builder.line_to(Point::new(x, y + h - inset));
            builder.line_to(Point::new(x, y + inset));
            builder.close();
            builder.build()
        }
        "plus" => {
            let arm_w = w * 0.28;
            let arm_h = h * 0.28;
            let cx = x + w * 0.5;
            let cy = y + h * 0.5;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(cx - arm_w / 2.0, y));
            builder.line_to(Point::new(cx + arm_w / 2.0, y));
            builder.line_to(Point::new(cx + arm_w / 2.0, cy - arm_h / 2.0));
            builder.line_to(Point::new(x + w, cy - arm_h / 2.0));
            builder.line_to(Point::new(x + w, cy + arm_h / 2.0));
            builder.line_to(Point::new(cx + arm_w / 2.0, cy + arm_h / 2.0));
            builder.line_to(Point::new(cx + arm_w / 2.0, y + h));
            builder.line_to(Point::new(cx - arm_w / 2.0, y + h));
            builder.line_to(Point::new(cx - arm_w / 2.0, cy + arm_h / 2.0));
            builder.line_to(Point::new(x, cy + arm_h / 2.0));
            builder.line_to(Point::new(x, cy - arm_h / 2.0));
            builder.line_to(Point::new(cx - arm_w / 2.0, cy - arm_h / 2.0));
            builder.close();
            builder.build()
        }
        "pentagon" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            let center_x = x + w / 2.0;
            let center_y = y + h / 2.0;
            let r = w.min(h) / 2.0;
            for i in 0..5 {
                let angle =
                    (i as f32 * 2.0 * std::f32::consts::PI / 5.0) - std::f32::consts::PI / 2.0;
                let px = center_x + angle.cos() * r;
                let py = center_y + angle.sin() * r;
                if i == 0 {
                    builder.move_to(Point::new(px, py));
                } else {
                    builder.line_to(Point::new(px, py));
                }
            }
            builder.close();
            builder.build()
        }
        "diamond" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + w / 2.0, y));
            builder.line_to(Point::new(x + w, y + h / 2.0));
            builder.line_to(Point::new(x + w / 2.0, y + h));
            builder.line_to(Point::new(x, y + h / 2.0));
            builder.close();
            builder.build()
        }
        "parallelogram" => {
            let skew = w * 0.25;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + skew, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w - skew, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "slanted_r" => {
            let inset = w * 0.18;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + inset, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w - inset, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "slanted_l" => {
            let inset = w * 0.18;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w - inset, y));
            builder.line_to(Point::new(x + w, y + h));
            builder.line_to(Point::new(x + inset, y + h));
            builder.close();
            builder.build()
        }
        "trapezoid" => {
            let inset = w * 0.15;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + inset, y));
            builder.line_to(Point::new(x + w - inset, y));
            builder.line_to(Point::new(x + w, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "split_rect" | "notch_tl" | "notch_tr" | "notch_bl" | "notch_br" => {
            Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), 3.0.into())
        }
        "chat_left" => {
            let notch = w * 0.18;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w, y + h * 0.75));
            builder.line_to(Point::new(x + notch * 1.8, y + h * 0.75));
            builder.line_to(Point::new(x + notch, y + h));
            builder.line_to(Point::new(x + notch, y + h * 0.75));
            builder.line_to(Point::new(x, y + h * 0.75));
            builder.close();
            builder.build()
        }
        "chat_right" => {
            let notch = w * 0.18;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w, y + h * 0.75));
            builder.line_to(Point::new(x + w - notch, y + h * 0.75));
            builder.line_to(Point::new(x + w - notch, y + h));
            builder.line_to(Point::new(x + w - notch * 1.8, y + h * 0.75));
            builder.line_to(Point::new(x, y + h * 0.75));
            builder.close();
            builder.build()
        }
        "file" => {
            let fold = w * 0.24;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w - fold, y));
            builder.line_to(Point::new(x + w, y + fold));
            builder.line_to(Point::new(x + w, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "folder" => {
            let tab_w = w * 0.34;
            let tab_h = h * 0.26;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y + tab_h));
            builder.line_to(Point::new(x + tab_w, y + tab_h));
            builder.line_to(Point::new(x + tab_w + w * 0.12, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "wave_doc" | "stacked_doc" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w, y + h * 0.8));
            builder.line_to(Point::new(x + w * 0.75, y + h));
            builder.line_to(Point::new(x + w * 0.5, y + h * 0.82));
            builder.line_to(Point::new(x + w * 0.25, y + h));
            builder.line_to(Point::new(x, y + h * 0.8));
            builder.close();
            builder.build()
        }
        "cylinder" | "delay" => {
            let radius = (h * 0.42).min(w * 0.36);
            Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), radius.into())
        }
        "offpage" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w, y + h * 0.65));
            builder.line_to(Point::new(x + w * 0.5, y + h));
            builder.line_to(Point::new(x, y + h * 0.65));
            builder.close();
            builder.build()
        }
        "manual_input" => {
            let inset = w * 0.16;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + inset, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w - inset, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "chevron" | "chevron_right" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w * 0.6, y));
            builder.line_to(Point::new(x + w, y + h / 2.0));
            builder.line_to(Point::new(x + w * 0.6, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.line_to(Point::new(x + w * 0.4, y + h / 2.0));
            builder.close();
            builder.build()
        }
        "capsule" | "capsule_h" => {
            let r = (h / 2.0).min(w / 2.0);
            Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), r.into())
        }
        "capsule_v" | "crosshair" | "ring_x" => {
            let r = (w / 2.0).min(h / 2.0);
            Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), r.into())
        }
        _ => {
            let max_r = w.min(h) / 2.0;
            let radius = radius.clamp(0.0, max_r);
            let is_square = (w - h).abs() <= SQUARE_EPS_PX;
            if is_square && radius >= max_r - SQUARE_EPS_PX {
                let center = Point::new(x + w / 2.0, y + h / 2.0);
                let r = max_r;
                Path::circle(center, r)
            } else {
                Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), radius.into())
            }
        }
    }
}

/// 公开的 element_path_radius 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn element_path_radius(kind: &str, x: f32, y: f32, w: f32, h: f32, radius: Radius) -> Path {
    match kind {
        "line" | "Line" => Path::line(Point::new(x, y), Point::new(x + w, y + h)),
        "arrow_left" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y + h / 2.0));
            builder.line_to(Point::new(x + w * 0.35, y));
            builder.line_to(Point::new(x + w * 0.35, y + h * 0.3));
            builder.line_to(Point::new(x + w, y + h * 0.3));
            builder.line_to(Point::new(x + w, y + h * 0.7));
            builder.line_to(Point::new(x + w * 0.35, y + h * 0.7));
            builder.line_to(Point::new(x + w * 0.35, y + h));
            builder.close();
            builder.build()
        }
        "arrow_right" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + w, y + h / 2.0));
            builder.line_to(Point::new(x + w * 0.65, y));
            builder.line_to(Point::new(x + w * 0.65, y + h * 0.3));
            builder.line_to(Point::new(x, y + h * 0.3));
            builder.line_to(Point::new(x, y + h * 0.7));
            builder.line_to(Point::new(x + w * 0.65, y + h * 0.7));
            builder.line_to(Point::new(x + w * 0.65, y + h));
            builder.close();
            builder.build()
        }
        "ellipse" | "circle" => {
            let center = Point::new(x + w / 2.0, y + h / 2.0);
            let r = w.min(h) / 2.0;
            Path::circle(center, r)
        }
        "rounded" => {
            let rounded = Radius::from(w.min(h) * 0.24);
            Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), rounded)
        }
        "triangle" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + w / 2.0, y));
            builder.line_to(Point::new(x + w, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "triangle_down" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w / 2.0, y + h));
            builder.close();
            builder.build()
        }
        "star" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            let center_x = x + w / 2.0;
            let center_y = y + h / 2.0;
            let outer_radius = w.min(h) / 2.0;
            let inner_radius = outer_radius * 0.382;
            let num_points = 5;
            for i in 0..(num_points * 2) {
                let radius = if i % 2 == 0 { outer_radius } else { inner_radius };
                let angle = (i as f32 * std::f32::consts::PI / num_points as f32)
                    - std::f32::consts::PI / 2.0;
                let px = center_x + angle.cos() * radius;
                let py = center_y + angle.sin() * radius;
                if i == 0 {
                    builder.move_to(Point::new(px, py));
                } else {
                    builder.line_to(Point::new(px, py));
                }
            }
            builder.close();
            builder.build()
        }
        "hexagon" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            let center_x = x + w / 2.0;
            let center_y = y + h / 2.0;
            let r = w.min(h) / 2.0;
            for i in 0..6 {
                let angle = (i as f32 * std::f32::consts::PI / 3.0) - std::f32::consts::PI / 2.0;
                let px = center_x + angle.cos() * r;
                let py = center_y + angle.sin() * r;
                if i == 0 {
                    builder.move_to(Point::new(px, py));
                } else {
                    builder.line_to(Point::new(px, py));
                }
            }
            builder.close();
            builder.build()
        }
        "octagon" => {
            let inset = w.min(h) * 0.24;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + inset, y));
            builder.line_to(Point::new(x + w - inset, y));
            builder.line_to(Point::new(x + w, y + inset));
            builder.line_to(Point::new(x + w, y + h - inset));
            builder.line_to(Point::new(x + w - inset, y + h));
            builder.line_to(Point::new(x + inset, y + h));
            builder.line_to(Point::new(x, y + h - inset));
            builder.line_to(Point::new(x, y + inset));
            builder.close();
            builder.build()
        }
        "plus" => {
            let arm_w = w * 0.28;
            let arm_h = h * 0.28;
            let cx = x + w * 0.5;
            let cy = y + h * 0.5;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(cx - arm_w / 2.0, y));
            builder.line_to(Point::new(cx + arm_w / 2.0, y));
            builder.line_to(Point::new(cx + arm_w / 2.0, cy - arm_h / 2.0));
            builder.line_to(Point::new(x + w, cy - arm_h / 2.0));
            builder.line_to(Point::new(x + w, cy + arm_h / 2.0));
            builder.line_to(Point::new(cx + arm_w / 2.0, cy + arm_h / 2.0));
            builder.line_to(Point::new(cx + arm_w / 2.0, y + h));
            builder.line_to(Point::new(cx - arm_w / 2.0, y + h));
            builder.line_to(Point::new(cx - arm_w / 2.0, cy + arm_h / 2.0));
            builder.line_to(Point::new(x, cy + arm_h / 2.0));
            builder.line_to(Point::new(x, cy - arm_h / 2.0));
            builder.line_to(Point::new(cx - arm_w / 2.0, cy - arm_h / 2.0));
            builder.close();
            builder.build()
        }
        "pentagon" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            let center_x = x + w / 2.0;
            let center_y = y + h / 2.0;
            let r = w.min(h) / 2.0;
            for i in 0..5 {
                let angle =
                    (i as f32 * 2.0 * std::f32::consts::PI / 5.0) - std::f32::consts::PI / 2.0;
                let px = center_x + angle.cos() * r;
                let py = center_y + angle.sin() * r;
                if i == 0 {
                    builder.move_to(Point::new(px, py));
                } else {
                    builder.line_to(Point::new(px, py));
                }
            }
            builder.close();
            builder.build()
        }
        "diamond" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + w / 2.0, y));
            builder.line_to(Point::new(x + w, y + h / 2.0));
            builder.line_to(Point::new(x + w / 2.0, y + h));
            builder.line_to(Point::new(x, y + h / 2.0));
            builder.close();
            builder.build()
        }
        "parallelogram" => {
            let skew = w * 0.25;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + skew, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w - skew, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "slanted_r" => {
            let inset = w * 0.18;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + inset, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w - inset, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "slanted_l" => {
            let inset = w * 0.18;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w - inset, y));
            builder.line_to(Point::new(x + w, y + h));
            builder.line_to(Point::new(x + inset, y + h));
            builder.close();
            builder.build()
        }
        "trapezoid" => {
            let inset = w * 0.15;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + inset, y));
            builder.line_to(Point::new(x + w - inset, y));
            builder.line_to(Point::new(x + w, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "split_rect" | "notch_tl" | "notch_tr" | "notch_bl" | "notch_br" => {
            Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), Radius::from(3.0))
        }
        "chat_left" => {
            let notch = w * 0.18;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w, y + h * 0.75));
            builder.line_to(Point::new(x + notch * 1.8, y + h * 0.75));
            builder.line_to(Point::new(x + notch, y + h));
            builder.line_to(Point::new(x + notch, y + h * 0.75));
            builder.line_to(Point::new(x, y + h * 0.75));
            builder.close();
            builder.build()
        }
        "chat_right" => {
            let notch = w * 0.18;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w, y + h * 0.75));
            builder.line_to(Point::new(x + w - notch, y + h * 0.75));
            builder.line_to(Point::new(x + w - notch, y + h));
            builder.line_to(Point::new(x + w - notch * 1.8, y + h * 0.75));
            builder.line_to(Point::new(x, y + h * 0.75));
            builder.close();
            builder.build()
        }
        "file" => {
            let fold = w * 0.24;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w - fold, y));
            builder.line_to(Point::new(x + w, y + fold));
            builder.line_to(Point::new(x + w, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "folder" => {
            let tab_w = w * 0.34;
            let tab_h = h * 0.26;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y + tab_h));
            builder.line_to(Point::new(x + tab_w, y + tab_h));
            builder.line_to(Point::new(x + tab_w + w * 0.12, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "wave_doc" | "stacked_doc" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w, y + h * 0.8));
            builder.line_to(Point::new(x + w * 0.75, y + h));
            builder.line_to(Point::new(x + w * 0.5, y + h * 0.82));
            builder.line_to(Point::new(x + w * 0.25, y + h));
            builder.line_to(Point::new(x, y + h * 0.8));
            builder.close();
            builder.build()
        }
        "cylinder" | "delay" => {
            let rounded = Radius::from((h * 0.42).min(w * 0.36));
            Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), rounded)
        }
        "offpage" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w, y + h * 0.65));
            builder.line_to(Point::new(x + w * 0.5, y + h));
            builder.line_to(Point::new(x, y + h * 0.65));
            builder.close();
            builder.build()
        }
        "manual_input" => {
            let inset = w * 0.16;
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x + inset, y));
            builder.line_to(Point::new(x + w, y));
            builder.line_to(Point::new(x + w - inset, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.close();
            builder.build()
        }
        "chevron" | "chevron_right" => {
            let mut builder = iced::widget::canvas::path::Builder::new();
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + w * 0.6, y));
            builder.line_to(Point::new(x + w, y + h / 2.0));
            builder.line_to(Point::new(x + w * 0.6, y + h));
            builder.line_to(Point::new(x, y + h));
            builder.line_to(Point::new(x + w * 0.4, y + h / 2.0));
            builder.close();
            builder.build()
        }
        "capsule" | "capsule_h" => {
            let r = (h / 2.0).min(w / 2.0);
            Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), r.into())
        }
        "capsule_v" | "crosshair" | "ring_x" => {
            let r = (w / 2.0).min(h / 2.0);
            Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), r.into())
        }
        _ => {
            let max_r = w.min(h) / 2.0;
            let r = Radius {
                top_left: radius.top_left.clamp(0.0, max_r),
                top_right: radius.top_right.clamp(0.0, max_r),
                bottom_right: radius.bottom_right.clamp(0.0, max_r),
                bottom_left: radius.bottom_left.clamp(0.0, max_r),
            };
            let is_square = (w - h).abs() <= SQUARE_EPS_PX;
            let all_max = (r.top_left - max_r).abs() <= SQUARE_EPS_PX
                && (r.top_right - max_r).abs() <= SQUARE_EPS_PX
                && (r.bottom_right - max_r).abs() <= SQUARE_EPS_PX
                && (r.bottom_left - max_r).abs() <= SQUARE_EPS_PX;
            if is_square && all_max {
                let center = Point::new(x + w / 2.0, y + h / 2.0);
                Path::circle(center, max_r)
            } else {
                Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), r)
            }
        }
    }
}

#[cfg(test)]
#[path = "path_tests.rs"]
mod path_tests;
