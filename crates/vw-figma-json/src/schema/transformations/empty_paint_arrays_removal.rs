use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除空的 fillPaints 和 strokePaints 数组。
///
/// 此转换会删除 `fillPaints` 和 `strokePaints` 字段
/// 包含空数组。隐形涂料被清除后，可能会出现空涂料阵列。
/// 被过滤掉，或者当节点明确没有填充或描边时。
///
/// 删除空绘制数组会导致 HTML/CSS 转换的 JSON 输出更清晰，
/// 因为缺少该字段在语义上等同于空数组(无填充/笔画)。
///
/// 此转换通常应在 `invisible_paints_removal` 之后运行以进行清理
/// 过滤产生的空数组。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_empty_paint_arrays;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "fillPaints": [],
///     "strokePaints": [],
///     "size": {"x": 100.0, "y": 100.0}
/// });
///
/// remove_empty_paint_arrays(&mut tree).unwrap();
///
/// assert!(tree.get("fillPaints").is_none());
/// assert!(tree.get("strokePaints").is_none());
/// assert!(tree.get("size").is_some());
/// ```
pub fn remove_empty_paint_arrays(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查并删除空的 fillPaints 数组
            if let Some(JsonValue::Array(paints)) = map.get("fillPaints")
                && paints.is_empty()
            {
                map.remove("fillPaints");
            }

            // 检查并删除空的 strokePaints 数组
            if let Some(JsonValue::Array(paints)) = map.get("strokePaints")
                && paints.is_empty()
            {
                map.remove("strokePaints");
            }

            // 递归到所有剩余值
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

#[cfg(test)]
#[path = "empty_paint_arrays_removal_tests.rs"]
mod empty_paint_arrays_removal_tests;
