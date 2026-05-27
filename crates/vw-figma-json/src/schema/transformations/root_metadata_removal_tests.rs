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
