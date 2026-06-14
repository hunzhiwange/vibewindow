fn element(kind: &str) -> crate::app::views::design::models::DesignElement {
    crate::app::views::design::models::DesignElement {
        id: "node".to_string(),
        kind: kind.to_string(),
        name: Some("Layer name".to_string()),
        content: Some("Text content".to_string()),
        ..Default::default()
    }
}

#[test]
fn content_sections_render_for_text_and_non_text_elements() {
    let text = element("text");
    let context_editor = iced::widget::text_editor::Content::with_text("Context");
    let content_editor = iced::widget::text_editor::Content::with_text("Text content");
    let _title = super::render_node_title(&text);
    let _context = super::render_context(&text, &context_editor, false);
    let _text_content = super::render_text_content(&text, &content_editor);

    let rect = element("rect");
    let _rect_title = super::render_node_title(&rect);
    let _rect_context = super::render_context(&rect, &context_editor, true);
}
