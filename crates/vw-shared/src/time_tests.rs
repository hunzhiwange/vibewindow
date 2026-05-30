#[test]
fn now_ms_is_close_to_system_time() {
    let before =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
            as u64;
    let actual = super::now_ms();
    let after =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
            as u64;

    assert!(actual >= before);
    assert!(actual <= after);
}

#[test]
fn now_returns_system_time_after_unix_epoch() {
    assert!(super::now().duration_since(std::time::UNIX_EPOCH).is_ok());
}
