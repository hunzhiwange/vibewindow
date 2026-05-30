//! 提供任务看板消息处理过程中复用的局部辅助逻辑。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::*;

/// 执行 to_file_url 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn to_file_url(path: &std::path::Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/").replace(' ', "%20");
    if normalized.starts_with('/') {
        format!("file://{}", normalized)
    } else {
        format!("file:///{}", normalized)
    }
}

/// 执行 split_output_and_git_metadata 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn split_output_and_git_metadata(
    output: &str,
) -> (String, Option<String>, Option<String>, Option<String>, Option<String>) {
    let mut summary: Option<String> = None;
    let mut source_branch: Option<String> = None;
    let mut target_branch: Option<String> = None;
    let mut worktree_path: Option<String> = None;
    let mut body_lines: Vec<String> = Vec::new();

    for line in output.lines() {
        if let Some(value) = line.strip_prefix("__VW_GIT_SUMMARY__:") {
            let value = value.trim();
            if !value.is_empty() {
                summary = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("__VW_GIT_SOURCE_BRANCH__:") {
            let value = value.trim();
            if !value.is_empty() {
                source_branch = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("__VW_GIT_TARGET_BRANCH__:") {
            let value = value.trim();
            if !value.is_empty() {
                target_branch = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("__VW_GIT_WORKTREE_PATH__:") {
            let value = value.trim();
            if !value.is_empty() {
                worktree_path = Some(value.to_string());
            }
        } else {
            body_lines.push(line.to_string());
        }
    }

    let body = body_lines.join("\n").trim().to_string();
    (body, summary, source_branch, target_branch, worktree_path)
}

/// 执行 validate_ready_for_merge 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn validate_ready_for_merge(
    task: &Task,
    project_path: Option<&str>,
) -> Result<(), String> {
    let Some(project_path) = project_path else {
        return Err("缺少项目路径，无法执行合并".to_string());
    };

    let source_branch = task.merge_source_branch.as_deref().unwrap_or("").trim();
    if source_branch.is_empty() || source_branch == "HEAD" {
        return Err("缺少有效 source branch，无法自动合并".to_string());
    }

    let target_branch = task.merge_target_branch.as_deref().unwrap_or("").trim();
    if target_branch.is_empty() || target_branch == "HEAD" {
        return Err("缺少有效 target branch，无法自动合并".to_string());
    }

    if source_branch == target_branch {
        return Err(format!(
            "source branch 与 target branch 相同，无法自动合并: {}",
            source_branch
        ));
    }

    let selected_worktree_path = task.selected_worktree_path.as_deref().unwrap_or("").trim();
    if selected_worktree_path.is_empty() {
        return Err("缺少任务 worktree，无法自动合并".to_string());
    }

    if !crate::app::task::task_has_live_worktree(project_path, &task.id) {
        return Err(format!(
            "任务 worktree 已失效或已被回收，无法自动合并: {}",
            selected_worktree_path
        ));
    }

    Ok(())
}

/// 执行 truncate_prompt_diff 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn truncate_prompt_diff(diff: &str, max_chars: usize) -> String {
    let count = diff.chars().count();
    if count <= max_chars {
        return diff.to_string();
    }
    let truncated = diff.chars().take(max_chars).collect::<String>();
    format!("{}\n\n[DIFF_TRUNCATED total_chars={}]", truncated, count)
}

/// 表示 CodeReviewPromptContext 相关的应用状态或派生数据。
pub(crate) struct CodeReviewPromptContext {
    pub(crate) prompt: String,
}

/// 执行 build_code_review_prompt_context 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn build_code_review_prompt_context(
    task: &Task,
    project_path: &str,
    max_diff_chars: Option<usize>,
) -> Result<CodeReviewPromptContext, String> {
    let (resolved_source_branch, resolved_target_branch, commit1, commit2, diff) =
        crate::app::task::build_review_diff_context(
            project_path,
            task.merge_source_branch.as_deref(),
            task.merge_target_branch.as_deref(),
        )?;
    let prompt_diff = max_diff_chars
        .map(|limit| truncate_prompt_diff(&diff, limit))
        .unwrap_or_else(|| diff.clone());
    let prompt = format!(
        "你是代码 Review 专家。请基于下面给出的 git diff commit1 commit2 结果做审查，不要自行假设额外改动。\nsource_branch={}\ntarget_branch={}\nresolved_source_ref={}\nresolved_target_ref={}\ncommit1={}\ncommit2={}\ntask_id={}\n\n以下是命令 `git diff commit1 commit2` 的输出：\n{}\n\n请先给出审查结论，再给出可执行建议。\n如果不建议合并，必须输出精确 JSON：{{\"status\":\"error\",\"error\":\"风险太高或者其他什么原因\"}}。\n如果建议合并，请先给出简短 review 结论，再给出精确 JSON：{{\"status\":\"ok\"}}。",
        task.merge_source_branch.as_deref().unwrap_or("待合并分支"),
        task.merge_target_branch.as_deref().unwrap_or("目标分支"),
        resolved_source_branch,
        resolved_target_branch,
        commit1,
        commit2,
        task.id,
        prompt_diff
    );
    Ok(CodeReviewPromptContext { prompt })
}

/// 执行 build_code_review_prompt 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn build_code_review_prompt(task: &Task, project_path: &str) -> Result<String, String> {
    Ok(build_code_review_prompt_context(task, project_path, Some(120_000))?.prompt)
}

/// 执行 build_task_prompt_from_input 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn build_task_prompt_from_input(
    raw: &str,
    _priority: Option<u32>,
    _subtasks: &[String],
) -> String {
    raw.trim().to_string()
}

/// 执行 parse_review_decision 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(crate) fn parse_review_decision(output: &str) -> Result<(), String> {
    fn from_json(value: &serde_json::Value) -> Option<Result<(), String>> {
        let status = value.get("status").and_then(|v| v.as_str())?;
        if status == "error" {
            let reason = value
                .get("error")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("风险太高")
                .to_string();
            return Some(Err(reason));
        }
        if status == "ok" {
            return Some(Ok(()));
        }
        None
    }

    fn collect_json_objects(text: &str) -> Vec<String> {
        let mut objects = Vec::new();
        let mut start: Option<usize> = None;
        let mut depth = 0usize;
        for (idx, ch) in text.char_indices() {
            if ch == '{' {
                if depth == 0 {
                    start = Some(idx);
                }
                depth = depth.saturating_add(1);
                continue;
            }
            if ch == '}' {
                if depth == 0 {
                    continue;
                }
                depth = depth.saturating_sub(1);
                if depth == 0
                    && let Some(begin) = start.take()
                {
                    objects.push(text[begin..=idx].to_string());
                }
            }
        }
        objects
    }

    fn parse_candidate_json(text: &str) -> Option<serde_json::Value> {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
            return Some(value);
        }
        let unescaped = text.replace("\\\"", "\"");
        if unescaped != text
            && let Ok(value) = serde_json::from_str::<serde_json::Value>(&unescaped)
        {
            return Some(value);
        }
        None
    }

    fn find_decision_in_value(value: &serde_json::Value) -> Option<Result<(), String>> {
        if let Some(decision) = from_json(value) {
            return Some(decision);
        }
        match value {
            serde_json::Value::Array(items) => items.iter().rev().find_map(find_decision_in_value),
            serde_json::Value::Object(map) => map.values().find_map(find_decision_in_value),
            serde_json::Value::String(text) => find_decision_in_text(text),
            _ => None,
        }
    }

    fn find_decision_in_text(text: &str) -> Option<Result<(), String>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(value) = parse_candidate_json(trimmed)
            && let Some(decision) = find_decision_in_value(&value)
        {
            return Some(decision);
        }
        for line in trimmed.lines().rev() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(value) = parse_candidate_json(line)
                && let Some(decision) = find_decision_in_value(&value)
            {
                return Some(decision);
            }
        }
        for json_text in collect_json_objects(trimmed).into_iter().rev() {
            if let Some(value) = parse_candidate_json(&json_text)
                && let Some(decision) = find_decision_in_value(&value)
            {
                return Some(decision);
            }
        }
        None
    }

    if let Some(decision) = find_decision_in_text(output) {
        return decision;
    }

    if output.contains("\"status\":\"error\"") {
        return Err("风险太高".to_string());
    }

    Err("审核结果缺少可解析结论".to_string())
}
#[cfg(test)]
#[path = "review_tests.rs"]
mod review_tests;
