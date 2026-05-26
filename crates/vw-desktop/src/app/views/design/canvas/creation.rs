//! 设计画布核心模块。
//!
//! 该模块承载节点创建、几何计算、命中测试与画布入口组织逻辑，是设计视图交互和渲染之间的核心边界。

use crate::app::views::design::models::{DesignDoc, DesignElement, StickyNoteKind, Stroke};
use iced::Point;

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_arch = "wasm32")]
use web_time::{SystemTime, UNIX_EPOCH};

const DEFAULT_SHAPE_SIZE: f32 = 160.0;
const DEFAULT_FRAME_WIDTH: f32 = 360.0;
const DEFAULT_FRAME_HEIGHT: f32 = 240.0;
const DEFAULT_STICKY_NOTE_WIDTH: f32 = 320.0;
const DEFAULT_STICKY_NOTE_HEIGHT: f32 = 220.0;
/// 公开的 DEFAULT_BRUSH_WIDTH_PX 常量，集中保存该模块复用的稳定取值。
pub const DEFAULT_BRUSH_WIDTH_PX: f32 = 3.0;
/// 公开的 DEFAULT_BRUSH_COLOR_HEX 常量，集中保存该模块复用的稳定取值。
pub const DEFAULT_BRUSH_COLOR_HEX: &str = "#111827";
/// 公开的 BRUSH_STROKE_CLASS 常量，集中保存该模块复用的稳定取值。
pub const BRUSH_STROKE_CLASS: &str = "vw-brush-stroke";

