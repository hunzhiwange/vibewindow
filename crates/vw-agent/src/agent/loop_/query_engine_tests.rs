use super::query_engine::QueryEngineUsage;

#[test]
fn query_engine_usage_totals_billable_tokens() {
    let usage = QueryEngineUsage {
        input_tokens: 10,
        output_tokens: 7,
        cached_tokens: 3,
        reasoning_tokens: 99,
        llm_calls: 2,
    };

    assert_eq!(usage.total_tokens(), 20);
    assert_eq!(usage.as_ui_token_usage().reasoning_tokens, 99);
}
