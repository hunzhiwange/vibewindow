//! Git 面板 UI 组件模块
//!
//! 本模块保留原有公开按钮构造函数作为稳定入口，
//! 具体实现按职责拆分到独立文件中，避免单文件继续膨胀。

mod disabled_buttons;
mod glyph_buttons;
mod icon_buttons;
mod shared;

#[cfg(test)]
mod disabled_buttons_tests;
#[cfg(test)]
mod glyph_buttons_tests;
#[cfg(test)]
mod icon_buttons_tests;
#[cfg(test)]
mod shared_tests;

pub use disabled_buttons::{disabled_square_content_button_tiny, disabled_square_icon_button_tiny};
pub use glyph_buttons::header_plain_glyph_button;
pub use icon_buttons::{
    small_plain_icon_button,
    square_icon_button_micro,
    square_icon_button_tiny,
};
