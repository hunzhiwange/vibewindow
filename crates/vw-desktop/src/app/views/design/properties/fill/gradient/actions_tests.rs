#[test]
fn task_1179_test_module_is_wired() {}

use super::*;
use crate::app::message::DesignMessage;

fn gradient_fill() -> GradientFill {
    GradientFill {
        gradient_type: "linear".to_string(),
        enabled: true,
        rotation: 12.0,
        colors: vec![
            GradientStop { color: "#000000".to_string(), position: 0.0 },
            GradientStop { color: "#ffffff".to_string(), position: 1.0 },
        ],
        center: None,
        size: None,
        size_h: None,
    }
}

fn fills() -> Vec<FillItem> {
    vec![FillItem::Object(FillObject::Gradient(gradient_fill()))]
}

fn updated_fills(message: Message) -> Vec<FillItem> {
    match message {
        Message::Design(DesignMessage::PropertyUpdate(id, key, value)) => {
            assert_eq!(id, "shape");
            assert_eq!(key, "fill");
            serde_json::from_value(value).expect("fill update should serialize")
        }
        _ => panic!("unexpected message"),
    }
}

fn updated_gradient(message: Message) -> GradientFill {
    match updated_fills(message).remove(0) {
        FillItem::Object(FillObject::Gradient(gradient)) => gradient,
        _ => panic!("expected gradient fill"),
    }
}

#[test]
fn update_gradient_type_updates_existing_gradient() {
    let gradient = updated_gradient(update_gradient_type(
        "shape".to_string(),
        fills(),
        0,
        "radial".to_string(),
    ));

    assert_eq!(gradient.gradient_type, "radial");
    assert_eq!(gradient.rotation, 12.0);
}

#[test]
fn update_gradient_type_converts_non_gradient_and_mesh() {
    let solid_fills =
        vec![FillItem::Object(FillObject::Solid { color: "#123456".to_string(), enabled: true })];
    let gradient = updated_gradient(update_gradient_type(
        "shape".to_string(),
        solid_fills,
        0,
        "angular".into(),
    ));
    assert_eq!(gradient.gradient_type, "angular");
    assert!(gradient.enabled);

    let mesh_fills = updated_fills(update_gradient_type(
        "shape".to_string(),
        fills(),
        0,
        "mesh_gradient".to_string(),
    ));
    assert!(
        matches!(mesh_fills.first(), Some(FillItem::Object(FillObject::Mesh(mesh))) if mesh.columns == 3 && mesh.rows == 3)
    );
}

#[test]
fn update_gradient_rotation_accepts_numbers_and_ignores_invalid_input() {
    let gradient =
        updated_gradient(update_gradient_rotation("shape".to_string(), fills(), 0, "45.5".into()));
    assert_eq!(gradient.rotation, 45.5);

    let gradient =
        updated_gradient(update_gradient_rotation("shape".to_string(), fills(), 0, "bad".into()));
    assert_eq!(gradient.rotation, 12.0);
}

#[test]
fn update_gradient_center_and_size_create_defaults() {
    let gradient =
        updated_gradient(update_gradient_center_x("shape".to_string(), fills(), 0, "25".into()));
    assert_eq!(gradient.center.unwrap(), GradientCenter { x: 25.0, y: 50.0 });

    let gradient =
        updated_gradient(update_gradient_center_y("shape".to_string(), fills(), 0, "75".into()));
    assert_eq!(gradient.center.unwrap(), GradientCenter { x: 50.0, y: 75.0 });

    let gradient =
        updated_gradient(update_gradient_size_w("shape".to_string(), fills(), 0, "33".into()));
    assert_eq!(gradient.size.unwrap().width, Some(33.0));

    let gradient =
        updated_gradient(update_gradient_size_v("shape".to_string(), fills(), 0, "66".into()));
    assert_eq!(gradient.size.unwrap().height, Some(66.0));
}

#[test]
fn update_gradient_size_h_trims_percent_and_clamps() {
    let gradient =
        updated_gradient(update_gradient_size_h("shape".to_string(), fills(), 0, "250%".into()));
    assert_eq!(gradient.size_h, Some(200.0));

    let gradient =
        updated_gradient(update_gradient_size_h("shape".to_string(), fills(), 0, "-5".into()));
    assert_eq!(gradient.size_h, Some(0.0));

    let gradient =
        updated_gradient(update_gradient_size_h("shape".to_string(), fills(), 0, "oops".into()));
    assert_eq!(gradient.size_h, None);
}

#[test]
fn gradient_stop_actions_mutate_only_valid_targets() {
    let gradient = updated_gradient(update_gradient_stop_color(
        "shape".to_string(),
        fills(),
        0,
        1,
        "#abcdef".to_string(),
    ));
    assert_eq!(gradient.colors[1].color, "#abcdef");

    let gradient = updated_gradient(update_gradient_stop_position(
        "shape".to_string(),
        fills(),
        0,
        0,
        "125%".to_string(),
    ));
    assert_eq!(gradient.colors[0].position, 1.0);

    let gradient = updated_gradient(update_gradient_stop_position(
        "shape".to_string(),
        fills(),
        0,
        1,
        "-10".to_string(),
    ));
    assert_eq!(gradient.colors[1].position, 0.0);

    let gradient = updated_gradient(remove_gradient_stop("shape".to_string(), fills(), 0, 0));
    assert_eq!(gradient.colors.len(), 1);
    assert_eq!(gradient.colors[0].color, "#ffffff");

    let gradient = updated_gradient(add_gradient_stop("shape".to_string(), fills(), 0));
    assert_eq!(gradient.colors.last().unwrap().color, "#ffffff");
    assert_eq!(gradient.colors.last().unwrap().position, 0.5);
}

#[test]
fn update_gradient_stops_replaces_stops_and_out_of_range_is_noop() {
    let replacement = vec![GradientStop { color: "#ff0000".to_string(), position: 0.25 }];
    let gradient = updated_gradient(update_gradient_stops(
        "shape".to_string(),
        fills(),
        0,
        replacement.clone(),
    ));
    assert_eq!(gradient.colors, replacement);

    let unchanged = updated_fills(update_gradient_stop_color(
        "shape".to_string(),
        fills(),
        99,
        0,
        "#badbad".to_string(),
    ));
    assert_eq!(unchanged, fills());
}
