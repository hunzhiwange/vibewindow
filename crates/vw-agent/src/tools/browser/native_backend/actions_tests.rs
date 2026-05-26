use super::*;

#[test]
fn action_helpers_remain_available_as_narrow_function_items() {
    let _ = find_element;
    let _ = wait_for_selector;
    let _ = click_with_recovery;
    let _ = fill_with_recovery;
    let _ = type_with_recovery;
    let _ = element_checked;
    let _ = prepare_interactable_element;
    let _ = hover_element;
}
