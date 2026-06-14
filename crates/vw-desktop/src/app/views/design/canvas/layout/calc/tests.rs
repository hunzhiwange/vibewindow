//! 设计画布布局模块。
//!
//! 该模块负责解析和计算画布节点布局，帮助渲染层获得稳定的几何信息。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::{compute_layout, resolve_element_size};
    use crate::app::views::design::canvas::types::{AlignMode, LayoutDirection};
    use crate::app::views::design::models::{DesignDoc, DesignElement};
    use iced::Size;

    #[test]
    fn fill_container_with_fallback_width_uses_fallback_without_parent() {
        let doc = DesignDoc { version: "test".to_string(), ..Default::default() };
        let element = DesignElement {
            kind: "frame".to_string(),
            width: Some(serde_json::Value::String("fill_container(640)".to_string())),
            ..Default::default()
        };
        let s = resolve_element_size(&element, None, &doc, None);
        assert_eq!(s.width, 640.0);
    }

    #[test]
    fn fill_container_with_fallback_width_uses_parent_when_available() {
        let doc = DesignDoc { version: "test".to_string(), ..Default::default() };
        let element = DesignElement {
            kind: "frame".to_string(),
            width: Some(serde_json::Value::String("fill_container(640)".to_string())),
            ..Default::default()
        };
        let s = resolve_element_size(&element, Some(Size::new(500.0, 300.0)), &doc, None);
        assert_eq!(s.width, 500.0);
    }

    #[test]
    fn fill_container_with_fallback_width_caps_to_fallback() {
        let doc = DesignDoc { version: "test".to_string(), ..Default::default() };
        let element = DesignElement {
            kind: "frame".to_string(),
            width: Some(serde_json::Value::String("fill_container(640)".to_string())),
            ..Default::default()
        };
        let s = resolve_element_size(&element, Some(Size::new(1200.0, 300.0)), &doc, None);
        assert_eq!(s.width, 640.0);
    }

    #[test]
    fn horizontal_two_fill_children_split_evenly_with_gap() {
        let doc = DesignDoc { version: "test".to_string(), ..Default::default() };
        let parent = DesignElement {
            kind: "frame".to_string(),
            justify_content: Some("center".to_string()),
            align_items: Some("center".to_string()),
            gap: Some(serde_json::Value::Number(serde_json::Number::from(12))),
            ..Default::default()
        };
        let child_a = DesignElement {
            kind: "frame".to_string(),
            width: Some(serde_json::Value::String("fill_container".to_string())),
            ..Default::default()
        };
        let child_b = DesignElement {
            kind: "frame".to_string(),
            width: Some(serde_json::Value::String("fill_container".to_string())),
            ..Default::default()
        };
        let layouts = compute_layout(
            LayoutDirection::Horizontal,
            &[child_a, child_b],
            Size::new(352.0, 40.0),
            &parent,
            &doc,
            None,
        );
        assert_eq!(layouts.len(), 2);
        assert_eq!(layouts[0].size.width, 170.0);
        assert_eq!(layouts[1].size.width, 170.0);
    }

    #[test]
    fn fit_content_with_min_height_returns_min_without_children_or_content() {
        let doc = DesignDoc { version: "test".to_string(), ..Default::default() };
        let element = DesignElement {
            kind: "frame".to_string(),
            height: Some(serde_json::Value::String("fit_content(44)".to_string())),
            ..Default::default()
        };
        let s = resolve_element_size(&element, None, &doc, None);
        assert_eq!(s.height, 44.0);
    }

    #[test]
    fn fit_content_with_min_height_clamps_measured_height() {
        let doc = DesignDoc { version: "test".to_string(), ..Default::default() };
        let child_small = DesignElement {
            kind: "frame".to_string(),
            height: Some(serde_json::json!(10)),
            ..Default::default()
        };
        let row = DesignElement {
            kind: "frame".to_string(),
            height: Some(serde_json::Value::String("fit_content(44)".to_string())),
            align_items: Some("center".to_string()),
            children: vec![child_small],
            ..Default::default()
        };
        let s = resolve_element_size(&row, None, &doc, None);
        assert_eq!(s.height, 44.0);

        let child_tall = DesignElement {
            kind: "frame".to_string(),
            height: Some(serde_json::json!(60)),
            ..Default::default()
        };
        let row = DesignElement {
            kind: "frame".to_string(),
            height: Some(serde_json::Value::String("fit_content(44)".to_string())),
            align_items: Some("center".to_string()),
            children: vec![child_tall],
            ..Default::default()
        };
        let s = resolve_element_size(&row, None, &doc, None);
        assert_eq!(s.height, 60.0);
    }

    #[test]
    fn helper_predicates_parse_fit_and_fill_variants() {
        assert!(super::is_fit_content(&Some(serde_json::json!("fit_content"))));
        assert!(super::is_fit_content(&Some(serde_json::json!("fit_content(44px)"))));
        assert!(!super::is_fit_content(&Some(serde_json::json!("fixed"))));

        assert!(super::is_fill_container(&Some(serde_json::json!("fill_container"))));
        assert_eq!(
            super::fill_container_weight(&Some(serde_json::json!("fill_container(120px)"))),
            Some(1.0)
        );
        assert_eq!(
            super::fill_container_fallback_size(&Some(serde_json::json!("fill_container(120px)"))),
            Some(120.0)
        );
        assert_eq!(
            super::fit_content_min_size(&Some(serde_json::json!("fit_content(12px)"))),
            Some(12.0)
        );
        assert_eq!(super::fit_content_min_size(&Some(serde_json::json!("fit_content(-1)"))), None);
    }

    #[test]
    fn guess_direction_and_inference_use_children_and_layout_hints() {
        let horizontal = vec![
            DesignElement { x: 0.0, y: 0.0, ..Default::default() },
            DesignElement { x: 100.0, y: 5.0, ..Default::default() },
        ];
        assert!(matches!(
            super::guess_direction_from_children(&horizontal),
            LayoutDirection::Horizontal
        ));

        let vertical = vec![
            DesignElement { x: 0.0, y: 0.0, ..Default::default() },
            DesignElement { x: 5.0, y: 100.0, ..Default::default() },
        ];
        assert!(matches!(
            super::guess_direction_from_children(&vertical),
            LayoutDirection::Vertical
        ));

        let none = DesignElement { layout: Some("none".to_string()), ..Default::default() };
        assert!(super::infer_container_layout_direction(&none).is_none());

        let hinted = DesignElement {
            gap: Some(serde_json::json!(8)),
            children: horizontal,
            ..Default::default()
        };
        assert!(matches!(
            super::infer_container_layout_direction(&hinted),
            Some(LayoutDirection::Horizontal)
        ));
    }

    #[test]
    fn resolve_ref_size_uses_reference_when_instance_has_no_explicit_size() {
        let source = DesignElement {
            id: "source".to_string(),
            width: Some(serde_json::json!(200)),
            height: Some(serde_json::json!(90)),
            ..Default::default()
        };
        let instance = DesignElement {
            id: "instance".to_string(),
            kind: "ref".to_string(),
            reference: Some("source".to_string()),
            ..Default::default()
        };
        let doc = DesignDoc {
            version: "test".to_string(),
            children: vec![source, instance.clone()],
            ..Default::default()
        };

        let size = resolve_element_size(&instance, None, &doc, None);
        assert_eq!(size, Size::new(200.0, 90.0));
    }

    #[test]
    fn resolve_fit_content_container_ignores_hidden_children() {
        let doc = DesignDoc { version: "test".to_string(), ..Default::default() };
        let visible = DesignElement {
            width: Some(serde_json::json!(40)),
            height: Some(serde_json::json!(20)),
            ..Default::default()
        };
        let hidden = DesignElement {
            visible: Some(false),
            width: Some(serde_json::json!(400)),
            height: Some(serde_json::json!(200)),
            ..Default::default()
        };
        let parent = DesignElement {
            layout: Some("horizontal".to_string()),
            gap: Some(serde_json::json!(10)),
            padding: Some(serde_json::json!([2, 3])),
            width: Some(serde_json::json!("fit_content")),
            height: Some(serde_json::json!("fit_content")),
            children: vec![visible, hidden],
            ..Default::default()
        };

        let size = resolve_element_size(&parent, None, &doc, None);
        assert_eq!(size, Size::new(46.0, 24.0));
    }

    #[test]
    fn resolve_text_content_measures_width_height_and_wraps_fixed_width_text() {
        let doc = DesignDoc { version: "test".to_string(), ..Default::default() };
        let text = DesignElement {
            content: Some("aa\nbb".to_string()),
            font_size: Some(serde_json::json!(10)),
            line_height: Some(serde_json::json!(2.0)),
            padding: Some(serde_json::json!(1)),
            ..Default::default()
        };
        let size = resolve_element_size(&text, None, &doc, None);
        assert!(size.width > 10.0);
        assert_eq!(size.height, 42.0);

        let wrapped = DesignElement {
            content: Some("alpha beta".to_string()),
            text_growth: Some("fixed-width".to_string()),
            width: Some(serde_json::json!(30)),
            font_size: Some(serde_json::json!(10)),
            line_height: Some(serde_json::json!(10)),
            ..Default::default()
        };
        let size = resolve_element_size(&wrapped, None, &doc, None);
        assert_eq!(size.width, 30.0);
        assert!(size.height > 10.0);
    }

    #[test]
    fn compute_layout_handles_visibility_alignment_and_space_distribution() {
        let doc = DesignDoc { version: "test".to_string(), ..Default::default() };
        let parent = DesignElement {
            justify_content: Some("space-between".to_string()),
            align_items: Some("end".to_string()),
            ..Default::default()
        };
        let children = vec![
            DesignElement {
                width: Some(serde_json::json!(20)),
                height: Some(serde_json::json!(10)),
                ..Default::default()
            },
            DesignElement {
                visible: Some(false),
                width: Some(serde_json::json!(100)),
                height: Some(serde_json::json!(100)),
                ..Default::default()
            },
            DesignElement {
                width: Some(serde_json::json!(30)),
                height: Some(serde_json::json!(20)),
                ..Default::default()
            },
        ];

        let layouts = compute_layout(
            LayoutDirection::Horizontal,
            &children,
            Size::new(100.0, 50.0),
            &parent,
            &doc,
            None,
        );
        assert_eq!(layouts[0].offset, iced::Vector::new(0.0, 40.0));
        assert_eq!(layouts[1].size, Size::new(0.0, 0.0));
        assert_eq!(layouts[2].offset, iced::Vector::new(70.0, 30.0));
    }

    #[test]
    fn compute_layout_supports_center_space_around_evenly_stretch_and_vertical_fill() {
        let doc = DesignDoc { version: "test".to_string(), ..Default::default() };
        let child = DesignElement {
            width: Some(serde_json::json!(20)),
            height: Some(serde_json::json!(10)),
            ..Default::default()
        };

        for (justify, expected_x) in
            [("center", 40.0), ("end", 80.0), ("space-around", 80.0), ("space-evenly", 80.0)]
        {
            let parent = DesignElement {
                justify_content: Some(justify.to_string()),
                align_items: Some("center".to_string()),
                ..Default::default()
            };
            let layouts = compute_layout(
                LayoutDirection::Horizontal,
                std::slice::from_ref(&child),
                Size::new(100.0, 50.0),
                &parent,
                &doc,
                None,
            );
            assert_eq!(layouts[0].offset.x, expected_x, "justify {justify}");
            assert_eq!(layouts[0].offset.y, 20.0);
        }

        let parent =
            DesignElement { align_items: Some("stretch".to_string()), ..Default::default() };
        let layouts = compute_layout(
            LayoutDirection::Horizontal,
            std::slice::from_ref(&child),
            Size::new(100.0, 50.0),
            &parent,
            &doc,
            None,
        );
        assert_eq!(layouts[0].size.height, 50.0);

        let fill = DesignElement {
            height: Some(serde_json::json!("fill_container")),
            width: Some(serde_json::json!("fill_container")),
            ..Default::default()
        };
        let layouts = compute_layout(
            LayoutDirection::Vertical,
            &[fill],
            Size::new(80.0, 120.0),
            &DesignElement::default(),
            &doc,
            None,
        );
        assert_eq!(layouts[0].size, Size::new(80.0, 120.0));

        assert!(matches!(AlignMode::Start, AlignMode::Start));
    }
}
