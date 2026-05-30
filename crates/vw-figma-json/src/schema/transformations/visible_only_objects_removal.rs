use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中删除仅包含visible 属性的对象
///
/// 递归遍历JSON树并移除只有一个key的对象
/// 名为 "visible"。这些对象通常出现在Figma的symbolOverrides数组中
/// 并用于隐藏/显示元素而不提供其他有意义的数据。
///
/// 仅具有 `visible` 的对象将从以下位置删除：
/// - 数组(仅可见的对象元素被过滤掉)
/// - 对象值(如果值只有 `visible`，则删除键值对)
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有仅可见对象
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_visible_only_objects;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "symbolOverrides": [
///         {"visible": false},
///         {"textData": {"characters": "Hello"}},
///         {"visible": true}
///     ]
/// });
/// remove_visible_only_objects(&mut tree).unwrap();
/// // symbolOverrides 现在只有 textData 对象
/// ```
pub fn remove_visible_only_objects(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree);
    Ok(())
}

/// 从 JSON 值中递归删除仅可见对象
fn transform_recursive(value: &mut JsonValue) {
    match value {
        JsonValue::Object(map) => {
            // 首先，递归到所有值
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in &keys {
                if let Some(val) = map.get_mut(key) {
                    transform_recursive(val);
                }
            }

            // 然后删除其值为仅可见对象的所有键
            map.retain(|_, v| !is_visible_only_object(v));
        }
        JsonValue::Array(arr) => {
            // 首先，递归到数组元素
            for val in arr.iter_mut() {
                transform_recursive(val);
            }

            // 然后从数组中过滤掉仅可见的对象
            arr.retain(|v| !is_visible_only_object(v));
        }
        _ => {
            // 原始值，无需处理
        }
    }
}

/// 检查 JSON 值是否是仅具有 "visible" 键的对象
fn is_visible_only_object(value: &JsonValue) -> bool {
    match value {
        JsonValue::Object(map) => map.len() == 1 && map.contains_key("visible"),
        _ => false,
    }
}

#[cfg(test)]
#[path = "visible_only_objects_removal_tests.rs"]
mod visible_only_objects_removal_tests;
