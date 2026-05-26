//! 设计画布渲染工具模块。
//!
//! 该模块提供路径、图片、文本和 Tailwind 样式转换等底层辅助函数，减少渲染主流程中的重复样板逻辑。

use iced::widget::image::Handle;
use iced::{
    Rectangle,
    widget::canvas::{Frame, Image},
};
use std::collections::HashMap;

/// 公开的 draw_image_from_cache 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn draw_image_from_cache(
    frame: &mut Frame,
    rect: Rectangle,
    images: &HashMap<String, Handle>,
    src: &str,
) -> bool {
    if let Some(handle) = images.get(src) {
        frame.draw_image(rect, Image::new(handle.clone()));
        true
    } else {
        false
    }
}

#[cfg(test)]
#[path = "image_tests.rs"]
mod image_tests;
