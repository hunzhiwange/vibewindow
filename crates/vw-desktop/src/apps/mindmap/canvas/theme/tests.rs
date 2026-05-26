use super::{THEME_GROUPS, get_theme};

#[test]
fn theme_module_exports_resolvable_groups() {
    for group in THEME_GROUPS {
        assert_eq!(get_theme(group.id, 0).id, group.variants[0].id);
    }
}
