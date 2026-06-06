#[test]
fn cleaner_cancel_flag_defaults_to_false() {
    assert!(!super::CLEANER_CANCEL_FLAG.load(std::sync::atomic::Ordering::Relaxed));
}
