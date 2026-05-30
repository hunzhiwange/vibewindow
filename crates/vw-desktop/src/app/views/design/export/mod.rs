//! 设计导出模块，负责把内部设计文档转换为 HTML、SVG 或共享的 CSS/尺寸表示。

mod html;
mod svg;
mod util;

pub use html::{generate_element_html, generate_html};
pub use svg::generate_element_svg;
