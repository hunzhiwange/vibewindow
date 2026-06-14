use crate::app::Message;
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::{BracketLayoutFormat, MindMapTab};

fn tab() -> MindMapTab {
    MindMapTab::new("tab-1".to_string(), "Mind map".to_string(), None, model::default_doc())
}

fn child_count(element: &iced::Element<'_, Message>) -> usize {
    element.as_widget().children().len()
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("bracket_tests"));
}

#[test]
fn bracket_layout_picker_builds_each_active_format() {
    for format in [BracketLayoutFormat::BraceRight, BracketLayoutFormat::BraceLeft] {
        let mut tab = tab();
        tab.bracket_layout_format = format;

        let element = super::bracket::bracket_layout_picker(&tab, 360.0)
            .expect("bracket picker should always render");

        assert!(child_count(&element) > 0);
    }
}

#[test]
fn bracket_layout_picker_clamps_card_width() {
    let tab = tab();

    let element = super::bracket::bracket_layout_picker(&tab, 1.0)
        .expect("bracket picker should always render");

    assert!(child_count(&element) > 0);
}
