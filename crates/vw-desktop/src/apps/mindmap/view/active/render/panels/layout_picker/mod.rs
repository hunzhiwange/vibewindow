//! 思维导图布局选择面板入口，组织不同图形格式的选择控件。

mod bracket;
mod fishbone;
mod mindmap;
mod org_chart;
mod timeline;
mod tree;

#[cfg(test)]
#[path = "bracket_tests.rs"]
mod bracket_tests;
#[cfg(test)]
#[path = "fishbone_tests.rs"]
mod fishbone_tests;
#[cfg(test)]
#[path = "mindmap_tests.rs"]
mod mindmap_tests;
#[cfg(test)]
#[path = "org_chart_tests.rs"]
mod org_chart_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[path = "timeline_tests.rs"]
mod timeline_tests;
#[cfg(test)]
#[path = "tree_tests.rs"]
mod tree_tests;

pub(super) use bracket::bracket_layout_picker;
pub(super) use fishbone::fishbone_layout_picker;
pub(super) use mindmap::mindmap_layout_picker;
pub(super) use org_chart::org_chart_layout_picker;
pub(super) use timeline::timeline_layout_picker;
pub(super) use tree::tree_layout_picker;
