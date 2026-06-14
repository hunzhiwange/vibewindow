use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    fn make_channel() -> WatiChannel {
        WatiChannel {
            api_token: "test-token".into(),
            api_url: "https://live-mt-server.wati.io".into(),
            tenant_id: None,
            allowed_numbers: vec!["+1234567890".into()],
            client: reqwest::Client::new(),
        }
    }

    fn make_wildcard_channel() -> WatiChannel {
        WatiChannel {
            api_token: "test-token".into(),
            api_url: "https://live-mt-server.wati.io".into(),
            tenant_id: None,
            allowed_numbers: vec!["*".into()],
            client: reqwest::Client::new(),
        }
    }

    #[test]
    fn wati_channel_name() {
        let ch = make_channel();
        assert_eq!(ch.name(), "wati");
    }

    #[test]
    fn wati_number_allowed_exact() {
        let ch = make_channel();
        assert!(ch.is_number_allowed("+1234567890"));
        assert!(!ch.is_number_allowed("+9876543210"));
    }

    #[test]
    fn wati_number_allowed_wildcard() {
        let ch = make_wildcard_channel();
        assert!(ch.is_number_allowed("+1234567890"));
        assert!(ch.is_number_allowed("+9999999999"));
    }

    #[test]
    fn wati_number_allowed_empty() {
        let ch = WatiChannel {
            api_token: "tok".into(),
            api_url: "https://live-mt-server.wati.io".into(),
            tenant_id: None,
            allowed_numbers: vec![],
            client: reqwest::Client::new(),
        };
        assert!(!ch.is_number_allowed("+1234567890"));
    }

    #[test]
    fn wati_build_target_with_tenant() {
        let ch = WatiChannel {
            api_token: "tok".into(),
            api_url: "https://live-mt-server.wati.io".into(),
            tenant_id: Some("tenant1".into()),
            allowed_numbers: vec![],
            client: reqwest::Client::new(),
        };
        assert_eq!(ch.build_target("+1234567890"), "tenant1:1234567890");
    }

    #[test]
    fn wati_build_target_without_tenant() {
        let ch = make_channel();
        assert_eq!(ch.build_target("+1234567890"), "1234567890");
    }

    #[test]
    fn wati_build_target_already_prefixed() {
        let ch = WatiChannel {
            api_token: "tok".into(),
            api_url: "https://live-mt-server.wati.io".into(),
            tenant_id: Some("tenant1".into()),
            allowed_numbers: vec![],
            client: reqwest::Client::new(),
        };
        // If the phone already has the tenant prefix, don't double it
        assert_eq!(ch.build_target("tenant1:1234567890"), "tenant1:1234567890");
    }

    #[test]
    fn wati_build_target_keeps_bare_number_without_plus() {
        let ch = make_channel();
        assert_eq!(ch.build_target("1234567890"), "1234567890");
    }

    #[test]
    fn wati_parse_valid_message() {
        let ch = make_channel();
        let payload = serde_json::json!({
            "text": "Hello from WATI!",
            "waId": "1234567890",
            "fromMe": false,
            "timestamp": 1_705_320_000_u64
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].sender, "+1234567890");
        assert_eq!(msgs[0].content, "Hello from WATI!");
        assert_eq!(msgs[0].channel, "wati");
        assert_eq!(msgs[0].reply_target, "+1234567890");
        assert_eq!(msgs[0].timestamp, 1_705_320_000);
    }

    #[test]
    fn wati_parse_skip_from_me() {
        let ch = make_wildcard_channel();
        let payload = serde_json::json!({
            "text": "My own message",
            "waId": "1234567890",
            "fromMe": true
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert!(msgs.is_empty(), "fromMe messages should be skipped");
    }

    #[test]
    fn wati_parse_skip_no_text() {
        let ch = make_wildcard_channel();
        let payload = serde_json::json!({
            "waId": "1234567890",
            "fromMe": false
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert!(msgs.is_empty(), "Messages without text should be skipped");
    }

    #[test]
    fn wati_parse_skip_blank_text_and_missing_sender() {
        let ch = make_wildcard_channel();
        let blank = serde_json::json!({
            "text": "   ",
            "waId": "1234567890",
            "fromMe": false
        });
        assert!(ch.parse_webhook_payload(&blank).is_empty());

        let missing_sender = serde_json::json!({
            "text": "hello",
            "fromMe": false
        });
        assert!(ch.parse_webhook_payload(&missing_sender).is_empty());
    }

    #[test]
    fn wati_parse_alternative_field_names() {
        let ch = make_wildcard_channel();

        // wa_id instead of waId, message.body instead of text
        let payload = serde_json::json!({
            "message": { "body": "Alt field test" },
            "wa_id": "1234567890",
            "from_me": false,
            "timestamp": 1_705_320_000_u64
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "Alt field test");
        assert_eq!(msgs[0].sender, "+1234567890");
    }

    #[test]
    fn wati_parse_timestamp_seconds() {
        let ch = make_wildcard_channel();
        let payload = serde_json::json!({
            "text": "Test",
            "waId": "1234567890",
            "timestamp": 1_705_320_000_u64
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert_eq!(msgs[0].timestamp, 1_705_320_000);
    }

    #[test]
    fn wati_parse_timestamp_milliseconds() {
        let ch = make_wildcard_channel();
        let payload = serde_json::json!({
            "text": "Test",
            "waId": "1234567890",
            "timestamp": 1_705_320_000_000_u64
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert_eq!(msgs[0].timestamp, 1_705_320_000);
    }

    #[test]
    fn wati_parse_timestamp_iso() {
        let ch = make_wildcard_channel();
        let payload = serde_json::json!({
            "text": "Test",
            "waId": "1234567890",
            "timestamp": "2025-01-15T12:00:00Z"
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert_eq!(msgs[0].timestamp, 1_736_942_400);
    }

    #[test]
    fn wati_parse_created_timestamp_fallbacks() {
        let ch = make_wildcard_channel();
        let payload = serde_json::json!({
            "text": "Created timestamp",
            "waId": "1234567890",
            "created": 1_705_320_000_000_u64
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].timestamp, 1_705_320_000);
    }

    #[test]
    fn wati_parse_normalizes_phone() {
        let ch = WatiChannel {
            api_token: "tok".into(),
            api_url: "https://live-mt-server.wati.io".into(),
            tenant_id: None,
            allowed_numbers: vec!["+1234567890".into()],
            client: reqwest::Client::new(),
        };

        // Phone without + prefix
        let payload = serde_json::json!({
            "text": "Hi",
            "waId": "1234567890",
            "fromMe": false
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].sender, "+1234567890");
    }

    #[test]
    fn wati_parse_empty_payload() {
        let ch = make_channel();
        let payload = serde_json::json!({});
        let msgs = ch.parse_webhook_payload(&payload);
        assert!(msgs.is_empty());
    }

    #[test]
    fn wati_parse_from_field_fallback() {
        let ch = make_wildcard_channel();
        // Uses "from" instead of "waId"
        let payload = serde_json::json!({
            "text": "Fallback test",
            "from": "1234567890",
            "fromMe": false
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].sender, "+1234567890");
    }

    #[test]
    fn wati_parse_message_text_fallback() {
        let ch = make_wildcard_channel();
        // Uses "message.text" instead of top-level "text"
        let payload = serde_json::json!({
            "message": { "text": "Nested text" },
            "waId": "1234567890",
            "fromMe": false
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "Nested text");
    }

    #[test]
    fn wati_parse_owner_field_as_from_me() {
        let ch = make_wildcard_channel();
        // Uses "owner" field as fromMe indicator
        let payload = serde_json::json!({
            "text": "Test",
            "waId": "1234567890",
            "owner": true
        });

        let msgs = ch.parse_webhook_payload(&payload);
        assert!(msgs.is_empty(), "owner=true messages should be skipped");
    }
}
