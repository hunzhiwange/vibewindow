//! 思维导图样式与布局预览组件。
//!
//! 本模块保持原有 `previews` 对外接口不变，仅将不同预览器按职责拆分到独立文件中。

mod bracket_layout;
mod edge_styles;
mod fishbone_layout;
mod mindmap_layout;
mod org_chart_layout;
mod timeline_layout;
mod tree_layout;

#[cfg(test)]
mod bracket_layout_tests;
#[cfg(test)]
mod edge_styles_tests;
#[cfg(test)]
mod fishbone_layout_tests;
#[cfg(test)]
mod mindmap_layout_tests;
#[cfg(test)]
mod org_chart_layout_tests;
#[cfg(test)]
#[path = "previews_tests.rs"]
mod previews_tests;
#[cfg(test)]
mod timeline_layout_tests;
#[cfg(test)]
mod tree_layout_tests;

pub(crate) use bracket_layout::BracketLayoutFormatPreview;
pub(crate) use edge_styles::{BorderStylePreview, LineStylePreview};
pub(crate) use fishbone_layout::FishboneLayoutFormatPreview;
pub(crate) use mindmap_layout::LayoutFormatPreview;
pub(crate) use org_chart_layout::OrgChartLayoutFormatPreview;
pub(crate) use timeline_layout::TimelineLayoutFormatPreview;
pub(crate) use tree_layout::TreeLayoutFormatPreview;
