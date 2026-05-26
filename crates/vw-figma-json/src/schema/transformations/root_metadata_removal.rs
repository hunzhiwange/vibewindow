use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 文档中删除根级元数据字段(版本和文件类型)
///
/// 这些字段是 Figma 特定的元数据，HTML/CSS 渲染不需要这些元数据。
/// 此函数仅从根级别删除它们，以保持输出干净和
/// 专注于可渲染内容。
///
/// 删除的字段：
/// - "version" - Figma 文件格式版本号
/// - "fileType" - 文件类型标识符(例如 "figma"、"figjam")
///
/// # 参数
/// * `json` - 根 JSON 对象(通常包含版本、文件类型、文档)
///
/// # 返回值
/// * `Ok(())` - 成功删除元数据字段(或者它们不存在)
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_root_metadata;
/// use serde_json::json;
///
/// let mut output = json!({
///     "version": 101,
///     "fileType": "figma",
///     "document": {"name": "Root"}
/// });
/// remove_root_metadata(&mut output).unwrap();
/// // 输出现在只有文档字段，版本和文件类型已删除
/// ```
pub fn remove_root_metadata(json: &mut JsonValue) -> Result<()> {
    if let Some(obj) = json.as_object_mut() {
        obj.remove("version");
        obj.remove("fileType");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_version_and_file_type() {
        let mut output = json!({
            "version": 101,
            "fileType": "figma",
            "document": {
                "name": "Root",
                "children": []
            }
        });

        remove_root_metadata(&mut output).unwrap();

        // 版本和文件类型
        assert!(output.get("version").is_none());
        assert!(output.get("fileType").is_none());

        // 文件应保留
        assert!(output.get("document").is_some());
        assert_eq!(output.get("document").unwrap().get("name").unwrap().as_str(), Some("Root"));
    }

    #[test]
    fn test_remove_only_version() {
        let mut output = json!({
            "version": 101,
            "document": {
                "name": "Root"
            }
        });

        remove_root_metadata(&mut output).unwrap();

        // 版本应删除
        assert!(output.get("version").is_none());

        // 文件应保留
        assert!(output.get("document").is_some());
    }

    #[test]
    fn test_remove_only_file_type() {
        let mut output = json!({
            "fileType": "figjam",
            "document": {
                "name": "Board"
            }
        });

        remove_root_metadata(&mut output).unwrap();

        // 文件类型应该被删除
        assert!(output.get("fileType").is_none());

        // 文件应保留
        assert!(output.get("document").is_some());
    }

    #[test]
    fn test_already_missing() {
        let mut output = json!({
            "document": {
                "name": "Document"
            }
        });

        // 如果版本和文件类型已经丢失，则不应失败
        remove_root_metadata(&mut output).unwrap();

        assert!(output.get("version").is_none());
        assert!(output.get("fileType").is_none());
        assert!(output.get("document").is_some());
    }

    #[test]
    fn test_preserves_other_root_fields() {
        let mut output = json!({
            "version": 101,
            "fileType": "figma",
            "document": {
                "name": "Document"
            },
            "metadata": {
                "custom": "field"
            },
            "otherData": [1, 2, 3]
        });

        remove_root_metadata(&mut output).unwrap();

        // 仅应删除版本和文件类型
        assert!(output.get("version").is_none());
        assert!(output.get("fileType").is_none());

        // 保留所有其他字段
        assert!(output.get("document").is_some());
        assert!(output.get("metadata").is_some());
        assert!(output.get("otherData").is_some());
        assert_eq!(output.get("metadata").unwrap().get("custom").unwrap().as_str(), Some("field"));
    }

    #[test]
    fn test_preserves_nested_version_and_file_type() {
        let mut output = json!({
            "version": 101,
            "fileType": "figma",
            "document": {
                "name": "Document",
                "version": 2,
                "metadata": {
                    "fileType": "custom"
                }
            }
        });

        remove_root_metadata(&mut output).unwrap();

        // 根级版本和文件类型已删除
        assert!(output.get("version").is_none());
        assert!(output.get("fileType").is_none());

        // 保留嵌套版本和文件类型
        assert_eq!(output["document"].get("version").unwrap().as_i64(), Some(2));
        assert_eq!(
            output["document"]["metadata"].get("fileType").unwrap().as_str(),
            Some("custom")
        );
    }

    #[test]
    fn test_different_version_values() {
        let versions = vec![48, 50, 101, 999];

        for version in versions {
            let mut output = json!({
                "version": version,
                "document": {}
            });

            remove_root_metadata(&mut output).unwrap();

            // 应删除所有版本值
            assert!(output.get("version").is_none());
        }
    }

    #[test]
    fn test_different_file_types() {
        let file_types = vec!["figma", "figjam", "whiteboard"];

        for file_type in file_types {
            let mut output = json!({
                "fileType": file_type,
                "document": {}
            });

            remove_root_metadata(&mut output).unwrap();

            // 应删除所有文件类型值
            assert!(output.get("fileType").is_none());
        }
    }

    #[test]
    fn test_not_an_object() {
        let mut output = json!([1, 2, 3]);

        // 非对象输入不应失败
        remove_root_metadata(&mut output).unwrap();

        // 数组应保持不变
        assert_eq!(output.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_empty_object() {
        let mut output = json!({});

        remove_root_metadata(&mut output).unwrap();

        // 空对象应保持为空
        assert_eq!(output.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_string_primitive() {
        let mut output = json!("document");

        // 原始输入不应失败
        remove_root_metadata(&mut output).unwrap();

        // 字符串应保持不变
        assert_eq!(output.as_str(), Some("document"));
    }
}
