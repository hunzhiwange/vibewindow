use super::*;
use axum::{
    Json as AxumJson, Router,
    extract::Query,
    routing::{get, post},
};
use serde_json::json;
use std::collections::BTreeMap;
use vw_api_types::data::{
    AiDataConnectionKind, AiDataCountMode, AiDataQueryKind, AiDataSourceMode,
};

fn connection(kind: AiDataConnectionKind) -> AiDataConnectionDto {
    AiDataConnectionDto {
        id: "conn-1".to_string(),
        name: "Primary".to_string(),
        kind,
        description: None,
        enabled: true,
        read_only: true,
        base_url: Some("https://api.example.test/root/".to_string()),
        connection_url: Some("postgres://example".to_string()),
        sqlite_path: None,
        default_path: Some("/default".to_string()),
        auth_token: Some("Bearer token".to_string()),
        headers: BTreeMap::from([("X-Test".to_string(), "yes".to_string())]),
        schema_hint: Some("items(id,name)".to_string()),
        updated_at_ms: 100,
        last_used_ms: None,
    }
}

fn sqlite_connection(path: &str) -> AiDataConnectionDto {
    AiDataConnectionDto {
        sqlite_path: Some(path.to_string()),
        base_url: None,
        connection_url: None,
        default_path: None,
        auth_token: None,
        headers: BTreeMap::new(),
        schema_hint: None,
        ..connection(AiDataConnectionKind::Sqlite)
    }
}

fn report() -> AiDataReportDto {
    AiDataReportDto {
        id: "report-1".to_string(),
        name: "Sales".to_string(),
        slug: "sales".to_string(),
        data_source: AiDataSourceMode::Normal,
        default_source_key: Some("main".to_string()),
        report_config: json!({
            "modules": [
                {
                    "type": "table",
                    "source": "main",
                    "template_code": "tpl",
                    "defaultSortFields": [
                        { "sortFieldId": "amount", "sortMethod": "desc" }
                    ],
                    "fields": [
                        { "code": "amount", "valueType": "sys_price" },
                        { "code": "ratio", "valueType": "float" }
                    ]
                },
                { "type": "chart", "show": false },
                { "type": "text", "show": "F" },
                { "type": "summary", "show": true }
            ],
            "alertMenu": { "show": true },
            "mock": {
                "goods_options": [
                    {
                        "company_id": "c1",
                        "goods_id": "g1",
                        "options_id": "1,2",
                        "options_name": "Red / XL"
                    },
                    { "options_id": "9", "options_name": "Fallback" }
                ]
            }
        }),
        sources: vec![AiDataReportSourceDto {
            source_key: "main".to_string(),
            connection_id: "conn-1".to_string(),
            query_kind: AiDataQueryKind::Sql,
            sql: Some("SELECT amount, ratio FROM sales".to_string()),
            count_sql: None,
            cube_query: None,
            http_method: "GET".to_string(),
            http_path: None,
            http_body: None,
            append_pagination: true,
        }],
        updated_at_ms: 200,
    }
}

fn base_resolved() -> ResolvedQuery {
    ResolvedQuery {
        connection: connection(AiDataConnectionKind::Sqlite),
        report_config: None,
        query_kind: AiDataQueryKind::Sql,
        sql: None,
        count_sql: None,
        cube_query: None,
        http_method: "GET".to_string(),
        http_path: None,
        http_body: None,
        params: BTreeMap::new(),
        page: 1,
        limit: 25,
        count_mode: AiDataCountMode::Disabled,
        order_by: None,
        search_fields: None,
        template_fields: None,
        transformer_fields: None,
        template_code: None,
        append_pagination: true,
        debug: false,
    }
}

fn make_sqlite_fixture(path: &std::path::Path) {
    let conn = rusqlite::Connection::open(path).unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE items (
            id INTEGER PRIMARY KEY,
            name TEXT,
            price REAL,
            active INTEGER,
            blob_value BLOB
        );
        INSERT INTO items (id, name, price, active, blob_value) VALUES
            (1, 'apple', 1.234, 1, x'0A0B'),
            (2, 'banana', 2.0, 0, x'0C'),
            (3, 'pear', 3.5, 1, x'0D');
        "#,
    )
    .unwrap();
}

