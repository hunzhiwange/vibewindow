//! 设计画布渲染入口模块。
//!
//! 该模块组织预览、形状、文本和工具函数等渲染子能力，是画布视觉输出路径的组合层。

pub mod overlay;
pub mod preview;
pub mod shapes;
pub mod svg;
pub mod text;
pub mod utils;

pub use overlay::{draw_grid, draw_hover_edit_overlay, draw_selection_box, draw_selection_overlay};
pub use preview::{draw_brush_preview_overlay, draw_eraser_overlay, draw_tool_preview_overlay};
pub use shapes::draw_shapes_tree;
pub use text::draw_texts_tree;
