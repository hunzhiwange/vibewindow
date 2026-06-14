use super::*;
use axum::http::StatusCode;

fn body(name: &str, workflow_yaml: &str) -> WorkflowRecordUpsertBody {
    WorkflowRecordUpsertBody {
        uuid: None,
        name: name.to_string(),
        description: "  description  ".to_string(),
        workflow_yaml: workflow_yaml.to_string(),
    }
}

fn db_path(name: &str) -> PathBuf {
    tempfile::tempdir().expect("temp dir").keep().join(name).join("workflow.sqlite")
}

#[tokio::test(flavor = "multi_thread")]
async fn workflow_store_crud_lists_orders_and_deletes_records() {
    let db = db_path("crud");
    let first_uuid = Uuid::new_v4().to_string();
    let second_uuid = Uuid::new_v4().to_string();

    assert!(list_records(db.clone()).await.expect("empty list").is_empty());

    let mut first_body = body("  First  ", "nodes: []");
    first_body.uuid = Some(first_uuid.clone());
    let first = create_record(db.clone(), first_body).await.expect("first create");
    assert_eq!(first.uuid, first_uuid);
    assert_eq!(first.name, "First");
    assert_eq!(first.description, "description");

    let mut second_body = body("Second", "nodes: [1]");
    second_body.uuid = Some(second_uuid.clone());
    let second = create_record(db.clone(), second_body).await.expect("second create");
    assert_eq!(second.uuid, second_uuid);

    let fetched = get_record(db.clone(), first_uuid.clone()).await.expect("get first");
    assert_eq!(fetched.workflow_yaml, "nodes: []");

    let updated =
        update_record(db.clone(), first_uuid.clone(), body("First Updated", "nodes: [2]"))
            .await
            .expect("update first");
    assert_eq!(updated.created_at_ms, first.created_at_ms);
    assert!(updated.updated_at_ms >= first.updated_at_ms);
    assert_eq!(updated.workflow_yaml, "nodes: [2]");

    let summaries = list_records(db.clone()).await.expect("list records");
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].uuid, first_uuid);
    assert_eq!(summaries[1].uuid, second.uuid);

    let deleted = delete_record(db.clone(), first_uuid.clone()).await.expect("delete first");
    assert_eq!(deleted.uuid, first_uuid);
    assert!(deleted.deleted);

    let missing_delete =
        delete_record(db.clone(), first_uuid.clone()).await.expect("delete missing is ok");
    assert!(!missing_delete.deleted);

    let missing = get_record(db, first_uuid).await.expect_err("missing get should fail");
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test(flavor = "multi_thread")]
async fn workflow_store_create_generates_uuid_and_validates_inputs() {
    let db = db_path("validation");

    let generated = create_record(db.clone(), body("Generated", "workflow: true"))
        .await
        .expect("generated create");
    Uuid::parse_str(&generated.uuid).expect("generated uuid should be valid");

    let invalid_uuid =
        get_record(db.clone(), "not-a-uuid".to_string()).await.expect_err("invalid uuid");
    assert_eq!(invalid_uuid.status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid_uuid.to_string(), "workflow uuid is invalid");

    let blank_name =
        create_record(db.clone(), body("   ", "workflow")).await.expect_err("blank name");
    assert_eq!(blank_name.status, StatusCode::BAD_REQUEST);
    assert_eq!(blank_name.to_string(), "workflow name is required");

    let blank_yaml = create_record(db, body("Name", "  ")).await.expect_err("blank yaml");
    assert_eq!(blank_yaml.status, StatusCode::BAD_REQUEST);
    assert_eq!(blank_yaml.to_string(), "workflow_yaml is required");
}

#[test]
fn workflow_store_blocking_helpers_cover_sql_and_integer_edges() {
    let db = db_path("blocking");
    let uuid = Uuid::new_v4().to_string();

    let record = upsert_record_blocking(db.clone(), &uuid, &body("Negative", "yaml"))
        .expect("upsert should create");
    assert_eq!(record.created_at_ms, record.updated_at_ms);

    let conn = open_db(&db).expect("db should open");
    conn.execute(
        "UPDATE dify_workflows SET created_at_ms = -1, updated_at_ms = -5 WHERE uuid = ?1",
        params![uuid],
    )
    .expect("manual update should succeed");
    drop(conn);

    let record = get_record_blocking(db.clone(), &uuid).expect("record should load");
    assert_eq!(record.created_at_ms, 0);
    assert_eq!(record.updated_at_ms, 0);

    let missing_uuid = Uuid::new_v4().to_string();
    let missing =
        get_record_blocking(db.clone(), &missing_uuid).expect_err("missing valid uuid should fail");
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    let invalid_update = update_record_blocking_for_test(db.clone(), "bad", body("Name", "yaml"));
    assert_eq!(invalid_update.expect_err("invalid uuid").status, StatusCode::BAD_REQUEST);

    let invalid_db = tempfile::tempdir().expect("temp dir");
    let sql = list_records_blocking(invalid_db.path().to_path_buf())
        .expect_err("opening directory as db should fail");
    assert_eq!(sql.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(sql.to_string().contains("workflow sqlite error"));

    let blocked_parent = tempfile::NamedTempFile::new().expect("parent placeholder");
    let create_dir_error = list_records_blocking(blocked_parent.path().join("workflow.sqlite"))
        .expect_err("file parent should block directory creation");
    assert_eq!(create_dir_error.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(create_dir_error.to_string().contains("create workflow db dir failed"));
}

#[tokio::test(flavor = "multi_thread")]
async fn workflow_store_maps_blocking_task_panics_to_internal_error() {
    let error = spawn_store_task(PathBuf::from("unused"), |_path| -> Result<(), ApiError> {
        panic!("store task panic");
    })
    .await
    .expect_err("panic should become internal error");

    assert_eq!(error.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(error.to_string().contains("workflow store task failed"));
}

fn update_record_blocking_for_test(
    db_path: PathBuf,
    uuid: &str,
    body: WorkflowRecordUpsertBody,
) -> Result<WorkflowRecord, ApiError> {
    upsert_record_blocking(db_path, uuid, &body)
}
