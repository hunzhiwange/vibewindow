use crate::error::Result;
use serde_json::Value as JsonValue;

/// 根据 Blob 的类型解析它
///
/// 采用 blob 类型(不带 "Blob" 后缀的字段名称)和 blob 对象，
/// 提取字节，并将它们解析为结构化 JSON 数据。
///
/// # 参数
/// * `blob_type` - blob 的类型(例如 "commands"、"vectorNetwork")
/// * `blob` - 包含字节的 blob 对象(可以是 base64 字符串或字节数组)
///
/// # 返回值
/// * `Ok(Some(JsonValue))` - 成功解析 blob 数据
/// * `Ok(None)` - 未知的 blob 类型或无法解析的数据
/// * `Err(FigError)` - 如果 blob 提取失败
pub fn parse_blob(blob_type: &str, blob: &JsonValue) -> Result<Option<JsonValue>> {
    // 从 blob 对象中提取字节
    let bytes = extract_blob_bytes(blob)?;

    // 根据类型解析
    match blob_type {
        "commands" => Ok(parse_commands(&bytes)),
        "vectorNetwork" => Ok(parse_vector_network(&bytes)),
        _ => Ok(None), // Unknown blob type, return None
    }
}

/// 从 blob 对象中提取字节
///
/// Blob 可以存储为：
/// - "bytes" 字段中的 Base64 字符串
/// - "bytes" 字段中的数字数组
fn extract_blob_bytes(blob: &JsonValue) -> Result<Vec<u8>> {
    let bytes_value = blob
        .get("bytes")
        .ok_or_else(|| crate::error::FigError::ZipError("Blob missing bytes field".to_string()))?;

    // 处理base64字符串
    if let Some(base64_str) = bytes_value.as_str() {
        use base64::{Engine as _, engine::general_purpose};
        return general_purpose::STANDARD.decode(base64_str).map_err(|e| {
            crate::error::FigError::ZipError(format!("Failed to decode base64: {}", e))
        });
    }

    // 处理数字数组
    if let Some(bytes_array) = bytes_value.as_array() {
        let bytes: Vec<u8> =
            bytes_array.iter().filter_map(|v| v.as_u64().map(|n| n as u8)).collect();
        return Ok(bytes);
    }

    Err(crate::error::FigError::ZipError(
        "Blob bytes field is neither string nor array".to_string(),
    ))
}

