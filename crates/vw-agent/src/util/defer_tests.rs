use super::defer::{Defer, defer};
use std::cell::Cell;

#[test]
fn runs_deferred_closure_on_drop() {
    let called = Cell::new(false);
    {
        let _guard = defer(|| called.set(true));
    }

    assert!(called.get());
}

#[test]
fn disarm_prevents_drop_callback() {
    let called = Cell::new(false);
    Defer::new(|| called.set(true)).disarm();

    assert!(!called.get());
}
