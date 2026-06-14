#[test]
fn task_1180_test_module_is_wired() {}

use super::*;
use crate::app::views::design::properties::fill::types::{
    FillObject, GradientCenter, GradientSize, GradientStop,
};

fn keep_element(element: Element<'static, Message>) {
    std::hint::black_box(element);
}

fn gradient(gradient_type: &str) -> GradientFill {
    GradientFill {
        gradient_type: gradient_type.to_string(),
        enabled: true,
        rotation: 30.0,
        colors: vec![
            GradientStop { color: "#000000ff".to_string(), position: 0.0 },
            GradientStop { color: "#ffffffff".to_string(), position: 1.0 },
        ],
        center: Some(GradientCenter { x: 45.0, y: 55.0 }),
        size: Some(GradientSize { width: Some(120.0), height: Some(80.0) }),
        size_h: Some(65.0),
    }
}

#[test]
fn gradient_type_option_displays_label() {
    let option = GradientTypeOption { label: "线性", value: "linear" };

    assert_eq!(option.to_string(), "线性");
}

#[test]
fn render_covers_linear_radial_angular_mesh_and_unknown_types() {
    for gradient_type in ["linear", "radial", "angular", "mesh_gradient", "custom"] {
        let gradient = gradient(gradient_type);
        let fills = vec![FillItem::Object(FillObject::Gradient(gradient.clone()))];

        keep_element(render(gradient, 0, fills, "shape".to_string()));
    }
}

#[test]
fn render_handles_empty_stops_and_missing_optional_fields() {
    let gradient = GradientFill {
        gradient_type: "linear".to_string(),
        enabled: false,
        rotation: 0.0,
        colors: vec![],
        center: None,
        size: None,
        size_h: None,
    };
    let fills = vec![FillItem::Object(FillObject::Gradient(gradient.clone()))];

    keep_element(render(gradient, 0, fills, "shape".to_string()));
}
