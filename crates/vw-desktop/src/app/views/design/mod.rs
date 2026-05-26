//! 设计视图模块入口，组织画布、导入导出、属性面板和设计工具的子模块。

pub mod canvas;
pub mod export;
pub mod image_import;
pub mod import;
pub mod layers;
pub mod models;
pub mod properties;
pub mod settings;
pub mod state;
pub mod sticky_note_create;
pub mod toolbar;
pub mod utils;
pub mod variables;

mod view;

pub use view::view;
