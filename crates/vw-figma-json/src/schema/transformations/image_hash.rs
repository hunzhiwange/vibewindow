use crate::error::Result;
use serde_json::Value as JsonValue;
use std::fs;
use std::io::Read;
use std::path::Path;

/// 将图像哈希数组转换为带有扩展名的文件名字符串
///
/// 递归遍历 JSON 树并转换 "image" 和 中的对象
/// "imageThumbnail" 字段：
/// - 将 "hash" 整数数组转换为十六进制编码的 "filename" 字符串
/// - 从文件头检测图像格式(PNG、JPEG、WebP、GIF、SVG)
/// - 重命名物理文件以包含适当的扩展名
/// - 更新 JSON 中的文件名以包含扩展名
/// - 删除 "hash" 字段
/// - 保留所有其他字段(包括 "name")
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
/// * `base_dir` - 图像文件所在的基本目录(相对于输出 JSON)
///
/// # 返回值
/// * `Ok(())` - 成功转换所有图像哈希值
///
/// # 示例
/// ```no_run
/// use fig2json::schema::transform_image_hashes;
/// use serde_json::json;
/// use std::path::Path;
///
/// let mut tree = json!({
///     "image": {
///         "hash": [96, 73, 161, 122],
///         "name": "Amazon-beast"
///     }
/// });
/// transform_image_hashes(&mut tree, Path::new("/output/dir")).unwrap();
/// // 树现在有 "image": {"filename": "images/6049a17a.jpg", "name": "Amazon-beast"}
/// ```
pub fn transform_image_hashes(tree: &mut JsonValue, base_dir: &Path) -> Result<()> {
    transform_recursive(tree, base_dir)
}

/// 递归地转换 JSON 值中的图像哈希值
fn transform_recursive(value: &mut JsonValue, base_dir: &Path) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 首先，检查该对象是否在 "image" 或 "imageThumbnail" 字段中
            // 我们需要转换我们找到的任何此类字段
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if key == "image" || key == "imageThumbnail" {
                    // 该字段可能需要转换
                    if let Some(image_obj) = map.get_mut(&key)
                        && let Some(obj) = image_obj.as_object_mut()
                    {
                        // 检查是否有 "hash" 字段
                        if let Some(hash_value) = obj.get("hash")
                            && let Some(hash_array) = hash_value.as_array()
                        {
                            // 将哈希数组转换为文件名
                            if let Some(mut filename) = hash_to_filename(hash_array) {
                                // 尝试检测格式并重命名物理文件
                                let file_path = base_dir.join(&filename);

                                if let Some(extension) = detect_image_format(&file_path) {
                                    // 用扩展名重命名物理文件
                                    let new_filename = format!("{}{}", filename, extension);
                                    let new_file_path = base_dir.join(&new_filename);

                                    // 尝试重命名文件
                                    // 如果失败，我们仍然会使用扩展名更新 JSON
                                    // (用户可能已经重命名文件，或者文件可能尚不存在)
                                    let _ = fs::rename(&file_path, &new_file_path);

                                    // 更新文件名以包含扩展名
                                    filename = new_filename;
                                }

                                // 删除哈希字段
                                obj.remove("hash");
                                // 添加文件名字段(带或不带扩展名)
                                obj.insert("filename".to_string(), JsonValue::String(filename));
                            }
                        }
                    }
                }

                // 递归到该值，不管
                if let Some(val) = map.get_mut(&key) {
                    transform_recursive(val, base_dir)?;
                }
            }
        }
        JsonValue::Array(arr) => {
            // 递归到数组元素
            for val in arr.iter_mut() {
                transform_recursive(val, base_dir)?;
            }
        }
        _ => {
            // 原始值，无需处理
        }
    }

    Ok(())
}

