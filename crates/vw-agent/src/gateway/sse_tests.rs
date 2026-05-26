use super::*;
use crate::app::agent::observability::{Observer, ObserverEvent};
use crate::observability::traits::ObserverMetric;

struct TestObserver;

impl Observer for TestObserver {
    fn record_event(&self, _event: &ObserverEvent) {}

    fn record_metric(&self, _metric: &ObserverMetric) {}

    fn name(&self) -> &str {
        "test"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn broadcast_observer_has_stable_name() {
    let (sender, _receiver) = tokio::sync::broadcast::channel(4);
    let observer = BroadcastObserver::new(Box::new(TestObserver), sender);

    assert_eq!(observer.name(), "gateway_broadcast");
}
