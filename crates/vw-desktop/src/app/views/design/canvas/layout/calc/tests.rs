//! 设计画布布局模块。
//!
//! 该模块负责解析和计算画布节点布局，帮助渲染层获得稳定的几何信息。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::{compute_layout, resolve_element_size};
    use crate::app::views::design::canvas::types::LayoutDirection;
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
}