/// 将整数哈希数组转换为文件名字符串
///
/// 将每个整数转换为其 2 位十六进制表示形式并连接
/// 它们带有 "images/" 前缀。
///
/// # 参数
/// * `hash` - 表示哈希值的整数数组
///
/// # 返回值
/// * `Some(String)` - 文件名字符串(例如 "images/6049a17a...")
/// * `None` - 如果任何元素不是有效的 u8 整数
fn hash_to_filename(hash: &[JsonValue]) -> Option<String> {
    let mut hex_string = String::with_capacity(hash.len() * 2);

    for value in hash {
        if let Some(num) = value.as_u64() {
            if num <= 255 {
                // 格式为 2 位小写十六进制
                hex_string.push_str(&format!("{:02x}", num));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(format!("images/{}", hex_string))
}

/// 从文件头检测图像格式(魔术字节)
///
/// 读取文件的前几个字节以识别图像格式。
///
/// # 参数
/// * `file_path` - 图像文件的路径
///
/// # 返回值
/// * `Some(String)` - 文件扩展名(例如 ".png"、".jpg"、".webp"、".gif"、".svg")
/// * `None` - 如果无法检测到格式或无法读取文件
fn detect_image_format(file_path: &Path) -> Option<String> {
    // 读取前 256 个字节以进行格式检测
    let mut file = fs::File::open(file_path).ok()?;
    let mut buffer = vec![0u8; 256];
    let bytes_read = file.read(&mut buffer).ok()?;

    if bytes_read < 4 {
        return None;
    }

    // PNG：89 50 4E 47 0D 0A 1A 0A
    if bytes_read >= 8 && buffer[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some(".png".to_string());
    }

    // JPEG：FF D8 FF
    if bytes_read >= 3 && buffer[0..3] == [0xFF, 0xD8, 0xFF] {
        return Some(".jpg".to_string());
    }

    // GIF：47 49 46 38 (GIF8)
    if bytes_read >= 4 && buffer[0..4] == [0x47, 0x49, 0x46, 0x38] {
        return Some(".gif".to_string());
    }

    // WebP：52 49 46 46 [4字节] 57 45 42 50(RIFF ....WEBP)
    if bytes_read >= 12
        && buffer[0..4] == [0x52, 0x49, 0x46, 0x46]
        && buffer[8..12] == [0x57, 0x45, 0x42, 0x50]
    {
        return Some(".webp".to_string());
    }

    // SVG：检查 XML/SVG 标记(基于文本)
    if let Ok(text) = std::str::from_utf8(&buffer[..bytes_read]) {
        let text_lower = text.to_lowercase();
        if text_lower.contains("<svg")
            || (text_lower.contains("<?xml") && text_lower.contains("svg"))
        {
            return Some(".svg".to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_hash_to_filename() {
        let hash = vec![
            JsonValue::from(96),
            JsonValue::from(73),
            JsonValue::from(161),
            JsonValue::from(122),
        ];

        let filename = hash_to_filename(&hash).unwrap();
        assert_eq!(filename, "images/6049a17a");
    }

    #[test]
    fn test_hash_to_filename_full() {
        let hash = vec![
            JsonValue::from(96),
            JsonValue::from(73),
            JsonValue::from(161),
            JsonValue::from(122),
            JsonValue::from(132),
            JsonValue::from(131),
            JsonValue::from(226),
            JsonValue::from(80),
            JsonValue::from(226),
            JsonValue::from(150),
            JsonValue::from(78),
            JsonValue::from(100),
            JsonValue::from(84),
            JsonValue::from(218),
            JsonValue::from(142),
            JsonValue::from(231),
            JsonValue::from(161),
            JsonValue::from(69),
            JsonValue::from(66),
            JsonValue::from(133),
        ];

        let filename = hash_to_filename(&hash).unwrap();
        assert_eq!(filename, "images/6049a17a8483e250e2964e6454da8ee7a1454285");
    }

    #[test]
    fn test_hash_to_filename_invalid() {
        let hash = vec![JsonValue::from(256)]; // Out of u8 range
        assert!(hash_to_filename(&hash).is_none());
    }

    #[test]
    fn test_transform_image_field() {
        let mut tree = json!({
            "name": "Rectangle",
            "image": {
                "hash": [96, 73, 161, 122],
                "name": "Amazon-beast"
            }
        });

        // 使用测试路径(文件不存在，因此不会添加扩展名)
        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

        let image = tree.get("image").unwrap();
        assert!(image.get("hash").is_none());
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/6049a17a"));
        assert_eq!(image.get("name").unwrap().as_str(), Some("Amazon-beast"));
    }

    #[test]
    fn test_transform_image_thumbnail_field() {
        let mut tree = json!({
            "name": "Rectangle",
            "imageThumbnail": {
                "hash": [96, 73, 161, 122, 132, 131],
                "name": "Test-Image"
            }
        });

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

        let thumbnail = tree.get("imageThumbnail").unwrap();
        assert!(thumbnail.get("hash").is_none());
        assert_eq!(thumbnail.get("filename").unwrap().as_str(), Some("images/6049a17a8483"));
        assert_eq!(thumbnail.get("name").unwrap().as_str(), Some("Test-Image"));
    }

    #[test]
    fn test_transform_nested_objects() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "image": {
                        "hash": [96, 73],
                        "name": "Image1"
                    }
                },
                {
                    "name": "Child2",
                    "fills": [
                        {
                            "image": {
                                "hash": [161, 122],
                                "name": "Image2"
                            }
                        }
                    ]
                }
            ]
        });

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

        // 检查第一个嵌套图像
        let child1_image = &tree["children"][0]["image"];
        assert!(child1_image.get("hash").is_none());
        assert_eq!(child1_image.get("filename").unwrap().as_str(), Some("images/6049"));

        // 检查深层嵌套图像
        let child2_image = &tree["children"][1]["fills"][0]["image"];
        assert!(child2_image.get("hash").is_none());
        assert_eq!(child2_image.get("filename").unwrap().as_str(), Some("images/a17a"));
    }

    #[test]
    fn test_transform_preserves_other_fields() {
        let mut tree = json!({
            "name": "Rectangle",
            "visible": true,
            "image": {
                "hash": [96, 73, 161, 122],
                "name": "Amazon-beast",
                "width": 100,
                "height": 200
            },
            "x": 10,
            "y": 20
        });

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

        // 检查非图像字段是否被保留
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));

        // 检查图像对象是否保留除哈希之外的所有字段
        let image = tree.get("image").unwrap();
        assert!(image.get("hash").is_none());
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/6049a17a"));
        assert_eq!(image.get("name").unwrap().as_str(), Some("Amazon-beast"));
        assert_eq!(image.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(image.get("height").unwrap().as_i64(), Some(200));
    }

    #[test]
    fn test_transform_no_hash_field() {
        let mut tree = json!({
            "name": "Rectangle",
            "image": {
                "name": "Amazon-beast",
                "url": "https://example.com/image.png"
            }
        });

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

        // 没有哈希值的图像应该保持不变
        let image = tree.get("image").unwrap();
        assert!(image.get("hash").is_none());
        assert!(image.get("filename").is_none());
        assert_eq!(image.get("name").unwrap().as_str(), Some("Amazon-beast"));
        assert_eq!(image.get("url").unwrap().as_str(), Some("https://example.com/image.png"));
    }

    #[test]
    fn test_transform_both_image_and_thumbnail() {
        let mut tree = json!({
            "name": "Rectangle",
            "image": {
                "hash": [96, 73],
                "name": "Main-Image"
            },
            "imageThumbnail": {
                "hash": [161, 122],
                "name": "Thumbnail"
            }
        });

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

        let image = tree.get("image").unwrap();
        assert!(image.get("hash").is_none());
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/6049"));
        assert_eq!(image.get("name").unwrap().as_str(), Some("Main-Image"));

        let thumbnail = tree.get("imageThumbnail").unwrap();
        assert!(thumbnail.get("hash").is_none());
        assert_eq!(thumbnail.get("filename").unwrap().as_str(), Some("images/a17a"));
        assert_eq!(thumbnail.get("name").unwrap().as_str(), Some("Thumbnail"));
    }

    #[test]
    fn test_transform_ignores_other_hash_fields() {
        let mut tree = json!({
            "name": "Node",
            "metadata": {
                "hash": [1, 2, 3, 4],
                "type": "checksum"
            },
            "image": {
                "hash": [96, 73],
                "name": "Real-Image"
            }
        });

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

        // metadata.hash 应保持不变(不在 "image" 或 "imageThumbnail" 字段中)
        let metadata = tree.get("metadata").unwrap();
        assert!(metadata.get("hash").is_some());
        assert!(metadata.get("filename").is_none());

        // image.hash 应该被转换
        let image = tree.get("image").unwrap();
        assert!(image.get("hash").is_none());
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/6049"));
    }

    // 使用实际文件测试图像格式检测

    #[test]
    fn test_detect_png_format() {
        use std::io::Write;

        // 创建临时目录
        let temp_dir = std::env::temp_dir().join("fig2json_test_png");
        let _ = fs::create_dir_all(&temp_dir);

        // 使用 PNG magic bytes 创建测试文件
        let test_file = temp_dir.join("images").join("6049a17a");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap();
        file.write_all(&[0; 100]).unwrap(); // Add some padding
        drop(file);

        // 测试转换
        let mut tree = json!({
            "image": {
                "hash": [96, 73, 161, 122],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/6049a17a.png"));

        // 验证文件已重命名
        assert!(temp_dir.join("images/6049a17a.png").exists());
        assert!(!test_file.exists());

        // 清理
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_jpeg_format() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_jpeg");
        let _ = fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("images").join("a17a6049");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(&[0xFF, 0xD8, 0xFF, 0xE0]).unwrap(); // JPEG magic bytes
        file.write_all(&[0; 100]).unwrap();
        drop(file);

        let mut tree = json!({
            "image": {
                "hash": [161, 122, 96, 73],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/a17a6049.jpg"));

        assert!(temp_dir.join("images/a17a6049.jpg").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_gif_format() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_gif");
        let _ = fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("images").join("12345678");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"GIF89a").unwrap(); // GIF magic bytes
        file.write_all(&[0; 100]).unwrap();
        drop(file);

        let mut tree = json!({
            "image": {
                "hash": [0x12, 0x34, 0x56, 0x78],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/12345678.gif"));

        assert!(temp_dir.join("images/12345678.gif").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_webp_format() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_webp");
        let _ = fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("images").join("abcdef12");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"RIFF").unwrap();
        file.write_all(&[0, 0, 0, 0]).unwrap(); // Size placeholder
        file.write_all(b"WEBP").unwrap();
        file.write_all(&[0; 100]).unwrap();
        drop(file);

        let mut tree = json!({
            "image": {
                "hash": [0xab, 0xcd, 0xef, 0x12],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/abcdef12.webp"));

        assert!(temp_dir.join("images/abcdef12.webp").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_svg_format() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_svg");
        let _ = fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("images").join("87654321");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\">")
            .unwrap();
        file.write_all(&[0; 100]).unwrap();
        drop(file);

        let mut tree = json!({
            "image": {
                "hash": [0x87, 0x65, 0x43, 0x21],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/87654321.svg"));

        assert!(temp_dir.join("images/87654321.svg").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_unknown_format_keeps_no_extension() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_unknown");
        let _ = fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("images").join("deadbeef");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"UNKNOWN FORMAT").unwrap(); // Unknown magic bytes
        file.write_all(&[0; 100]).unwrap();
        drop(file);

        let mut tree = json!({
            "image": {
                "hash": [0xde, 0xad, 0xbe, 0xef],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        // 未知格式应保留不带扩展名的文件名
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/deadbeef"));

        // 原始文件应该仍然存在(未重命名)
        assert!(test_file.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_multiple_images_different_formats() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_multi");
        let _ = fs::create_dir_all(&temp_dir);

        // 创建 PNG 文件
        let png_file = temp_dir.join("images").join("6049");
        fs::create_dir_all(png_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&png_file).unwrap();
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap();
        drop(file);

        // 创建 JPEG 文件
        let jpg_file = temp_dir.join("images").join("a17a");
        let mut file = fs::File::create(&jpg_file).unwrap();
        file.write_all(&[0xFF, 0xD8, 0xFF, 0xE0]).unwrap();
        drop(file);

        let mut tree = json!({
            "children": [
                {
                    "image": {
                        "hash": [96, 73],
                        "name": "Image1"
                    }
                },
                {
                    "fills": [
                        {
                            "image": {
                                "hash": [161, 122],
                                "name": "Image2"
                            }
                        }
                    ]
                }
            ]
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let child1_image = &tree["children"][0]["image"];
        assert_eq!(child1_image.get("filename").unwrap().as_str(), Some("images/6049.png"));

        let child2_image = &tree["children"][1]["fills"][0]["image"];
        assert_eq!(child2_image.get("filename").unwrap().as_str(), Some("images/a17a.jpg"));

        assert!(temp_dir.join("images/6049.png").exists());
        assert!(temp_dir.join("images/a17a.jpg").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
