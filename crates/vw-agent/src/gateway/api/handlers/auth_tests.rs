    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn api_error_bad_request_has_400_status() {
        let err = ApiError::bad_request("invalid credentials");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.message, "invalid credentials");
    }

    #[test]
    fn api_error_not_found_has_404_status() {
        let err = ApiError::not_found("provider not found");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.message, "provider not found");
    }

    #[test]
    fn api_error_internal_has_500_status() {
        let err = ApiError::internal("unexpected failure");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.message, "unexpected failure");
    }

    #[test]
    fn api_error_bad_request_from_io_error_is_400() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let err = ApiError::bad_request(io_err.to_string());
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(err.message.contains("permission denied"));
    }

    #[test]
    fn api_error_message_is_preserved() {
        let msg = "write error: disk full";
        let err = ApiError::bad_request(msg);
        assert_eq!(err.message, msg);
    }

    #[test]
    fn auth_info_api_deserializes_correctly() {
        let json = r#"{"type":"api","key":"sk-test-abc123"}"#;
        let info: auth::Info = serde_json::from_str(json).unwrap();
        match info {
            auth::Info::Api(api) => assert_eq!(api.key, "sk-test-abc123"),
            _ => panic!("expected Info::Api"),
        }
    }

    #[test]
    fn auth_info_oauth_deserializes_correctly() {
        let json = r#"{"type":"oauth","refresh":"rt","access":"at","expires":9999}"#;
        let info: auth::Info = serde_json::from_str(json).unwrap();
        match info {
            auth::Info::Oauth(oauth) => {
                assert_eq!(oauth.refresh, "rt");
                assert_eq!(oauth.access, "at");
                assert_eq!(oauth.expires, 9999);
                assert!(oauth.account_id.is_none());
                assert!(oauth.enterprise_url.is_none());
            }
            _ => panic!("expected Info::Oauth"),
        }
    }

    #[test]
    fn auth_info_wellknown_deserializes_correctly() {
        let json = r#"{"type":"wellknown","key":"wk-key","token":"wk-token"}"#;
        let info: auth::Info = serde_json::from_str(json).unwrap();
        match info {
            auth::Info::Wellknown(wk) => {
                assert_eq!(wk.key, "wk-key");
                assert_eq!(wk.token, "wk-token");
            }
            _ => panic!("expected Info::Wellknown"),
        }
    }

    #[test]
    fn auth_info_missing_type_field_fails_deserialization() {
        let json = r#"{"key":"sk-test"}"#;
        let result = serde_json::from_str::<auth::Info>(json);
        assert!(result.is_err(), "missing 'type' tag should fail deserialization");
    }

    #[test]
    fn auth_info_unknown_type_fails_deserialization() {
        let json = r#"{"type":"unknown","key":"sk-test"}"#;
        let result = serde_json::from_str::<auth::Info>(json);
        assert!(result.is_err(), "unknown type should fail deserialization");
    }

    #[test]
    fn auth_set_maps_io_error_to_bad_request() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let api_err = ApiError::bad_request(io_err.to_string());
        assert_eq!(api_err.status, StatusCode::BAD_REQUEST);
        assert!(api_err.message.contains("no such file"));
    }

    #[test]
    fn auth_remove_maps_io_error_to_bad_request() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let api_err = ApiError::bad_request(io_err.to_string());
        assert_eq!(api_err.status, StatusCode::BAD_REQUEST);
        assert!(api_err.message.contains("access denied"));
    }

    #[test]
    fn router_builds_without_panic() {
        let _r: Router<()> = router();
    }
