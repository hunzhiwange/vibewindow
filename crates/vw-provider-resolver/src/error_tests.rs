use crate::error::{ApiCallError, ParsedApiCallError, parse_api_call_error};

#[test]
fn alibaba_market_activation_error_is_rewritten_to_actionable_message() {
    let parsed = parse_api_call_error(
        "alibaba-cn",
        ApiCallError {
            message:
                "Aliyun market app does not exist, the user may not have activated the service."
                    .to_string(),
            status_code: Some(400),
            is_retryable: false,
            url: None,
            response_headers: None,
            response_body: None,
        },
    );

    let ParsedApiCallError::ApiError { message, .. } = parsed else {
        panic!("expected api error");
    };

    assert!(message.contains("激活"));
    assert!(message.contains("siliconflow-cn"));
}

#[test]
fn unrelated_alibaba_error_message_is_preserved() {
    let parsed = parse_api_call_error(
        "alibaba-cn",
        ApiCallError {
            message: "some other upstream failure".to_string(),
            status_code: Some(500),
            is_retryable: true,
            url: None,
            response_headers: None,
            response_body: None,
        },
    );

    let ParsedApiCallError::ApiError { message, .. } = parsed else {
        panic!("expected api error");
    };

    assert_eq!(message, "some other upstream failure");
}