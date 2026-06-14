use serde_json::json;

use crate::client::test_support;

#[tokio::test]
async fn provider_list_reads_directory_scoped_provider_state() {
    let server = test_support::server(vec![(
        200,
        json!({
            "all": [{
                "id": "openai",
                "name": "OpenAI",
                "source": "env",
                "env": ["OPENAI_API_KEY"],
                "models": {}
            }],
            "default": {"chat": "openai"},
            "connected": ["openai"]
        }),
    )]);

    let response =
        server.client().provider_list(Some("/tmp/project")).await.expect("provider list");

    assert_eq!(response.all[0].id, "openai");
    assert_eq!(response.default["chat"], "openai");
    assert_eq!(response.connected, vec!["openai".to_string()]);
    let request = server.take_request();
    assert_eq!(request.method, "GET");
    assert_eq!(request.path, "/v1/provider?directory=%2Ftmp%2Fproject");
    assert_eq!(request.body, serde_json::Value::Null);
    server.join();
}
