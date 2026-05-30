//! 设计画布形状渲染模块。
//!
//! 该模块封装填充、描边、阴影和形状树遍历等绘制细节，让上层渲染流程可以按节点语义组合图形输出。

use iced::{
    Color, Point, Rectangle,
    widget::canvas::{Frame, Image, Path, Stroke, path::Builder},
};

use crate::app::views::design::{
    canvas::parse::parse_color,
    models::{DesignDoc, VariableDef},
    properties::fill::types::{FillItem, FillObject, ImageFill, MeshFill},
};

/// 模块内部可见的 extract_first_enabled_image_fill 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn extract_first_enabled_image_fill(
    fill: &Option<serde_json::Value>,
) -> Option<ImageFill> {
    match fill {
        Some(serde_json::Value::Object(map)) => {
            if let Ok(FillItem::Object(FillObject::Image(img))) =
                serde_json::from_value(serde_json::Value::Object(map.clone()))
                && img.enabled
                && !img.url.trim().is_empty()
            {
                return Some(img);
            }
            None
        }
        Some(serde_json::Value::Array(arr)) => {
            for item in arr {
                if let Ok(FillItem::Object(FillObject::Image(img))) =
                    serde_json::from_value(item.clone())
                    && img.enabled
                    && !img.url.trim().is_empty()
                {
                    return Some(img);
                }
            }
            None
        }
        _ => None,
    }
}

/// 模块内部可见的 extract_first_enabled_mesh_fill 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn extract_first_enabled_mesh_fill(
    fill: &Option<serde_json::Value>,
) -> Option<MeshFill> {
    match fill {
        Some(serde_json::Value::Object(map)) => {
            if let Ok(FillItem::Object(FillObject::Mesh(mesh))) =
                serde_json::from_value(serde_json::Value::Object(map.clone()))
                && mesh.enabled
            {
                return Some(mesh);
            }
            None
        }
        Some(serde_json::Value::Array(arr)) => {
            for item in arr {
                if let Ok(FillItem::Object(FillObject::Mesh(mesh))) =
                    serde_json::from_value(item.clone())
                    && mesh.enabled
                {
                    return Some(mesh);
                }
            }
            None
        }
        _ => None,
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color { r: lerp(a.r, b.r, t), g: lerp(a.g, b.g, t), b: lerp(a.b, b.b, t), a: lerp(a.a, b.a, t) }
}

fn bilerp_point(p00: Point, p10: Point, p01: Point, p11: Point, u: f32, v: f32) -> Point {
    let top = Point::new(lerp(p00.x, p10.x, u), lerp(p00.y, p10.y, u));
    let bottom = Point::new(lerp(p01.x, p11.x, u), lerp(p01.y, p11.y, u));
    Point::new(lerp(top.x, bottom.x, v), lerp(top.y, bottom.y, v))
}

/// 模块内部可见的 draw_mesh_fill 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn draw_mesh_fill(
    frame: &mut Frame,
    bounds: Rectangle,
    mesh: &MeshFill,
    variables: &std::collections::HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
    draw_outline: bool,
) {
    let columns = mesh.columns.max(2);
    let rows = mesh.rows.max(2);
    let expected = columns.saturating_mul(rows);
    if expected == 0 {
        return;
    }

    let mut colors: Vec<Color> =
        mesh.colors.iter().map(|s| parse_color(s, variables, theme_mode)).collect();
    if colors.is_empty() {
        return;
    }
    if colors.len() < expected {
        let last = colors.last().copied().unwrap_or(Color::WHITE);
        colors.resize(expected, last);
    } else if colors.len() > expected {
        colors.truncate(expected);
    }

    let (mut points, mut handles) = MeshFill::default_points_and_handles(columns, rows);
    let copy = mesh.points.len().min(expected);
    for i in 0..copy {
        if let Some(p) = mesh.points.get(i) {
            let x = p.first().copied().unwrap_or(points[i][0]);
            let y = p.get(1).copied().unwrap_or(points[i][1]);
            points[i] = vec![x, y];
        }
    }

    let copy_h = mesh.handles.len().min(expected);
    for i in 0..copy_h {
        if let Some(h) = mesh.handles.get(i)
            && h.len() >= 8
        {
            handles[i] = vec![h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]];
        }
    }

    let pts: Vec<Point> = points
        .iter()
        .map(|p| {
            let x = p.first().copied().unwrap_or(0.0).clamp(0.0, 1.0) as f32;
            let y = p.get(1).copied().unwrap_or(0.0).clamp(0.0, 1.0) as f32;
            Point::new(bounds.x + x * bounds.width, bounds.y + y * bounds.height)
        })
        .collect();

    let steps: usize = 6;
    for r in 0..rows.saturating_sub(1) {
        for c in 0..columns.saturating_sub(1) {
            let i00 = r * columns + c;
            let i10 = r * columns + (c + 1);
            let i01 = (r + 1) * columns + c;
            let i11 = (r + 1) * columns + (c + 1);

            let p00 = *pts.get(i00).unwrap_or(&Point::new(bounds.x, bounds.y));
            let p10 = *pts.get(i10).unwrap_or(&Point::new(bounds.x + bounds.width, bounds.y));
            let p01 = *pts.get(i01).unwrap_or(&Point::new(bounds.x, bounds.y + bounds.height));
            let p11 = *pts
                .get(i11)
                .unwrap_or(&Point::new(bounds.x + bounds.width, bounds.y + bounds.height));

            let c00 = *colors.get(i00).unwrap_or(&Color::WHITE);
            let c10 = *colors.get(i10).unwrap_or(&c00);
            let c01 = *colors.get(i01).unwrap_or(&c00);
            let c11 = *colors.get(i11).unwrap_or(&c00);

            for sy in 0..steps {
                for sx in 0..steps {
                    let u0 = sx as f32 / steps as f32;
                    let v0 = sy as f32 / steps as f32;
                    let u1 = (sx + 1) as f32 / steps as f32;
                    let v1 = (sy + 1) as f32 / steps as f32;

                    let q00 = bilerp_point(p00, p10, p01, p11, u0, v0);
                    let q10 = bilerp_point(p00, p10, p01, p11, u1, v0);
                    let q11 = bilerp_point(p00, p10, p01, p11, u1, v1);
                    let q01 = bilerp_point(p00, p10, p01, p11, u0, v1);

                    let tu = (u0 + u1) * 0.5;
                    let tv = (v0 + v1) * 0.5;
                    let top = lerp_color(c00, c10, tu);
                    let bottom = lerp_color(c01, c11, tu);
                    let col = lerp_color(top, bottom, tv);

                    let patch = Path::new(|b: &mut Builder| {
                        b.move_to(q00);
                        b.line_to(q10);
                        b.line_to(q11);
                        b.line_to(q01);
                        b.close();
                    });
                    frame.fill(&patch, col);
                }
            }
        }
    }

    if draw_outline {
        let grid_color = Color::from_rgba(0.0, 0.0, 0.0, 0.12);
        let handle_pt = |idx: usize, hi: usize| -> Point {
            let h = handles.get(idx);
            if let Some(h) = h
                && h.len() >= 8
                && hi < 4
            {
                let x = (h[hi * 2] as f32).clamp(0.0, 1.0);
                let y = (h[hi * 2 + 1] as f32).clamp(0.0, 1.0);
                Point::new(bounds.x + x * bounds.width, bounds.y + y * bounds.height)
            } else {
                *pts.get(idx).unwrap_or(&Point::ORIGIN)
            }
        };

        for r in 0..rows {
            for c in 0..columns {
                let idx = r * columns + c;
                if c + 1 < columns {
                    let a = *pts.get(idx).unwrap_or(&Point::ORIGIN);
                    let b = *pts.get(idx + 1).unwrap_or(&Point::ORIGIN);
                    let cp1 = handle_pt(idx, 2);
                    let cp2 = handle_pt(idx + 1, 0);
                    let path = Path::new(|builder| {
                        builder.move_to(a);
                        builder.bezier_curve_to(cp1, cp2, b);
                    });
                    frame.stroke(&path, Stroke::default().with_color(grid_color).with_width(1.0));
                }

                if r + 1 < rows {
                    let a = *pts.get(idx).unwrap_or(&Point::ORIGIN);
                    let b = *pts.get(idx + columns).unwrap_or(&Point::ORIGIN);
                    let cp1 = handle_pt(idx, 3);
                    let cp2 = handle_pt(idx + columns, 1);
                    let path = Path::new(|builder| {
                        builder.move_to(a);
                        builder.bezier_curve_to(cp1, cp2, b);
                    });
                    frame.stroke(&path, Stroke::default().with_color(grid_color).with_width(1.0));
                }
            }
        }
    }
}

