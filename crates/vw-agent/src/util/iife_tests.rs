use super::iife::iife;

#[test]
fn returns_closure_result() {
    assert_eq!(iife(|| 2 + 3), 5);
}
