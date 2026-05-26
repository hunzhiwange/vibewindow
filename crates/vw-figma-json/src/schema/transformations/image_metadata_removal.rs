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
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_thumb_hash() {
        let mut tree = json!({
            "name": "Image",
            "thumbHash": [],
            "visible": true
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("thumbHash").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_animation_frame() {
        let mut tree = json!({
            "name": "Image",
            "animationFrame": 0,
            "opacity": 1.0
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("animationFrame").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_remove_color_manage_flag() {
        let mut tree = json!({
            "name": "Image",
            "imageShouldColorManage": true,
            "visible": true
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("imageShouldColorManage").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_image_scale_mode() {
        let mut tree = json!({
            "name": "Image",
            "imageScaleMode": {
                "__enum__": "ImageScaleMode",
                "value": "FILL"
            },
            "opacity": 1.0
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("imageScaleMode").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_remove_original_dimensions() {
        let mut tree = json!({
            "name": "Image",
            "originalImageWidth": 300,
            "originalImageHeight": 300,
            "visible": true
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("originalImageWidth").is_none());
        assert!(tree.get("originalImageHeight").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_alt_text() {
        let mut tree = json!({
            "name": "Image",
            "altText": "",
            "visible": true
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("altText").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_image_paint_rotation_scale() {
        let mut tree = json!({
            "type": {
                "__enum__": "PaintType",
                "value": "IMAGE"
            },
            "rotation": 0.0,
            "scale": 0.5,
            "opacity": 1.0
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("rotation").is_none());
        assert!(tree.get("scale").is_none());
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_preserve_non_image_rotation_scale() {
        let mut tree = json!({
            "name": "Frame",
            "rotation": 45.0,
            "scale": 2.0,
            "type": "FRAME"
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        // 应为非图像对象保留旋转和缩放
        assert_eq!(tree.get("rotation").unwrap().as_f64(), Some(45.0));
        assert_eq!(tree.get("scale").unwrap().as_f64(), Some(2.0));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_remove_all_image_metadata() {
        let mut tree = json!({
            "name": "ComplexImage",
            "thumbHash": [],
            "animationFrame": 0,
            "imageShouldColorManage": true,
            "imageScaleMode": {
                "__enum__": "ImageScaleMode",
                "value": "FILL"
            },
            "originalImageWidth": 300,
            "originalImageHeight": 300,
            "altText": "",
            "type": {
                "__enum__": "PaintType",
                "value": "IMAGE"
            },
            "rotation": 0.0,
            "scale": 0.5,
            "opacity": 1.0
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        // 删除所有元数据字段
        assert!(tree.get("thumbHash").is_none());
        assert!(tree.get("animationFrame").is_none());
        assert!(tree.get("imageShouldColorManage").is_none());
        assert!(tree.get("imageScaleMode").is_none());
        assert!(tree.get("originalImageWidth").is_none());
        assert!(tree.get("originalImageHeight").is_none());
        assert!(tree.get("altText").is_none());
        assert!(tree.get("rotation").is_none());
        assert!(tree.get("scale").is_none());

        // 保留其他字段
        assert_eq!(tree.get("name").unwrap().as_str(), Some("ComplexImage"));
        assert!(tree.get("type").is_some());
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_nested_image_metadata() {
        let mut tree = json!({
            "name": "Root",
            "fillPaints": [
                {
                    "type": {
                        "__enum__": "PaintType",
                        "value": "IMAGE"
                    },
                    "thumbHash": [],
                    "animationFrame": 0,
                    "rotation": 0.0,
                    "scale": 0.5,
                    "opacity": 1.0
                }
            ]
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        // 检查嵌套图像绘制
        let paint = &tree["fillPaints"][0];
        assert!(paint.get("thumbHash").is_none());
        assert!(paint.get("animationFrame").is_none());
        assert!(paint.get("rotation").is_none());
        assert!(paint.get("scale").is_none());
        assert!(paint.get("type").is_some());
        assert_eq!(paint.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_no_image_metadata() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200,
            "visible": true
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        // 没有图像元数据的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_image_thumbnail() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "IMAGE",
                    "image": {
                        "filename": "images/abc123",
                        "name": "Photo"
                    },
                    "imageThumbnail": {
                        "filename": "images/abc123",
                        "name": "Photo"
                    }
                }
            ]
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        let paint = &tree["fillPaints"][0];
        // imageThumbnail 应删除(图像重复)
        assert!(paint.get("imageThumbnail").is_none());
        // 图像应保留
        assert!(paint.get("image").is_some());
    }

    #[test]
    fn test_remove_image_thumbnail_nested() {
        let mut tree = json!({
            "children": [
                {
                    "fillPaints": [
                        {
                            "image": {"filename": "images/abc"},
                            "imageThumbnail": {"filename": "images/abc"}
                        }
                    ]
                }
            ]
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        let paint = &tree["children"][0]["fillPaints"][0];
        assert!(paint.get("imageThumbnail").is_none());
        assert!(paint.get("image").is_some());
    }
}
