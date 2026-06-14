use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 fillPaints 和 strokePaints 数组中删除不可见的绘制。
///
/// 此转换从 `fillPaints` 和 `strokePaints` 中过滤掉绘制对象
/// 数组，其中 `visible` 属性显式设置为 `false`。隐形涂料
/// 不会在最终输出中呈现，并且 HTML/CSS 转换不需要。
///
/// 没有 `visible` 属性的绘制对象被假定为可见并被保留。
/// 具有 `visible: true` 的绘制对象也被保留(尽管 `visible` 属性
/// 本身可能会被处理默认值的其他转换删除)。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_invisible_paints;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "fillPaints": [
///         {
///             "color": "#ffffff",
///             "type": "SOLID",
///             "visible": false
///         },
///         {
///             "color": "#000000",
///             "type": "SOLID"
///         }
///     ]
/// });
///
/// remove_invisible_paints(&mut tree).unwrap();
///
/// let fills = tree.get("fillPaints").unwrap().as_array().unwrap();
/// assert_eq!(fills.len(), 1);
/// assert_eq!(fills[0].get("color").unwrap().as_str(), Some("#000000"));
/// ```
pub fn remove_invisible_paints(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 过滤 fillPaints 数组
            if let Some(JsonValue::Array(paints)) = map.get_mut("fillPaints") {
                paints.retain(|paint| !is_invisible(paint));
            }

            // 过滤笔画数组
            if let Some(JsonValue::Array(paints)) = map.get_mut("strokePaints") {
                paints.retain(|paint| !is_invisible(paint));
            }

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

/// 检查paint对象是否不可见(可见： false)
fn is_invisible(paint: &JsonValue) -> bool {
    if let Some(visible) = paint.get("visible")
        && let Some(visible_bool) = visible.as_bool()
    {
        return !visible_bool;
    }
    false
}

#[cfg(test)]
#[path = "invisible_paints_removal_tests.rs"]
mod invisible_paints_removal_tests;
