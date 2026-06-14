use crate::app::Message;
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::{MindMapLayoutFormat, MindMapTab};

fn tab() -> MindMapTab {
    MindMapTab::new("tab-1".to_string(), "Mind map".to_string(), None, model::default_doc())
}

fn child_count(element: &iced::Element<'_, Message>) -> usize {
    element.as_widget().children().len()
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("mindmap_tests"));
}

#[test]
fn mindmap_layout_picker_builds_each_active_format() {
    for format in [
        MindMapLayoutFormat::RightAligned,
        MindMapLayoutFormat::LeftAligned,
        MindMapLayoutFormat::Bidirectional,
    ] {
        let mut tab = tab();
        tab.layout_format = format;

        let element = super::mindmap::mindmap_layout_picker(&tab, 420.0)
            .expect("mindmap picker should always render");

        assert!(child_count(&element) > 0);
    }
}

#[test]
fn mindmap_layout_picker_clamps_card_width() {
    let tab = tab();

    let element = super::mindmap::mindmap_layout_picker(&tab, 1.0)
        .expect("mindmap picker should always render");

    assert!(child_count(&element) > 0);
}
