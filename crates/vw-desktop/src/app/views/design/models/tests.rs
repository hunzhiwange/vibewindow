//! 设计文档模型测试模块
//!
//! 本模块包含设计文档变量解析和主题反序列化的单元测试。
//! 主要测试以下功能：
//! - 动态主题变量的解析
//! - 主题条件缺失时的反序列化行为
//! - 设计文档主题的反序列化处理

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    /// 测试动态主题变量的解析
    ///
    /// 验证 `resolve_variable` 函数能够根据不同的主题模式正确解析变量值，
    /// 并在未匹配到主题时正确回退到默认值。
    ///
    /// # 测试场景
    /// - Dark 主题：应返回 "#000000"
    /// - Variant-1 主题：应返回 "#111111"
    /// - Custom-Theme 自定义主题：应返回 "#222222"
    /// - 未知主题：应回退到默认值 "#FFFFFF"
    /// - 无主题：应返回默认值 "#FFFFFF"
    #[test]
    fn test_resolve_variable_dynamic_themes() {
        let mut variables = HashMap::new();

        // 定义变量配置，包含多个主题条件值
        let var_def = VariableDef {
            kind: "color".to_string(),
            collection: None,
            value: vec![
                // Dark 主题对应的颜色值
                VariableValue {
                    value: "#000000".to_string(),
                    theme: Some(ThemeCondition { mode: "Dark".to_string() }),
                },
                // Variant-1 主题对应的颜色值
                VariableValue {
                    value: "#111111".to_string(),
                    theme: Some(ThemeCondition { mode: "Variant-1".to_string() }),
                },
                // 自定义主题对应的颜色值，验证动态主题支持
                VariableValue {
                    value: "#222222".to_string(),
                    theme: Some(ThemeCondition { mode: "Custom-Theme".to_string() }),
                },
                // 默认颜色值（无主题条件）
                VariableValue { value: "#FFFFFF".to_string(), theme: None },
            ],
        };
        variables.insert("bg-color".to_string(), var_def);

        // 测试 Dark 主题模式
        assert_eq!(
            resolve_variable("bg-color", &variables, Some("Dark")),
            Some(&"#000000".to_string())
        );

        // 测试 Variant-1 主题
        assert_eq!(
            resolve_variable("bg-color", &variables, Some("Variant-1")),
            Some(&"#111111".to_string())
        );

        // 测试 Custom-Theme 自定义主题（验证动态主题支持）
        assert_eq!(
            resolve_variable("bg-color", &variables, Some("Custom-Theme")),
            Some(&"#222222".to_string())
        );

        // 测试回退行为（未知主题）
        assert_eq!(
            resolve_variable("bg-color", &variables, Some("Unknown")),
            Some(&"#FFFFFF".to_string())
        );

        // 测试无主题情况
        assert_eq!(resolve_variable("bg-color", &variables, None), Some(&"#FFFFFF".to_string()));
    }

    /// 测试主题条件缺失 `mode` 字段时的反序列化行为
    ///
    /// 验证当 JSON 中的 `theme` 字段缺少 `mode` 属性时，反序列化应将其视为 `None`。
    ///
    /// # 测试场景
    /// - 包含错误主题结构 `{ "Base": "Gray" }`：应反序列化为 `None`
    /// - 包含正确主题结构 `{ "mode": "Dark" }`：应正确反序列化
    /// - 包含字符串形式主题 `"Light"`：应正确反序列化
    /// - 缺少 `theme` 字段：应为 `None`
    #[test]
    fn test_deserialize_theme_missing_mode_treated_as_none() {
        // 定义包含多种主题条件格式的 JSON
        let json = r##"
            {
              "type": "color",
              "value": [
                { "value": "#111111", "theme": { "Base": "Gray" } },
                { "value": "#222222", "theme": { "mode": "Dark" } },
                { "value": "#333333", "theme": "Light" },
                { "value": "#FFFFFF" }
              ]
            }
            "##;

        // 反序列化 JSON
        let def: VariableDef = serde_json::from_str(json).unwrap();

        // 验证第一个值：主题字段缺少 mode，应为 None
        assert!(def.value[0].theme.is_none());
        // 验证第二个值：主题字段包含 mode="Dark"
        assert_eq!(def.value[1].theme.as_ref().map(|t| t.mode.as_str()), Some("Dark"));
        // 验证第三个值：主题字段为字符串 "Light"
        assert_eq!(def.value[2].theme.as_ref().map(|t| t.mode.as_str()), Some("Light"));
        // 验证第四个值：缺少主题字段，应为 None
        assert!(def.value[3].theme.is_none());
    }

    /// 测试设计文档主题缺失 `mode` 字段时的反序列化行为
    ///
    /// 验证当设计文档 JSON 中的 `theme` 字段缺少 `mode` 属性时，
    /// 反序列化应将其视为 `None` 而不是报错。
    ///
    /// # 测试场景
    /// - 设计文档包含错误主题结构 `{ "Base": "Gray" }`：应反序列化为 `None`
    #[test]
    fn test_deserialize_doc_theme_missing_mode_treated_as_none() {
        // 定义包含错误主题结构的设计文档 JSON
        let json = r#"
            {
              "version": "1.0",
              "children": [],
              "variables": {},
              "theme": { "Base": "Gray" }
            }
            "#;

        // 反序列化 JSON
        let doc: DesignDoc = serde_json::from_str(json).unwrap();

        // 验证主题字段缺失 mode，应为 None
        assert!(doc.theme.is_none());
    }

    #[test]
    fn test_deserialize_design_element_accepts_string_stroke() {
        let json = r#"
            {
              "type": "frame",
              "id": "case-module",
              "stroke": "run",
              "children": []
            }
            "#;
        let element: DesignElement = serde_json::from_str(json).unwrap();
        assert_eq!(element.stroke.as_ref().and_then(|stroke| stroke.fill.as_deref()), Some("run"));
    }

    #[test]
    fn test_deserialize_design_element_ignores_invalid_children_shape() {
        let json = r#"
            {
              "type": "frame",
              "id": "case-module",
              "children": "run"
            }
            "#;
        let element: DesignElement = serde_json::from_str(json).unwrap();
        assert!(element.children.is_empty());
    }

    #[test]
    fn test_normalize_groups_adds_default_group_for_legacy_docs() {
        let mut doc = DesignDoc {
            version: "1.0".to_string(),
            children: vec![DesignElement {
                id: "page-1".to_string(),
                kind: "frame".to_string(),
                children: vec![],
                ..Default::default()
            }],
            ..Default::default()
        };

        doc.normalize_groups();

        assert_eq!(doc.groups.len(), 1);
        assert_eq!(doc.groups[0].id, 0);
        assert_eq!(doc.groups[0].name, "默认页面");
        assert_eq!(doc.first_group_id(), 0);
    }

    #[test]
    fn test_filtered_for_group_keeps_only_matching_top_level_children() {
        let doc = DesignDoc {
            version: "1.0".to_string(),
            children: vec![
                DesignElement {
                    id: "group-0".to_string(),
                    kind: "frame".to_string(),
                    group_id: 0,
                    ..Default::default()
                },
                DesignElement {
                    id: "group-2".to_string(),
                    kind: "frame".to_string(),
                    group_id: 2,
                    ..Default::default()
                },
            ],
            groups: vec![
                DesignGroup { id: 0, name: "默认页面".to_string() },
                DesignGroup { id: 2, name: "业务A".to_string() },
            ],
            ..Default::default()
        };

        let filtered = doc.filtered_for_group(2);

        assert_eq!(filtered.children.len(), 1);
        assert_eq!(filtered.children[0].id, "group-2");
        assert_eq!(filtered.groups.len(), 2);
    }

    #[test]
    fn test_deserialize_design_element_accepts_snake_case_group_id() {
        let json = r#"
            {
              "type": "frame",
              "id": "case-module",
              "group_id": 3,
              "children": []
            }
            "#;

        let element: DesignElement = serde_json::from_str(json).unwrap();

        assert_eq!(element.group_id, 3);
    }

    #[test]
    fn test_color_format_display_labels() {
        assert_eq!(ColorFormat::Hex.to_string(), "HEX");
        assert_eq!(ColorFormat::Rgba.to_string(), "RGBA");
        assert_eq!(ColorFormat::Hsl.to_string(), "HSL");
        assert_eq!(ColorFormat::Css.to_string(), "CSS");
    }

    #[test]
    fn test_design_tool_icon_names_cover_all_variants() {
        let cases = [
            (DesignTool::Move, "cursor-fill.svg"),
            (DesignTool::Line, "minus.svg"),
            (DesignTool::Rectangle, "square.svg"),
            (DesignTool::Ellipse, "circle.svg"),
            (DesignTool::Triangle, "triangle.svg"),
            (DesignTool::Diamond, "diamond.svg"),
            (DesignTool::Star, "star.svg"),
            (DesignTool::Pentagon, "pentagon.svg"),
            (DesignTool::Hexagon, "hexagon.svg"),
            (DesignTool::Parallelogram, "parallelogram.svg"),
            (DesignTool::Trapezoid, "trapezoid.svg"),
            (DesignTool::Chevron, "chevron.svg"),
            (DesignTool::Capsule, "capsule.svg"),
            (DesignTool::Icon, "gem.svg"),
            (DesignTool::ImportImage, "image.svg"),
            (DesignTool::ImportFigma, "file-code.svg"),
            (DesignTool::Pen, "pen.svg"),
            (DesignTool::Eraser, "eraser.svg"),
            (DesignTool::Text, "fonts.svg"),
            (DesignTool::Frame, "bounding-box.svg"),
            (DesignTool::StickyNote, "sticky.svg"),
            (DesignTool::Hand, "arrows-move.svg"),
        ];

        for (tool, icon) in cases {
            assert_eq!(tool.icon_name(), icon);
        }
    }

    #[test]
    fn test_sticky_note_kind_labels_colors_and_parsing() {
        assert_eq!(
            StickyNoteKind::ALL,
            [StickyNoteKind::Note, StickyNoteKind::Context, StickyNoteKind::Prompt]
        );
        assert_eq!(StickyNoteKind::Note.label(), "Note");
        assert_eq!(StickyNoteKind::Context.label_zh(), "上下文");
        assert_eq!(StickyNoteKind::Prompt.bilingual_label(), "Prompt 提示词");
        assert_eq!(StickyNoteKind::Note.fill_color(), "#FFF1CC");
        assert_eq!(StickyNoteKind::Context.stroke_color(), "#8A8A8A");
        assert_eq!(StickyNoteKind::Prompt.text_color(), "#0B6FBD");
        assert_eq!(StickyNoteKind::from_str(" prompt "), Some(StickyNoteKind::Prompt));
        assert_eq!(
            StickyNoteKind::from_value(&serde_json::json!("context")),
            Some(StickyNoteKind::Context)
        );
        assert_eq!(StickyNoteKind::from_str("missing"), None);
        assert_eq!(StickyNoteKind::Note.to_string(), "Note 笔记");
    }

    #[test]
    fn test_variable_collection_names_merge_sources_uniquely() {
        let mut variables = HashMap::new();
        variables.insert(
            "color-a".to_string(),
            VariableDef {
                kind: "color".to_string(),
                collection: Some("Brand".to_string()),
                value: vec![VariableValue { value: "#fff".to_string(), theme: None }],
            },
        );
        variables.insert(
            "color-b".to_string(),
            VariableDef {
                kind: "color".to_string(),
                collection: Some("brand".to_string()),
                value: vec![VariableValue { value: "#000".to_string(), theme: None }],
            },
        );
        let mut doc = DesignDoc {
            variables,
            variable_collections: Some(VariableCollections {
                names: vec!["Theme".to_string(), " ".to_string()],
            }),
            ..Default::default()
        };

        assert_eq!(
            doc.variable_collection_names()
                .into_iter()
                .map(|name| name.to_ascii_lowercase())
                .collect::<Vec<_>>(),
            vec!["theme".to_string(), "brand".to_string()]
        );
        assert_eq!(
            doc.ensure_variable_collections()
                .into_iter()
                .map(|name| name.to_ascii_lowercase())
                .collect::<Vec<_>>(),
            vec!["theme".to_string(), "brand".to_string()]
        );
    }

    #[test]
    fn test_variable_theme_modes_merge_theme_sources_uniquely() {
        let mut variables = HashMap::new();
        variables.insert(
            "color".to_string(),
            VariableDef {
                kind: "color".to_string(),
                collection: None,
                value: vec![
                    VariableValue {
                        value: "#000".to_string(),
                        theme: Some(ThemeCondition { mode: "Dark".to_string() }),
                    },
                    VariableValue {
                        value: "#111".to_string(),
                        theme: Some(ThemeCondition { mode: "dark".to_string() }),
                    },
                ],
            },
        );
        let mut doc = DesignDoc {
            variables,
            theme: Some(ThemeCondition { mode: "Light".to_string() }),
            themes: Some(DesignThemes { mode: vec!["Base".to_string(), " ".to_string()] }),
            ..Default::default()
        };

        assert_eq!(
            doc.variable_theme_modes(),
            vec!["Base".to_string(), "Light".to_string(), "Dark".to_string()]
        );
        assert_eq!(
            doc.ensure_variable_themes(),
            vec!["Base".to_string(), "Light".to_string(), "Dark".to_string()]
        );
    }

    #[test]
    fn test_normalize_groups_deduplicates_and_adds_used_ids() {
        let mut doc = DesignDoc {
            children: vec![
                DesignElement { id: "a".to_string(), group_id: 2, ..Default::default() },
                DesignElement { id: "b".to_string(), group_id: 3, ..Default::default() },
            ],
            groups: vec![
                DesignGroup { id: 2, name: "".to_string() },
                DesignGroup { id: 2, name: "Duplicate".to_string() },
            ],
            ..Default::default()
        };

        doc.normalize_groups();

        assert_eq!(doc.groups.len(), 2);
        assert_eq!((doc.groups[0].id, doc.groups[0].name.as_str()), (2, "页面 2"));
        assert_eq!((doc.groups[1].id, doc.groups[1].name.as_str()), (3, "页面 3"));
        assert_eq!(doc.next_group_id(), 4);
        assert_eq!(doc.group_name(2), Some("页面 2"));
    }

    #[test]
    fn test_group_helpers_find_top_level_and_nested_elements() {
        let doc = DesignDoc {
            children: vec![
                DesignElement {
                    id: "root-a".to_string(),
                    group_id: 1,
                    children: vec![DesignElement {
                        id: "nested".to_string(),
                        group_id: 1,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                DesignElement { id: "root-b".to_string(), group_id: 2, ..Default::default() },
            ],
            groups: vec![
                DesignGroup { id: 1, name: "One".to_string() },
                DesignGroup { id: 2, name: "Two".to_string() },
            ],
            ..Default::default()
        };

        assert_eq!(doc.first_top_level_in_group(2).map(|el| el.id.as_str()), Some("root-b"));
        assert_eq!(doc.top_level_children_count_in_group(1), 1);
        assert_eq!(doc.group_id_for_element("nested"), Some(1));
        assert_eq!(doc.find_element("nested").map(|el| el.id.as_str()), Some("nested"));
        assert_eq!(doc.find_path_to_element("nested"), Some(vec!["root-a".to_string()]));
    }

    #[test]
    fn test_get_bounds_uses_variables_and_ignores_empty_docs() {
        let mut variables = HashMap::new();
        variables.insert(
            "w".to_string(),
            VariableDef {
                kind: "number".to_string(),
                collection: None,
                value: vec![VariableValue { value: "30".to_string(), theme: None }],
            },
        );
        let doc = DesignDoc {
            variables,
            children: vec![
                DesignElement {
                    id: "a".to_string(),
                    x: 10.0,
                    y: 20.0,
                    width: Some(serde_json::json!("$-w")),
                    height: Some(serde_json::json!("fill_container(40)")),
                    ..Default::default()
                },
                DesignElement {
                    id: "b".to_string(),
                    x: -5.0,
                    y: 5.0,
                    width: Some(serde_json::json!(10)),
                    height: Some(serde_json::json!(10)),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        assert_eq!(DesignDoc::default().get_bounds(), None);
        assert_eq!(doc.get_bounds(), Some((-5.0, 5.0, 40.0, 60.0)));
    }

    #[test]
    fn test_update_property_updates_all_supported_core_fields() {
        let mut element = DesignElement { id: "a".to_string(), ..Default::default() };

        assert!(element.update_property("a", "name", serde_json::json!("Name")));
        assert!(element.update_property("a", "x", serde_json::json!(12.0)));
        assert!(element.update_property("a", "y", serde_json::json!(13.0)));
        assert!(element.update_property("a", "width", serde_json::json!(14.0)));
        assert!(element.update_property("a", "height", serde_json::json!(15.0)));
        assert!(element.update_property("a", "color", serde_json::json!("#fff")));
        assert!(element.update_property("a", "class", serde_json::json!("p-4")));
        assert!(element.update_property("a", "rotation", serde_json::json!(45.0)));
        assert!(element.update_property("a", "content", serde_json::json!("Body")));
        assert!(element.update_property("a", "context", serde_json::json!("Ctx")));
        assert!(element.update_property("a", "noteType", serde_json::json!("prompt")));
        assert!(element.update_property("a", "fontFamily", serde_json::json!("Inter")));
        assert!(element.update_property("a", "fontSize", serde_json::json!(16)));
        assert!(element.update_property("a", "fontWeight", serde_json::json!(700)));
        assert!(element.update_property("a", "weight", serde_json::json!(600)));
        assert!(element.update_property("a", "fontStyle", serde_json::json!("italic")));
        assert!(element.update_property("a", "textDecoration", serde_json::json!("underline")));
        assert!(element.update_property("a", "lineHeight", serde_json::json!(1.5)));
        assert!(element.update_property("a", "letterSpacing", serde_json::json!(0.2)));
        assert!(element.update_property("a", "textAlign", serde_json::json!("center")));
        assert!(element.update_property("a", "textAlignVertical", serde_json::json!("middle")));
        assert!(element.update_property("a", "textGrowth", serde_json::json!("auto")));
        assert!(element.update_property("a", "fill", serde_json::json!("#000")));
        assert!(element.update_property("a", "iconFontName", serde_json::json!("star")));
        assert!(element.update_property("a", "iconFontFamily", serde_json::json!("lucide")));
        assert!(element.update_property("a", "opacity", serde_json::json!(0.5)));
        assert!(element.update_property("a", "fillWidth", serde_json::json!(true)));
        assert!(element.update_property("a", "fillHeight", serde_json::json!(true)));
        assert!(element.update_property("a", "visible", serde_json::json!(false)));
        assert!(element.update_property("a", "effect", serde_json::json!([{"type": "shadow"}])));
        assert!(element.update_property("a", "theme", serde_json::json!({"mode": "Dark"})));
        assert!(element.update_property("a", "export", serde_json::json!({"png": true})));
        assert!(element.update_property(
            "a",
            "stroke",
            serde_json::json!({"align": "inside", "thickness": 1, "fill": "#ddd"})
        ));

        assert_eq!(element.name.as_deref(), Some("Name"));
        assert_eq!((element.x, element.y), (12.0, 13.0));
        assert_eq!(element.width, Some(serde_json::json!(14.0)));
        assert_eq!(element.height, Some(serde_json::json!(15.0)));
        assert_eq!(element.color.as_deref(), Some("#fff"));
        assert_eq!(element.class.as_deref(), Some("p-4"));
        assert_eq!(element.rotation, Some(45.0));
        assert_eq!(element.content.as_deref(), Some("Body"));
        assert_eq!(element.context.as_deref(), Some("Ctx"));
        assert_eq!(element.note_type, Some(StickyNoteKind::Prompt));
        assert_eq!(element.font_family.as_deref(), Some("Inter"));
        assert_eq!(element.text_growth.as_deref(), Some("auto"));
        assert_eq!(element.icon_font_name.as_deref(), Some("star"));
        assert_eq!(element.opacity, Some(0.5));
        assert_eq!(element.fill_width, Some(true));
        assert_eq!(element.fill_height, Some(true));
        assert_eq!(element.visible, Some(false));
        assert_eq!(element.stroke.as_ref().and_then(|stroke| stroke.fill.as_deref()), Some("#ddd"));

        assert!(element.update_property("a", "textGrowth", serde_json::Value::Null));
        assert!(element.text_growth.is_none());
        assert!(!element.update_property("missing", "x", serde_json::json!(1)));
    }

    #[test]
    fn test_doc_update_property_recurses_into_children() {
        let mut doc = DesignDoc {
            children: vec![DesignElement {
                id: "root".to_string(),
                children: vec![DesignElement { id: "child".to_string(), ..Default::default() }],
                ..Default::default()
            }],
            ..Default::default()
        };

        doc.update_property("child", "x", serde_json::json!(42.0));

        assert_eq!(doc.find_element("child").map(|el| el.x), Some(42.0));
    }

    #[test]
    fn test_fill_flags_and_group_id_apply_recursively() {
        let mut element = DesignElement {
            width: Some(serde_json::json!("fill_container")),
            height: Some(serde_json::json!("fill_container(10)")),
            children: vec![DesignElement {
                width: Some(serde_json::json!("fill_container")),
                ..Default::default()
            }],
            ..Default::default()
        };

        element.normalize_fill_flags();
        assert_eq!(element.fill_width, Some(true));
        assert_eq!(element.fill_height, Some(true));
        assert_eq!(element.children[0].fill_width, Some(true));

        element.set_group_id_recursive(7);
        assert_eq!(element.group_id, 7);
        assert_eq!(element.children[0].group_id, 7);
    }

    #[test]
    fn test_compute_tree_metrics_uses_names_refs_and_depth() {
        let doc = DesignDoc {
            children: vec![
                DesignElement {
                    id: "component".to_string(),
                    kind: "component".to_string(),
                    name: Some("ReusableComponent".to_string()),
                    children: vec![DesignElement {
                        id: "component-child".to_string(),
                        kind: "text".to_string(),
                        name: Some("DeepChild".to_string()),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                DesignElement {
                    id: "instance".to_string(),
                    kind: "ref".to_string(),
                    reference: Some("component".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        assert_eq!(compute_tree_metrics(&doc), ("ReusableComponent".len(), 1));
    }

    #[test]
    fn test_parse_val_handles_numbers_variables_fill_container_and_invalid_values() {
        let mut variables = HashMap::new();
        variables.insert(
            "size".to_string(),
            VariableDef {
                kind: "number".to_string(),
                collection: None,
                value: vec![VariableValue { value: "24".to_string(), theme: None }],
            },
        );

        assert_eq!(parse_val(&Some(serde_json::json!(12)), &variables, None), Some(12.0));
        assert_eq!(parse_val(&Some(serde_json::json!("13.5")), &variables, None), Some(13.5));
        assert_eq!(parse_val(&Some(serde_json::json!("$-size")), &variables, None), Some(24.0));
        assert_eq!(
            parse_val(&Some(serde_json::json!("fill_container(88)")), &variables, None),
            Some(88.0)
        );
        assert_eq!(parse_val(&Some(serde_json::json!("auto")), &variables, None), None);
        assert_eq!(parse_val(&None, &variables, None), None);
    }
}
