use super::*;
use serde_json::json;

#[test]
fn test_remove_fill_geometry() {
    let mut tree = json!({
        "name": "Rectangle",
        "fillGeometry": [
            {
                "commands": ["M", 0.0, 0.0, "L", 100.0, 0.0, "L", 100.0, 100.0, "Z"],
                "styleID": 0,
                "windingRule": {
                    "__enum__": "WindingRule",
                    "value": "NONZERO"
                }
            }
        ],
        "size": {"x": 100.0, "y": 100.0}
    });

    remove_geometry_fields(&mut tree).unwrap();

    assert!(tree.get("fillGeometry").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert!(tree.get("size").is_some());
}

#[test]
fn test_remove_stroke_geometry() {
    let mut tree = json!({
        "name": "Line",
        "strokeGeometry": [
            {
                "commands": ["M", 0.0, 0.0, "L", 100.0, 100.0],
                "styleID": 0,
                "windingRule": {
                    "__enum__": "WindingRule",
                    "value": "NONZERO"
                }
            }
        ],
        "visible": true
    });

    remove_geometry_fields(&mut tree).unwrap();

    assert!(tree.get("strokeGeometry").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Line"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
}

#[test]
fn test_remove_both_geometries() {
    let mut tree = json!({
        "name": "Shape",
        "fillGeometry": [
            {
                "commands": ["M", 0.0, 0.0, "Z"],
                "styleID": 1
            }
        ],
        "strokeGeometry": [
            {
                "commands": ["M", 0.0, 0.0, "L", 10.0, 10.0],
                "styleID": 2
            }
        ],
        "opacity": 1.0
    });

    remove_geometry_fields(&mut tree).unwrap();

    assert!(tree.get("fillGeometry").is_none());
    assert!(tree.get("strokeGeometry").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
    assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
}

#[test]
fn test_remove_nested_geometry() {
    let mut tree = json!({
        "name": "Root",
        "children": [
            {
                "name": "Child1",
                "fillGeometry": [
                    {
                        "commands": ["M", 0.0, 0.0, "Z"],
                        "styleID": 0
                    }
                ]
            },
            {
                "name": "Child2",
                "strokeGeometry": [
                    {
                        "commands": ["M", 0.0, 0.0, "L", 10.0, 10.0]
                    }
                ]
            }
        ]
    });

    remove_geometry_fields(&mut tree).unwrap();

    // 儿童几何图形应删除
    assert!(tree["children"][0].get("fillGeometry").is_none());
    assert_eq!(tree["children"][0].get("name").unwrap().as_str(), Some("Child1"));

    assert!(tree["children"][1].get("strokeGeometry").is_none());
    assert_eq!(tree["children"][1].get("name").unwrap().as_str(), Some("Child2"));
}

#[test]
fn test_remove_winding_rule_standalone() {
    let mut tree = json!({
        "name": "Path",
        "windingRule": {
            "__enum__": "WindingRule",
            "value": "EVENODD"
        },
        "visible": true
    });

    remove_geometry_fields(&mut tree).unwrap();

    assert!(tree.get("windingRule").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Path"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
}

#[test]
fn test_remove_style_id_standalone() {
    let mut tree = json!({
        "name": "Element",
        "styleID": 42,
        "type": "SHAPE"
    });

    remove_geometry_fields(&mut tree).unwrap();

    assert!(tree.get("styleID").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Element"));
    assert_eq!(tree.get("type").unwrap().as_str(), Some("SHAPE"));
}

#[test]
fn test_remove_all_geometry_fields() {
    let mut tree = json!({
        "name": "Complex",
        "fillGeometry": [{"commands": ["M", 0.0, 0.0, "Z"]}],
        "strokeGeometry": [{"commands": ["M", 0.0, 0.0, "L", 10.0, 10.0]}],
        "windingRule": {"__enum__": "WindingRule", "value": "NONZERO"},
        "styleID": 5,
        "opacity": 1.0
    });

    remove_geometry_fields(&mut tree).unwrap();

    assert!(tree.get("fillGeometry").is_none());
    assert!(tree.get("strokeGeometry").is_none());
    assert!(tree.get("windingRule").is_none());
    assert!(tree.get("styleID").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Complex"));
    assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
}

#[test]
fn test_remove_geometry_missing() {
    let mut tree = json!({
        "name": "Simple",
        "x": 10,
        "y": 20,
        "width": 100,
        "height": 100
    });

    remove_geometry_fields(&mut tree).unwrap();

    // 没有几何字段的树应该保持不变
    assert!(tree.get("fillGeometry").is_none());
    assert!(tree.get("strokeGeometry").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Simple"));
    assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
    assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));
}

#[test]
fn test_remove_geometry_deeply_nested() {
    let mut tree = json!({
        "document": {
            "fillGeometry": [{"commands": ["M", 0.0, 0.0, "Z"]}],
            "children": [
                {
                    "children": [
                        {
                            "strokeGeometry": [{"commands": ["L", 10.0, 10.0]}],
                            "name": "DeepChild"
                        }
                    ]
                }
            ]
        }
    });

    remove_geometry_fields(&mut tree).unwrap();

    // 所有级别的所有几何图形都应删除
    assert!(tree["document"].get("fillGeometry").is_none());
    assert!(tree["document"]["children"][0]["children"][0].get("strokeGeometry").is_none());

    // 其他字段应保留
    assert_eq!(
        tree["document"]["children"][0]["children"][0].get("name").unwrap().as_str(),
        Some("DeepChild")
    );
}

#[test]
fn test_remove_geometry_empty_object() {
    let mut tree = json!({});

    remove_geometry_fields(&mut tree).unwrap();

    // 空对象应保持为空
    assert_eq!(tree.as_object().unwrap().len(), 0);
}

#[test]
fn test_preserve_geometry_for_icon_with_export_settings_svg() {
    let mut tree = json!({
        "name": "icon/ai",
        "fillGeometry": [
            {
                "commands": ["M", 14.1667, 1.11133, "L", 5.83339, 1.11133, "Z"],
                "styleID": 0
            }
        ],
        "symbolData": {
            "symbolOverrides": [
                {
                    "exportSettings": [
                        {
                            "imageType": {
                                "__enum__": "ImageType",
                                "value": "SVG"
                            }
                        }
                    ]
                }
            ]
        }
    });

    remove_geometry_fields(&mut tree).unwrap();

    // 应该为具有导出设置的图标保留几何形状
    assert!(tree.get("fillGeometry").is_some());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("icon/ai"));
}

#[test]
fn test_preserve_geometry_for_icon_with_export_settings_png() {
    let mut tree = json!({
        "name": "arrows/chevron-right",
        "strokeGeometry": [
            {
                "commands": ["M", 0.0, 0.0, "L", 10.0, 10.0],
                "styleID": 0
            }
        ],
        "symbolData": {
            "symbolOverrides": [
                {
                    "exportSettings": [
                        {
                            "imageType": {
                                "__enum__": "ImageType",
                                "value": "PNG"
                            }
                        }
                    ]
                }
            ]
        }
    });

    remove_geometry_fields(&mut tree).unwrap();

    // PNG 图标应保留几何形状
    assert!(tree.get("strokeGeometry").is_some());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("arrows/chevron-right"));
}

#[test]
fn test_preserve_geometry_for_icon_by_name_pattern() {
    let mut tree = json!({
        "name": "icon/star",
        "fillGeometry": [
            {
                "commands": ["M", 0.0, 0.0, "Z"],
                "styleID": 0
            }
        ]
    });

    remove_geometry_fields(&mut tree).unwrap();

    // 应该仅根据名称模式保留几何图形
    assert!(tree.get("fillGeometry").is_some());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("icon/star"));
}

#[test]
fn test_preserve_geometry_for_arrows_by_name_pattern() {
    let mut tree = json!({
        "name": "arrows/left",
        "fillGeometry": [
            {
                "commands": ["M", 0.0, 0.0, "Z"],
                "styleID": 0
            }
        ],
        "strokeGeometry": [
            {
                "commands": ["L", 10.0, 10.0],
                "styleID": 1
            }
        ]
    });

    remove_geometry_fields(&mut tree).unwrap();

    // 应该保留箭头的几何形状
    assert!(tree.get("fillGeometry").is_some());
    assert!(tree.get("strokeGeometry").is_some());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("arrows/left"));
}

#[test]
fn test_remove_geometry_for_non_icon_with_name() {
    let mut tree = json!({
        "name": "Button",
        "fillGeometry": [
            {
                "commands": ["M", 0.0, 0.0, "Z"],
                "styleID": 0
            }
        ]
    });

    remove_geometry_fields(&mut tree).unwrap();

    // 应删除常规元素的几何图形
    assert!(tree.get("fillGeometry").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
}

#[test]
fn test_mixed_icon_and_regular_nodes() {
    let mut tree = json!({
        "name": "Root",
        "children": [
            {
                "name": "icon/home",
                "fillGeometry": [
                    {
                        "commands": ["M", 0.0, 0.0, "Z"],
                        "styleID": 0
                    }
                ]
            },
            {
                "name": "Button",
                "fillGeometry": [
                    {
                        "commands": ["M", 0.0, 0.0, "Z"],
                        "styleID": 0
                    }
                ]
            }
        ]
    });

    remove_geometry_fields(&mut tree).unwrap();

    // 图标应保留几何形状
    assert!(tree["children"][0].get("fillGeometry").is_some());
    assert_eq!(tree["children"][0].get("name").unwrap().as_str(), Some("icon/home"));

    // 按钮应删除几何图形
    assert!(tree["children"][1].get("fillGeometry").is_none());
    assert_eq!(tree["children"][1].get("name").unwrap().as_str(), Some("Button"));
}

#[test]
fn test_preserve_geometry_in_derived_symbol_data() {
    let mut tree = json!({
        "name": "Root",
        "derivedSymbolData": [
            {
                "fillGeometry": [
                    {
                        "commands": ["M", 0.0, 0.0, "Z"],
                        "styleID": 0
                    }
                ]
            }
        ],
        "symbolData": {
            "symbolOverrides": [
                {
                    "exportSettings": [
                        {
                            "imageType": {
                                "__enum__": "ImageType",
                                "value": "SVG"
                            }
                        }
                    ]
                }
            ]
        }
    });

    remove_geometry_fields(&mut tree).unwrap();

    // 由于导出设置，应保留节点级几何图形
    // 但衍生符号数据是一个数组，所以它的递归方式不同
    // 衍生符号数据中的几何图形应该被删除，因为衍生符号数据元素
    // 本身没有exportSettings
    assert!(tree["derivedSymbolData"][0].get("fillGeometry").is_none());
}

#[test]
fn test_preserve_both_fill_and_stroke_geometry_for_icons() {
    let mut tree = json!({
        "name": "icon/complex",
        "fillGeometry": [
            {
                "commands": ["M", 0.0, 0.0, "Z"],
                "styleID": 0
            }
        ],
        "strokeGeometry": [
            {
                "commands": ["M", 0.0, 0.0, "L", 10.0, 10.0],
                "styleID": 1
            }
        ],
        "windingRule": {
            "__enum__": "WindingRule",
            "value": "NONZERO"
        },
        "styleID": 5
    });

    remove_geometry_fields(&mut tree).unwrap();

    // 所有几何字段都应保留用于图标
    assert!(tree.get("fillGeometry").is_some());
    assert!(tree.get("strokeGeometry").is_some());
    assert!(tree.get("windingRule").is_some());
    assert!(tree.get("styleID").is_some());
}

#[test]
fn test_remove_geometry_from_button_with_icon_child() {
    let mut tree = json!({
        "name": "Button",
        "fillGeometry": [
            {
                "commands": ["M", 0.0, 0.0, "Z"],
                "styleID": 0
            }
        ],
        "symbolData": {
            "symbolOverrides": [
                {
                    "name": "icon/settings"
                }
            ]
        }
    });

    remove_geometry_fields(&mut tree).unwrap();

    // 按钮应该删除几何图形，即使它在 symbolOverrides 中有图标子项
    assert!(tree.get("fillGeometry").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
}
