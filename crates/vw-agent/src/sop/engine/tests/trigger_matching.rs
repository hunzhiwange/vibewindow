//! SOP 触发器匹配测试。
//!
//! 覆盖 manual、MQTT、webhook 与 cron 触发器，特别验证 MQTT 通配符和 payload
//! 条件失败关闭，避免没有数据时误触发自动化流程。

use super::fixtures::{engine_with_sops, manual_event, mqtt_event, test_sop};
use super::*;

#[test]
fn match_manual_trigger() {
    let engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal)]);
    let matches = engine.match_trigger(&manual_event());
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].name, "s1");
}

#[test]
fn no_match_for_wrong_source() {
    let engine =
        engine_with_sops(vec![test_sop("s1", SopExecutionMode::Auto, SopPriority::Normal)]);
    let event = mqtt_event("sensors/temp", "{}");
    let matches = engine.match_trigger(&event);
    assert!(matches.is_empty());
}

#[test]
fn match_mqtt_trigger_exact() {
    let sop = Sop {
        triggers: vec![SopTrigger::Mqtt { topic: "plant/pump/pressure".into(), condition: None }],
        ..test_sop("pressure-sop", SopExecutionMode::Auto, SopPriority::Critical)
    };
    let engine = engine_with_sops(vec![sop]);
    let matches = engine.match_trigger(&mqtt_event("plant/pump/pressure", "87.3"));
    assert_eq!(matches.len(), 1);
}

#[test]
fn match_mqtt_wildcard_plus() {
    let sop = Sop {
        triggers: vec![SopTrigger::Mqtt { topic: "plant/+/pressure".into(), condition: None }],
        ..test_sop("wildcard-sop", SopExecutionMode::Auto, SopPriority::Normal)
    };
    let engine = engine_with_sops(vec![sop]);
    assert_eq!(engine.match_trigger(&mqtt_event("plant/pump_3/pressure", "87")).len(), 1);
    assert!(engine.match_trigger(&mqtt_event("plant/pump_3/temperature", "50")).is_empty());
}

#[test]
fn match_mqtt_wildcard_hash() {
    let sop = Sop {
        triggers: vec![SopTrigger::Mqtt { topic: "plant/#".into(), condition: None }],
        ..test_sop("hash-sop", SopExecutionMode::Auto, SopPriority::Normal)
    };
    let engine = engine_with_sops(vec![sop]);
    assert_eq!(engine.match_trigger(&mqtt_event("plant/pump/pressure", "87")).len(), 1);
    assert_eq!(engine.match_trigger(&mqtt_event("plant/a/b/c/d", "x")).len(), 1);
}

#[test]
fn mqtt_topic_matching_edge_cases() {
    assert!(mqtt_topic_matches("a/b/c", "a/b/c"));
    assert!(!mqtt_topic_matches("a/b/c", "a/b/d"));
    assert!(!mqtt_topic_matches("a/b/c", "a/b"));
    assert!(!mqtt_topic_matches("a/b", "a/b/c"));
    assert!(mqtt_topic_matches("+/+/+", "a/b/c"));
    assert!(!mqtt_topic_matches("+/+", "a/b/c"));
    assert!(mqtt_topic_matches("#", "a/b/c"));
    assert!(mqtt_topic_matches("a/#", "a/b/c"));
    assert!(!mqtt_topic_matches("b/#", "a/b/c"));
}

#[test]
fn webhook_trigger_matches_exact_path() {
    let sop = Sop {
        triggers: vec![SopTrigger::Webhook { path: "/webhook".into() }],
        ..test_sop("webhook-sop", SopExecutionMode::Auto, SopPriority::Normal)
    };
    let engine = engine_with_sops(vec![sop]);

    let event = SopEvent {
        source: SopTriggerSource::Webhook,
        topic: Some("/webhook".into()),
        payload: None,
        timestamp: now_iso8601(),
    };
    assert_eq!(engine.match_trigger(&event).len(), 1);
}

