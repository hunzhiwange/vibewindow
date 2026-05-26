use std::sync::{Arc, Mutex};

use serde_json::json;

use super::{define, publish, publish_value, subscribe, subscribe_all};

#[test]
fn publish_notifies_typed_and_wildcard_subscribers() {
    let def = define("test.plan6.bus.event");
    let typed_seen = Arc::new(Mutex::new(Vec::new()));
    let wildcard_seen = Arc::new(Mutex::new(Vec::new()));

    let typed_slot = Arc::clone(&typed_seen);
    let typed_unsub = subscribe(def, move |payload| {
        typed_slot.lock().unwrap().push(payload);
    });
    let wildcard_slot = Arc::clone(&wildcard_seen);
    let wildcard_unsub = subscribe_all(move |payload| {
        wildcard_slot.lock().unwrap().push(payload);
    });

    publish(def, json!({"value": 7}), Some("/workspace".to_string())).unwrap();

    assert_eq!(typed_seen.lock().unwrap()[0]["properties"]["value"], 7);
    assert!(
        wildcard_seen
            .lock()
            .unwrap()
            .iter()
            .any(|payload| payload["type"] == "test.plan6.bus.event")
    );

    typed_unsub();
    wildcard_unsub();
}

#[test]
fn unsubscribe_stops_future_events() {
    let def = define("test.plan6.bus.unsubscribe");
    let seen = Arc::new(Mutex::new(0usize));
    let slot = Arc::clone(&seen);
    let unsubscribe = subscribe(def, move |_| {
        *slot.lock().unwrap() += 1;
    });

    publish_value(def, json!({}), None);
    unsubscribe();
    publish_value(def, json!({}), None);

    assert_eq!(*seen.lock().unwrap(), 1);
}
