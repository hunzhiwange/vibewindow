use super::*;

use serde_json::{Map, Value};
use std::thread;

#[test]
fn global_logger_initializes_and_accepts_llm_tags() {
    let mut extra = Map::new();
    extra.insert("requestID".to_string(), Value::String("logging-test".to_string()));

    LOGGER
        .clone_logger()
        .tag("providerID", "test-provider")
        .tag("modelID", "test-model")
        .info("llm logging test", Some(extra));
}

#[test]
fn cloned_loggers_can_be_used_from_multiple_threads() {
    let handles = (0..4)
        .map(|idx| {
            thread::spawn(move || {
                LOGGER
                    .clone_logger()
                    .tag("worker", &idx.to_string())
                    .info("llm logging thread test", None);
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.join().expect("logger thread should not panic");
    }
}
