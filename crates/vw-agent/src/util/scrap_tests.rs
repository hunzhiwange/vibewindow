use super::scrap::{BAR, FOO, dummy_function, random_helper};

#[test]
fn dummy_exports_are_callable() {
    dummy_function();
    let _ = random_helper();
    assert_eq!(FOO, "42");
    assert_eq!(BAR, 123);
}
