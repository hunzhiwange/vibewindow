#[test]
fn task_1187_test_module_is_wired() {}

use super::*;
use crate::app::message::DesignMessage;

fn keep_element<'a>(element: Element<'a, Message>) {
    std::hint::black_box(element);
}

fn element(fill: Option<serde_json::Value>) -> DesignElement {
    serde_json::from_value(serde_json::json!({
        "type": "rect",
        "id": "shape",
        "fill": fill,
    }))
    .expect("design element should deserialize")
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

fn sample_gradient() -> FillItem {
    FillItem::Object(FillObject::Gradient(GradientFill {
        gradient_type: "linear".to_string(),
        enabled: true,
        rotation: 0.0,
        colors: vec![
            GradientStop { color: "#000000ff".to_string(), position: 0.0 },
            GradientStop { color: "#ffffffff".to_string(), position: 1.0 },
        ],
        center: None,
        size: None,
        size_h: None,
    }))
}

fn sample_image() -> FillItem {
    FillItem::Object(FillObject::Image(ImageFill {
        enabled: true,
        url: "https://example.test/a.png".to_string(),
        mode: "cover".to_string(),
    }))
}

#[test]
fn parse_fills_handles_none_single_fill_array_and_invalid_values() {
    assert!(parse_fills(&None).is_empty());
    assert!(parse_fills(&Some(serde_json::json!({"bad": true}))).is_empty());

    assert_eq!(
        parse_fills(&Some(serde_json::json!("#123456"))),
        vec![FillItem::Color("#123456".to_string())]
    );

    let fills = parse_fills(&Some(serde_json::json!([
        "#000000",
        {"type": "solid", "color": "#ffffff", "enabled": false}
    ])));
    assert_eq!(fills.len(), 2);
    assert!(matches!(fills[1], FillItem::Object(FillObject::Solid { enabled: false, .. })));
}

#[test]
fn parse_fills_normalizes_mesh_objects() {
    let fills = parse_fills(&Some(serde_json::json!([{
        "type": "mesh_gradient",
        "enabled": true,
        "columns": 1,
        "rows": 9,
        "colors": [],
        "points": [],
        "handles": [],
        "mirroring": "Y",
        "outline": true,
        "selected_point_index": 99
    }])));

    match &fills[0] {
        FillItem::Object(FillObject::Mesh(mesh)) => {
            assert_eq!((mesh.columns, mesh.rows), (2, 6));
            assert_eq!(mesh.colors.len(), 12);
            assert_eq!(mesh.points.len(), 12);
            assert_eq!(mesh.handles.len(), 12);
            assert_eq!(mesh.selected_point_index, None);
            assert_eq!(mesh.mirroring.as_deref(), Some("y"));
        }
        _ => panic!("expected mesh fill"),
    }
}

#[test]
fn render_and_popover_cover_list_selection_and_empty_popover() {
    let fill = serde_json::json!([
        "#000000ff",
        {"type": "gradient", "gradientType": "linear", "colors": []},
        {"type": "image", "url": "https://example.test/a.png", "mode": "cover", "enabled": true}
    ]);
    let element = element(Some(fill));

    keep_element(render(&element, Some(1)));
    keep_element(render_popover(
        &element,
        0,
        ColorFormat::Hex,
        false,
        iced::Vector::new(0.0, 0.0),
        1.0,
    ));
    keep_element(render_popover(
        &element,
        99,
        ColorFormat::Rgba,
        true,
        iced::Vector::new(10.0, 20.0),
        0.75,
    ));
}

#[test]
fn render_picker_covers_each_fill_kind_and_tab_buttons() {
    let fills = vec![
        FillItem::Color("#000000ff".to_string()),
        FillItem::Object(FillObject::Solid { color: "#111111ff".to_string(), enabled: true }),
        FillItem::Object(FillObject::Color { color: "#222222ff".to_string(), enabled: false }),
        sample_gradient(),
        FillItem::Object(FillObject::Mesh(MeshFill::new_random(3, 3))),
        sample_image(),
    ];

    for (index, item) in fills.clone().into_iter().enumerate() {
        keep_element(render_picker(
            item,
            index,
            fills.clone(),
            "shape".to_string(),
            ColorFormat::Css,
            false,
            iced::Vector::new(4.0, 5.0),
            1.5,
        ));
    }
}

#[test]
fn fill_list_action_helpers_add_remove_toggle_and_update_color() {
    let fills = vec![
        FillItem::Color("#000000ff".to_string()),
        FillItem::Object(FillObject::Solid { color: "#111111ff".to_string(), enabled: true }),
        sample_gradient(),
    ];

    let added = updated_fills(add_fill("shape".to_string(), &fills));
    assert_eq!(added.len(), 4);
    assert_eq!(added.last(), Some(&FillItem::Color("#000000ff".to_string())));

    let removed = updated_fills(remove_fill("shape".to_string(), &fills, 1));
    assert_eq!(removed.len(), 2);
    assert!(matches!(removed[1], FillItem::Object(FillObject::Gradient(_))));

    let unchanged = updated_fills(remove_fill("shape".to_string(), &fills, 99));
    assert_eq!(unchanged, fills);

    let toggled = updated_fills(toggle_fill("shape".to_string(), &fills, 1));
    assert!(matches!(toggled[1], FillItem::Object(FillObject::Solid { enabled: false, .. })));

    let color = updated_fills(update_fill_color(
        "shape".to_string(),
        fills.clone(),
        0,
        "#abcdef".to_string(),
    ));
    assert_eq!(color[0], FillItem::Color("#abcdef".to_string()));

    let color = updated_fills(update_fill_color(
        "shape".to_string(),
        fills.clone(),
        2,
        "#abcdef".to_string(),
    ));
    assert_eq!(color[2], fills[2]);
}

#[test]
fn change_fill_type_replaces_supported_tabs_and_ignores_bad_index() {
    let fills = vec![sample_gradient()];

    assert_eq!(
        updated_fills(change_fill_type("shape".to_string(), fills.clone(), 0, FillTab::Color))[0],
        FillItem::Color("#000000ff".to_string())
    );

    assert!(matches!(
        updated_fills(change_fill_type("shape".to_string(), fills.clone(), 0, FillTab::Gradient))
            [0],
        FillItem::Object(FillObject::Gradient(_))
    ));
    assert!(matches!(
        updated_fills(change_fill_type("shape".to_string(), fills.clone(), 0, FillTab::Mesh))[0],
        FillItem::Object(FillObject::Mesh(_))
    ));
    assert!(matches!(
        updated_fills(change_fill_type("shape".to_string(), fills.clone(), 0, FillTab::Image))[0],
        FillItem::Object(FillObject::Image(_))
    ));

    assert_eq!(
        updated_fills(change_fill_type("shape".to_string(), fills.clone(), 9, FillTab::Color)),
        fills
    );
}

#[test]
fn gradient_type_label_maps_known_and_unknown_types() {
    assert_eq!(gradient_type_label("linear"), "线性");
    assert_eq!(gradient_type_label("radial"), "径向");
    assert_eq!(gradient_type_label("angular"), "角向");
    assert_eq!(gradient_type_label("mesh_gradient"), "网格");
    assert_eq!(gradient_type_label("custom"), "custom");
}
