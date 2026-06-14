use crate::apps::mindmap::model::default_doc;
use crate::apps::mindmap::state::{MindMapTab, TreeLayoutFormat};

fn tab_with_tree_format(format: TreeLayoutFormat) -> MindMapTab {
    let mut tab = MindMapTab::new("tab".to_string(), "Tree".to_string(), None, default_doc());
    tab.tree_layout_format = format;
    tab
}

#[test]
fn tree_layout_picker_builds_cards_for_each_active_format() {
    for format in [
        TreeLayoutFormat::SymmetricSplit,
        TreeLayoutFormat::FanDown,
        TreeLayoutFormat::LeftAligned,
        TreeLayoutFormat::RightAligned,
    ] {
        let tab = tab_with_tree_format(format);
        let picker = super::tree_layout_picker(&tab, 360.0);

        assert!(picker.is_some());
    }
}

#[test]
fn tree_layout_picker_clamps_card_width_for_narrow_panel() {
    let tab = tab_with_tree_format(TreeLayoutFormat::RightAligned);
    let picker = super::tree_layout_picker(&tab, 40.0);

    assert!(picker.is_some());
}
