use serde_json::json;

use crate::client::test_support;

#[tokio::test]
async fn question_api_lists_replies_and_rejects_requests() {
    let server = test_support::server(vec![
        (
            200,
            json!([{
                "id": "q1",
                "sessionID": "s1",
                "questions": [{
                    "question": "Pick one",
                    "header": "Choice",
                    "options": [{"label": "A", "description": "Alpha"}],
                    "multiple": false
                }],
                "tool": {"messageID": "m1", "callID": "c1"}
            }]),
        ),
        (200, json!(true)),
        (200, json!(false)),
    ]);

    let questions = server.client().question_list().await.expect("questions");
    assert_eq!(questions[0].id, "q1");
    assert_eq!(questions[0].questions[0].options[0].label, "A");
    assert!(
        server
            .client()
            .question_reply(
                "q1",
                vec![vec!["A".to_string()], vec!["custom".to_string(), "B".to_string()]],
            )
            .await
            .expect("reply")
    );
    assert!(!server.client().question_reject("q1").await.expect("reject"));

    assert_eq!(server.take_request().path, "/v1/question");
    let request = server.take_request();
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/v1/question/q1/reply");
    assert_eq!(request.body, json!({"answers": [["A"], ["custom", "B"]]}));
    let request = server.take_request();
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/v1/question/q1/reject");
    assert_eq!(request.body, json!({}));
    server.join();
}
