//! Figma 导入模块，负责把 Figma JSON 中的节点、几何、样式和辅助字段转换为设计模型。

use serde_json::{Map, Value, json};

use super::figma_support::value_to_f64;

/// 执行 read_vector_geometry 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn read_vector_geometry(object: &Map<String, Value>) -> Option<String> {
    if let Some(fill_geometry) = object.get("fillGeometry").and_then(Value::as_array)
        && let Some(geometry) = build_geometry_from_fill_geometry(fill_geometry)
    {
        return Some(geometry);
    }

    let network = object
        .get("vectorData")
        .and_then(Value::as_object)
        .and_then(|vector_data| vector_data.get("vectorNetwork"))
        .and_then(Value::as_object)?;
    let vertices = network.get("vertices").and_then(Value::as_array)?;
    let segments = network.get("segments").and_then(Value::as_array)?;
    let regions = network.get("regions").and_then(Value::as_array).map(Vec::as_slice);
    build_geometry_from_regions(vertices, segments, regions)
        .or_else(|| build_geometry_from_segments(vertices, segments))
}

fn build_geometry_from_fill_geometry(fill_geometry: &[Value]) -> Option<String> {
    let mut commands = Vec::new();

    for geometry in fill_geometry {
        let object = geometry.as_object()?;
        let path_commands = object.get("commands").and_then(Value::as_array)?;
        let serialized = serialize_fill_geometry_commands(path_commands)?;
        commands.push(serialized);
    }

    if commands.is_empty() { None } else { Some(commands.join(" ")) }
}

fn serialize_fill_geometry_commands(commands: &[Value]) -> Option<String> {
    let mut serialized = Vec::new();

    for command in commands {
        match command {
            Value::String(value) => serialized.push(value.clone()),
            Value::Number(number) => serialized.push(format_number(number.as_f64()?)),
            _ => return None,
        }
    }

    Some(serialized.join(" "))
}

fn build_geometry_from_regions(
    vertices: &[Value],
    segments: &[Value],
    regions: Option<&[Value]>,
) -> Option<String> {
    let mut commands = Vec::new();
    for region in regions? {
        let loops = region.get("loops").and_then(Value::as_array)?;
        for loop_value in loops {
            let loop_segments = loop_value.get("segments").and_then(Value::as_array)?;
            let mut segment_commands =
                build_geometry_for_segment_list(vertices, segments, loop_segments)?;
            if !segment_commands.ends_with('Z') {
                segment_commands.push_str(" Z");
            }
            commands.push(segment_commands);
        }
    }
    if commands.is_empty() { None } else { Some(commands.join(" ")) }
}

fn build_geometry_from_segments(vertices: &[Value], segments: &[Value]) -> Option<String> {
    let segment_ids: Vec<Value> = (0..segments.len()).map(|index| json!(index)).collect();
    build_geometry_for_segment_list(vertices, segments, &segment_ids)
}

fn build_geometry_for_segment_list(
    vertices: &[Value],
    segments: &[Value],
    segment_ids: &[Value],
) -> Option<String> {
    let mut commands = Vec::new();
    let mut started = false;

    for segment_id in segment_ids {
        let raw_index = segment_id.as_i64()?;
        let reversed = raw_index < 0;
        let segment = segments.get(raw_index.unsigned_abs() as usize)?;
        let (start_point, start_handle, end_handle, end_point) =
            resolve_segment_points(segment, vertices, reversed)?;

        if !started {
            commands.push(format!(
                "M {} {}",
                format_number(start_point.0),
                format_number(start_point.1)
            ));
            started = true;
        }

        if is_same_point(start_point, start_handle) && is_same_point(end_handle, end_point) {
            commands.push(format!(
                "L {} {}",
                format_number(end_point.0),
                format_number(end_point.1)
            ));
        } else {
            commands.push(format!(
                "C {} {}, {} {}, {} {}",
                format_number(start_handle.0),
                format_number(start_handle.1),
                format_number(end_handle.0),
                format_number(end_handle.1),
                format_number(end_point.0),
                format_number(end_point.1)
            ));
        }
    }

    if commands.is_empty() { None } else { Some(commands.join(" ")) }
}

fn resolve_segment_points(
    segment: &Value,
    vertices: &[Value],
    reversed: bool,
) -> Option<((f64, f64), (f64, f64), (f64, f64), (f64, f64))> {
    let object = segment.as_object()?;
    let start = object.get("start")?.as_object()?;
    let end = object.get("end")?.as_object()?;
    let start_vertex = read_vertex(vertices, start.get("vertex")?.as_u64()? as usize)?;
    let end_vertex = read_vertex(vertices, end.get("vertex")?.as_u64()? as usize)?;
    let start_handle = (
        start_vertex.0 + start.get("dx").and_then(value_to_f64).unwrap_or(0.0),
        start_vertex.1 + start.get("dy").and_then(value_to_f64).unwrap_or(0.0),
    );
    let end_handle = (
        end_vertex.0 + end.get("dx").and_then(value_to_f64).unwrap_or(0.0),
        end_vertex.1 + end.get("dy").and_then(value_to_f64).unwrap_or(0.0),
    );

    if reversed {
        Some((end_vertex, end_handle, start_handle, start_vertex))
    } else {
        Some((start_vertex, start_handle, end_handle, end_vertex))
    }
}

fn read_vertex(vertices: &[Value], index: usize) -> Option<(f64, f64)> {
    let vertex = vertices.get(index)?.as_object()?;
    Some((vertex.get("x").and_then(value_to_f64)?, vertex.get("y").and_then(value_to_f64)?))
}

fn format_number(value: f64) -> String {
    let rounded = (value * 1000.0).round() / 1000.0;
    if rounded.fract().abs() < f64::EPSILON {
        format!("{}", rounded as i64)
    } else {
        let mut text = rounded.to_string();
        while text.contains('.') && text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
        text
    }
}

fn is_same_point(left: (f64, f64), right: (f64, f64)) -> bool {
    (left.0 - right.0).abs() < 0.0001 && (left.1 - right.1).abs() < 0.0001
}

#[cfg(test)]
#[path = "figma_geometry_tests.rs"]
mod figma_geometry_tests;
