#[test]
fn image_fill_editor_renders_for_image_object() {
    let image = super::super::types::ImageFill {
        url: "https://example.test/a.png".to_string(),
        mode: "cover".to_string(),
        enabled: true,
    };
    let fills = vec![super::super::types::FillItem::Object(
        super::super::types::FillObject::Image(image.clone()),
    )];

    let _element = super::render(image, 0, fills, "shape".to_string());
}
