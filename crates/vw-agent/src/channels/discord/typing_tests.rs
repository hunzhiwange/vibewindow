use super::*;

#[test]
fn new_typing_handles_starts_empty() {
    let handles = new_typing_handles();
    assert!(handles.lock().is_empty());
}
