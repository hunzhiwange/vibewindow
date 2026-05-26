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
}