#[test]
fn webhook_trigger_rejects_different_path() {
    let sop = Sop {
        triggers: vec![SopTrigger::Webhook { path: "/sop/deploy".into() }],
        ..test_sop("deploy-sop", SopExecutionMode::Auto, SopPriority::Normal)
    };
    let engine = engine_with_sops(vec![sop]);

    let event = SopEvent {
        source: SopTriggerSource::Webhook,
        topic: Some("/webhook".into()),
        payload: None,
        timestamp: now_iso8601(),
    };
    assert!(engine.match_trigger(&event).is_empty());

    let event = SopEvent {
        source: SopTriggerSource::Webhook,
        topic: Some("/sop/deploy".into()),
        payload: None,
        timestamp: now_iso8601(),
    };
    assert_eq!(engine.match_trigger(&event).len(), 1);
}

#[test]
fn cron_trigger_matches_only_matching_expression() {
    let sop = Sop {
        triggers: vec![SopTrigger::Cron { expression: "0 */5 * * *".into() }],
        ..test_sop("cron-sop", SopExecutionMode::Auto, SopPriority::Normal)
    };
    let engine = engine_with_sops(vec![sop]);

    let event = SopEvent {
        source: SopTriggerSource::Cron,
        topic: Some("0 */5 * * *".into()),
        payload: None,
        timestamp: now_iso8601(),
    };
    assert_eq!(engine.match_trigger(&event).len(), 1);

    let event = SopEvent {
        source: SopTriggerSource::Cron,
        topic: Some("0 */10 * * *".into()),
        payload: None,
        timestamp: now_iso8601(),
    };
    assert!(engine.match_trigger(&event).is_empty());

    let event = SopEvent {
        source: SopTriggerSource::Cron,
        topic: None,
        payload: None,
        timestamp: now_iso8601(),
    };
    assert!(engine.match_trigger(&event).is_empty());
}

#[test]
fn mqtt_condition_filters_by_payload() {
    let sop = Sop {
        triggers: vec![SopTrigger::Mqtt {
            topic: "sensors/pressure".into(),
            condition: Some("$.value > 85".into()),
        }],
        ..test_sop("cond-sop", SopExecutionMode::Auto, SopPriority::Critical)
    };
    let engine = engine_with_sops(vec![sop]);

    let matches = engine.match_trigger(&mqtt_event("sensors/pressure", r#"{"value": 90}"#));
    assert_eq!(matches.len(), 1);

    let matches = engine.match_trigger(&mqtt_event("sensors/pressure", r#"{"value": 50}"#));
    assert!(matches.is_empty());
}

#[test]
fn mqtt_no_condition_matches_any_payload() {
    let sop = Sop {
        triggers: vec![SopTrigger::Mqtt { topic: "sensors/temp".into(), condition: None }],
        ..test_sop("no-cond", SopExecutionMode::Auto, SopPriority::Normal)
    };
    let engine = engine_with_sops(vec![sop]);

    let matches = engine.match_trigger(&mqtt_event("sensors/temp", "anything"));
    assert_eq!(matches.len(), 1);
}

#[test]
fn mqtt_condition_no_payload_fails_closed() {
    let sop = Sop {
        triggers: vec![SopTrigger::Mqtt {
            topic: "sensors/temp".into(),
            condition: Some("$.value > 0".into()),
        }],
        ..test_sop("no-payload", SopExecutionMode::Auto, SopPriority::Normal)
    };
    let engine = engine_with_sops(vec![sop]);

    let event = SopEvent {
        source: SopTriggerSource::Mqtt,
        topic: Some("sensors/temp".into()),
        payload: None,
        timestamp: now_iso8601(),
    };
    // 带条件的 MQTT 触发器在缺少 payload 时不匹配，避免条件无法验证却放行。
    assert!(engine.match_trigger(&event).is_empty());
}
