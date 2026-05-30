//! 处理设计画布中的画笔路径擦除、颜色提取和笔刷几何转换。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::views::design::canvas::creation::{
    BRUSH_STROKE_CLASS, DEFAULT_BRUSH_COLOR_HEX, DEFAULT_BRUSH_WIDTH_PX, create_brush_path_element,
};
use crate::app::views::design::models::DesignElement;
use iced::Point;

fn is_brush_path(element: &DesignElement) -> bool {
    element.kind == "path"
        && element.class.as_deref().is_some_and(|class_name| {
            class_name.split_whitespace().any(|token| token == BRUSH_STROKE_CLASS)
        })
}

fn parse_brush_color(element: &DesignElement) -> String {
    element
        .stroke
        .as_ref()
        .and_then(|stroke| stroke.fill.clone())
        .filter(|fill| !fill.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_BRUSH_COLOR_HEX.to_string())
}

fn parse_brush_width(element: &DesignElement) -> f32 {
    element
        .stroke
        .as_ref()
        .and_then(|stroke| stroke.thickness.as_ref())
        .and_then(|value| {
            value
                .as_f64()
                .map(|number| number as f32)
                .or_else(|| value.as_i64().map(|number| number as f32))
                .or_else(|| value.as_u64().map(|number| number as f32))
                .or_else(|| value.as_str().and_then(|raw| raw.parse::<f32>().ok()))
        })
        .unwrap_or(DEFAULT_BRUSH_WIDTH_PX)
        .clamp(1.0, 18.0)
}

fn parse_brush_points(geometry: &str) -> Option<Vec<Point>> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in geometry.chars() {
        if ch.is_ascii_alphabetic() {
            if !current.trim().is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            tokens.push(ch.to_string());
            continue;
        }

        if ch == '-' && !current.is_empty() && !current.ends_with('e') && !current.ends_with('E') {
            tokens.push(std::mem::take(&mut current));
        }

        if ch == ',' || ch.is_ascii_whitespace() {
            if !current.trim().is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }

    if !current.trim().is_empty() {
        tokens.push(current);
    }

    let mut points = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        match tokens[index].as_str() {
            "M" | "L" | "m" | "l" => {
                if index + 2 >= tokens.len() {
                    break;
                }
                let x = tokens[index + 1].parse::<f32>().ok()?;
                let y = tokens[index + 2].parse::<f32>().ok()?;
                points.push(Point::new(x, y));
                index += 3;
            }
            _ => index += 1,
        }
    }

    (points.len() >= 2).then_some(points)
}

fn split_brush_segments(
    points: &[Point],
    center_world: Point,
    radius_world: f32,
) -> Vec<Vec<Point>> {
    let mut out = Vec::new();
    let mut current = Vec::new();
    let radius_sq = radius_world * radius_world;

    for point in points {
        let dx = point.x - center_world.x;
        let dy = point.y - center_world.y;
        if dx * dx + dy * dy > radius_sq {
            current.push(*point);
        } else if current.len() >= 2 {
            out.push(std::mem::take(&mut current));
        } else {
            current.clear();
        }
    }

    if current.len() >= 2 {
        out.push(current);
    }

    out
}

/// erase_brush_nodes 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn erase_brush_nodes(
    children: &mut Vec<DesignElement>,
    parent_offset: Point,
    center_world: Point,
    radius_world: f32,
) -> bool {
    let mut changed = false;
    let mut index = 0;

    // 递归擦除会在遍历中替换节点，因此使用索引循环避免迭代器失效。

    while index < children.len() {
        let child_offset =
            Point::new(parent_offset.x + children[index].x, parent_offset.y + children[index].y);

        if erase_brush_nodes(
            &mut children[index].children,
            child_offset,
            center_world,
            radius_world,
        ) {
            changed = true;
        }

        if !is_brush_path(&children[index]) {
            index += 1;
            continue;
        }

        let Some(local_points) = children[index].geometry.as_deref().and_then(parse_brush_points)
        else {
            index += 1;
            continue;
        };

        let world_points: Vec<Point> = local_points
            .into_iter()
            .map(|point| Point::new(child_offset.x + point.x, child_offset.y + point.y))
            .collect();
        let segments = split_brush_segments(&world_points, center_world, radius_world);

        if segments.len() == 1 && segments[0].len() == world_points.len() {
            index += 1;
            continue;
        }

        changed = true;
        let base = children[index].clone();
        let color = parse_brush_color(&base);
        let width = parse_brush_width(&base);
        let mut replacements = Vec::new();

        for (segment_index, segment_world) in segments.iter().enumerate() {
            let segment_local: Vec<Point> = segment_world
                .iter()
                .map(|point| Point::new(point.x - parent_offset.x, point.y - parent_offset.y))
                .collect();
            if let Some(mut replacement) = create_brush_path_element(&segment_local, &color, width)
            {
                if segment_index == 0 {
                    replacement.id = base.id.clone();
                }
                replacement.group_id = base.group_id;
                replacement.name = base.name.clone();
                replacement.visible = base.visible;
                replacement.enabled = base.enabled;
                replacement.opacity = base.opacity;
                replacement.class = base.class.clone();
                replacement.theme = base.theme.clone();
                replacement.export = base.export.clone();
                replacements.push(replacement);
            }
        }

        if replacements.is_empty() {
            children.remove(index);
            continue;
        }

        let insert_count = replacements.len().saturating_sub(1);
        children[index] = replacements.remove(0);
        for (offset, replacement) in replacements.into_iter().enumerate() {
            children.insert(index + 1 + offset, replacement);
        }
        index += 1 + insert_count;
    }

    changed
}

/// extract_hex_token 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn extract_hex_token(input: &str) -> Option<String> {
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '#' {
            continue;
        }
        let mut token = String::new();
        token.push('#');
        while let Some(next) = chars.peek().copied() {
            if next.is_ascii_hexdigit() && token.len() < 9 {
                token.push(next);
                chars.next();
            } else {
                break;
            }
        }
        if token.len() == 7 || token.len() == 9 {
            return Some(token);
        }
    }
    None
}

/// with_alpha 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn with_alpha(hex: &str, alpha: u8) -> String {
    let raw = hex.trim().trim_start_matches('#');
    match raw.len() {
        6 => format!("#{raw}{alpha:02X}"),
        8 => format!("#{}{alpha:02X}", &raw[0..6]),
        _ => format!("#000000{alpha:02X}"),
    }
}
#[cfg(test)]
#[path = "brush_tests.rs"]
mod brush_tests;