async fn spawn_data_server() -> String {
    async fn items_get(Query(query): Query<BTreeMap<String, String>>) -> AxumJson<Value> {
        AxumJson(json!({
            "items": [
                { "method": "GET", "q": query.get("q").cloned().unwrap_or_default() }
            ],
            "total": 1
        }))
    }

    async fn items_post(AxumJson(body): AxumJson<Value>) -> AxumJson<Value> {
        AxumJson(json!({
            "data": [
                { "method": "POST", "body": body }
            ],
            "count": 1
        }))
    }

    async fn cube_meta() -> AxumJson<Value> {
        AxumJson(json!({ "cubes": [{ "name": "Orders" }] }))
    }

    async fn cube_load(AxumJson(body): AxumJson<Value>) -> AxumJson<Value> {
        AxumJson(json!({
            "data": [
                { "cube": "Orders", "query": body["query"].clone() }
            ],
            "total": 1
        }))
    }

    async fn text() -> &'static str {
        "plain text"
    }

    async fn fail() -> (axum::http::StatusCode, &'static str) {
        (axum::http::StatusCode::IM_A_TEAPOT, "short failure")
    }

    let app = Router::new()
        .route("/items", get(items_get).post(items_post))
        .route("/cubejs-api/v1/meta", get(cube_meta))
        .route("/cubejs-api/v1/load", post(cube_load))
        .route("/text", get(text))
        .route("/fail", get(fail));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn execute_query_sqlite_applies_templates_count_pagination_and_debug() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("items.db");
    make_sqlite_fixture(&db_path);
    let settings =
        AiDataSettings { default_limit: 2, default_timeout_secs: 5, ..AiDataSettings::default() };
    let connections = vec![sqlite_connection(db_path.to_str().unwrap())];
    let mut params = BTreeMap::new();
    params.insert("active".to_string(), json!(true));

    let response = execute_query(
        &settings,
        &connections,
        &[],
        AiDataQueryRequest {
            connection_id: Some("conn-1".to_string()),
            sql: Some(
                "SELECT id, name, price, blob_value FROM items WHERE active = {{ active }}"
                    .to_string(),
            ),
            params,
            page: Some(1),
            limit: Some(1),
            count: Some(AiDataCountMode::Enabled),
            order_by: Some("id DESC".to_string()),
            search_fields: Some(vec!["id".to_string(), "price".to_string()]),
            template_fields: Some(vec![AiDataTemplateFieldDto {
                code: "price".to_string(),
                value_type: FORMAT_SYS_PRICE,
            }]),
            debug: Some(true),
            ..AiDataQueryRequest::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(response.page.total_record, 2);
    assert_eq!(response.page.total_page, 2);
    assert_eq!(response.page.from, 1);
    assert!(response.has_next_page);
    assert_eq!(response.items, vec![json!({ "id": 3, "price": 3.5 })]);
    assert_eq!(response.debug.as_ref().unwrap()["query_kind"], "sql");
}

#[tokio::test]
async fn sqlite_connection_test_and_catalog_cover_success_and_missing_path() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("items.db");
    make_sqlite_fixture(&db_path);
    let sqlite = sqlite_connection(db_path.to_str().unwrap());

    let message = test_connection(&sqlite, 1).await.unwrap();
    assert!(message.contains("SQLite OK"));

    let catalog = connection_catalog(&sqlite, 1).await.unwrap();
    assert_eq!(catalog["tables"][0]["name"], "items");
    assert!(sqlite_path(&connection(AiDataConnectionKind::Sqlite)).is_err());
}

#[tokio::test]
async fn execute_query_rejects_invalid_resolution_and_unsupported_backends() {
    let settings = AiDataSettings::default();
    let disabled = AiDataConnectionDto { enabled: false, ..connection(AiDataConnectionKind::Http) };

    let err = execute_query(&settings, &[], &[], AiDataQueryRequest::default()).await.unwrap_err();
    assert!(err.contains("connection_id 或 report_id"));

    let err = execute_query(
        &settings,
        &[disabled],
        &[],
        AiDataQueryRequest { connection_id: Some("conn-1".to_string()), ..Default::default() },
    )
    .await
    .unwrap_err();
    assert!(err.contains("已被禁用"));

    let mysql = connection(AiDataConnectionKind::Mysql);
    assert!(test_connection(&mysql, 1).await.unwrap_err().contains("MySQL"));
    assert!(
        connection_catalog(&connection(AiDataConnectionKind::Postgres), 1)
            .await
            .unwrap_err()
            .contains("PostgreSQL")
    );
}

