use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除与几何相关的字段
///
/// 递归遍历 JSON 树并删除几何字段：
/// - "fillGeometry" - 填充形状的路径命令(M、L、Q、Z 等)
/// - "strokeGeometry" - 笔划形状的路径命令
/// - "windingRule" - SVG 缠绕规则属性
/// - "styleID" - 内部样式参考
///
/// 这些字段包含详细的路径几何形状，简单的 HTML/CSS 形状渲染不需要它们。
///
/// **例外**：为图标和图像保留几何形状，这些图标和图像通过以下方式标识：
/// - 在 symbolData.symbolOverrides 中使用 imageType (SVG/PNG) 进行导出设置
/// - 节点名称以 "icon/" 或 "arrows/" 开头
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有几何字段(图标/图像除外)
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_geometry_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "fillGeometry": [
///         {
///             "commands": ["M", 0.0, 0.0, "L", 100.0, 0.0, "Z"],
///             "styleID": 0,
///             "windingRule": {
///                 "__enum__": "WindingRule",
///                 "value": "NONZERO"
///             }
///         }
///     ],
///     "size": {"x": 100.0, "y": 100.0}
/// });
/// remove_geometry_fields(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "size" 字段
/// ```
pub fn remove_geometry_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 确定是否应为此节点保留几何数据
///
/// 为图标和图像保留几何形状，这些图标和图像通过以下方式标识：
/// 1. 在顶层设置带有 imageType 字段的导出设置(表示 SVG/PNG 导出)
/// 2. 名称以 "icon/" 或 "arrows/" 开头(常见图标命名模式)
///
/// 注意：我们只检查节点自己的名称，而不检查 symbolOverrides 中代表的名称
/// 子组件。
///
/// # 参数
/// * `value` - 要检查的 JSON 节点
///
/// # 返回值
/// * `true` 如果应保留几何图形，`false` 如果应删除几何图形
fn should_preserve_geometry(value: &JsonValue) -> bool {
    if let Some(obj) = value.as_object() {
        // 检查 1：查找图标/图像名称模式(此节点的名称，而不是子节点的名称)
        if let Some(name) = obj.get("name")
            && let Some(name_str) = name.as_str()
            && (name_str.starts_with("icon/") || name_str.starts_with("arrows/"))
        {
            return true; // Icon or arrow, preserve geometry
        }

        // 检查2：在symbolData.symbolOverrides中查找带有imageType的exportSettings
        // 这检查此特定节点是否具有导出设置(不是子覆盖)
        if let Some(symbol_data) = obj.get("symbolData")
            && let Some(overrides) = symbol_data.get("symbolOverrides")
            && let Some(overrides_array) = overrides.as_array()
        {
            for override_item in overrides_array {
                if let Some(export_settings) = override_item.get("exportSettings")
                    && let Some(settings_array) = export_settings.as_array()
                {
                    for setting in settings_array {
                        if setting.get("imageType").is_some() {
                            return true; // Has imageType, preserve geometry
                        }
                    }
                }
            }
        }
    }

    false // Not an icon/image, remove geometry
}

/// 从 JSON 值中递归删除几何字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    // 检查我们是否应该在匹配之前保留几何图形(以避免借用检查器问题)
    let preserve = should_preserve_geometry(value);

    match value {
        JsonValue::Object(map) => {
            // 仅删除不是图标/图像节点的几何图形
            if !preserve {
                map.remove("fillGeometry");
                map.remove("strokeGeometry");
                map.remove("windingRule");
                map.remove("styleID");
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
}
