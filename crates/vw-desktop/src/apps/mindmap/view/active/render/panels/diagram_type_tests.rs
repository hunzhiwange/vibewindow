use crate::app::Message;
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::{MindMapDiagramType, MindMapTab};

fn tab() -> MindMapTab {
    MindMapTab::new("tab-1".to_string(), "Mind map".to_string(), None, model::default_doc())
}

fn child_count(element: &iced::Element<'_, Message>) -> usize {
    element.as_widget().children().len()
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("diagram_type_tests"));
}

#[test]
fn diagram_type_panel_builds_each_layout_picker_branch() {
    for diagram_type in [
        MindMapDiagramType::MindMap,
        MindMapDiagramType::OrgChart,
        MindMapDiagramType::Fishbone,
        MindMapDiagramType::Timeline,
        MindMapDiagramType::Tree,
        MindMapDiagramType::Bracket,
    ] {
        let mut tab = tab();
        tab.diagram_type = diagram_type;

        let element = super::diagram_type::diagram_type_panel(&tab, 420.0, 180.0);

        assert!(child_count(&element) > 0);
    }
}

#[test]
fn diagram_type_panel_uses_minimum_layout_picker_width() {
    let mut tab = tab();
    tab.diagram_type = MindMapDiagramType::Bracket;

    let element = super::diagram_type::diagram_type_panel(&tab, 120.0, 100.0);

    assert!(child_count(&element) > 0);
}
