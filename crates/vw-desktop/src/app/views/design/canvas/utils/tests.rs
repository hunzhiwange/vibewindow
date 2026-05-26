//! 设计画布工具测试模块，验证元素克隆、引用解析和 Tailwind 应用等辅助逻辑。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    #[test]
    fn ref_instance_applies_children_and_fit_content_size() {
        let base = DesignElement {
            kind: "frame".to_string(),
            id: "Kbr4h".to_string(),
            width: Some(serde_json::json!(93)),
            height: Some(serde_json::json!(40)),
            slot: Some(serde_json::json!(["A", "B"])),
            clip: Some(true),
            ..Default::default()
        };
        let inst_ref = DesignElement {
            kind: "ref".to_string(),
            id: "Hygdd".to_string(),
            reference: Some("Kbr4h".to_string()),
            width: Some(serde_json::json!("fit_content")),
            height: Some(serde_json::json!(40)),
            children: vec![
                DesignElement {
                    kind: "ref".to_string(),
                    id: "c1".to_string(),
                    reference: Some("KbyBJ".to_string()),
                    ..Default::default()
                },
                DesignElement {
                    kind: "ref".to_string(),
                    id: "c2".to_string(),
                    reference: Some("BdBJJ".to_string()),
                    ..Default::default()
                },
                DesignElement {
                    kind: "ref".to_string(),
                    id: "c3".to_string(),
                    reference: Some("BdBJJ".to_string()),
                    ..Default::default()
                },
                DesignElement {
                    kind: "ref".to_string(),
                    id: "c4".to_string(),
                    reference: Some("BdBJJ".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let out = resolve_ref_instance(&inst_ref, &[base], None).unwrap();
        assert_eq!(out.width, Some(serde_json::json!("fit_content")));
        assert_eq!(out.height, Some(serde_json::json!(40)));
        assert_eq!(out.children.len(), 4);
    }

    #[test]
    fn ref_instance_can_clear_fill_and_effect() {
        let base = DesignElement {
            kind: "frame".to_string(),
            id: "KbyBJ".to_string(),
            fill: Some(serde_json::json!("$--background")),
            effect: Some(serde_json::json!({"type":"shadow"})),
            ..Default::default()
        };
        let inst_ref = DesignElement {
            kind: "ref".to_string(),
            id: "BdBJJ".to_string(),
            reference: Some("KbyBJ".to_string()),
            fill: Some(serde_json::json!([])),
            effect: Some(serde_json::json!([])),
            ..Default::default()
        };

        let out = resolve_ref_instance(&inst_ref, &[base], None).unwrap();
        assert_eq!(out.fill, Some(serde_json::json!([])));
        assert_eq!(out.effect, Some(serde_json::json!([])));
    }

    #[test]
    fn nested_ref_descendants_override_propagates_to_base() {
        let base = DesignElement {
            kind: "frame".to_string(),
            id: "KbyBJ".to_string(),
            children: vec![DesignElement {
                kind: "text".to_string(),
                id: "248ys".to_string(),
                content: Some("Tab Item".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let mid = DesignElement {
            kind: "ref".to_string(),
            id: "BdBJJ".to_string(),
            reference: Some("KbyBJ".to_string()),
            descendants: Some(serde_json::json!({
                "248ys": { "fill": "$--muted-foreground" }
            })),
            ..Default::default()
        };

        let leaf = DesignElement {
            kind: "ref".to_string(),
            id: "CEFDt".to_string(),
            reference: Some("BdBJJ".to_string()),
            descendants: Some(serde_json::json!({
                "248ys": { "content": "Profile" }
            })),
            ..Default::default()
        };

        let inst = resolve_ref_instance(&leaf, &[base.clone(), mid], None).unwrap();
        assert_eq!(inst.kind, "ref");
        assert_eq!(inst.reference.as_deref(), Some("KbyBJ"));

        let final_inst = resolve_ref_instance(&inst, &[base], None).unwrap();
        assert_eq!(final_inst.children.first().and_then(|c| c.content.as_deref()), Some("Profile"));
    }

    #[test]
    fn descendants_override_can_change_node_type() {
        let base = DesignElement {
            kind: "frame".to_string(),
            id: "base".to_string(),
            children: vec![DesignElement {
                kind: "text".to_string(),
                id: "label".to_string(),
                content: Some("1".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let inst_ref = DesignElement {
            kind: "ref".to_string(),
            id: "inst".to_string(),
            reference: Some("base".to_string()),
            descendants: Some(serde_json::json!({
                "label": {
                    "type": "frame",
                    "width": 24,
                    "height": 24,
                    "children": [
                        { "type": "path", "id": "p1", "geometry": "M0 1l0-1" }
                    ]
                }
            })),
            ..Default::default()
        };

        let out = resolve_ref_instance(&inst_ref, &[base], None).unwrap();
        assert_eq!(out.children.len(), 1);
        assert_eq!(out.children[0].id, "label");
        assert_eq!(out.children[0].kind, "frame");
        assert_eq!(out.children[0].content.as_deref(), None);
        assert_eq!(out.children[0].children.len(), 1);
        assert_eq!(out.children[0].children[0].kind, "path");
    }

    #[test]
    fn descendants_override_can_replace_node_and_clear_old_descendants() {
        let base = DesignElement {
            kind: "frame".to_string(),
            id: "yoahP".to_string(),
            children: vec![DesignElement {
                kind: "ref".to_string(),
                id: "mhIeP".to_string(),
                reference: Some("u61z6".to_string()),
                fill: Some(serde_json::json!("$--background")),
                descendants: Some(serde_json::json!({
                    "mB2s3": { "enabled": false }
                })),
                ..Default::default()
            }],
            ..Default::default()
        };

        let selected = DesignElement {
            kind: "ref".to_string(),
            id: "hMm4B".to_string(),
            reference: Some("yoahP".to_string()),
            descendants: Some(serde_json::json!({
                "mhIeP": {
                    "id": "Z7y8W",
                    "type": "ref",
                    "ref": "u61z6",
                    "x": 0,
                    "y": 4
                }
            })),
            ..Default::default()
        };

        let out = resolve_ref_instance(&selected, &[base], None).unwrap();
        assert_eq!(out.kind, "frame");
        assert_eq!(out.children.len(), 1);
        assert_eq!(out.children[0].id, "Z7y8W");
        assert_eq!(out.children[0].kind, "ref");
        assert_eq!(out.children[0].reference.as_deref(), Some("u61z6"));
        assert_eq!(out.children[0].fill, None);
        assert_eq!(out.children[0].descendants, None);
        assert_eq!(out.children[0].y, 4.0);
    }
}
