use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中的所有节点中删除 `type` 字段。
///
/// `type` 字段指示节点类型(例如，"FRAME"、"INSTANCE"、"TEXT"、"RECTANGLE")。
/// 虽然这提供了有关节点结构的语义信息，但这不是必需的
/// 用于 HTML/CSS 渲染，其中元素类型通常由其他属性确定
/// (例如，文本内容、布局属性、视觉属性)。
///
/// 常见类型值包括：
/// - `FRAME`：容器节点
/// - `INSTANCE`：组件实例
/// - `TEXT`：文本节点
/// - `RECTANGLE`：矩形
/// - `ELLIPSE`：椭圆形
/// - `VECTOR`：矢量路径
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_type;
///
/// let mut tree = json!({
///     "name": "MyFrame",
///     "type": "FRAME",
///     "size": {"x": 100.0, "y": 100.0}
/// });
///
/// remove_type(&mut tree).unwrap();
///
/// assert!(tree.get("type").is_none());
/// assert!(tree.get("name").is_some());
/// assert!(tree.get("size").is_some());
/// ```
pub fn remove_type(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除type 字段
            map.remove("type");

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
#[path = "type_removal_tests.rs"]
mod type_removal_tests;
