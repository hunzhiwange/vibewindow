use serde_json::json;
use vw_api_types::workflow::WorkflowRecordUpsertBody;

use crate::client::test_support;

fn workflow_record() -> serde_json::Value {
    json!({
        "uuid": "wf-1",
        "name": "Deploy",
        "description": "Ship workflow",
        "workflow_yaml": "steps: []",
        "created_at_ms": 1,
        "updated_at_ms": 2
    })
}

#[tokio::test]
async fn workflow_api_routes_application_crud_methods() {
    let server = test_support::server(vec![
        (
            200,
            json!([{
                "uuid": "wf-1",
                "name": "Deploy",
                "description": "Ship workflow",
                "created_at_ms": 1,
                "updated_at_ms": 2
            }]),
        ),
        (200, workflow_record()),
        (200, workflow_record()),
        (200, workflow_record()),
        (200, json!({"uuid": "wf-1", "deleted": true})),
    ]);
    let body = WorkflowRecordUpsertBody {
        uuid: Some("wf-1".to_string()),
        name: "Deploy".to_string(),
        description: "Ship workflow".to_string(),
        workflow_yaml: "steps: []".to_string(),
    };

    assert_eq!(server.client().workflow_applications_list().await.expect("list")[0].uuid, "wf-1");
    assert_eq!(server.client().workflow_application_get("wf-1").await.expect("get").name, "Deploy");
    assert_eq!(
        server.client().workflow_application_create(&body).await.expect("create").uuid,
        "wf-1"
    );
    assert_eq!(
        server
            .client()
            .workflow_application_update("wf-1", &body)
            .await
            .expect("update")
            .workflow_yaml,
        "steps: []"
    );
    assert!(server.client().workflow_application_delete("wf-1").await.expect("delete").deleted);

    assert_eq!(server.take_request().path, "/v1/workflow/applications");
    assert_eq!(server.take_request().path, "/v1/workflow/applications/wf-1");
    let request = server.take_request();
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/v1/workflow/applications");
    assert_eq!(request.body["uuid"], "wf-1");
    let request = server.take_request();
    assert_eq!(request.method, "PUT");
    assert_eq!(request.path, "/v1/workflow/applications/wf-1");
    assert_eq!(request.body["name"], "Deploy");
    let request = server.take_request();
    assert_eq!(request.method, "DELETE");
    assert_eq!(request.path, "/v1/workflow/applications/wf-1");
    assert_eq!(request.body, json!({}));
    server.join();
}
