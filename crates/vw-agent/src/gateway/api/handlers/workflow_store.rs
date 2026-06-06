//! Workflow 本地 SQLite 存储。

use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;
use vw_api_types::workflow::{
    WorkflowRecord, WorkflowRecordDeleteResponse, WorkflowRecordSummary, WorkflowRecordUpsertBody,
};

use crate::app::agent::gateway::ApiError;

const WORKFLOW_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS dify_workflows (
    uuid TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    workflow_yaml TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
"#;

pub(super) async fn list_records(db_path: PathBuf) -> Result<Vec<WorkflowRecordSummary>, ApiError> {
    spawn_store_task(db_path, list_records_blocking).await
}

pub(super) async fn get_record(db_path: PathBuf, uuid: String) -> Result<WorkflowRecord, ApiError> {
    spawn_store_task(db_path, move |path| get_record_blocking(path, &uuid)).await
}

pub(super) async fn create_record(
    db_path: PathBuf,
    body: WorkflowRecordUpsertBody,
) -> Result<WorkflowRecord, ApiError> {
    spawn_store_task(db_path, move |path| {
        let uuid = body.uuid.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        upsert_record_blocking(path, &uuid, &body)
    })
    .await
}

pub(super) async fn update_record(
    db_path: PathBuf,
    uuid: String,
    body: WorkflowRecordUpsertBody,
) -> Result<WorkflowRecord, ApiError> {
    spawn_store_task(db_path, move |path| upsert_record_blocking(path, &uuid, &body)).await
}

pub(super) async fn delete_record(
    db_path: PathBuf,
    uuid: String,
) -> Result<WorkflowRecordDeleteResponse, ApiError> {
    spawn_store_task(db_path, move |path| delete_record_blocking(path, &uuid)).await
}

async fn spawn_store_task<T: Send + 'static>(
    db_path: PathBuf,
    task: impl FnOnce(PathBuf) -> Result<T, ApiError> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(move || task(db_path))
        .await
        .map_err(|error| ApiError::internal(format!("workflow store task failed: {error}")))?
}

fn list_records_blocking(db_path: PathBuf) -> Result<Vec<WorkflowRecordSummary>, ApiError> {
    let conn = open_db(&db_path)?;
    let mut stmt = conn
        .prepare(
            "SELECT uuid, name, description, created_at_ms, updated_at_ms \
             FROM dify_workflows ORDER BY updated_at_ms DESC, name ASC",
        )
        .map_err(sql_error)?;

    let rows = stmt
        .query_map([], |row| {
            Ok(WorkflowRecordSummary {
                uuid: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at_ms: read_u64(row, 3)?,
                updated_at_ms: read_u64(row, 4)?,
            })
        })
        .map_err(sql_error)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(sql_error)
}

fn get_record_blocking(db_path: PathBuf, uuid: &str) -> Result<WorkflowRecord, ApiError> {
    validate_uuid(uuid)?;
    let conn = open_db(&db_path)?;
    query_record(&conn, uuid)?
        .ok_or_else(|| ApiError::not_found(format!("workflow record not found: {uuid}")))
}

fn upsert_record_blocking(
    db_path: PathBuf,
    uuid: &str,
    body: &WorkflowRecordUpsertBody,
) -> Result<WorkflowRecord, ApiError> {
    validate_uuid(uuid)?;
    validate_body(body)?;
    let conn = open_db(&db_path)?;
    let now = now_ms();
    let created_at_ms =
        query_record(&conn, uuid)?.map(|record| record.created_at_ms).unwrap_or(now);

    conn.execute(
        "INSERT INTO dify_workflows \
         (uuid, name, description, workflow_yaml, created_at_ms, updated_at_ms) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         ON CONFLICT(uuid) DO UPDATE SET \
         name = excluded.name, \
         description = excluded.description, \
         workflow_yaml = excluded.workflow_yaml, \
         updated_at_ms = excluded.updated_at_ms",
        params![
            uuid,
            body.name.trim(),
            body.description.trim(),
            body.workflow_yaml.as_str(),
            created_at_ms,
            now,
        ],
    )
    .map_err(sql_error)?;

    query_record(&conn, uuid)?.ok_or_else(|| ApiError::internal("workflow record was not saved"))
}

fn delete_record_blocking(
    db_path: PathBuf,
    uuid: &str,
) -> Result<WorkflowRecordDeleteResponse, ApiError> {
    validate_uuid(uuid)?;
    let conn = open_db(&db_path)?;
    let changed = conn
        .execute("DELETE FROM dify_workflows WHERE uuid = ?1", params![uuid])
        .map_err(sql_error)?;

    Ok(WorkflowRecordDeleteResponse { uuid: uuid.to_string(), deleted: changed > 0 })
}

fn query_record(conn: &Connection, uuid: &str) -> Result<Option<WorkflowRecord>, ApiError> {
    conn.query_row(
        "SELECT uuid, name, description, workflow_yaml, created_at_ms, updated_at_ms \
         FROM dify_workflows WHERE uuid = ?1",
        params![uuid],
        |row| {
            Ok(WorkflowRecord {
                uuid: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                workflow_yaml: row.get(3)?,
                created_at_ms: read_u64(row, 4)?,
                updated_at_ms: read_u64(row, 5)?,
            })
        },
    )
    .optional()
    .map_err(sql_error)
}

fn open_db(db_path: &PathBuf) -> Result<Connection, ApiError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            ApiError::internal(format!("create workflow db dir failed: {error}"))
        })?;
    }
    let conn = Connection::open(db_path).map_err(sql_error)?;
    conn.execute_batch(WORKFLOW_SCHEMA).map_err(sql_error)?;
    Ok(conn)
}

fn read_u64(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    Ok(u64::try_from(value).unwrap_or_default())
}

fn validate_uuid(uuid: &str) -> Result<(), ApiError> {
    Uuid::parse_str(uuid).map(|_| ()).map_err(|_| ApiError::bad_request("workflow uuid is invalid"))
}

fn validate_body(body: &WorkflowRecordUpsertBody) -> Result<(), ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::bad_request("workflow name is required"));
    }
    if body.workflow_yaml.trim().is_empty() {
        return Err(ApiError::bad_request("workflow_yaml is required"));
    }
    Ok(())
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn sql_error(error: rusqlite::Error) -> ApiError {
    ApiError::internal(format!("workflow sqlite error: {error}"))
}
