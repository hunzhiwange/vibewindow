use super::*;
use crate::memory::traits::MemoryCategory;

#[test]
fn postgres_memory_type_name_remains_stable() {
    assert!(std::any::type_name::<PostgresMemory>().contains("PostgresMemory"));
}

#[test]
fn store_values_copy_user_input_and_serialize_category() {
    let values = PostgresMemory::store_values(
        "preference",
        "likes compact layouts",
        MemoryCategory::Custom("profile".into()),
        Some("session-1"),
    );

    assert_eq!(
        values,
        StoreValues {
            key: "preference".into(),
            content: "likes compact layouts".into(),
            category: "profile".into(),
            session_id: Some("session-1".into()),
        }
    );

    let values = PostgresMemory::store_values("daily", "", MemoryCategory::Daily, None);
    assert_eq!(values.category, "daily");
    assert!(values.session_id.is_none());
}

#[test]
fn store_statement_uses_upsert_with_bound_parameters() {
    let sql = PostgresMemory::store_statement("\"agent\".\"memories\"");

    assert!(sql.contains("INSERT INTO \"agent\".\"memories\""));
    assert!(sql.contains("(id, key, content, category, created_at, updated_at, session_id)"));
    assert!(sql.contains("VALUES"));
    assert!(sql.contains("($1, $2, $3, $4, $5, $6, $7)"));
    assert!(sql.contains("ON CONFLICT (key) DO UPDATE SET"));
    assert!(sql.contains("content = EXCLUDED.content"));
    assert!(sql.contains("session_id = EXCLUDED.session_id"));
}

#[test]
fn recall_query_is_trimmed_before_database_lookup() {
    assert_eq!(PostgresMemory::normalized_recall_query("  key words \n"), "key words");
    assert_eq!(PostgresMemory::normalized_recall_query("   "), "");
}

#[test]
fn recall_statement_scores_key_and_content_matches() {
    let sql = PostgresMemory::recall_statement("\"agent\".\"memories\"");

    assert!(sql.contains("SELECT id, key, content, category, created_at, session_id"));
    assert!(sql.contains("CASE WHEN key ILIKE '%' || $1 || '%' THEN 2.0 ELSE 0.0 END"));
    assert!(sql.contains("CASE WHEN content ILIKE '%' || $1 || '%' THEN 1.0 ELSE 0.0 END"));
    assert!(sql.contains("FROM \"agent\".\"memories\""));
    assert!(sql.contains("($2::TEXT IS NULL OR session_id = $2)"));
    assert!(
        sql.contains("($1 = '' OR key ILIKE '%' || $1 || '%' OR content ILIKE '%' || $1 || '%')")
    );
    assert!(sql.contains("ORDER BY score DESC, updated_at DESC"));
    assert!(sql.contains("LIMIT $3"));
}

#[test]
fn limit_to_i64_preserves_supported_limits() {
    assert_eq!(PostgresMemory::limit_to_i64(0), 0);
    assert_eq!(PostgresMemory::limit_to_i64(25), 25);
}

#[test]
fn get_statement_fetches_one_row_by_key() {
    let sql = PostgresMemory::get_statement("\"agent\".\"memories\"");

    assert!(sql.contains("SELECT id, key, content, category, created_at, session_id"));
    assert!(sql.contains("FROM \"agent\".\"memories\""));
    assert!(sql.contains("WHERE key = $1"));
    assert!(sql.contains("LIMIT 1"));
}

#[test]
fn list_filters_serialize_optional_category_and_session() {
    let category = MemoryCategory::Conversation;
    let filters = PostgresMemory::list_filters(Some(&category), Some("session-2"));

    assert_eq!(
        filters,
        ListFilters { category: Some("conversation".into()), session_id: Some("session-2".into()) }
    );

    assert_eq!(
        PostgresMemory::list_filters(None, None),
        ListFilters { category: None, session_id: None }
    );
}

#[test]
fn list_statement_applies_optional_category_and_session_filters() {
    let sql = PostgresMemory::list_statement("\"agent\".\"memories\"");

    assert!(sql.contains("SELECT id, key, content, category, created_at, session_id"));
    assert!(sql.contains("FROM \"agent\".\"memories\""));
    assert!(sql.contains("($1::TEXT IS NULL OR category = $1)"));
    assert!(sql.contains("($2::TEXT IS NULL OR session_id = $2)"));
    assert!(sql.contains("ORDER BY updated_at DESC"));
}

#[test]
fn forget_and_count_statements_use_qualified_table() {
    assert_eq!(
        PostgresMemory::forget_statement("\"agent\".\"memories\""),
        "DELETE FROM \"agent\".\"memories\" WHERE key = $1"
    );
    assert_eq!(
        PostgresMemory::count_statement("\"agent\".\"memories\""),
        "SELECT COUNT(*) FROM \"agent\".\"memories\""
    );
}
