//! 设计画布核心模块。
//!
//! 该模块承载节点创建、几何计算、命中测试与画布入口组织逻辑，是设计视图交互和渲染之间的核心边界。

pub mod creation;
pub mod geometry;
pub mod hit;
pub mod layout;
pub mod parse;
pub mod program;
pub mod rendering;
pub mod tailwind;
pub mod types;
pub mod utils;

pub use geometry::get_element_screen_bounds;
pub use parse::parse_font_size;
pub use program::DesignCanvas;
pub use utils::find_element_by_id;
