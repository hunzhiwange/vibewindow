use crate::app::agent::config::Config;

use super::runner::process_message;

#[tokio::test]
async fn process_message_returns_error_for_unknown_provider() {
    let config =
        Config { default_provider: Some("__missing_provider__".to_string()), ..Config::default() };

    let result = process_message(config, "hello", "session").await;

    assert!(result.is_err());
}
