//! 设计画布形状渲染模块。
//!
//! 该模块封装填充、描边、阴影和形状树遍历等绘制细节，让上层渲染流程可以按节点语义组合图形输出。

mod fills;
mod helpers;
mod shadow;
mod stroke;
mod tree;

pub use tree::draw_shapes_tree;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
