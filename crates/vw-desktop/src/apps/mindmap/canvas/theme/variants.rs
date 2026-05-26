//! 思维导图主题变体定义模块
//!
//! 本模块仅负责聚合各主题分组的静态变体定义，保持外部常量名和调用方式不变。

use super::MindMapTheme;

mod classic_family;
mod expressive_family;
mod fruit_family;
mod professional_family;

#[cfg(test)]
mod classic_family_tests;
#[cfg(test)]
mod expressive_family_tests;
#[cfg(test)]
mod fruit_family_tests;
#[cfg(test)]
mod professional_family_tests;

pub(crate) use classic_family::{CLASSIC_VARIANTS, RETRO_VARIANTS, VITALITY_VARIANTS};
pub(crate) use expressive_family::{CLASH_VARIANTS, ROSE_VARIANTS};
pub(crate) use fruit_family::{CHERRY_VARIANTS, PURPLE_VARIANTS};
pub(crate) use professional_family::{BUSINESS_VARIANTS, SOFT_VARIANTS};
