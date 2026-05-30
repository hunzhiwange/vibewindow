//! AI-DATA 自然语言查询支持。
//!
//! 该模块把用户问题、连接上下文、报表上下文和可选目录信息交给模型生成
//! 执行计划，并在计划通过授权校验后复用常规查询运行时执行。

use axum::Json;
use axum::extract::State;
use serde_json::{Value, json};
use vw_api_types::data::{
    AiDataAiQueryRequest, AiDataAiQueryResponse, AiDataConnectionDto, AiDataExecutionPlan,
    AiDataQueryRequest, AiDataReportDto, AiDataSettings,
};

use super::runtime;
use crate::app::agent::gateway::ApiError;
use crate::app::agent::gateway::state::AppState;
use crate::app::agent::providers::ChatMessage;

/// 处理 AI-DATA 自然语言查询请求。
///
/// # 参数
///
/// - `state`: 网关应用状态，包含模型 provider。
/// - `settings`: AI-DATA 当前设置。
/// - `connections`: 可用连接列表。
/// - `reports`: 可用报表列表。
/// - `body`: 用户自然语言查询请求。
///
/// # 返回值
///
/// 返回模型生成的执行计划、实际查询结果、可选结果总结和原始模型响应。
///
/// # 错误
///
/// 当上下文无法解析、模型规划失败、规划 JSON 非法、计划越权或查询执行失败时返回 `ApiError`。
pub(super) async fn handle_ai_query(
    State(state): State<AppState>,
    settings: AiDataSettings,
    connections: Vec<AiDataConnectionDto>,
    reports: Vec<AiDataReportDto>,
    body: AiDataAiQueryRequest,
) -> Result<Json<AiDataAiQueryResponse>, ApiError> {
    let (connection, report, source) = resolve_ai_context(&connections, &reports, &body)?;
    let catalog =
        runtime::connection_catalog(&connection, settings.default_timeout_secs).await.ok();
    let raw_model_response = plan_with_model(
        &state,
        &body,
        &connection,
        report.as_ref(),
        source.as_ref(),
        catalog.as_ref(),
    )
    .await?;
    let mut plan = parse_execution_plan(&raw_model_response)?;

    if plan.connection_id.trim().is_empty() {
        plan.connection_id = connection.id.clone();
    }
    if plan.connection_id != connection.id {
        // 模型输出不可信，必须重新核对连接范围，避免提示注入切换到其他连接。
        return Err(ApiError::bad_request("AI 规划返回了未授权的 connection_id"));
    }

    let query_request = AiDataQueryRequest {
        report_id: body.report_id.clone(),
        source_key: body.source_key.clone(),
        connection_id: Some(plan.connection_id.clone()),
        query_kind: Some(plan.query_kind.clone()),
        sql: plan.sql.clone(),
        count_sql: plan.count_sql.clone(),
        cube_query: plan.cube_query.clone(),
        http_method: Some(plan.http_method.clone()),
        http_path: plan.http_path.clone(),
        http_body: plan.http_body.clone(),
        params: body.params.clone(),
        page: Some(1),
        limit: body.limit,
        count: None,
        order_by: None,
        search_fields: None,
        template_fields: None,
        transformer_fields: None,
        template_code: None,
        search_condition: None,
        debug: Some(true),
    };
    let result = runtime::execute_query(&settings, &connections, &reports, query_request)
        .await
        .map_err(ApiError::bad_request)?;
    let answer = summarize_result(&state, &body.prompt, &plan, &result).await.ok();

    Ok(Json(AiDataAiQueryResponse {
        plan,
        result,
        answer,
        raw_model_response: Some(raw_model_response),
    }))
}

fn resolve_ai_context(
    connections: &[AiDataConnectionDto],
    reports: &[AiDataReportDto],
    body: &AiDataAiQueryRequest,
) -> Result<(AiDataConnectionDto, Option<AiDataReportDto>, Option<Value>), ApiError> {
    if let Some((report, source)) = runtime::report_source_context(
        reports,
        body.report_id.as_deref(),
        body.source_key.as_deref(),
    ) {
        let connection = connections
            .iter()
            .find(|item| item.id == source.connection_id)
            .cloned()
            .ok_or_else(|| ApiError::bad_request("AI-DATA 报表引用了不存在的连接"))?;
        return Ok((connection, Some(report), Some(json!(source))));
    }

    let connection_id = body
        .connection_id
        .as_deref()
        .ok_or_else(|| ApiError::bad_request("AI 查询必须提供 report_id 或 connection_id"))?;
    let connection = connections
        .iter()
        .find(|item| item.id == connection_id)
        .cloned()
        .ok_or_else(|| ApiError::bad_request("AI-DATA 连接不存在"))?;
    Ok((connection, None, None))
}

