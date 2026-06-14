use iced::Element;

use super::header::render;
use super::*;
use crate::apps::mindmap::state::{MindMapDiagramType, MindMapTab};

fn keep_element(element: Element<'_, Message>) {
    let _ = std::hint::black_box(element);
}

fn tab_with_state(
    title: &str,
    diagram_type: MindMapDiagramType,
    show_type_picker: bool,
    show_markdown_import: bool,
) -> MindMapTab {
    let mut tab = MindMapTab::new(
        "tab-1".to_string(),
        title.to_string(),
        None,
        crate::apps::mindmap::model::default_doc(),
    );
    tab.zoom = 1.25;
    tab.diagram_type = diagram_type;
    tab.show_diagram_type_picker = show_type_picker;
    tab.show_markdown_import = show_markdown_import;
    tab.markdown_import_editor = iced::widget::text_editor::Content::with_text("- one\n- two");
    tab
}

#[test]
fn render_builds_default_header_without_active_tab() {
    keep_element(render(None));
}

#[test]
fn render_builds_header_with_closed_overlays() {
    let tab = tab_with_state("产品路线图", MindMapDiagramType::Timeline, false, false);

    keep_element(render(Some(&tab)));
}

#[test]
fn render_builds_header_with_diagram_type_picker() {
    let tab = tab_with_state("组织结构", MindMapDiagramType::OrgChart, true, false);

    keep_element(render(Some(&tab)));
}

#[test]
fn render_builds_header_with_markdown_import_overlay() {
    let tab = tab_with_state("导入大纲", MindMapDiagramType::MindMap, false, true);

    keep_element(render(Some(&tab)));
}
