//! # 覆盖层模块
//!
//! 本模块把设计视图中的各种覆盖层实现拆分到独立文件，
//! 对外继续保持原有函数接口不变。

#[path = "overlay_basic_pickers.rs"]
mod basic_pickers;
#[path = "overlay_context.rs"]
mod context;
#[path = "overlay_font_and_tailwind.rs"]
mod font_and_tailwind;
#[path = "overlay_icon.rs"]
mod icon;
#[path = "overlay_shared.rs"]
mod shared;
#[path = "overlay_text.rs"]
mod text;

pub use basic_pickers::{color_picker_layers, effect_picker_layers, fill_picker_layers};
pub use context::{canvas_context_menu_layers, context_toolbar_layers};
pub use font_and_tailwind::{font_picker_layers, tailwind_class_picker_layers};
pub use icon::icon_picker_layers;
pub use text::{html_preview_layers, inline_text_editor_overlay};
