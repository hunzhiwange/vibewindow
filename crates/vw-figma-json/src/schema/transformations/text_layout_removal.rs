use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从衍生文本数据对象中删除详细的文本布局数据
///
/// 递归遍历 JSON 树并从中删除详细的布局字段
/// "derivedTextData" 对象：
/// - "baselines" - 精确的线基线定位
/// - "logicalIndexToCharacterOffsetMap" - 角色位置图
/// - "fontMetaData" - 字体摘要和元数据数组
/// - "derivedLines" - 线路方向性信息
/// - "truncatedHeight" - 截断高度值
/// - "truncationStartIndex" - 截断起始索引
///
/// 这些字段包含精确的文本布局数据，这些数据不需要
/// 基本 HTML/CSS 文本渲染。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有文本布局字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_text_layout_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "derivedTextData": {
///         "baselines": [{"lineY": 10.0, "width": 100.0}],
///         "logicalIndexToCharacterOffsetMap": [0.0, 10.0, 20.0],
///         "fontMetaData": [{"fontDigest": [1, 2, 3]}],
///         "layoutSize": {"x": 100.0, "y": 50.0}
///     }
/// });
/// remove_text_layout_fields(&mut tree).unwrap();
/// // derivedTextData 现在只有 "layoutSize"
/// ```
pub fn remove_text_layout_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除文本布局字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查该对象是否有 "derivedTextData" 字段
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if key == "derivedTextData" {
                    // 该字段可能包含要删除的布局数据
                    if let Some(derived_text_data) = map.get_mut(&key)
                        && let Some(obj) = derived_text_data.as_object_mut()
                    {
                        // 删除所有详细布局字段
                        obj.remove("baselines");
                        obj.remove("logicalIndexToCharacterOffsetMap");
                        obj.remove("fontMetaData");
                        obj.remove("derivedLines");
                        obj.remove("truncatedHeight");
                        obj.remove("truncationStartIndex");
                    }
                }

                // 递归到该值，不管
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
#[path = "text_layout_removal_tests.rs"]
mod text_layout_removal_tests;
