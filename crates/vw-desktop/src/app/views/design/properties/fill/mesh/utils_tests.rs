#[test]
fn task_1186_test_module_is_wired() {}

use super::*;

#[test]
fn next_seed_is_deterministic_and_mixes_bits() {
    assert_eq!(next_seed(0), 0);
    assert_eq!(next_seed(1), 1082269761);
    assert_ne!(next_seed(0x1234_5678_9abc_def0), 0x1234_5678_9abc_def0);
}

#[test]
fn mirroring_flags_parse_none_empty_and_case_insensitive_values() {
    assert_eq!(mirroring_flags(None), (false, false));
    assert_eq!(mirroring_flags(Some("")), (false, false));
    assert_eq!(mirroring_flags(Some(" none ")), (false, false));
    assert_eq!(mirroring_flags(Some("X")), (true, false));
    assert_eq!(mirroring_flags(Some(" y ")), (false, true));
    assert_eq!(mirroring_flags(Some("XY")), (true, true));
    assert_eq!(mirroring_flags(Some("axis-y")), (true, true));
}

#[test]
fn mirroring_value_serializes_flag_pairs() {
    assert_eq!(mirroring_value(false, false), None);
    assert_eq!(mirroring_value(true, false), Some("x".to_string()));
    assert_eq!(mirroring_value(false, true), Some("y".to_string()));
    assert_eq!(mirroring_value(true, true), Some("xy".to_string()));
}
