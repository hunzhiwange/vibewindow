use super::*;

#[test]
fn tracker_counts_recorded_actions() {
    let tracker = ActionTracker::new();
    tracker.record();
    tracker.record();
    assert_eq!(tracker.count(), 2);
}

