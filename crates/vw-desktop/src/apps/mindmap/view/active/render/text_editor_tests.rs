use crate::app::Message;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::model::default_doc;
use crate::apps::mindmap::state::MindMapTab;
use iced::widget::{Space, container};
use iced::{Element, Length};

fn base_element<'a>() -> Element<'a, Message> {
    container(Space::new().width(Length::Fill).height(Length::Fill)).into()
}

fn base_tab() -> MindMapTab {
    MindMapTab::new("tab".to_string(), "Text".to_string(), None, default_doc())
}

#[test]
fn text_editor_overlay_adds_backdrop_when_selected_rect_is_missing() {
    let mut tab = base_tab();
    tab.selected_path = None;

    let overlay = super::text_editor::with_text_editor_overlay(&tab, base_element());
    std::hint::black_box(overlay);

    tab.selected_path = Some(vec![99]);
    let overlay = super::text_editor::with_text_editor_overlay(&tab, base_element());
    std::hint::black_box(overlay);
}

#[test]
fn text_editor_overlay_builds_root_editor_with_zoom_clamp_and_default_colors() {
    let mut tab = base_tab();
    tab.selected_path = Some(vec![]);

    for zoom in [0.1, 1.0, 10.0] {
        tab.zoom = zoom;
        let overlay = super::text_editor::with_text_editor_overlay(&tab, base_element());
        std::hint::black_box(overlay);
    }
}

#[test]
fn text_editor_overlay_builds_child_editor_with_custom_node_colors() {
    let mut doc = default_doc();
    doc.children.push(MindNode { text: "child".to_string(), children: Vec::new() });

    let mut tab = MindMapTab::new("tab".to_string(), "Text".to_string(), None, doc);
    tab.selected_path = Some(vec![0]);
    tab.node_fills.insert(vec![0], 0x111827FF);
    tab.node_text_colors.insert(vec![0], 0xFFFFFFFF);

    for zoom in [0.1, 1.0, 10.0] {
        tab.zoom = zoom;
        let overlay = super::text_editor::with_text_editor_overlay(&tab, base_element());
        std::hint::black_box(overlay);
    }
}
