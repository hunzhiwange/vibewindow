use crate::app::App;
use crate::app::views::design::models::{
    ColorFormat, ColorPickerTarget, DesignDoc, DesignElement, Stroke, VariableDef, VariableValue,
};
use crate::app::views::design::properties::fill::types::FillItem;
use crate::app::views::design::state::DesignState;
use iced::{Color, Point};
use serde_json::json;

fn app_with_doc(doc: DesignDoc) -> App {
    let mut app = App::new().0;
    let tab_id = "design".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, DesignState::new(doc));
    app
}

fn element_with_fill() -> DesignElement {
    DesignElement {
        id: "shape".to_string(),
        fill: Some(json!([{"type":"solid","color":"#000000","enabled":true}])),
        effect: Some(
            json!([{"type":"dropShadow","color":"#000000","offset":{"x":1.0,"y":2.0},"blur":3.0}]),
        ),
        ..Default::default()
    }
}

#[test]
fn font_picker_open_select_and_close_update_picker_and_element() {
    let doc = DesignDoc {
        children: vec![DesignElement {
            id: "text".to_string(),
            font_weight: Some(json!("700")),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);
    app.cursor_position = Point::new(4.0, 5.0);

    let _ = super::pickers::set_font_filter(&mut app, "inter".to_string());
    assert_eq!(app.font_filter_query, "inter");

    let _ = super::pickers::open_font_picker(&mut app, "text".to_string(), None);
    assert_eq!(app.active_font_picker.as_ref().unwrap().position, Point::new(4.0, 5.0));
    assert!(app.font_filter_query.is_empty());

    let _ = super::pickers::select_font(&mut app, "text".to_string(), "Inter".to_string());
    let element = app.active_design_state().unwrap().doc.find_element("text").unwrap();
    assert_eq!(element.font_family.as_deref(), Some("Inter"));
    assert!(element.font_weight.is_some());
    assert!(app.active_font_picker.is_none());

    let _ =
        super::pickers::open_font_picker(&mut app, "text".to_string(), Some(Point::new(1.0, 2.0)));
    let _ = super::pickers::close_font_picker(&mut app);
    assert!(app.active_font_picker.is_none());
}

#[test]
fn icon_picker_selects_family_and_icon_with_weight_fallback() {
    let doc = DesignDoc {
        children: vec![DesignElement {
            id: "icon".to_string(),
            icon_font_family: Some("lucide".to_string()),
            weight: Some(json!("regular")),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);

    let _ = super::pickers::set_icon_picker_filter(&mut app, "home".to_string());
    assert_eq!(app.icon_picker_filter_query, "home");

    let _ =
        super::pickers::open_icon_picker(&mut app, "icon".to_string(), Some(Point::new(2.0, 3.0)));
    assert_eq!(app.active_icon_picker.as_ref().unwrap().position, Point::new(2.0, 3.0));
    assert_eq!(app.icon_picker_family_tab, "lucide");

    let _ = super::pickers::set_icon_picker_family_tab(&mut app, "material".to_string());
    assert_eq!(app.icon_picker_family_tab, "material");

    let _ = super::pickers::select_icon(
        &mut app,
        "icon".to_string(),
        "lucide".to_string(),
        "house".to_string(),
    );
    let element = app.active_design_state().unwrap().doc.find_element("icon").unwrap();
    assert_eq!(element.icon_font_family.as_deref(), Some("lucide"));
    assert_eq!(element.icon_font_name.as_deref(), Some("house"));
    assert!(app.active_icon_picker.is_none());

    let _ = super::pickers::select_icon_family(&mut app, "icon".to_string(), "lucide".to_string());
    assert_eq!(
        app.active_design_state()
            .unwrap()
            .doc
            .find_element("icon")
            .unwrap()
            .icon_font_family
            .as_deref(),
        Some("lucide")
    );
}

#[test]
fn help_modal_open_and_close_store_text() {
    let mut app = app_with_doc(DesignDoc::default());

    let _ = super::pickers::show_help_modal(&mut app, "details".to_string());
    assert_eq!(app.design_help_text.as_deref(), Some("details"));

    let _ = super::pickers::close_help_modal(&mut app);
    assert_eq!(app.design_help_text, None);
}

#[test]
fn fill_picker_tracks_selection_format_and_color() {
    let mut app =
        app_with_doc(DesignDoc { children: vec![element_with_fill()], ..Default::default() });

    let _ = super::pickers::open_fill_picker(
        &mut app,
        "shape".to_string(),
        0,
        Some(Point::new(1.0, 1.0)),
    );
    assert_eq!(app.active_fill_picker.as_ref().unwrap().fill_index, 0);
    assert_eq!(app.active_design_state().unwrap().selected_fill_index, Some(0));

    let _ = super::pickers::toggle_fill_picker_eyedropper(&mut app);
    assert!(app.active_fill_picker.as_ref().unwrap().picking);

    let _ = super::pickers::change_fill_picker_format(&mut app, ColorFormat::Rgba);
    assert_eq!(app.active_fill_picker.as_ref().unwrap().format, ColorFormat::Rgba);

    let _ =
        super::pickers::change_fill_picker_color(&mut app, Color::from_rgba(1.0, 0.0, 0.0, 1.0));
    let fill =
        app.active_design_state().unwrap().doc.find_element("shape").unwrap().fill.clone().unwrap();
    let fills: Vec<FillItem> = serde_json::from_value(fill).unwrap();
    assert!(matches!(
        &fills[0],
        FillItem::Object(
            crate::app::views::design::properties::fill::types::FillObject::Solid {
                color,
                ..
            }
        ) if color == "#FF0000FF"
    ));

    let _ = super::pickers::close_fill_picker(&mut app);
    assert!(app.active_fill_picker.is_none());
}

#[test]
fn color_picker_updates_each_target_kind() {
    let mut doc = DesignDoc {
        children: vec![DesignElement {
            id: "shape".to_string(),
            fill: Some(json!([
                {"type":"solid","color":"#000000","enabled":true},
                {"type":"gradient","gradientType":"linear","colors":[{"color":"#111111","position":0.0},{"color":"#222222","position":1.0}]},
                {"type":"mesh_gradient","columns":1,"rows":1,"colors":["#333333"],"points":[[0.0,0.0]]}
            ])),
            effect: Some(json!([{"type":"dropShadow","color":"#000000","offset":{"x":0.0,"y":0.0},"blur":1.0}])),
            stroke: Some(Stroke {
                align: Some("inside".to_string()),
                thickness: Some(json!(1.0)),
                fill: Some("[{\"type\":\"solid\",\"color\":\"#000000\",\"opacity\":1.0,\"dashArray\":[4,4]}]".to_string()),
            }),
            ..Default::default()
        }],
        ..Default::default()
    };
    doc.variables.insert(
        "brand".to_string(),
        VariableDef {
            kind: "color".to_string(),
            collection: None,
            value: vec![VariableValue { theme: None, value: "#000000".to_string() }],
        },
    );
    let mut app = app_with_doc(doc);

    for target in [
        ColorPickerTarget::Fill { element_id: "shape".to_string(), fill_index: 0 },
        ColorPickerTarget::GradientStop {
            element_id: "shape".to_string(),
            fill_index: 1,
            stop_index: 1,
        },
        ColorPickerTarget::MeshPoint {
            element_id: "shape".to_string(),
            fill_index: 2,
            point_index: 0,
        },
        ColorPickerTarget::Effect { element_id: "shape".to_string(), effect_index: 0 },
        ColorPickerTarget::ContextFill { element_id: "shape".to_string() },
        ColorPickerTarget::ContextBorder { element_id: "shape".to_string() },
        ColorPickerTarget::ContextText { element_id: "shape".to_string() },
        ColorPickerTarget::VariableValue { variable_name: "brand".to_string(), mode: None },
    ] {
        let _ = super::pickers::open_color_picker(
            &mut app,
            Color::BLACK,
            target,
            Some(Point::new(9.0, 9.0)),
        );
        let _ = super::pickers::change_color_picker_color(
            &mut app,
            Color::from_rgba(0.0, 1.0, 0.0, 1.0),
        );
    }

    let element = app.active_design_state().unwrap().doc.find_element("shape").unwrap();
    assert_eq!(element.color.as_deref(), Some("#00FF00FF"));
    assert!(element.stroke.as_ref().unwrap().fill.as_ref().unwrap().contains("dashArray"));
    assert_eq!(
        app.active_design_state().unwrap().doc.variables["brand"].value[0].value,
        "#00FF00FF"
    );

    let _ = super::pickers::toggle_color_picker_eyedropper(&mut app);
    assert!(app.active_color_picker.as_ref().unwrap().picking);

    let _ = super::pickers::change_color_picker_format(&mut app, ColorFormat::Hsl);
    assert_eq!(app.active_color_picker.as_ref().unwrap().format, ColorFormat::Hsl);

    let _ = super::pickers::close_color_picker(&mut app);
    assert!(app.active_color_picker.is_none());
}

#[test]
fn select_fill_and_effect_store_indices() {
    let mut app = app_with_doc(DesignDoc::default());

    let _ = super::pickers::select_fill(&mut app, Some(2));
    let _ = super::pickers::select_effect(&mut app, Some(1));

    let state = app.active_design_state().unwrap();
    assert_eq!(state.selected_fill_index, Some(2));
    assert_eq!(state.selected_effect_index, Some(1));
}
