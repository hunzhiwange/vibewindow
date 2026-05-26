//! 汇总清理工具的消息处理入口和公开类型。
//! 本模块为清理工具子模块提供稳定导出，调用侧无需感知内部拆分。

mod fs;
mod run;
mod scan;
mod state;
mod types;

pub use state::{selected_scan_totals, update};
pub use types::{
    CleanerPlatform, CleanerScanDetail, CleanerScanGroup, CleanerScanItem, CleanerScanReport,
    CleanerToolMessage, current_platform, format_bytes,
};
#[cfg(test)]
#[path = "cleaner_tool_tests.rs"]
mod cleaner_tool_tests;
