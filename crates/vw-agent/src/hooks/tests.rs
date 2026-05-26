use super::*;

#[test]
fn public_hook_types_are_exported() {
    fn assert_send<T: Send>() {}
    assert_send::<HookRunner>();
}
