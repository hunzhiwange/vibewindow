//! 设计导入测试模块，验证 Tailwind 和通用导入路径生成稳定的设计元素树。

use super::import_html_as_positioned_elements;
use iced::Size;
use serde_json::Value;

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.01, "expected {expected}, got {actual}");
}

fn value_as_f32(value: &Option<Value>) -> f32 {
    value
        .as_ref()
        .and_then(|value| match value {
            Value::Number(number) => number.as_f64().map(|number| number as f32),
            Value::String(string) => string.parse::<f32>().ok(),
            _ => None,
        })
        .expect("expected numeric size value")
}

#[test]
fn converts_flex_children_using_renderer_geometry() {
    let elements = import_html_as_positioned_elements(
        "<div class=\"flex justify-center w-40 h-10\"><div class=\"w-8 h-4\"></div><div class=\"w-4 h-4\"></div></div>",
        Size::new(160.0, 40.0),
    );

    assert_eq!(elements.len(), 1);

    let root = &elements[0];
    assert_eq!(root.kind, "frame");
    assert_eq!(root.layout.as_deref(), Some("none"));
    assert_eq!(root.children.len(), 2);
    assert_close(value_as_f32(&root.width), 160.0);
    assert_close(value_as_f32(&root.height), 40.0);

    let first = &root.children[0];
    assert_close(first.x, 56.0);
    assert_close(first.y, 0.0);
    assert_close(value_as_f32(&first.width), 32.0);
    assert_close(value_as_f32(&first.height), 16.0);

    let second = &root.children[1];
    assert_close(second.x, 88.0);
    assert_close(second.y, 0.0);
    assert_close(value_as_f32(&second.width), 16.0);
    assert_close(value_as_f32(&second.height), 16.0);
}

#[test]
fn converts_padding_to_static_offsets_without_double_applying() {
    let elements = import_html_as_positioned_elements(
        "<div class=\"p-4 w-20 h-20\"><div class=\"w-4 h-4\"></div></div>",
        Size::new(80.0, 80.0),
    );

    assert_eq!(elements.len(), 1);

    let root = &elements[0];
    assert_eq!(root.kind, "frame");
    assert!(root.padding.is_none());
    assert_eq!(root.children.len(), 1);

    let child = &root.children[0];
    assert_close(child.x, 16.0);
    assert_close(child.y, 16.0);
    assert_close(value_as_f32(&child.width), 16.0);
    assert_close(value_as_f32(&child.height), 16.0);
}

#[test]
fn converts_absolute_position_using_renderer_layout() {
    let elements = import_html_as_positioned_elements(
        "<div class=\"relative w-40 h-20\"><div class=\"absolute right-4 bottom-2 w-8 h-4\"></div></div>",
        Size::new(160.0, 80.0),
    );

    assert_eq!(elements.len(), 1);

    let child = &elements[0].children[0];
    assert_close(child.x, 112.0);
    assert_close(child.y, 56.0);
    assert_close(value_as_f32(&child.width), 32.0);
    assert_close(value_as_f32(&child.height), 16.0);
}

#[test]
fn converts_svg_paths_to_explicit_vector_layers() {
    let elements = import_html_as_positioned_elements(
        "<svg class=\"w-20\" viewBox=\"0 0 24 12\"><path d=\"M0 0 L24 0 L24 12 Z\" /></svg>",
        Size::new(240.0, 120.0),
    );

    assert_eq!(elements.len(), 1);

    let svg = &elements[0];
    assert_eq!(svg.kind, "frame");
    assert_eq!(svg.name.as_deref(), Some("Icon"));
    assert_close(value_as_f32(&svg.width), 80.0);
    assert_close(value_as_f32(&svg.height), 40.0);
    assert_eq!(svg.children.len(), 1);

    let path = &svg.children[0];
    assert_eq!(path.kind, "path");
    assert_close(path.x, 0.0);
    assert_close(path.y, 0.0);
    assert_close(value_as_f32(&path.width), 80.0);
    assert_close(value_as_f32(&path.height), 40.0);
}
