//! 设计画布布局模块。
//!
//! 该模块负责解析和计算画布节点布局，帮助渲染层获得稳定的几何信息。

pub mod calc;
pub mod parse;

pub use calc::{compute_layout, resolve_element_size};
pub use parse::{parse_layout, parse_padding};
