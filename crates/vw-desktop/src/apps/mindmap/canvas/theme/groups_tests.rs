use super::custom::MindMapCustomTheme;
use super::groups::{
    CUSTOM_THEME_GROUP_ID, THEME_GROUPS, get_theme, resolve_theme, theme_group_variant_count,
};

fn custom_theme() -> MindMapCustomTheme {
    MindMapCustomTheme {
        background_color: 0x010203FF,
        root_fill: 0x111111FF,
        root_text: 0xFFFFFFFF,
        branch_fills: vec![0x222222FF, 0x333333FF],
        branch_text: 0xEEEEEEFF,
        leaf_fill: 0x444444FF,
        leaf_text: 0xDDDDDDFF,
        line_color: Some(0x555555FF),
        is_dark: true,
    }
}

#[test]
fn get_theme_falls_back_to_classic_group() {
    assert_eq!(get_theme("missing", 0), get_theme("classic", 0));
}

#[test]
fn get_theme_wraps_variant_index_within_group() {
    let count = theme_group_variant_count("classic");

    assert_eq!(get_theme("classic", count + 2), get_theme("classic", 2));
}

#[test]
fn theme_group_variant_count_returns_known_count_or_one() {
    assert_eq!(theme_group_variant_count("classic"), 8);
    assert_eq!(theme_group_variant_count("missing"), 1);
}

#[test]
fn resolve_theme_uses_custom_theme_when_available() {
    let themes = vec![custom_theme()];
    let resolved = resolve_theme(CUSTOM_THEME_GROUP_ID, 3, &themes);

    assert_eq!(resolved.background_color, 0x010203FF);
    assert_eq!(resolved.palette(1), 0x333333FF);
    assert!(resolved.is_dark);
}

#[test]
fn resolve_theme_wraps_custom_theme_index() {
    let themes =
        vec![custom_theme(), MindMapCustomTheme { background_color: 0xABCDEF12, ..custom_theme() }];
    let resolved = resolve_theme(CUSTOM_THEME_GROUP_ID, 3, &themes);

    assert_eq!(resolved.background_color, 0xABCDEF12);
}

#[test]
fn resolve_theme_falls_back_to_preset_when_custom_list_is_empty() {
    let resolved = resolve_theme(CUSTOM_THEME_GROUP_ID, 0, &[]);
    let classic = get_theme("classic", 0);

    assert_eq!(resolved.background_color, classic.background_color);
    assert_eq!(resolved.root_fill, classic.root_fill);
}

#[test]
fn theme_groups_expose_non_empty_variant_sets() {
    assert!(THEME_GROUPS.iter().all(|group| !group.id.is_empty()));
    assert!(THEME_GROUPS.iter().all(|group| !group.name.is_empty()));
    assert!(THEME_GROUPS.iter().all(|group| !group.variants.is_empty()));
}
