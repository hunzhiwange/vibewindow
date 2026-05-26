use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中删除文档级属性字段
///
/// 递归遍历 JSON 树并删除文档级配置：
/// - "documentColorProfile" - 颜色配置文件设置(SRGB 等)
///
/// 这些字段包含文档级元数据，这些元数据不需要
/// 基本 HTML/CSS 渲染。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有文档属性字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_document_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "document": {
///         "name": "Document",
///         "documentColorProfile": {
///             "__enum__": "DocumentColorProfile",
///             "value": "SRGB"
///         }
///     }
/// });
/// remove_document_properties(&mut tree).unwrap();
/// // 文档现在只有 "name" 字段
/// ```
pub fn remove_document_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除文档属性字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除文档属性字段(如果存在)
            map.remove("documentColorProfile");

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
    fn test_remove_document_color_profile() {
        let mut tree = json!({
            "name": "Document",
            "documentColorProfile": {
                "__enum__": "DocumentColorProfile",
                "value": "SRGB"
            },
            "children": []
        });

        remove_document_properties(&mut tree).unwrap();

        assert!(tree.get("documentColorProfile").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Document"));
        assert!(tree.get("children").is_some());
    }

    #[test]
    fn test_remove_document_color_profile_nested() {
        let mut tree = json!({
            "document": {
                "name": "Document",
                "documentColorProfile": {
                    "__enum__": "DocumentColorProfile",
                    "value": "SRGB"
                },
                "children": [
                    {
                        "name": "Canvas",
                        "documentColorProfile": {
                            "__enum__": "DocumentColorProfile",
                            "value": "DISPLAY_P3"
                        }
                    }
                ]
            }
        });

        remove_document_properties(&mut tree).unwrap();

        // 应删除根文档颜色配置文件
        assert!(tree["document"].get("documentColorProfile").is_none());
        assert_eq!(tree["document"].get("name").unwrap().as_str(), Some("Document"));

        // 应删除嵌套颜色配置文件
        assert!(tree["document"]["children"][0].get("documentColorProfile").is_none());
        assert_eq!(tree["document"]["children"][0].get("name").unwrap().as_str(), Some("Canvas"));
    }

    #[test]
    fn test_no_document_properties() {
        let mut tree = json!({
            "document": {
                "name": "Document",
                "children": []
            }
        });

        remove_document_properties(&mut tree).unwrap();

        // 没有文档属性的树应该保持不变
        assert_eq!(tree["document"].get("name").unwrap().as_str(), Some("Document"));
        assert!(tree["document"].get("children").is_some());
        assert!(tree["document"].get("documentColorProfile").is_none());
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "document": {
                "name": "Document",
                "documentColorProfile": {
                    "__enum__": "DocumentColorProfile",
                    "value": "SRGB"
                },
                "type": "DOCUMENT",
                "opacity": 1.0,
                "visible": true
            }
        });

        remove_document_properties(&mut tree).unwrap();

        // 仅应删除 documentColorProfile
        assert!(tree["document"].get("documentColorProfile").is_none());

        // 保留所有其他字段
        assert_eq!(tree["document"].get("name").unwrap().as_str(), Some("Document"));
        assert_eq!(tree["document"].get("type").unwrap().as_str(), Some("DOCUMENT"));
        assert_eq!(tree["document"].get("opacity").unwrap().as_f64(), Some(1.0));
        assert_eq!(tree["document"].get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_different_color_profile_values() {
        let mut tree = json!({
            "doc1": {
                "documentColorProfile": {
                    "__enum__": "DocumentColorProfile",
                    "value": "SRGB"
                }
            },
            "doc2": {
                "documentColorProfile": {
                    "__enum__": "DocumentColorProfile",
                    "value": "DISPLAY_P3"
                }
            },
            "doc3": {
                "documentColorProfile": {
                    "__enum__": "DocumentColorProfile",
                    "value": "UNMANAGED"
                }
            }
        });

        remove_document_properties(&mut tree).unwrap();

        // documentColorProfile 的所有变体均应删除
        assert!(tree["doc1"].get("documentColorProfile").is_none());
        assert!(tree["doc2"].get("documentColorProfile").is_none());
        assert!(tree["doc3"].get("documentColorProfile").is_none());
    }

    #[test]
    fn test_deeply_nested_color_profile() {
        let mut tree = json!({
            "root": {
                "children": [
                    {
                        "children": [
                            {
                                "documentColorProfile": {
                                    "__enum__": "DocumentColorProfile",
                                    "value": "SRGB"
                                },
                                "name": "DeepNode"
                            }
                        ]
                    }
                ]
            }
        });

        remove_document_properties(&mut tree).unwrap();

        // 应删除深层嵌套的颜色配置文件
        assert!(tree["root"]["children"][0]["children"][0].get("documentColorProfile").is_none());
        assert_eq!(
            tree["root"]["children"][0]["children"][0].get("name").unwrap().as_str(),
            Some("DeepNode")
        );
    }

    #[test]
    fn test_multiple_documents() {
        let mut tree = json!({
            "documents": [
                {
                    "name": "Doc1",
                    "documentColorProfile": {
                        "__enum__": "DocumentColorProfile",
                        "value": "SRGB"
                    }
                },
                {
                    "name": "Doc2",
                    "documentColorProfile": {
                        "__enum__": "DocumentColorProfile",
                        "value": "DISPLAY_P3"
                    }
                }
            ]
        });

        remove_document_properties(&mut tree).unwrap();

        // 数组中的所有颜色配置文件应被删除
        assert!(tree["documents"][0].get("documentColorProfile").is_none());
        assert_eq!(tree["documents"][0].get("name").unwrap().as_str(), Some("Doc1"));

        assert!(tree["documents"][1].get("documentColorProfile").is_none());
        assert_eq!(tree["documents"][1].get("name").unwrap().as_str(), Some("Doc2"));
    }

    #[test]
    fn test_empty_object() {
        let mut tree = json!({});

        remove_document_properties(&mut tree).unwrap();

        // 空对象应保持为空
        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_primitives() {
        let mut tree = json!("document");

        remove_document_properties(&mut tree).unwrap();

        // 原始值应保持不变
        assert_eq!(tree.as_str(), Some("document"));
    }
}
