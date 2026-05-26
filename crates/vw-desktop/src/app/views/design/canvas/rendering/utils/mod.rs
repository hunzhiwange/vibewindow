//! 设计画布渲染工具模块。
//!
//! 该模块提供路径、图片、文本和 Tailwind 样式转换等底层辅助函数，减少渲染主流程中的重复样板逻辑。

mod image;
mod path;
mod tailwind;
mod text;

pub use image::draw_image_from_cache;
pub use path::{element_path, element_path_radius};
pub use tailwind::{draw_tailwind_box, draw_tailwind_outline};
pub use text::{
    apply_text_transform, compute_line_width, draw_text_decoration, wrap_text_words,
};