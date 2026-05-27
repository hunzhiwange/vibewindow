use crate::error::Result;
use serde_json::Value as JsonValue;

/// 将 RGBA 颜色对象转换为 CSS 十六进制颜色字符串
///
/// 递归遍历JSON树并用r、g、b转换任何对象
/// (以及可选的 a)字段：
/// - 将浮点值 (0.0-1.0) 转换为十六进制字节 (00-ff)
/// - 用十六进制字符串替换整个对象："#rrggbb" 或 "#rrggbbaa"
/// - 当 alpha 为 1.0 或缺失时使用 #rrggbb 格式
/// - 当 alpha 不是 1.0 时使用 #rrggbbaa 格式
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功转换所有颜色对象
///
/// # 示例
/// ```no_run
/// use fig2json::schema::transform_colors_to_css;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "color": {
///         "r": 0.8725961446762085,
///         "g": 0.06292760372161865,
///         "b": 0.06292760372161865,
///         "a": 1.0
///     }
/// });
/// transform_colors_to_css(&mut tree).unwrap();
/// // 树现在有 "color": "#df1010"
/// ```
pub fn transform_colors_to_css(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 递归转换 JSON 值中的颜色对象
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 收集键以避免借用检查器问题
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if let Some(val) = map.get(&key) {
                    // 检查该值是否是颜色对象
                    if let Some(obj) = val.as_object()
                        && is_color_object(obj)
                    {
                        // 将颜色对象转换为 CSS 十六进制字符串
                        if let Some(css_color) = convert_color_to_css(obj) {
                            map.insert(key.clone(), JsonValue::String(css_color));
                            continue; // Skip recursion since we replaced the object
                        }
                    }
                }

                // 如果未替换则递归到该值
                if let Some(val) = map.get_mut(&key) {
                    transform_recursive(val)?;
                }
            }
        }
        JsonValue::Array(arr) => {
            // 递归到数组元素
            for val in arr.iter_mut() {
                transform_recursive(val)?;
            }
        }
        _ => {
            // 原始值，无需处理
        }
    }

    Ok(())
}

/// 检查一个对象是否是颜色对象(有 r、g、b 字段)
fn is_color_object(obj: &serde_json::Map<String, JsonValue>) -> bool {
    obj.contains_key("r") && obj.contains_key("g") && obj.contains_key("b")
}

/// 将颜色对象转换为 CSS 十六进制字符串
///
/// 将 RGBA 值(0.0-1.0 范围)转换为十六进制格式。
/// 如果 alpha 为 1.0 或缺失，则返回 #rrggbb，否则返回 #rrggbbaa。
///
/// # 参数
/// * `obj` - 具有 r、g、b 和可选 a 字段的颜色对象
///
/// # 返回值
/// * `Some(String)` - CSS 十六进制颜色字符串
/// * `None` - 如果缺少任何必填字段或不是有效的 f64
fn convert_color_to_css(obj: &serde_json::Map<String, JsonValue>) -> Option<String> {
    // 提取r、g、b值(必填)
    let r = obj.get("r")?.as_f64()?;
    let g = obj.get("g")?.as_f64()?;
    let b = obj.get("b")?.as_f64()?;

    // 提取 alpha 值(可选，默认为 1.0)
    let a = obj.get("a").and_then(|v| v.as_f64()).unwrap_or(1.0);

    // 将 0.0-1.0 范围转换为 0-255 范围
    let r_byte = float_to_byte(r);
    let g_byte = float_to_byte(g);
    let b_byte = float_to_byte(b);
    let a_byte = float_to_byte(a);

    // 格式为十六进制字符串
    // 当 alpha 为 1.0 时使用 #rrggbb 格式(完全不透明)
    // 当 alpha 不是 1.0 时使用 #rrggbbaa 格式
    if (a - 1.0).abs() < 0.001 {
        // Alpha约为1.0，使用6字符格式
        Some(format!("#{:02x}{:02x}{:02x}", r_byte, g_byte, b_byte))
    } else {
        // Alpha 不是 1.0，以 8 字符格式包含它
        Some(format!("#{:02x}{:02x}{:02x}{:02x}", r_byte, g_byte, b_byte, a_byte))
    }
}

/// 将 0.0-1.0 范围内的浮点转换为 0-255 范围内的字节
///
/// 将输入限制在 [0.0, 1.0] 范围内并四舍五入到最接近的整数。
fn float_to_byte(value: f64) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * 255.0).round() as u8
}

#[cfg(test)]
#[path = "color_to_css_tests.rs"]
mod color_to_css_tests;
