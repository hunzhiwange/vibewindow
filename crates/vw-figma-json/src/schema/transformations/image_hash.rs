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
#[path = "image_hash_tests.rs"]
mod image_hash_tests;
