//! 文本渲染模块。
//!
//! 对外继续暴露 `draw_texts_tree`，内部按职责拆分为文本树遍历、普通文本渲染、
//! 便签文本渲染和字形网格采样几个独立文件。

mod mesh;
mod sticky_note;
mod tree;
mod typography;
mod wrap;

pub use tree::draw_texts_tree;

#[cfg(test)]
pub(super) use tree::clamp_child_size_to_content;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
