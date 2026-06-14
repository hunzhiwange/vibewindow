use iced::Size;

#[test]
fn clamp_child_size_keeps_non_negative_bounds() {
    let size = super::clamp_child_size_to_content(
        Size::new(20.0, 30.0),
        100.0,
        120.0,
        Size::new(200.0, 200.0),
    );
    assert!(size.width >= 0.0);
    assert!(size.height >= 0.0);
}

#[test]
fn clamp_child_size_keeps_child_inside_content_unchanged() {
    let size = super::clamp_child_size_to_content(
        Size::new(200.0, 120.0),
        20.0,
        10.0,
        Size::new(80.0, 40.0),
    );

    assert_eq!(size, Size::new(80.0, 40.0));
}

#[test]
fn clamp_child_size_clips_overflowing_child_to_remaining_content() {
    let size = super::clamp_child_size_to_content(
        Size::new(200.0, 120.0),
        150.0,
        90.0,
        Size::new(80.0, 40.0),
    );

    assert_eq!(size, Size::new(50.0, 30.0));
}

#[test]
fn clamp_child_size_allows_negative_offsets_to_keep_full_child_size() {
    let size = super::clamp_child_size_to_content(
        Size::new(200.0, 120.0),
        -20.0,
        -10.0,
        Size::new(80.0, 40.0),
    );

    assert_eq!(size, Size::new(80.0, 40.0));
}

#[test]
fn clamp_child_size_returns_zero_for_zero_content() {
    let size =
        super::clamp_child_size_to_content(Size::new(0.0, 0.0), 0.0, 0.0, Size::new(80.0, 40.0));

    assert_eq!(size, Size::new(0.0, 0.0));
}

#[test]
fn expand_slot_children_returns_none_without_slot_array() {
    let element = crate::app::views::design::models::DesignElement::default();

    assert!(super::expand_slot_children(&element).is_none());
}

#[test]
fn expand_slot_children_returns_none_for_empty_slot_array() {
    let element = crate::app::views::design::models::DesignElement {
        slot: Some(serde_json::json!([])),
        ..Default::default()
    };

    assert!(super::expand_slot_children(&element).is_none());
}

#[test]
fn expand_slot_children_returns_none_when_slot_array_has_no_string_ids() {
    let element = crate::app::views::design::models::DesignElement {
        slot: Some(serde_json::json!([false, 12, null, {"id": "ignored"}])),
        ..Default::default()
    };

    assert!(super::expand_slot_children(&element).is_none());
}

#[test]
fn expand_slot_children_creates_ordered_ref_elements_for_string_ids() {
    let element = crate::app::views::design::models::DesignElement {
        id: "host".to_string(),
        slot: Some(serde_json::json!(["first", false, "second"])),
        ..Default::default()
    };

    let children = super::expand_slot_children(&element).expect("slot refs");

    assert_eq!(children.len(), 2);
    assert_eq!(children[0].kind, "ref");
    assert_eq!(children[0].id, "host__slot__0");
    assert_eq!(children[0].reference.as_deref(), Some("first"));
    assert_eq!(children[1].kind, "ref");
    assert_eq!(children[1].id, "host__slot__1");
    assert_eq!(children[1].reference.as_deref(), Some("second"));
}
