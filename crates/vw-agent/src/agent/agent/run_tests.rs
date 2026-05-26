use super::run::run;
use crate::app::agent::config::Config;

#[tokio::test]
async fn run_rejects_missing_message_before_starting_agent() {
    let error = run(Config::default(), None, None, None, 0.7).await.unwrap_err();

    assert!(error.to_string().contains("provide a message"));
}
