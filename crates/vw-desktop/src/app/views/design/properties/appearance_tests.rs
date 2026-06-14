#[test]
fn task_1169_test_module_is_wired() {}

use iced::Point;
use serde_json::{Value, json};

use crate::app::Message;
use crate::app::message::DesignMessage;
use crate::app::views::design::models::{DesignElement, Effect, Offset};

fn element_with_effect(effect: Option<Value>) -> DesignElement {
    DesignElement {
        id: "shape".to_string(),
        kind: "rect".to_string(),
        effect,
        ..Default::default()
    }
}

fn shadow(visible: Option<bool>) -> Effect {
    Effect {
        kind: "shadow".to_string(),
        shadow_type: Some("outer".to_string()),
        color: Some("#10203040".to_string()),
        offset: Some(Offset { x: 1.0, y: 2.0 }),
        blur: Some(3.0),
        spread: Some(4.0),
        radius: None,
        visible,
        enabled: Some(true),
    }
}

fn blur(kind: &str) -> Effect {
    Effect {
        kind: kind.to_string(),
        shadow_type: None,
        color: None,
        offset: None,
        blur: None,
        spread: None,
        radius: Some(6.0),
        visible: Some(true),
        enabled: Some(true),
    }
}

fn effects_from_message(message: Message) -> (String, Vec<Effect>) {
    match message {
        Message::Design(DesignMessage::PropertyUpdate(id, prop, value)) => {
            assert_eq!(prop, "effect");
            (id, serde_json::from_value(value).expect("effect payload should deserialize"))
        }
        _ => panic!("unexpected message"),
    }
}

#[test]
fn format_opacity_clamps_and_trims() {
    assert_eq!(super::format_opacity(-5.0), "0");
    assert_eq!(super::format_opacity(0.0), "0");
    assert_eq!(super::format_opacity(12.50), "12.5");
    assert_eq!(super::format_opacity(100.0), "100");
    assert_eq!(super::format_opacity(120.0), "100");
}

#[test]
fn parse_effects_accepts_none_array_single_object_and_invalid_json() {
    assert!(super::parse_effects(&None).is_empty());
    assert!(super::parse_effects(&Some(json!("bad"))).is_empty());

    let effects = vec![shadow(Some(true)), blur("layer_blur")];
    let parsed = super::parse_effects(&Some(serde_json::to_value(&effects).unwrap()));
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].kind, "shadow");
    assert_eq!(parsed[1].kind, "layer_blur");

    let parsed_one = super::parse_effects(&Some(serde_json::to_value(shadow(None)).unwrap()));
    assert_eq!(parsed_one.len(), 1);
    assert_eq!(parsed_one[0].shadow_type.as_deref(), Some("outer"));
}

#[test]
fn parse_hex_to_rgba_accepts_rgb_rgba_and_falls_back_to_black() {
    assert_eq!(super::parse_hex_to_rgba("not-hex"), (0.0, 0.0, 0.0, 1.0));
    assert_eq!(super::parse_hex_to_rgba("#zz2030"), (0.0, 0.0, 0.0, 1.0));

    let (r, g, b, a) = super::parse_hex_to_rgba("#102030");
    assert!((r - 0x10 as f32 / 255.0).abs() < 0.001);
    assert!((g - 0x20 as f32 / 255.0).abs() < 0.001);
    assert!((b - 0x30 as f32 / 255.0).abs() < 0.001);
    assert_eq!(a, 1.0);

    let (_, _, _, alpha) = super::parse_hex_to_rgba("#10203080");
    assert!((alpha - 128.0 / 255.0).abs() < 0.001);

    let (_, _, _, fallback_alpha) = super::parse_hex_to_rgba("#102030zz");
    assert_eq!(fallback_alpha, 1.0);
}

#[test]
fn add_effect_of_kind_supports_all_labels_and_rejects_unknown() {
    let existing = vec![shadow(Some(true))];
    let cases = [
        ("Drop shadow", "shadow", Some("outer")),
        ("投影", "shadow", Some("outer")),
        ("Inner shadow", "shadow", Some("inner")),
        ("内阴影", "shadow", Some("inner")),
        ("Layer blur", "layer_blur", None),
        ("图层模糊", "layer_blur", None),
        ("Background blur", "background_blur", None),
        ("背景模糊", "background_blur", None),
    ];

    for (label, expected_kind, expected_shadow_type) in cases {
        let (id, effects) =
            effects_from_message(super::add_effect_of_kind("shape".to_string(), &existing, label));
        assert_eq!(id, "shape");
        assert_eq!(effects.len(), 2);
        let added = effects.last().unwrap();
        assert_eq!(added.kind, expected_kind);
        assert_eq!(added.shadow_type.as_deref(), expected_shadow_type);
        assert_eq!(added.visible, Some(true));
        assert_eq!(added.enabled, Some(true));
    }

    assert!(matches!(
        super::add_effect_of_kind("shape".to_string(), &existing, "unknown"),
        Message::None
    ));
    let (_, effects) = effects_from_message(super::add_effect("shape".to_string(), &existing));
    assert_eq!(effects.last().unwrap().shadow_type.as_deref(), Some("outer"));
}

