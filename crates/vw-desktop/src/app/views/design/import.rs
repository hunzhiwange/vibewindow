//! 设计导入入口模块，负责把外部 HTML 或结构化数据转换为内部设计元素。

mod figma;
mod figma_geometry;
mod figma_node;
mod figma_style;
mod figma_support;
mod html;
mod shared;

pub use figma::{
    FigmaImportProgress, count_figma_pages, figma_json_to_design_doc_with_progress,
    figma_to_design_doc, figma_to_design_doc_with_base_dir,
    figma_to_design_doc_with_base_dir_and_progress, figma_to_design_doc_with_elements_progress,
    figma_to_design_doc_with_progress, figma_to_elements,
};
pub use html::{import_html_as_elements, import_html_as_positioned_elements};

#[cfg(test)]
#[path = "import_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "import_tailwind_fixture_tests.rs"]
mod tailwind_fixture_tests;

#[cfg(test)]
#[path = "import_tailwind_layer_conversion_tests.rs"]
mod tailwind_layer_conversion_tests;
