use super::select_case_handle;
use crate::workflow::variables::VariablePool;
use serde_json::{Value, json};

#[test]
fn select_case_handle_supports_dify_common_operators() {
    let mut pool = VariablePool::default();
    pool.insert_selector(
        &["start".to_string(), "text".to_string()],
        Value::String("订单查询".to_string()),
    );
    pool.insert_selector(&["code".to_string(), "success".to_string()], Value::Bool(true));

    let cases = json!([
        {
            "case_id": "first",
            "logical_operator": "and",
            "conditions": [
                {
                    "comparison_operator": "contains",
                    "variable_selector": ["start", "text"],
                    "value": "订单"
                },
                {
                    "comparison_operator": "=",
                    "variable_selector": ["code", "success"],
                    "value": "1"
                }
            ]
        }
    ]);

    assert_eq!(
        select_case_handle(cases.as_array().expect("cases"), &pool).expect("selected"),
        "first"
    );
}

#[test]
fn select_case_handle_rejects_unsupported_operator() {
    let pool = VariablePool::default();
    let cases = json!([
        {
            "case_id": "bad",
            "conditions": [
                {
                    "comparison_operator": "around",
                    "variable_selector": ["start", "text"],
                    "value": "订单"
                }
            ]
        }
    ]);

    let error = select_case_handle(cases.as_array().expect("cases"), &pool).expect_err("error");
    assert!(error.contains("不支持的 if-else 比较运算符"));
}