async fn plan_with_model(
    state: &AppState,
    body: &AiDataAiQueryRequest,
    connection: &AiDataConnectionDto,
    report: Option<&AiDataReportDto>,
    source: Option<&Value>,
    catalog: Option<&Value>,
) -> Result<String, ApiError> {
    let connection_json = serde_json::to_string_pretty(connection).unwrap_or_default();
    let report_json = report
        .map(|value| serde_json::to_string_pretty(value).unwrap_or_default())
        .unwrap_or_default();
    let source_json = source
        .map(|value| serde_json::to_string_pretty(value).unwrap_or_default())
        .unwrap_or_default();
    let catalog_json = catalog
        .map(|value| serde_json::to_string_pretty(value).unwrap_or_default())
        .unwrap_or_default();
    let params_json = serde_json::to_string_pretty(&body.params).unwrap_or_default();

    let system_prompt = format!(
        "你是 AI-DATA 查询规划器。\n\
只输出一个 JSON 对象，不要使用 Markdown，不要输出解释性文字。\n\
必须遵守以下约束：\n\
1. connection_id 必须使用 {connection_id}\n\
2. 如果连接是 sqlite/mysql/postgres，则 query_kind 只能是 sql，且 sql 必须是只读 SELECT/CTE。\n\
3. 如果连接是 cube，则 query_kind 只能是 cube，cube_query 必须是 Cube load API 的 query 对象。\n\
4. 如果连接是 http，则 query_kind 只能是 http，http_method 只能是 GET/POST/HEAD。\n\
5. 优先利用已有报表/source 模板和 schema_hint。\n\
JSON 结构如下：\n\
{{\n  \"connection_id\": \"{connection_id}\",\n  \"query_kind\": \"sql|cube|http\",\n  \"sql\": \"可选\",\n  \"count_sql\": \"可选\",\n  \"cube_query\": {{}} ,\n  \"http_method\": \"GET|POST|HEAD\",\n  \"http_path\": \"可选\",\n  \"http_body\": {{}} ,\n  \"explanation\": \"一句话说明\"\n}}",
        connection_id = connection.id,
    );
    let user_prompt = format!(
        "用户问题：\n{question}\n\n附加参数：\n{params}\n\n连接信息：\n{connection}\n\n报表信息：\n{report}\n\n数据源信息：\n{source}\n\n目录信息：\n{catalog}",
        question = body.prompt,
        params = runtime::truncate_text(&params_json, 8_000),
        connection = runtime::truncate_text(&connection_json, 8_000),
        report = runtime::truncate_text(&report_json, 8_000),
        source = runtime::truncate_text(&source_json, 8_000),
        catalog = runtime::truncate_text(&catalog_json, 12_000),
    );

    let messages = vec![ChatMessage::system(system_prompt), ChatMessage::user(user_prompt)];
    state
        .provider
        .chat_with_history(&messages, &state.model, state.temperature)
        .await
        .map_err(|e| ApiError::bad_request(format!("AI 规划失败: {e}")))
}

fn parse_execution_plan(raw: &str) -> Result<AiDataExecutionPlan, ApiError> {
    if let Ok(plan) = serde_json::from_str::<AiDataExecutionPlan>(raw.trim()) {
        return Ok(plan);
    }
    let values = crate::app::agent::agent::loop_::extract_json_values(raw);
    for value in values {
        if let Ok(plan) = serde_json::from_value::<AiDataExecutionPlan>(value) {
            return Ok(plan);
        }
    }
    Err(ApiError::bad_request("AI 规划结果不是合法的执行计划 JSON"))
}

async fn summarize_result(
    state: &AppState,
    question: &str,
    plan: &AiDataExecutionPlan,
    result: &vw_api_types::data::AiDataQueryResponse,
) -> Result<String, ApiError> {
    let sample = json!({
        "page": result.page,
        "items": result.items.iter().take(20).cloned().collect::<Vec<_>>()
    });
    let system_prompt = "你是 AI-DATA 结果总结器。根据用户问题和查询结果，用简洁中文回答。不要编造结果中不存在的信息。";
    let user_prompt = format!(
        "用户问题：\n{question}\n\n执行计划：\n{plan}\n\n查询结果样本：\n{sample}",
        plan = serde_json::to_string_pretty(plan).unwrap_or_default(),
        sample = serde_json::to_string_pretty(&sample).unwrap_or_default(),
    );
    let messages = vec![ChatMessage::system(system_prompt), ChatMessage::user(user_prompt)];
    state
        .provider
        .chat_with_history(&messages, &state.model, state.temperature)
        .await
        .map_err(|e| ApiError::bad_request(format!("AI 总结失败: {e}")))
}
