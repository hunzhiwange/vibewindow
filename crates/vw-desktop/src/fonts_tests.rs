#[test]
fn module_test_anchor() {
    let stable_value = 1 + 1;
    assert_eq!(stable_value, 2);
}

#[test]
fn load_all_registers_regular_bold_and_cjk_fonts() {
    assert_eq!(super::load_all().len(), 4);
}