#[test]
fn remove_toggle_and_shadow_type_updates_handle_valid_and_invalid_indexes() {
    let effects = vec![shadow(Some(true)), blur("background_blur")];

    let (_, removed) = effects_from_message(super::remove_effect("shape".to_string(), &effects, 0));
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].kind, "background_blur");

    let (_, unchanged) =
        effects_from_message(super::remove_effect("shape".to_string(), &effects, 99));
    assert_eq!(unchanged.len(), 2);

    let (_, toggled) = effects_from_message(super::toggle_effect("shape".to_string(), &effects, 0));
    assert_eq!(toggled[0].visible, Some(false));

    let effects_without_visibility = vec![shadow(None)];
    let (_, toggled_default) = effects_from_message(super::toggle_effect(
        "shape".to_string(),
        &effects_without_visibility,
        0,
    ));
    assert_eq!(toggled_default[0].visible, Some(false));

    let (_, inner) =
        effects_from_message(super::update_shadow_type("shape".to_string(), &effects, 0, true));
    assert_eq!(inner[0].shadow_type.as_deref(), Some("inner"));

    let (_, outer) =
        effects_from_message(super::update_shadow_type("shape".to_string(), &effects, 0, false));
    assert_eq!(outer[0].shadow_type.as_deref(), Some("outer"));
}

#[test]
fn effect_field_update_closures_parse_values_and_create_missing_offset() {
    let effects = vec![Effect { offset: None, ..shadow(Some(true)) }];

    let (_, updated_x) =
        effects_from_message(super::update_effect_offset_x("shape".to_string(), &effects, 0)(
            "12.5".to_string(),
        ));
    assert_eq!(updated_x[0].offset.as_ref().unwrap().x, 12.5);
    assert_eq!(updated_x[0].offset.as_ref().unwrap().y, 0.0);

    let (_, updated_y) =
        effects_from_message(super::update_effect_offset_y("shape".to_string(), &effects, 0)(
            "-3".to_string(),
        ));
    assert_eq!(updated_y[0].offset.as_ref().unwrap().y, -3.0);

    let (_, blur_updated) =
        effects_from_message(super::update_effect_blur("shape".to_string(), &effects, 0)(
            "bad".to_string(),
        ));
    assert_eq!(blur_updated[0].blur, Some(0.0));

    let (_, spread) =
        effects_from_message(super::update_effect_spread("shape".to_string(), &effects, 0)(
            "7.25".to_string(),
        ));
    assert_eq!(spread[0].spread, Some(7.25));

    let blur_effects = vec![blur("layer_blur")];
    let (_, radius) =
        effects_from_message(super::update_effect_radius("shape".to_string(), &blur_effects, 0)(
            "9".to_string(),
        ));
    assert_eq!(radius[0].radius, Some(9.0));

    let (_, color) =
        effects_from_message(super::update_effect_color("shape".to_string(), &effects, 0)(
            "#abcdef".to_string(),
        ));
    assert_eq!(color[0].color.as_deref(), Some("#abcdef"));
}

#[test]
fn render_entry_points_construct_for_empty_shadow_blur_and_invalid_indexes() {
    let empty = element_with_effect(None);
    let appearance_element = DesignElement {
        id: "shape".to_string(),
        opacity: Some(0.335),
        ..Default::default()
    };
    let _appearance = super::render_appearance(&appearance_element);
    let _effects_empty = super::render_effects(&empty, None);
    let _invalid_popover = super::render_popover(&empty, 10);

    let effects = vec![
        shadow(Some(true)),
        Effect {
            kind: "shadow".to_string(),
            shadow_type: Some("inner".to_string()),
            color: None,
            offset: None,
            blur: None,
            spread: None,
            radius: None,
            visible: Some(false),
            enabled: Some(true),
        },
        blur("layer_blur"),
        blur("background_blur"),
        Effect { kind: "custom".to_string(), ..shadow(Some(true)) },
    ];
    let element = element_with_effect(Some(serde_json::to_value(effects).unwrap()));
    let _effects = super::render_effects(&element, Some(1));
    for index in 0..5 {
        let _popover = super::render_popover(&element, index);
    }
}

#[test]
fn active_effect_picker_is_cloneable_debug_state() {
    let picker = super::ActiveEffectPicker {
        element_id: "shape".to_string(),
        effect_index: 2,
        position: Point::new(10.0, 20.0),
    };
    let cloned = picker.clone();
    assert_eq!(cloned.element_id, "shape");
    assert_eq!(cloned.effect_index, 2);
    assert!(format!("{picker:?}").contains("shape"));
}