/// 将二进制路径命令解析为 JSON 数组
///
/// 将二进制路径命令数据转换为以下格式的平面 JSON 数组：
/// `["M", x, y, "L", x, y, "Q", cx, cy, x, y, "C", cx1, cy1, cx2, cy2, x, y, "Z"]`
///
/// 命令类型：
/// - 0：Z(闭合路径，无坐标)
/// - 1: M(移动到，2 个浮点数：x, y)
/// - 2：L(行到，2个浮点数：x，y)
/// - 3：Q(二次曲线，4个浮点数：cx、cy、x、y)
/// - 4：C(三次曲线，6 个浮点：cx1、cy1、cx2、cy2、x、y)
///
/// 所有坐标均存储为小端 f32 值。
///
/// # 参数
/// * `bytes` - 二进制命令数据
///
/// # 返回值
/// * `Some(JsonValue)` - 命令和坐标数组
/// * `None` - 如果数据无效或不完整
pub fn parse_commands(bytes: &[u8]) -> Option<JsonValue> {
    let mut commands = Vec::new();
    let mut offset = 0;

    while offset < bytes.len() {
        let cmd_type = bytes[offset];
        offset += 1;

        match cmd_type {
            0 => {
                // Z - 闭合路径
                commands.push(JsonValue::String("Z".to_string()));
            }
            1 => {
                // M - 移动到 (x, y)
                if offset + 8 > bytes.len() {
                    return None;
                }
                let x = f32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                let y = f32::from_le_bytes([
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                offset += 8;
                commands.push(JsonValue::String("M".to_string()));
                commands.push(json_number(x));
                commands.push(json_number(y));
            }
            2 => {
                // L - 到 (x, y) 的线
                if offset + 8 > bytes.len() {
                    return None;
                }
                let x = f32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                let y = f32::from_le_bytes([
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                offset += 8;
                commands.push(JsonValue::String("L".to_string()));
                commands.push(json_number(x));
                commands.push(json_number(y));
            }
            3 => {
                // Q - 二次曲线 (cx, cy, x, y)
                if offset + 16 > bytes.len() {
                    return None;
                }
                let cx = f32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                let cy = f32::from_le_bytes([
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                let x = f32::from_le_bytes([
                    bytes[offset + 8],
                    bytes[offset + 9],
                    bytes[offset + 10],
                    bytes[offset + 11],
                ]);
                let y = f32::from_le_bytes([
                    bytes[offset + 12],
                    bytes[offset + 13],
                    bytes[offset + 14],
                    bytes[offset + 15],
                ]);
                offset += 16;
                commands.push(JsonValue::String("Q".to_string()));
                commands.push(json_number(cx));
                commands.push(json_number(cy));
                commands.push(json_number(x));
                commands.push(json_number(y));
            }
            4 => {
                // C - 三次曲线 (cx1, cy1, cx2, cy2, x, y)
                if offset + 24 > bytes.len() {
                    return None;
                }
                let cx1 = f32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                let cy1 = f32::from_le_bytes([
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                let cx2 = f32::from_le_bytes([
                    bytes[offset + 8],
                    bytes[offset + 9],
                    bytes[offset + 10],
                    bytes[offset + 11],
                ]);
                let cy2 = f32::from_le_bytes([
                    bytes[offset + 12],
                    bytes[offset + 13],
                    bytes[offset + 14],
                    bytes[offset + 15],
                ]);
                let x = f32::from_le_bytes([
                    bytes[offset + 16],
                    bytes[offset + 17],
                    bytes[offset + 18],
                    bytes[offset + 19],
                ]);
                let y = f32::from_le_bytes([
                    bytes[offset + 20],
                    bytes[offset + 21],
                    bytes[offset + 22],
                    bytes[offset + 23],
                ]);
                offset += 24;
                commands.push(JsonValue::String("C".to_string()));
                commands.push(json_number(cx1));
                commands.push(json_number(cy1));
                commands.push(json_number(cx2));
                commands.push(json_number(cy2));
                commands.push(json_number(x));
                commands.push(json_number(y));
            }
            _ => {
                // 未知的命令类型
                return None;
            }
        }
    }

    Some(JsonValue::Array(commands))
}

/// 将二进制向量网络解析为 JSON 对象
///
/// 将二进制向量网络数据转换为结构化 JSON 对象：
/// ```json
/// {
///   "vertices": [{"styleID": 0, "x": 1.0, "y": 2.0}, ...],
///   "segments": [{"styleID": 0, "start": {...}, "end": {...}}, ...],
///   "regions": [{"styleID": 0, "windingRule": "ODD", "loops": [...]}, ...]
/// }
/// ```
///
/// 二进制格式：
/// - 标头：vertexCount (u32)、segmentCount (u32)、regionCount (u32)
/// - 顶点：styleID (u32), x (f32), y (f32) - 重复 vertexCount 次
/// - 段：styleID (u32)、startVertex (u32)、start.dx (f32)、start.dy (f32)、
///   endVertex (u32), end.dx (f32), end.dy (f32) - 重复segmentCount次
/// - 区域：styleID+windingRule (u32)、loopCount (u32)、
///   然后对于每个循环：indexCount (u32)，indices (u32[]) - 重复regionCount次
///
/// 所有值都是小端序。
///
/// # 参数
/// * `bytes` - 二进制向量网络数据
///
/// # 返回值
/// * `Some(JsonValue)` - 具有顶点、线段和区域的对象
/// * `None` - 如果数据无效或不完整
pub fn parse_vector_network(bytes: &[u8]) -> Option<JsonValue> {
    if bytes.len() < 12 {
        return None;
    }

    // 读取标头
    let vertex_count = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let segment_count = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let region_count = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;

    let mut offset = 12;

    // 解析顶点
    let mut vertices = Vec::new();
    for _ in 0..vertex_count {
        if offset + 12 > bytes.len() {
            return None;
        }
        let style_id = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let x = f32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        let y = f32::from_le_bytes([
            bytes[offset + 8],
            bytes[offset + 9],
            bytes[offset + 10],
            bytes[offset + 11],
        ]);
        offset += 12;

        vertices.push(serde_json::json!({
            "styleID": style_id,
            "x": json_number(x),
            "y": json_number(y)
        }));
    }

    // 解析段
    let mut segments = Vec::new();
    for _ in 0..segment_count {
        if offset + 28 > bytes.len() {
            return None;
        }
        let style_id = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let start_vertex = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        let start_dx = f32::from_le_bytes([
            bytes[offset + 8],
            bytes[offset + 9],
            bytes[offset + 10],
            bytes[offset + 11],
        ]);
        let start_dy = f32::from_le_bytes([
            bytes[offset + 12],
            bytes[offset + 13],
            bytes[offset + 14],
            bytes[offset + 15],
        ]);
        let end_vertex = u32::from_le_bytes([
            bytes[offset + 16],
            bytes[offset + 17],
            bytes[offset + 18],
            bytes[offset + 19],
        ]);
        let end_dx = f32::from_le_bytes([
            bytes[offset + 20],
            bytes[offset + 21],
            bytes[offset + 22],
            bytes[offset + 23],
        ]);
        let end_dy = f32::from_le_bytes([
            bytes[offset + 24],
            bytes[offset + 25],
            bytes[offset + 26],
            bytes[offset + 27],
        ]);
        offset += 28;

        // 验证顶点索引
        if start_vertex as usize >= vertex_count || end_vertex as usize >= vertex_count {
            return None;
        }

        segments.push(serde_json::json!({
            "styleID": style_id,
            "start": {
                "vertex": start_vertex,
                "dx": json_number(start_dx),
                "dy": json_number(start_dy)
            },
            "end": {
                "vertex": end_vertex,
                "dx": json_number(end_dx),
                "dy": json_number(end_dy)
            }
        }));
    }

    // 解析区域
    let mut regions = Vec::new();
    for _ in 0..region_count {
        if offset + 8 > bytes.len() {
            return None;
        }

        // styleID和缠绕规则打包到一个u32中
        let style_and_rule = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let winding_rule = if style_and_rule & 1 != 0 { "NONZERO" } else { "ODD" };
        let style_id = style_and_rule >> 1;

        let loop_count = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]) as usize;
        offset += 8;

        let mut loops = Vec::new();
        for _ in 0..loop_count {
            if offset + 4 > bytes.len() {
                return None;
            }

            let index_count = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + index_count * 4 > bytes.len() {
                return None;
            }

            let mut indices = Vec::new();
            for _ in 0..index_count {
                let segment_index = u32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                offset += 4;

                // 验证段索引
                if segment_index as usize >= segment_count {
                    return None;
                }

                indices.push(JsonValue::Number(segment_index.into()));
            }

            loops.push(serde_json::json!({
                "segments": indices
            }));
        }

        regions.push(serde_json::json!({
            "styleID": style_id,
            "windingRule": winding_rule,
            "loops": loops
        }));
    }

    Some(serde_json::json!({
        "vertices": vertices,
        "segments": segments,
        "regions": regions
    }))
}

/// 将f32转换为JSON数字，处理特殊值
fn json_number(value: f32) -> JsonValue {
    if value.is_nan() || value.is_infinite() {
        JsonValue::Null
    } else {
        serde_json::Number::from_f64(value as f64).map(JsonValue::Number).unwrap_or(JsonValue::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commands_simple_path() {
        // M 10 20 L 30 40 Z
        let mut bytes = Vec::new();
        bytes.push(1); // M
        bytes.extend_from_slice(&10.0f32.to_le_bytes());
        bytes.extend_from_slice(&20.0f32.to_le_bytes());
        bytes.push(2); // L
        bytes.extend_from_slice(&30.0f32.to_le_bytes());
        bytes.extend_from_slice(&40.0f32.to_le_bytes());
        bytes.push(0); // Z

        let result = parse_commands(&bytes).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 7);
        assert_eq!(arr[0].as_str(), Some("M"));
        assert_eq!(arr[1].as_f64(), Some(10.0));
        assert_eq!(arr[2].as_f64(), Some(20.0));
        assert_eq!(arr[3].as_str(), Some("L"));
        assert_eq!(arr[4].as_f64(), Some(30.0));
        assert_eq!(arr[5].as_f64(), Some(40.0));
        assert_eq!(arr[6].as_str(), Some("Z"));
    }

    #[test]
    fn test_parse_commands_quadratic() {
        // Q 1 2 3 4
        let mut bytes = Vec::new();
        bytes.push(3); // Q
        bytes.extend_from_slice(&1.0f32.to_le_bytes());
        bytes.extend_from_slice(&2.0f32.to_le_bytes());
        bytes.extend_from_slice(&3.0f32.to_le_bytes());
        bytes.extend_from_slice(&4.0f32.to_le_bytes());

        let result = parse_commands(&bytes).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_str(), Some("Q"));
        assert_eq!(arr[1].as_f64(), Some(1.0));
        assert_eq!(arr[2].as_f64(), Some(2.0));
        assert_eq!(arr[3].as_f64(), Some(3.0));
        assert_eq!(arr[4].as_f64(), Some(4.0));
    }

    #[test]
    fn test_parse_commands_cubic() {
        // C 1 2 3 4 5 6
        let mut bytes = Vec::new();
        bytes.push(4); // C
        bytes.extend_from_slice(&1.0f32.to_le_bytes());
        bytes.extend_from_slice(&2.0f32.to_le_bytes());
        bytes.extend_from_slice(&3.0f32.to_le_bytes());
        bytes.extend_from_slice(&4.0f32.to_le_bytes());
        bytes.extend_from_slice(&5.0f32.to_le_bytes());
        bytes.extend_from_slice(&6.0f32.to_le_bytes());

        let result = parse_commands(&bytes).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 7);
        assert_eq!(arr[0].as_str(), Some("C"));
        assert_eq!(arr[1].as_f64(), Some(1.0));
        assert_eq!(arr[2].as_f64(), Some(2.0));
        assert_eq!(arr[3].as_f64(), Some(3.0));
        assert_eq!(arr[4].as_f64(), Some(4.0));
        assert_eq!(arr[5].as_f64(), Some(5.0));
        assert_eq!(arr[6].as_f64(), Some(6.0));
    }

    #[test]
    fn test_parse_commands_invalid() {
        // 无效的命令类型
        let bytes = vec![99];
        assert!(parse_commands(&bytes).is_none());

        // 数据不完整
        let bytes = vec![1, 0]; // M with incomplete coordinates
        assert!(parse_commands(&bytes).is_none());
    }

    #[test]
    fn test_parse_vector_network_simple() {
        let mut bytes = Vec::new();

        // 标头：2 个顶点，1 个线段，0 个区域
        bytes.extend_from_slice(&2u32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());

        // 顶点 0：styleID=0，x=10，y=20
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&10.0f32.to_le_bytes());
        bytes.extend_from_slice(&20.0f32.to_le_bytes());

        // 顶点 1：styleID=0，x=30，y=40
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&30.0f32.to_le_bytes());
        bytes.extend_from_slice(&40.0f32.to_le_bytes());

        // 段 0：styleID=0，开始=(顶点=0，dx=0，dy=0)，结束=(顶点=1，dx=0，dy=0)
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());

        let result = parse_vector_network(&bytes).unwrap();

        assert!(result.get("vertices").is_some());
        assert!(result.get("segments").is_some());
        assert!(result.get("regions").is_some());

        let vertices = result.get("vertices").unwrap().as_array().unwrap();
        assert_eq!(vertices.len(), 2);

        let segments = result.get("segments").unwrap().as_array().unwrap();
        assert_eq!(segments.len(), 1);

        let regions = result.get("regions").unwrap().as_array().unwrap();
        assert_eq!(regions.len(), 0);
    }
}
