//! 设计画布形状渲染模块。
//!
//! 该模块封装填充、描边、阴影和形状树遍历等绘制细节，让上层渲染流程可以按节点语义组合图形输出。

use iced::widget::canvas::Frame;

use crate::app::views::design::canvas::{rendering::utils::element_path, types::ShadowSpec};

/// 模块内部可见的 draw_shadow 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn draw_shadow(
    frame: &mut Frame,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    shadow: ShadowSpec,
    kind: &str,
) {
    let shadow_path = element_path(
        kind,
        x + shadow.offset.x,
        y + shadow.offset.y,
        w + shadow.spread * 2.0,
        h + shadow.spread * 2.0,
        radius,
    );
    frame.fill(&shadow_path, shadow.color);
}

#[cfg(test)]
#[path = "shadow_tests.rs"]
mod shadow_tests;
