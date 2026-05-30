use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除图像元数据字段
///
/// 递归遍历 JSON 树并删除图像相关元数据：
/// - "thumbHash" - 缩略图哈希数组
/// - "animationFrame" - 动画帧编号
/// - "imageShouldColorManage" - 颜色管理标志
/// - "imageScaleMode" - 图像缩放模式
/// - "originalImageWidth" - 原始图像宽度
/// - "originalImageHeight" - 原始图像高度
/// - "altText" - 图像的替代文本
/// - "imageThumbnail" - 缩略图(图像字段的副本)
/// - "rotation" - 图像旋转(在paint对象内部时)
/// - "scale" - 图像比例(在paint对象内部时)
///
/// 这些字段包含图像元数据，对于基本功能来说不是必需的
/// HTML/CSS 渲染。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有图像元数据字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_image_metadata_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Image",
///     "thumbHash": [],
///     "animationFrame": 0,
///     "imageShouldColorManage": true,
///     "imageScaleMode": {
///         "__enum__": "ImageScaleMode",
///         "value": "FILL"
///     },
///     "visible": true
/// });
/// remove_image_metadata_fields(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "visible" 字段
/// ```
pub fn remove_image_metadata_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除图像元数据字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除图像元数据字段(如果存在)
            map.remove("thumbHash");
            map.remove("animationFrame");
            map.remove("imageShouldColorManage");
            map.remove("imageScaleMode");
            map.remove("originalImageWidth");
            map.remove("originalImageHeight");
            map.remove("altText");
            map.remove("imageThumbnail");

            // 检查这是否是具有图像属性的paint对象
            // (仅在某些情况下才应删除旋转和缩放)
            if map.contains_key("type")
                && let Some(type_val) = map.get("type")
                && let Some(type_obj) = type_val.as_object()
                && let Some(value_str) = type_obj.get("value").and_then(|v| v.as_str())
                && value_str == "IMAGE"
            {
                // 这是一个图像绘制对象，删除旋转和缩放
                map.remove("rotation");
                map.remove("scale");
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
#[path = "image_metadata_removal_tests.rs"]
mod image_metadata_removal_tests;
