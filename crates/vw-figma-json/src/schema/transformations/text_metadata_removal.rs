use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除文本元数据字段
///
/// 递归遍历JSON树并移除文本配置字段：
/// - "textBidiVersion" - 双向文本版本
/// - "textExplicitLayoutVersion" - 显式布局版本
/// - "textUserLayoutVersion" - 用户布局版本
/// - "textDecorationSkipInk" - 文本修饰跳过墨迹设置
/// - "fontVariantCommonLigatures" - 字体连字设置
/// - "fontVariantContextualLigatures" - 上下文连字设置
/// - "fontVariantNumericFigure" - 数字变量(LINING、OLDSTYLE 等)
/// - "fontVariantNumericSpacing" - 数字间距变量(比例、表格等)
/// - "fontVariations" - 字体变化数组
/// - "fontVersion" - 字体版本字符串
/// - "emojiImageSet" - 表情符号图像集枚举
/// - "autoRename" - 自动重命名标志
/// - "textTracking" - 文本跟踪值
/// - "textAlignVertical" - 垂直文本对齐(Figma 自动布局)
/// - "textAutoResize" - 文本自动调整大小行为(Figma 自动布局)
///
/// 这些字段包含不需要的文本渲染配置
/// 基本 HTML/CSS 文本渲染。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有文本元数据字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_text_metadata_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Text",
///     "textBidiVersion": 1,
///     "textUserLayoutVersion": 5,
///     "autoRename": true,
///     "fontSize": 16.0
/// });
/// remove_text_metadata_fields(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "fontSize" 字段
/// ```
pub fn remove_text_metadata_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除文本元数据字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除所有文本元数据字段(如果存在)
            map.remove("textBidiVersion");
            map.remove("textExplicitLayoutVersion");
            map.remove("textUserLayoutVersion");
            map.remove("textDecorationSkipInk");
            map.remove("fontVariantCommonLigatures");
            map.remove("fontVariantContextualLigatures");
            map.remove("fontVariantNumericFigure");
            map.remove("fontVariantNumericSpacing");
            map.remove("fontVariations");
            map.remove("fontVersion");
            map.remove("emojiImageSet");
            map.remove("autoRename");
            map.remove("textTracking");
            map.remove("textAlignVertical");
            map.remove("textAutoResize");

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
#[path = "text_metadata_removal_tests.rs"]
mod text_metadata_removal_tests;
