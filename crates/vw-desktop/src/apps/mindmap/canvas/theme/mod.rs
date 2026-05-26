//! 思维导图主题模块入口，提供主题解析、颜色计算和自定义主题结构。

mod custom;
mod groups;
mod types;
mod variants;

#[cfg(test)]
#[path = "custom_tests.rs"]
mod custom_tests;
#[cfg(test)]
#[path = "groups_tests.rs"]
mod groups_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
#[cfg(test)]
#[path = "variants_tests.rs"]
mod variants_tests;

pub use custom::{MindMapCustomTheme, default_custom_themes};
pub use groups::{
    CUSTOM_THEME_GROUP_ID, CUSTOM_THEME_GROUP_NAME, THEME_GROUPS, get_theme, resolve_theme,
    theme_group_variant_count,
};
pub use types::{MindMapTheme, MindMapThemeGroup, MindMapThemeView};
