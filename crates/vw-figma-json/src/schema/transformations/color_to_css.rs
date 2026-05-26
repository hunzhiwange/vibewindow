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
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_float_to_byte() {
        assert_eq!(float_to_byte(0.0), 0);
        assert_eq!(float_to_byte(1.0), 255);
        assert_eq!(float_to_byte(0.5), 128);
        assert_eq!(float_to_byte(0.8725961446762085), 223); // From example
        assert_eq!(float_to_byte(0.06292760372161865), 16);
    }

    #[test]
    fn test_float_to_byte_clamping() {
        assert_eq!(float_to_byte(-0.5), 0); // Negative clamped to 0
        assert_eq!(float_to_byte(1.5), 255); // Over 1.0 clamped to 255
    }

    #[test]
    fn test_is_color_object() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.5,
            "g": 0.5,
            "b": 0.5,
            "a": 1.0
        }))
        .unwrap();
        assert!(is_color_object(&color));

        let color_no_alpha = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.5,
            "g": 0.5,
            "b": 0.5
        }))
        .unwrap();
        assert!(is_color_object(&color_no_alpha));

        let not_color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "x": 10,
            "y": 20
        }))
        .unwrap();
        assert!(!is_color_object(&not_color));

        let incomplete_color =
            serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
                "r": 0.5,
                "g": 0.5
            }))
            .unwrap();
        assert!(!is_color_object(&incomplete_color));
    }

    #[test]
    fn test_convert_color_to_css_opaque() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.8725961446762085,
            "g": 0.06292760372161865,
            "b": 0.06292760372161865,
            "a": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#df1010");
    }

    #[test]
    fn test_convert_color_to_css_transparent() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 1.0,
            "g": 0.0,
            "b": 0.0,
            "a": 0.5
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#ff000080");
    }

    #[test]
    fn test_convert_color_to_css_no_alpha() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.0,
            "g": 0.5,
            "b": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#0080ff");
    }

    #[test]
    fn test_convert_color_to_css_black() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.0,
            "g": 0.0,
            "b": 0.0,
            "a": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#000000");
    }

    #[test]
    fn test_convert_color_to_css_white() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 1.0,
            "g": 1.0,
            "b": 1.0,
            "a": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#ffffff");
    }

    #[test]
    fn test_transform_simple_color() {
        let mut tree = json!({
            "name": "Rectangle",
            "color": {
                "r": 0.8725961446762085,
                "g": 0.06292760372161865,
                "b": 0.06292760372161865,
                "a": 1.0
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree.get("color").unwrap().as_str(), Some("#df1010"));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    }

    #[test]
    fn test_transform_multiple_colors() {
        let mut tree = json!({
            "backgroundColor": {
                "r": 1.0,
                "g": 0.0,
                "b": 0.0,
                "a": 1.0
            },
            "foregroundColor": {
                "r": 0.0,
                "g": 1.0,
                "b": 0.0,
                "a": 0.5
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree.get("backgroundColor").unwrap().as_str(), Some("#ff0000"));
        assert_eq!(tree.get("foregroundColor").unwrap().as_str(), Some("#00ff0080"));
    }

    #[test]
    fn test_transform_nested_colors() {
        let mut tree = json!({
            "name": "Root",
            "style": {
                "fill": {
                    "r": 1.0,
                    "g": 0.0,
                    "b": 0.0,
                    "a": 1.0
                },
                "stroke": {
                    "r": 0.0,
                    "g": 0.0,
                    "b": 1.0,
                    "a": 0.8
                }
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree["style"]["fill"].as_str(), Some("#ff0000"));
        assert_eq!(tree["style"]["stroke"].as_str(), Some("#0000ffcc"));
    }

    #[test]
    fn test_transform_colors_in_array() {
        let mut tree = json!({
            "fills": [
                {
                    "type": "solid",
                    "color": {
                        "r": 1.0,
                        "g": 0.0,
                        "b": 0.0,
                        "a": 1.0
                    }
                },
                {
                    "type": "solid",
                    "color": {
                        "r": 0.0,
                        "g": 1.0,
                        "b": 0.0,
                        "a": 0.5
                    }
                }
            ]
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree["fills"][0]["color"].as_str(), Some("#ff0000"));
        assert_eq!(tree["fills"][1]["color"].as_str(), Some("#00ff0080"));
    }

    #[test]
    fn test_transform_preserves_non_color_objects() {
        let mut tree = json!({
            "name": "Rectangle",
            "position": {
                "x": 10,
                "y": 20
            },
            "color": {
                "r": 1.0,
                "g": 0.0,
                "b": 0.0,
                "a": 1.0
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        // 颜色应该转换
        assert_eq!(tree.get("color").unwrap().as_str(), Some("#ff0000"));

        // 位置应保持不变
        assert_eq!(tree["position"]["x"].as_i64(), Some(10));
        assert_eq!(tree["position"]["y"].as_i64(), Some(20));

        // 名称应保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    }

    #[test]
    fn test_transform_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "name": "Frame",
                        "fills": [
                            {
                                "type": "solid",
                                "color": {
                                    "r": 0.5,
                                    "g": 0.5,
                                    "b": 0.5,
                                    "a": 1.0
                                }
                            }
                        ]
                    }
                ]
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree["document"]["children"][0]["fills"][0]["color"].as_str(), Some("#808080"));
    }
}