#[tokio::test]
async fn execute_query_covers_http_and_cube_network_paths() {
    let base_url = spawn_data_server().await;
    let settings =
        AiDataSettings { default_limit: 10, default_timeout_secs: 2, ..AiDataSettings::default() };

    let http = AiDataConnectionDto {
        base_url: Some(base_url.clone()),
        default_path: Some("/items".to_string()),
        ..connection(AiDataConnectionKind::Http)
    };
    let mut params = BTreeMap::new();
    params.insert("q".to_string(), json!("search"));
    let response = execute_query(
        &settings,
        &[http.clone()],
        &[],
        AiDataQueryRequest {
            connection_id: Some("conn-1".to_string()),
            params,
            debug: Some(true),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(response.items, vec![json!({"method": "GET", "q": "search"})]);
    assert_eq!(response.debug.unwrap()["query_kind"], "http");

    let response = execute_query(
        &settings,
        &[http.clone()],
        &[],
        AiDataQueryRequest {
            connection_id: Some("conn-1".to_string()),
            http_method: Some("POST".to_string()),
            http_path: Some("/items".to_string()),
            http_body: Some(json!({ "tenant": "{{ tenant }}" })),
            params: BTreeMap::from([("tenant".to_string(), json!("t1"))]),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(response.items[0]["method"], "POST");
    assert_eq!(response.items[0]["body"]["tenant"], "t1");

    assert!(
        execute_query(
            &settings,
            &[http.clone()],
            &[],
            AiDataQueryRequest {
                connection_id: Some("conn-1".to_string()),
                count: Some(AiDataCountMode::Only),
                ..Default::default()
            },
        )
        .await
        .unwrap_err()
        .contains("HTTP 查询暂不支持")
    );

    let cube = AiDataConnectionDto {
        id: "cube-1".to_string(),
        base_url: Some(base_url),
        default_path: None,
        ..connection(AiDataConnectionKind::Cube)
    };
    assert!(test_connection(&cube, 2).await.unwrap().contains("Cube OK"));
    assert_eq!(connection_catalog(&cube, 2).await.unwrap()["cubes"][0]["name"], "Orders");

    let response = execute_query(
        &settings,
        &[cube],
        &[],
        AiDataQueryRequest {
            connection_id: Some("cube-1".to_string()),
            cube_query: Some(json!({ "measures": ["Orders.count"] })),
            debug: Some(true),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(response.items[0]["cube"], "Orders");
    assert_eq!(response.debug.unwrap()["query_kind"], "cube");
}

#[tokio::test]
async fn http_helpers_parse_raw_text_and_error_responses() {
    let base_url = spawn_data_server().await;
    let http = AiDataConnectionDto {
        base_url: Some(base_url.clone()),
        ..connection(AiDataConnectionKind::Http)
    };

    let (_, raw) =
        send_json_request(&http, reqwest::Method::GET, Some(&format!("{base_url}/text")), None, 2)
            .await
            .unwrap();
    assert_eq!(raw, json!({ "raw": "plain text" }));

    let err =
        send_json_request(&http, reqwest::Method::GET, Some(&format!("{base_url}/fail")), None, 2)
            .await
            .unwrap_err();
    assert!(err.contains("HTTP 418"));
    assert!(err.contains("short failure"));
    assert!(
        send_json_request(&http, reqwest::Method::GET, None, None, 2)
            .await
            .unwrap_err()
            .contains("请求 URL")
    );
}

#[test]
fn merge_search_condition_and_template_rendering_handle_edge_cases() {
    let mut params = BTreeMap::from([
        ("name".to_string(), json!("O'Reilly")),
        ("nested".to_string(), json!({ "id": 7 })),
        ("tags".to_string(), json!(["a", "b"])),
    ]);
    let merged =
        merge_search_condition(params.clone(), Some(json!(r#"{"name":"pear","extra":3}"#)))
            .unwrap();
    assert_eq!(merged["name"], "pear");
    assert_eq!(merged["extra"], 3);
    assert!(merge_search_condition(BTreeMap::new(), Some(json!([1]))).is_err());
    assert!(merge_search_condition(BTreeMap::new(), Some(json!("[1]"))).is_err());

    assert_eq!(
        render_sql_template("name={{name}} AND id={{nested.id}} AND missing={{none}}", &params)
            .unwrap(),
        "name='O''Reilly' AND id=7 AND missing=NULL"
    );
    assert_eq!(render_string_template("/items/{{nested.id}}/{{tags}}", &params), "/items/7/a,b");
    assert_eq!(render_json_template(&json!("{{ nested.id }}"), &params), json!(7));
    assert_eq!(
        render_json_template(&json!({ "q": "hello {{name}}" }), &params),
        json!({"q": "hello O'Reilly"})
    );

    params.insert("obj".to_string(), json!({"a": "b"}));
    assert_eq!(sql_literal(&params["obj"]), r#"'{\"a\":\"b\"}'"#.replace("\\\"", "\""));
}

#[test]
fn report_config_visibility_defaults_sorting_and_source_context_are_resolved() {
    let report = report();
    let prepared = prepared_report(&report);
    let modules = prepared.report_config["modules"].as_array().unwrap();
    assert_eq!(modules.len(), 2);
    assert!(prepared.report_config["alertMenu"]["menu"].as_array().unwrap().is_empty());
    assert!(prepared.report_config["alertMenu"]["variable"].is_object());
    assert_eq!(
        report_default_order_by(Some(&prepared.report_config)).as_deref(),
        Some("amount DESC")
    );
    assert_eq!(
        report_default_template_code(Some(&prepared.report_config), Some("main")).as_deref(),
        Some("tpl")
    );

    assert_eq!(is_visible_flag(None), true);
    assert_eq!(is_visible_flag(Some(&json!("0"))), false);
    assert_eq!(is_visible_flag(Some(&json!(0))), false);
    assert_eq!(module_is_visible(&json!("not-object")), true);

    let (resolved_report, source) = report_source_context(&[report], Some("sales"), None).unwrap();
    assert_eq!(resolved_report.slug, "sales");
    assert_eq!(source.source_key, "main");
    assert!(report_source_context(&[], Some("missing"), None).is_none());
}

#[test]
fn sql_safety_order_by_pagination_and_endpoints_are_sanitized() {
    ensure_read_only_sql(" SELECT * FROM items").unwrap();
    ensure_read_only_sql("WITH x AS (SELECT 1) SELECT * FROM x").unwrap();
    assert!(ensure_read_only_sql("DELETE FROM items").is_err());
    assert!(ensure_read_only_sql("SELECT * FROM a; DROP TABLE a").is_err());
    assert_eq!(sanitize_order_by(Some("name desc, id")).unwrap().unwrap(), "name DESC, id ASC");
    assert!(sanitize_order_by(Some("name; drop")).is_err());

    assert_eq!(
        build_page(0, 1, 25, 0),
        AiDataPageDto {
            per_page: 25,
            current_page: 1,
            total_page: 0,
            total_record: 0,
            from: 0,
            to: 0,
        }
    );
    assert_eq!(build_page(51, 3, 25, 1).to, 51);

    let conn = connection(AiDataConnectionKind::Http);
    assert_eq!(
        http_endpoint(&conn, Some("/v1/items")).unwrap(),
        "https://api.example.test/root/v1/items"
    );
    assert_eq!(cube_endpoint(&conn, "meta").unwrap(), "https://api.example.test/root/default/meta");
    assert!(http_endpoint(&AiDataConnectionDto { base_url: None, ..conn }, None).is_err());
    assert_eq!(parse_http_method("head").unwrap(), reqwest::Method::HEAD);
    assert!(parse_http_method("put").is_err());
}

#[test]
fn post_processing_formats_filters_and_transforms_items() {
    let mut resolved = base_resolved();
    resolved.report_config = Some(report().report_config);
    resolved.template_code = Some("tpl".to_string());
    resolved.search_fields = Some(vec![
        "amount".to_string(),
        "ratio".to_string(),
        "options_name".to_string(),
        "options_id".to_string(),
    ]);
    resolved.params.insert("company_id".to_string(), json!("c1"));
    resolved.params.insert("goods_id".to_string(), json!("g1"));
    resolved.transformer_fields = Some(vec![
        AiDataTransformerFieldDto {
            code: "ratio".to_string(),
            transformer_type: "percentage".to_string(),
            transformer_args: BTreeMap::from([("decimals".to_string(), json!(1))]),
        },
        AiDataTransformerFieldDto {
            code: "options_name".to_string(),
            transformer_type: "goods_options_name".to_string(),
            transformer_args: BTreeMap::from([("goods_field".to_string(), json!("goods_id"))]),
        },
    ]);
    let mut items = vec![json!({
        "amount": "12.345",
        "ratio": 0.125,
        "options_id": "[\"1\",\"2\"]",
        "options_name": "",
        "goods_id": "g1",
        "hidden": "remove"
    })];

    post_process_items(&mut items, &resolved).unwrap();

    assert_eq!(
        items,
        vec![json!({
            "amount": 12.35,
            "ratio": "12.5%",
            "options_id": "[\"1\",\"2\"]",
            "options_name": "Red / XL"
        })]
    );

    resolved.transformer_fields = Some(vec![AiDataTransformerFieldDto {
        code: "amount".to_string(),
        transformer_type: "unknown".to_string(),
        transformer_args: BTreeMap::new(),
    }]);
    assert!(post_process_items(&mut items, &resolved).unwrap_err().contains("暂不支持"));
}

#[test]
fn template_sources_value_formats_and_goods_fallbacks_are_supported() {
    let report_config = json!({
        "mockTemplateSource": {
            "templates": [
                {
                    "templateCode": "mock-tpl",
                    "fields": [
                        { "field": "CountValue", "sysFormat": "int" },
                        { "key": "name", "value_type": "string" }
                    ]
                }
            ]
        }
    });
    let formats = resolve_template_formats(Some(&report_config), None, Some("mock-tpl"));
    assert_eq!(formats["count_value"], FORMAT_INT);
    assert_eq!(formats["name"], FORMAT_STRING);
    assert_eq!(format_value_by_type(json!("2.9"), FORMAT_INT), json!(2));
    assert_eq!(format_value_by_type(json!(2), FORMAT_STRING), json!("2"));
    assert_eq!(normalize_field_code("goodsOptions Name"), "goods_options_name");
    assert_eq!(normalize_options_lookup_key(r#"["1","2"]"#), "1,2");
    assert_eq!(value_lookup_string(&json!(["a", 1, true])).unwrap(), "a,1,true");
    assert_eq!(
        format_goods_options_name_value(json!({"color":"red","size":""})),
        json!("color:red / size")
    );
}

#[test]
fn extract_items_total_and_body_merge_cover_response_shapes() {
    assert_eq!(extract_items_and_total(json!([{"id": 1}])).1, Some(1));
    assert_eq!(extract_items_and_total(json!({"data": [{"id": 1}], "total": 9})).1, Some(9));
    assert_eq!(extract_items_and_total(json!({"items": [{"id": 2}], "count": 3})).0[0]["id"], 2);
    assert_eq!(
        extract_items_and_total(json!({"list": [{"id": 3}], "page": {"total_record": 4}})).1,
        Some(4)
    );
    assert_eq!(extract_items_and_total(json!("raw")).0, vec![json!("raw")]);

    let params = BTreeMap::from([("page".to_string(), json!(1))]);
    assert_eq!(
        merge_json_object(Some(json!({"q": "x"})), &params),
        Some(json!({"q": "x", "page": 1}))
    );
    assert_eq!(merge_json_object(Some(json!([1])), &params), Some(json!([1])));
    assert_eq!(merge_json_object(None, &params), Some(json!({"page": 1})));
    assert_eq!(merge_json_object(None, &BTreeMap::new()), None);
    assert_eq!(truncate_text("你好abcdef", 4), "你好ab");
}
