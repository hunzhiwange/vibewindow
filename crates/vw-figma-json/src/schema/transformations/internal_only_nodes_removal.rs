use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中删除仅限内部的节点
///
/// 递归遍历JSON树，过滤掉有的节点
/// `internalOnly: true`。这些是 Figma 内部节点，并不意味着
/// 用于渲染，不应包含在最终输出中。
///
/// 仅内部节点通常从 "children" 数组中删除。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有内部节点
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_internal_only_nodes;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "children": [
///         {"name": "Visible", "visible": true},
///         {"name": "Internal", "internalOnly": true}
///     ]
/// });
/// remove_internal_only_nodes(&mut tree).unwrap();
/// // 子数组现在只包含 "Visible" 节点
/// ```
pub fn remove_internal_only_nodes(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除仅限内部的节点
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 递归到所有值
            for val in map.values_mut() {
                transform_recursive(val)?;
            }

            // 递归后，删除internalOnly字段本身
            // (仅用于过滤，输出中不需要)
            map.remove("internalOnly");
        }
        JsonValue::Array(arr) => {
            // 用internalOnly: true FIRST 过滤掉节点(在递归之前)
            arr.retain(|node| {
                if let Some(obj) = node.as_object() {
                    // 如果internalOnly不为true则保留节点
                    !obj.get("internalOnly").and_then(|v| v.as_bool()).unwrap_or(false)
                } else {
                    // 保留非对象值
                    true
                }
            });

            // 然后递归到剩余的数组元素
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
#[path = "internal_only_nodes_removal_tests.rs"]
mod internal_only_nodes_removal_tests;
