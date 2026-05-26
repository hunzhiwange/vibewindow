//! 思维导图主题分组定义，维护预设主题族和可选主题入口。

use super::custom::MindMapCustomTheme;
use super::types::{MindMapTheme, MindMapThemeGroup, MindMapThemeView};
use super::variants::{
    BUSINESS_VARIANTS, CHERRY_VARIANTS, CLASH_VARIANTS, CLASSIC_VARIANTS, PURPLE_VARIANTS,
    RETRO_VARIANTS, ROSE_VARIANTS, SOFT_VARIANTS, VITALITY_VARIANTS,
};

pub const CUSTOM_THEME_GROUP_ID: &str = "custom";
pub const CUSTOM_THEME_GROUP_NAME: &str = "自定义组合";

pub const THEME_GROUPS: &[MindMapThemeGroup] = &[
    MindMapThemeGroup { id: "classic", name: "经典推荐", variants: &CLASSIC_VARIANTS },
    MindMapThemeGroup { id: "retro", name: "复古彩虹", variants: &RETRO_VARIANTS },
    MindMapThemeGroup { id: "vitality", name: "活力幻彩", variants: &VITALITY_VARIANTS },
    MindMapThemeGroup { id: "business", name: "简约商务", variants: &BUSINESS_VARIANTS },
    MindMapThemeGroup { id: "soft", name: "柔和雅丽", variants: &SOFT_VARIANTS },
    MindMapThemeGroup { id: "rose", name: "紫红魅丽", variants: &ROSE_VARIANTS },
    MindMapThemeGroup { id: "clash", name: "活力对撞", variants: &CLASH_VARIANTS },
    MindMapThemeGroup { id: "cherry", name: "浪漫樱花", variants: &CHERRY_VARIANTS },
    MindMapThemeGroup { id: "purple", name: "紫色果韵", variants: &PURPLE_VARIANTS },
];

/// 构建或更新 get theme 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn get_theme(group_id: &str, variant: usize) -> MindMapTheme {
    let group = THEME_GROUPS.iter().find(|g| g.id == group_id).unwrap_or(&THEME_GROUPS[0]);
    let idx = if group.variants.is_empty() { 0 } else { variant % group.variants.len() };
    group.variants.get(idx).copied().unwrap_or(group.variants[0])
}

/// 构建或更新 theme group variant count 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn theme_group_variant_count(group_id: &str) -> usize {
    THEME_GROUPS.iter().find(|g| g.id == group_id).map(|g| g.variants.len()).unwrap_or(1)
}

/// 构建或更新 resolve theme 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn resolve_theme<'a>(
    group_id: &str,
    variant: usize,
    custom_themes: &'a [MindMapCustomTheme],
) -> MindMapThemeView<'a> {
    if group_id == CUSTOM_THEME_GROUP_ID {
        let idx = if custom_themes.is_empty() { 0 } else { variant % custom_themes.len() };
        if let Some(t) = custom_themes.get(idx) {
            return MindMapThemeView {
                background_color: t.background_color,
                root_fill: t.root_fill,
                root_text: t.root_text,
                branch_fills: &t.branch_fills,
                branch_text: t.branch_text,
                leaf_fill: t.leaf_fill,
                leaf_text: t.leaf_text,
                line_color: t.line_color,
                is_dark: t.is_dark,
            };
        }
    }

    let t = get_theme(group_id, variant);
    MindMapThemeView {
        background_color: t.background_color,
        root_fill: t.root_fill,
        root_text: t.root_text,
        branch_fills: t.branch_fills,
        branch_text: t.branch_text,
        leaf_fill: t.leaf_fill,
        leaf_text: t.leaf_text,
        line_color: t.line_color,
        is_dark: t.is_dark,
    }
}
