use super::*;

fn element(kind: &str, children: Vec<DesignElement>) -> DesignElement {
    DesignElement {
        id: format!("{kind}-id"),
        kind: kind.to_string(),
        children,
        ..Default::default()
    }
}

#[test]
fn dark_theme_detection_uses_background_luminance() {
    assert!(is_dark_theme(&Theme::Dark));
    assert!(!is_dark_theme(&Theme::Light));
}

#[test]
fn layer_hover_background_differs_for_light_and_dark_themes() {
    let light = layer_item_hover_bg(&Theme::Light);
    let dark = layer_item_hover_bg(&Theme::Dark);

    assert_eq!(light, Color::from_rgb8(240, 240, 240));
    assert!(dark.a > 0.0);
    assert_ne!(light, dark);
}

#[test]
fn can_switch_page_only_for_shallow_container_layers() {
    let child = element("rect", vec![]);
    let frame = element("frame", vec![child.clone()]);
    let group = element("group", vec![child.clone()]);
    let component = element("component", vec![child.clone()]);
    let reference = element("ref", vec![child.clone()]);
    let leaf_frame = element("frame", vec![]);
    let deep_frame = element("frame", vec![child]);

    assert!(can_switch_page(&frame, 0));
    assert!(can_switch_page(&group, 1));
    assert!(can_switch_page(&component, 1));
    assert!(can_switch_page(&reference, 1));
    assert!(!can_switch_page(&leaf_frame, 0));
    assert!(!can_switch_page(&deep_frame, 2));
    assert!(!can_switch_page(&element("rect", vec![element("text", vec![])]), 0));
}

#[test]
fn page_count_badge_and_context_menu_can_be_constructed() {
    let _badge: Element<'_, Message> = page_count_badge(3, true);
    let _menu: Element<'_, Message> = render_page_context_menu(2);
}

#[test]
fn layer_item_can_render_plain_and_reference_children() {
    let referenced = DesignElement {
        id: "source".to_string(),
        kind: "frame".to_string(),
        children: vec![DesignElement {
            id: "source-child".to_string(),
            kind: "text".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let reference = DesignElement {
        id: "ref".to_string(),
        kind: "ref".to_string(),
        reference: Some("source".to_string()),
        ..Default::default()
    };
    let doc = DesignDoc { children: vec![referenced, reference.clone()], ..Default::default() };
    let mut expanded = std::collections::HashSet::new();
    expanded.insert("ref".to_string());

    let _item: Element<'_, Message> = render_layer_item(
        &reference,
        &doc,
        0,
        Some("ref"),
        &expanded,
        Some("ref"),
        Some("source-child"),
        Some("ref"),
        Some("ref"),
        Some(Point::new(4.0, 5.0)),
        240.0,
    );
}

#[test]
fn page_card_can_render_normal_and_renaming_states() {
    let doc = DesignDoc {
        groups: vec![crate::app::views::design::models::DesignGroup {
            id: 1,
            name: "Page".to_string(),
        }],
        ..Default::default()
    };
    let mut state = DesignState::new(doc);
    state.doc.groups =
        vec![crate::app::views::design::models::DesignGroup { id: 1, name: "Page".to_string() }];
    state.active_group_id = 1;
    let group = state.doc.groups[0].clone();

    let normal: Element<'_, Message> = render_page_card(&state, &group);
    drop(normal);

    state.renaming_page_id = Some(1);
    state.renaming_page_name = "Renamed".to_string();
    let _renaming: Element<'_, Message> = render_page_card(&state, &group);
}
