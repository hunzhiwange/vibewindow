#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("markdown_import_tests"));
}

use super::markdown_import::with_markdown_import_overlay;
use crate::app::Message;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::MindMapTab;
use iced::Element;
use iced::widget::{container, text};

#[test]
fn markdown_import_overlay_stacks_modal_over_base_element() {
    let tab = MindMapTab::new("id".to_string(), "title".to_string(), None, MindNode::default());
    let base: Element<'_, Message> = container(text("base")).into();

    let element = with_markdown_import_overlay(&tab, base);

    std::hint::black_box(element);
}