/// 公开的 create_text_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_text_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "text".to_string(),
        id: generate_id("text"),
        x: position.x,
        y: position.y,
        name: Some("Text".to_string()),
        width: Some(serde_json::json!(220.0)),
        height: Some(serde_json::json!(40.0)),
        content: Some("输入文本".to_string()),
        font_size: Some(serde_json::json!(16.0)),
        font_family: Some("JetBrains Mono".to_string()),
        font_weight: Some(serde_json::json!(400)),
        line_height: Some(serde_json::json!(1.5)),
        color: Some("#111827".to_string()),
        text_growth: Some("auto".to_string()),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_line_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_line_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "line".to_string(),
        id: generate_id("line"),
        x: position.x,
        y: position.y,
        name: Some("Line".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(2.0)),
        fill: Some(serde_json::json!("#3B82F6")),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_rectangle_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_rectangle_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "rectangle".to_string(),
        id: generate_id("rectangle"),
        x: position.x,
        y: position.y,
        name: Some("Rectangle".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        fill: Some(serde_json::json!("#60A5FA")),
        corner_radius: Some(serde_json::json!(16.0)),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_ellipse_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_ellipse_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "ellipse".to_string(),
        id: generate_id("ellipse"),
        x: position.x,
        y: position.y,
        name: Some("Ellipse".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        fill: Some(serde_json::json!("#34D399")),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_triangle_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_triangle_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "triangle".to_string(),
        id: generate_id("triangle"),
        x: position.x,
        y: position.y,
        name: Some("Triangle".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        fill: Some(serde_json::json!("#FBBF24")),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_diamond_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_diamond_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "diamond".to_string(),
        id: generate_id("diamond"),
        x: position.x,
        y: position.y,
        name: Some("Diamond".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        fill: Some(serde_json::json!("#A78BFA")),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_star_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_star_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "star".to_string(),
        id: generate_id("star"),
        x: position.x,
        y: position.y,
        name: Some("Star".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        fill: Some(serde_json::json!("#F59E0B")),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_pentagon_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_pentagon_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "pentagon".to_string(),
        id: generate_id("pentagon"),
        x: position.x,
        y: position.y,
        name: Some("Pentagon".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        fill: Some(serde_json::json!("#EC4899")),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_hexagon_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_hexagon_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "hexagon".to_string(),
        id: generate_id("hexagon"),
        x: position.x,
        y: position.y,
        name: Some("Hexagon".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        fill: Some(serde_json::json!("#06B6D4")),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_parallelogram_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_parallelogram_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "parallelogram".to_string(),
        id: generate_id("parallelogram"),
        x: position.x,
        y: position.y,
        name: Some("Parallelogram".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(DEFAULT_SHAPE_SIZE * 0.6)),
        fill: Some(serde_json::json!("#8B5CF6")),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_trapezoid_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_trapezoid_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "trapezoid".to_string(),
        id: generate_id("trapezoid"),
        x: position.x,
        y: position.y,
        name: Some("Trapezoid".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(DEFAULT_SHAPE_SIZE * 0.6)),
        fill: Some(serde_json::json!("#10B981")),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_chevron_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_chevron_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "chevron".to_string(),
        id: generate_id("chevron"),
        x: position.x,
        y: position.y,
        name: Some("Chevron".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(DEFAULT_SHAPE_SIZE * 0.8)),
        fill: Some(serde_json::json!("#3B82F6")),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_capsule_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_capsule_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "capsule".to_string(),
        id: generate_id("capsule"),
        x: position.x,
        y: position.y,
        name: Some("Capsule".to_string()),
        width: Some(serde_json::json!(DEFAULT_SHAPE_SIZE)),
        height: Some(serde_json::json!(DEFAULT_SHAPE_SIZE * 0.5)),
        fill: Some(serde_json::json!("#EF4444")),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_icon_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_icon_element(position: Point) -> DesignElement {
    DesignElement {
        kind: "icon_font".to_string(),
        id: generate_id("icon"),
        x: position.x,
        y: position.y,
        name: Some("Icon".to_string()),
        width: Some(serde_json::json!(48.0)),
        height: Some(serde_json::json!(48.0)),
        fill: Some(serde_json::json!("#111827")),
        icon_font_name: Some("star".to_string()),
        icon_font_family: Some("lucide".to_string()),
        weight: Some(serde_json::json!(400)),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_frame_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_frame_element(position: Point, doc: &DesignDoc) -> DesignElement {
    let index = doc.children.iter().filter(|element| element.kind == "frame").count() + 1;

    DesignElement {
        kind: "frame".to_string(),
        id: generate_id("frame"),
        x: position.x,
        y: position.y,
        name: Some(format!("画板 {}", index)),
        width: Some(serde_json::json!(DEFAULT_FRAME_WIDTH)),
        height: Some(serde_json::json!(DEFAULT_FRAME_HEIGHT)),
        fill: Some(serde_json::json!("#FFFFFF")),
        corner_radius: Some(serde_json::json!(24.0)),
        stroke: Some(Stroke {
            align: Some("inside".to_string()),
            thickness: Some(serde_json::json!(1)),
            fill: Some("#D1D5DB".to_string()),
        }),
        clip: Some(true),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_sticky_note_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_sticky_note_element(position: Point, note_type: StickyNoteKind) -> DesignElement {
    DesignElement {
        kind: "sticky_note".to_string(),
        id: generate_id("sticky-note"),
        x: position.x,
        y: position.y,
        name: Some(note_type.bilingual_label()),
        width: Some(serde_json::json!(DEFAULT_STICKY_NOTE_WIDTH)),
        height: Some(serde_json::json!(DEFAULT_STICKY_NOTE_HEIGHT)),
        fill: Some(serde_json::json!(note_type.fill_color())),
        stroke: Some(Stroke {
            align: Some("inside".to_string()),
            thickness: Some(serde_json::json!(1)),
            fill: Some(note_type.stroke_color().to_string()),
        }),
        corner_radius: Some(serde_json::json!(18.0)),
        content: Some("输入便签内容".to_string()),
        note_type: Some(note_type),
        font_size: Some(serde_json::json!(16.0)),
        font_family: Some("JetBrains Mono".to_string()),
        font_weight: Some(serde_json::json!(400)),
        line_height: Some(serde_json::json!(1.45)),
        color: Some(note_type.text_color().to_string()),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_image_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_image_element(
    position: Point,
    source: String,
    size_opt: Option<(u32, u32)>,
) -> DesignElement {
    let (width, height) = match size_opt {
        Some((w, h)) if w > 0 && h > 0 => {
            let mut width = w as f32;
            let mut height = h as f32;
            let scale = (320.0 / width).min(240.0 / height).min(1.0);
            width = (width * scale).max(48.0);
            height = (height * scale).max(48.0);
            (width, height)
        }
        _ => (240.0, 180.0),
    };

    DesignElement {
        kind: "image".to_string(),
        id: generate_id("image"),
        x: position.x,
        y: position.y,
        name: Some("Image".to_string()),
        width: Some(serde_json::json!(width)),
        height: Some(serde_json::json!(height)),
        fill: Some(serde_json::json!({
            "type": "image",
            "enabled": true,
            "url": source,
            "mode": "fit"
        })),
        visible: Some(true),
        ..Default::default()
    }
}

/// 公开的 create_brush_path_element 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn create_brush_path_element(
    points: &[Point],
    stroke_hex: &str,
    width_px: f32,
) -> Option<DesignElement> {
    if points.len() < 2 {
        return None;
    }

    let stroke_width = width_px.clamp(1.0, 18.0);
    let padding = stroke_width * 0.5 + 2.0;

    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    let origin_x = min_x - padding;
    let origin_y = min_y - padding;
    let width = (max_x - min_x + padding * 2.0).max(1.0);
    let height = (max_y - min_y + padding * 2.0).max(1.0);
    let local_points: Vec<Point> =
        points.iter().map(|point| Point::new(point.x - origin_x, point.y - origin_y)).collect();
    let geometry = build_polyline_geometry(&local_points)?;

    Some(DesignElement {
        kind: "path".to_string(),
        id: generate_id("brush"),
        x: origin_x,
        y: origin_y,
        name: Some("画笔".to_string()),
        width: Some(serde_json::json!(width)),
        height: Some(serde_json::json!(height)),
        geometry: Some(geometry),
        stroke: Some(Stroke {
            align: Some("center".to_string()),
            thickness: Some(serde_json::json!(stroke_width)),
            fill: Some(stroke_hex.to_string()),
        }),
        class: Some(BRUSH_STROKE_CLASS.to_string()),
        visible: Some(true),
        ..Default::default()
    })
}

fn build_polyline_geometry(points: &[Point]) -> Option<String> {
    let mut iter = points.iter();
    let first = iter.next()?;
    let mut geometry = format!("M {:.2} {:.2}", first.x, first.y);
    for point in iter {
        geometry.push_str(&format!(" L {:.2} {:.2}", point.x, point.y));
    }
    Some(geometry)
}

fn generate_id(prefix: &str) -> String {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
    let nanos = since_epoch.as_nanos();

    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let counter = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    format!("{prefix}-{nanos:x}-{counter:08x}")
}

#[cfg(test)]
#[path = "creation_tests.rs"]
mod creation_tests;
