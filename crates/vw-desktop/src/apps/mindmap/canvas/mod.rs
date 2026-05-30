//! # 思维导图画布模块
//!
//! 本模块提供了思维导图的渲染、布局计算和导出功能。
//!
//! ## 核心功能
//!
//! - **布局计算** (`layout`): 负责计算节点和边的位置
//! - **画布程序** (`program`): 实现画布的交互逻辑和状态管理
//! - **样式系统** (`style`): 提供节点、边、文本等的样式计算
//! - **主题系统** (`theme`): 支持多种视觉主题和自定义主题
//! - **变换工具** (`transform`): 处理视图的缩放、平移等变换
//!
//! ## 主要导出类型
//!
//! - `MindMapCanvas`: 画布组件，负责渲染思维导图
//! - `MindMapCanvasState`: 画布状态，管理视图和交互状态
//! - `Layout`: 布局数据结构，包含节点和边的位置信息
//! - `NodeLayout` / `EdgeLayout`: 单个节点/边的布局信息

mod export;
pub(crate) mod layout;
mod program;
mod rasterize;
mod style;
pub mod theme;
mod transform;

#[cfg(test)]
#[path = "rasterize_tests.rs"]
mod rasterize_tests;
#[cfg(test)]
#[path = "style_tests.rs"]
mod style_tests;
#[cfg(test)]
#[path = "transform_tests.rs"]
mod transform_tests;

pub(crate) use export::export_svg;
pub(crate) use layout::selected_node_rect_screen;
pub use layout::{EdgeLayout, Layout, NodeLayout};
pub use program::{MindMapCanvas, MindMapCanvasState};
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use rasterize::render_svg_to_png;
pub(crate) use style::dash_segments_px;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
