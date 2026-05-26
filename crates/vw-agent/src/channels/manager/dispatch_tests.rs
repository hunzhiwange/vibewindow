use super::*;

#[test]
fn compute_max_in_flight_messages_clamps_to_configured_bounds() {
    assert_eq!(compute_max_in_flight_messages(0), CHANNEL_MIN_IN_FLIGHT_MESSAGES);
    assert_eq!(compute_max_in_flight_messages(usize::MAX), CHANNEL_MAX_IN_FLIGHT_MESSAGES);
}
