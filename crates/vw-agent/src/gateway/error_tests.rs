use super::*;
use axum::response::IntoResponse;

#[test]
fn constructors_set_status_and_message() {
    let error = ApiError::bad_request("bad input");

    assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(error.to_string(), "bad input");
}

#[test]
fn into_response_uses_error_json_body() {
    let response = ApiError::not_found("missing").into_response();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}
