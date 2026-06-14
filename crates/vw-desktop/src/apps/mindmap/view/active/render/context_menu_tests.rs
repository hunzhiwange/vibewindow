#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("context_menu_tests"));
}

use super::context_menu::context_menu_overlay;

#[test]
fn context_menu_overlay_builds_with_all_actions_disabled() {
    let element = context_menu_overlay(false, false, false, false);

    std::hint::black_box(element);
}

#[test]
fn context_menu_overlay_builds_with_all_actions_enabled() {
    let element = context_menu_overlay(true, true, true, true);

    std::hint::black_box(element);
}

#[test]
fn context_menu_overlay_builds_mixed_action_states() {
    let element = context_menu_overlay(true, false, true, false);

    std::hint::black_box(element);
}
