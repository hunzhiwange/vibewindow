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

#[test]
fn test_image_metadata_primitive_value() {
    let mut tree = json!(false);

    remove_image_metadata_fields(&mut tree).unwrap();

    assert_eq!(tree.as_bool(), Some(false));
}

#[test]
fn test_preserve_rotation_scale_for_non_image_enum_paint() {
    let mut tree = json!({
        "type": {
            "__enum__": "PaintType",
            "value": "SOLID"
        },
        "rotation": 30.0,
        "scale": 2.0
    });

    remove_image_metadata_fields(&mut tree).unwrap();

    assert_eq!(tree.get("rotation").unwrap().as_f64(), Some(30.0));
    assert_eq!(tree.get("scale").unwrap().as_f64(), Some(2.0));
}
