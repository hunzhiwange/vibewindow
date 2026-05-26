use crate::error::Result;
use serde_json::Value as JsonValue;
use std::f64::consts::PI;

/// 将 2D 仿射变换矩阵转换为 CSS 定位属性
///
/// 递归遍历 JSON 树并通过以下方式转换 "transform" 对象：
/// - 将矩阵 [m00, m01, m02, m10, m11, m12] 分解为 CSS 属性
/// - 将矩阵字段替换为：x、y、rotation、scaleX、scaleY、skewX
///
/// 分解遵循标准 2D 仿射变换分解：
/// - 翻译：x = m02，y = m12
/// - 从线性变换矩阵中提取的缩放和旋转
/// - 根据剩余组件计算的偏差
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功转换所有矩阵变换
///
/// # 示例
/// ```no_run
/// use fig2json::schema::transform_matrix_to_css;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "transform": {
///         "m00": 1.0,
///         "m01": 0.0,
///         "m02": 100.0,
///         "m10": 0.0,
///         "m11": 1.0,
///         "m12": 50.0
///     }
/// });
/// transform_matrix_to_css(&mut tree).unwrap();
/// // 树现在有 "transform": {"x": 100.0, "y": 50.0, "rotation": 0.0, ...}
/// ```
pub fn transform_matrix_to_css(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 以 JSON 值递归变换矩阵变换
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查这是否是具有矩阵字段的 "transform" 对象
            if let Some(transform_value) = map.get("transform")
                && let Some(transform_obj) = transform_value.as_object()
            {
                // 检查是否有矩阵字段
                if has_matrix_fields(transform_obj) {
                    // 提取矩阵值
                    if let Some(css_transform) = extract_and_decompose_matrix(transform_obj) {
                        // 替换变换对象
                        map.insert("transform".to_string(), css_transform);
                    }
                }
            }

            // 递归到所有值
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
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

/// 检查对象是否具有所有必需的矩阵字段
fn has_matrix_fields(obj: &serde_json::Map<String, JsonValue>) -> bool {
    obj.contains_key("m00")
        && obj.contains_key("m01")
        && obj.contains_key("m02")
        && obj.contains_key("m10")
        && obj.contains_key("m11")
        && obj.contains_key("m12")
}

/// 提取矩阵值并分解为 CSS 属性
///
/// 仅包含与其默认值不同的属性：
/// - x, y：始终包含在内
/// - 旋转：仅当不是 ~0.0 时
/// - scaleX：仅当不是~1.0时
/// - scaleY：仅当不是 ~1.0 时
/// - skewX：仅当不是 ~0.0 时
fn extract_and_decompose_matrix(obj: &serde_json::Map<String, JsonValue>) -> Option<JsonValue> {
    // 提取矩阵分量
    let m00 = obj.get("m00")?.as_f64()?;
    let m01 = obj.get("m01")?.as_f64()?;
    let m02 = obj.get("m02")?.as_f64()?;
    let m10 = obj.get("m10")?.as_f64()?;
    let m11 = obj.get("m11")?.as_f64()?;
    let m12 = obj.get("m12")?.as_f64()?;

    // 将矩阵分解为 CSS 属性
    let css = decompose_matrix(m00, m01, m02, m10, m11, m12);

    // 构建结果对象，仅包含非默认值
    let mut result = serde_json::Map::new();

    // 始终包含 x 和 y
    result.insert("x".to_string(), serde_json::json!(css.x));
    result.insert("y".to_string(), serde_json::json!(css.y));

    // 仅包含非默认值(使用浮点比较的公差)
    const EPSILON: f64 = 1e-10;

    if css.rotation.abs() > EPSILON {
        result.insert("rotation".to_string(), serde_json::json!(css.rotation));
    }

    if (css.scale_x - 1.0).abs() > EPSILON {
        result.insert("scaleX".to_string(), serde_json::json!(css.scale_x));
    }

    if (css.scale_y - 1.0).abs() > EPSILON {
        result.insert("scaleY".to_string(), serde_json::json!(css.scale_y));
    }

    if css.skew_x.abs() > EPSILON {
        result.insert("skewX".to_string(), serde_json::json!(css.skew_x));
    }

    Some(JsonValue::Object(result))
}

/// CSS 变换属性
#[derive(Debug)]
struct CssTransform {
    x: f64,
    y: f64,
    rotation: f64, // in degrees
    scale_x: f64,
    scale_y: f64,
    skew_x: f64, // in degrees
}

/// 将 2D 仿射变换矩阵分解为 CSS 属性
///
/// 矩阵格式：
/// [m00  m01  m02]   [a  c  tx]
/// [m10  m11  m12] = [b  d  ty]
/// [0    0    1  ]   [0  0  1 ]
///
/// 分解算法：
/// 1. 翻译：tx,ty直接是m02,m12
/// 2. 根据第一列的大小计算scale_x
/// 3. 根据第一列的角度计算旋转
/// 4. 根据行列式除以scale_x计算scale_y
/// 5. 根据列的点积计算 skew_x
fn decompose_matrix(m00: f64, m01: f64, m02: f64, m10: f64, m11: f64, m12: f64) -> CssTransform {
    // 翻译很简单
    let x = m02;
    let y = m12;

    // 计算scale_x作为第一列向量的大小[m00, m10]
    let scale_x = (m00 * m00 + m10 * m10).sqrt();

    // 从第一列向量的角度计算旋转
    // atan2(m10, m00) 给出以弧度为单位的旋转
    let rotation_rad = m10.atan2(m00);
    let rotation = rotation_rad * (180.0 / PI);

    // 根据行列式计算scale_y
    // det = m00*m11 - m01*m10
    // 比例_y = det / 比例_x
    let determinant = m00 * m11 - m01 * m10;
    let scale_y = if scale_x.abs() > 1e-10 {
        determinant / scale_x
    } else {
        // 如果scale_x接近零，则使用第二列的大小
        (m01 * m01 + m11 * m11).sqrt()
    };

    // 根据列向量的点积计算 skew_x
    // 偏斜 = atan((m00*m01 + m10*m11) / (m00^2 + m10^2))
    let skew_x_rad = if scale_x.abs() > 1e-10 {
        let dot_product = m00 * m01 + m10 * m11;
        let scale_x_squared = m00 * m00 + m10 * m10;
        (dot_product / scale_x_squared).atan()
    } else {
        0.0
    };
    let skew_x = skew_x_rad * (180.0 / PI);

    CssTransform { x, y, rotation, scale_x, scale_y, skew_x }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // 用于比较浮点数与公差的辅助函数
    fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

    #[test]
    fn test_identity_matrix() {
        let mut tree = json!({
            "transform": {
                "m00": 1.0,
                "m01": 0.0,
                "m02": 0.0,
                "m10": 0.0,
                "m11": 1.0,
                "m12": 0.0
            }
        });

        transform_matrix_to_css(&mut tree).unwrap();

        let transform = tree.get("transform").unwrap();
        assert!(approx_eq(transform["x"].as_f64().unwrap(), 0.0, 1e-10));
        assert!(approx_eq(transform["y"].as_f64().unwrap(), 0.0, 1e-10));
        // 不应出现默认值
        assert!(transform.get("rotation").is_none());
        assert!(transform.get("scaleX").is_none());
        assert!(transform.get("scaleY").is_none());
        assert!(transform.get("skewX").is_none());
    }

    #[test]
    fn test_pure_translation() {
        let mut tree = json!({
            "transform": {
                "m00": 1.0,
                "m01": 0.0,
                "m02": 100.0,
                "m10": 0.0,
                "m11": 1.0,
                "m12": 50.0
            }
        });

        transform_matrix_to_css(&mut tree).unwrap();

        let transform = tree.get("transform").unwrap();
        assert!(approx_eq(transform["x"].as_f64().unwrap(), 100.0, 1e-10));
        assert!(approx_eq(transform["y"].as_f64().unwrap(), 50.0, 1e-10));
        // 不应出现默认值
        assert!(transform.get("rotation").is_none());
        assert!(transform.get("scaleX").is_none());
        assert!(transform.get("scaleY").is_none());
        assert!(transform.get("skewX").is_none());
    }

    #[test]
    fn test_pure_scale() {
        let mut tree = json!({
            "transform": {
                "m00": 2.0,
                "m01": 0.0,
                "m02": 0.0,
                "m10": 0.0,
                "m11": 3.0,
                "m12": 0.0
            }
        });

        transform_matrix_to_css(&mut tree).unwrap();

        let transform = tree.get("transform").unwrap();
        assert!(approx_eq(transform["x"].as_f64().unwrap(), 0.0, 1e-10));
        assert!(approx_eq(transform["y"].as_f64().unwrap(), 0.0, 1e-10));
        assert!(approx_eq(transform["scaleX"].as_f64().unwrap(), 2.0, 1e-10));
        assert!(approx_eq(transform["scaleY"].as_f64().unwrap(), 3.0, 1e-10));
        // 不应出现默认旋转和倾斜
        assert!(transform.get("rotation").is_none());
        assert!(transform.get("skewX").is_none());
    }

    #[test]
    fn test_pure_rotation_45_degrees() {
        // 45度旋转：cos(45°) ≈ 0.7071，sin(45°) ≈ 0.7071
        let cos45 = std::f64::consts::FRAC_1_SQRT_2;
        let sin45 = std::f64::consts::FRAC_1_SQRT_2;

        let mut tree = json!({
            "transform": {
                "m00": cos45,
                "m01": -sin45,
                "m02": 0.0,
                "m10": sin45,
                "m11": cos45,
                "m12": 0.0
            }
        });

        transform_matrix_to_css(&mut tree).unwrap();

        let transform = tree.get("transform").unwrap();
        assert!(approx_eq(transform["x"].as_f64().unwrap(), 0.0, 1e-10));
        assert!(approx_eq(transform["y"].as_f64().unwrap(), 0.0, 1e-10));
        assert!(approx_eq(transform["rotation"].as_f64().unwrap(), 45.0, 1e-8));
        // 不应出现默认比例和倾斜
        assert!(transform.get("scaleX").is_none());
        assert!(transform.get("scaleY").is_none());
        assert!(transform.get("skewX").is_none());
    }

    #[test]
    fn test_pure_rotation_90_degrees() {
        // 90 度旋转：cos(90°) = 0，sin(90°) = 1
        let mut tree = json!({
            "transform": {
                "m00": 0.0,
                "m01": -1.0,
                "m02": 0.0,
                "m10": 1.0,
                "m11": 0.0,
                "m12": 0.0
            }
        });

        transform_matrix_to_css(&mut tree).unwrap();

        let transform = tree.get("transform").unwrap();
        assert!(approx_eq(transform["x"].as_f64().unwrap(), 0.0, 1e-10));
        assert!(approx_eq(transform["y"].as_f64().unwrap(), 0.0, 1e-10));
        assert!(approx_eq(transform["rotation"].as_f64().unwrap(), 90.0, 1e-8));
        // 不应出现默认比例和倾斜
        assert!(transform.get("scaleX").is_none());
        assert!(transform.get("scaleY").is_none());
        assert!(transform.get("skewX").is_none());
    }

    #[test]
    fn test_combined_translation_scale_rotation() {
        // 30 度旋转，比例为 2x, 3y，翻译为 (100, 50)
        let angle = 30.0 * PI / 180.0;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let sx = 2.0;
        let sy = 3.0;

        let mut tree = json!({
            "transform": {
                "m00": sx * cos_a,
                "m01": -sy * sin_a,
                "m02": 100.0,
                "m10": sx * sin_a,
                "m11": sy * cos_a,
                "m12": 50.0
            }
        });

        transform_matrix_to_css(&mut tree).unwrap();

        let transform = tree.get("transform").unwrap();
        assert!(approx_eq(transform["x"].as_f64().unwrap(), 100.0, 1e-10));
        assert!(approx_eq(transform["y"].as_f64().unwrap(), 50.0, 1e-10));
        assert!(approx_eq(transform["rotation"].as_f64().unwrap(), 30.0, 1e-8));
        assert!(approx_eq(transform["scaleX"].as_f64().unwrap(), 2.0, 1e-10));
        assert!(approx_eq(transform["scaleY"].as_f64().unwrap(), 3.0, 1e-10));
        // 不应出现默认倾斜
        assert!(transform.get("skewX").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "name": "Root",
            "transform": {
                "m00": 1.0,
                "m01": 0.0,
                "m02": 10.0,
                "m10": 0.0,
                "m11": 1.0,
                "m12": 20.0
            },
            "children": [
                {
                    "name": "Child1",
                    "transform": {
                        "m00": 2.0,
                        "m01": 0.0,
                        "m02": 5.0,
                        "m10": 0.0,
                        "m11": 2.0,
                        "m12": 10.0
                    }
                }
            ]
        });

        transform_matrix_to_css(&mut tree).unwrap();

        // 检查根变换(仅平移，应该只有 x 和 y)
        let root_transform = tree.get("transform").unwrap();
        assert!(approx_eq(root_transform["x"].as_f64().unwrap(), 10.0, 1e-10));
        assert!(approx_eq(root_transform["y"].as_f64().unwrap(), 20.0, 1e-10));
        assert!(root_transform.get("rotation").is_none());
        assert!(root_transform.get("scaleX").is_none());
        assert!(root_transform.get("scaleY").is_none());
        assert!(root_transform.get("skewX").is_none());

        // 检查子变换(有比例，应该有x，y，scaleX，scaleY)
        let child_transform = &tree["children"][0]["transform"];
        assert!(approx_eq(child_transform["x"].as_f64().unwrap(), 5.0, 1e-10));
        assert!(approx_eq(child_transform["y"].as_f64().unwrap(), 10.0, 1e-10));
        assert!(approx_eq(child_transform["scaleX"].as_f64().unwrap(), 2.0, 1e-10));
        assert!(approx_eq(child_transform["scaleY"].as_f64().unwrap(), 2.0, 1e-10));
        assert!(child_transform.get("rotation").is_none());
        assert!(child_transform.get("skewX").is_none());
    }

    #[test]
    fn test_non_transform_object_unchanged() {
        let mut tree = json!({
            "name": "Rectangle",
            "position": {
                "x": 10,
                "y": 20
            }
        });

        let original = tree.clone();
        transform_matrix_to_css(&mut tree).unwrap();

        // 应保持不变
        assert_eq!(tree, original);
    }

    #[test]
    fn test_transform_without_matrix_fields_unchanged() {
        let mut tree = json!({
            "transform": {
                "x": 10,
                "y": 20
            }
        });

        let original = tree.clone();
        transform_matrix_to_css(&mut tree).unwrap();

        // 应保持不变，因为它没有矩阵字段
        assert_eq!(tree, original);
    }

    #[test]
    fn test_negative_scale() {
        // 负比例(反射)
        let mut tree = json!({
            "transform": {
                "m00": -1.0,
                "m01": 0.0,
                "m02": 0.0,
                "m10": 0.0,
                "m11": 1.0,
                "m12": 0.0
            }
        });

        transform_matrix_to_css(&mut tree).unwrap();

        let transform = tree.get("transform").unwrap();
        // x 和 y 应该存在
        assert!(approx_eq(transform["x"].as_f64().unwrap(), 0.0, 1e-10));
        assert!(approx_eq(transform["y"].as_f64().unwrap(), 0.0, 1e-10));
        // scaleX应为1.0(大小)，旋转应为180度
        assert!(approx_eq(transform["rotation"].as_f64().unwrap(), 180.0, 1e-8));
        assert!(approx_eq(transform["scaleY"].as_f64().unwrap(), -1.0, 1e-10));
        // scaleX 为 1.0(默认)，因此不应存在
        assert!(transform.get("scaleX").is_none());
        assert!(transform.get("skewX").is_none());
    }

    #[test]
    fn test_real_world_example() {
        // 来自实际的 example.canvas.fig：translate(248, -7)
        let mut tree = json!({
            "transform": {
                "m00": 1.0,
                "m01": 0.0,
                "m02": 248.0,
                "m10": 0.0,
                "m11": 1.0,
                "m12": -7.0
            }
        });

        transform_matrix_to_css(&mut tree).unwrap();

        let transform = tree.get("transform").unwrap();
        assert!(approx_eq(transform["x"].as_f64().unwrap(), 248.0, 1e-10));
        assert!(approx_eq(transform["y"].as_f64().unwrap(), -7.0, 1e-10));
        // 不应出现默认值
        assert!(transform.get("rotation").is_none());
        assert!(transform.get("scaleX").is_none());
        assert!(transform.get("scaleY").is_none());
        assert!(transform.get("skewX").is_none());
    }
}
