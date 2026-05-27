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
