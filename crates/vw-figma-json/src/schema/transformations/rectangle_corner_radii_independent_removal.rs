use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除矩形CornerRadiiIndependent 字段
///
/// 递归遍历 JSON 树并删除所有 "rectangleCornerRadiiIndependent" 字段。
/// 该标志表示拐角半径是否独立设置，当
/// 存在实际的拐角半径值。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有矩形CornerRadii独立字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_rectangle_corner_radii_independent;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "cornerRadius": 16.0,
///     "rectangleCornerRadiiIndependent": true,
///     "rectangleTopLeftCornerRadius": 16.0,
///     "rectangleTopRightCornerRadius": 16.0,
///     "visible": true
/// });
/// remove_rectangle_corner_radii_independent(&mut tree).unwrap();
/// // 树现在有cornerRadius和特定的半径字段，但没有标志
/// ```
pub fn remove_rectangle_corner_radii_independent(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除矩形CornerRadiiIndependent 字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除 "rectangleCornerRadiiIndependent" 字段(如果存在)
            map.remove("rectangleCornerRadiiIndependent");

            // 递归到所有剩余值
            for val in map.values_mut() {
                transform_recursive(val)?;
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

#[cfg(test)]
#[path = "rectangle_corner_radii_independent_removal_tests.rs"]
mod rectangle_corner_radii_independent_removal_tests;
