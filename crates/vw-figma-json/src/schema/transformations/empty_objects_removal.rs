use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中删除空对象 {}
///
/// 递归遍历 JSON 树并删除空对象(没有键的对象)。
/// 这通过消除无意义的空对象文字来减少输出大小，这些空对象文字提供
/// 没有信息。
///
/// 空对象被移除：
/// - 数组(空对象元素被过滤掉)
/// - 对象值(如果值为 {}，则删除键值对)
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有空对象
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_empty_objects;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "data": {},
///     "items": [1, {}, 2, {}]
/// });
/// remove_empty_objects(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "items": [1, 2]
/// ```
pub fn remove_empty_objects(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree);
    Ok(())
}

/// 从 JSON 值中递归删除空对象
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

            // 然后删除任何值为空对象的键
            map.retain(|_, v| !is_empty_object(v));
        }
        JsonValue::Array(arr) => {
            // 首先，递归到数组元素
            for val in arr.iter_mut() {
                transform_recursive(val);
            }

            // 然后从数组中过滤掉空对象
            arr.retain(|v| !is_empty_object(v));
        }
        _ => {
            // 原始值，无需处理
        }
    }
}

/// 检查 JSON 值是否为空对象 {}
fn is_empty_object(value: &JsonValue) -> bool {
    match value {
        JsonValue::Object(map) => map.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
#[path = "empty_objects_removal_tests.rs"]
mod empty_objects_removal_tests;
