//! Tailwind CSS 解析器测试模块。
//!
//! 测试按能力族拆成独立文件，覆盖 spacing/size、layout、color/border、variant/unsupported
//! 四类主要路径，避免后续 parser 调整只能靠手工回归。

pub(super) use super::{
    TailwindParseAnalysis,
    TailwindParser,
    TailwindTokenIssue,
    TailwindTokenSupport,
};
pub(super) use super::super::TailwindColors;

#[path = "tests/color_border.rs"]
mod color_border;
#[path = "tests/layout.rs"]
mod layout;
#[path = "tests/spacing_size.rs"]
mod spacing_size;
#[path = "tests/variant_unsupported.rs"]
mod variant_unsupported;
