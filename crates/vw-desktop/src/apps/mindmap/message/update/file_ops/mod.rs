//! 思维导图文件操作更新入口，组织导入、导出和保存相关消息处理。

mod export_ops;
mod json_format;
mod open_save;
mod tab_restore;

#[cfg(test)]
#[path = "export_ops_tests.rs"]
mod export_ops_tests;
#[cfg(test)]
#[path = "json_format_tests.rs"]
mod json_format_tests;
#[cfg(test)]
#[path = "open_save_tests.rs"]
mod open_save_tests;
#[cfg(test)]
#[path = "tab_restore_tests.rs"]
mod tab_restore_tests;
#[cfg(test)]
mod tests;

pub(crate) use export_ops::{export_finished, export_jpeg, export_png, export_svg};
pub(crate) use open_save::{
    file_opened, file_saved, new_tab, open, save, save_as, save_as_json, save_finished,
};