/// 模块内部可见的 draw_image_fill 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn draw_image_fill(
    frame: &mut Frame,
    bounds: Rectangle,
    doc: &DesignDoc,
    img: &ImageFill,
) -> bool {
    let Some(handle) = doc.images.get(&img.url) else {
        return false;
    };

    let (iw, ih) = doc.image_sizes.get(&img.url).copied().unwrap_or((0, 0));
    let mut dest = bounds;
    if iw > 0 && ih > 0 {
        let iw = iw as f32;
        let ih = ih as f32;
        let cw = bounds.width.max(0.0);
        let ch = bounds.height.max(0.0);

        let (dw, dh) = match img.mode.as_str() {
            "fit" => {
                let s = (cw / iw).min(ch / ih);
                (iw * s, ih * s)
            }
            "fill" => {
                let s = (cw / iw).max(ch / ih);
                (iw * s, ih * s)
            }
            "fill_width" => {
                let s = cw / iw;
                (cw, ih * s)
            }
            "stretch" => (cw, ch),
            _ => (cw, ch),
        };

        let dx = bounds.x + (cw - dw) / 2.0;
        let dy = bounds.y + (ch - dh) / 2.0;
        dest = Rectangle { x: dx, y: dy, width: dw, height: dh };
    }

    frame.draw_image(dest, Image::new(handle.clone()));
    true
}

#[cfg(test)]
#[path = "fills_tests.rs"]
mod fills_tests;
