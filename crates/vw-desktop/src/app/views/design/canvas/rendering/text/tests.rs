//! 设计画布文本渲染模块。
//!
//! 该模块处理文本节点的排版、网格、树结构或便签绘制逻辑，确保 DOM 风格输入能够稳定映射到画布中的可见文本。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    #[test]
    fn clamp_child_size_to_content_limits_overflowing_size() {
        let clipped = clamp_child_size_to_content(
            iced::Size::new(204.0, 32.0),
            12.0,
            5.0,
            iced::Size::new(392.0, 22.0),
        );
        assert_eq!(clipped.width, 192.0);
        assert_eq!(clipped.height, 22.0);
    }

    #[test]
    fn clamp_child_size_to_content_returns_zero_when_child_starts_outside() {
        let clipped = clamp_child_size_to_content(
            iced::Size::new(204.0, 32.0),
            240.0,
            40.0,
            iced::Size::new(30.0, 20.0),
        );
        assert_eq!(clipped.width, 0.0);
        assert_eq!(clipped.height, 0.0);
    }
}
