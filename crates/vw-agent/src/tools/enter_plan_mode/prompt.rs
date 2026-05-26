//! 计划模式工具返回给模型的固定提示文本。
//!
//! 这些文本集中在独立模块中，避免工具执行逻辑和长提示词交织；调用方只通过
//! 函数获取消息、步骤列表和最终结果文本。

/// 进入计划模式工具的工具描述。
///
/// 该描述会暴露给模型，用于判断何时主动进入计划模式，以及进入后必须保持
/// 只读探索。它不参与运行时权限判断。
pub(super) const ENTER_PLAN_MODE_DESCRIPTION: &str = concat!(
    "Use this tool proactively when you're about to start a non-trivial implementation task. ",
    "Getting alignment on the approach before writing code prevents wasted effort and ensures the implementation matches the user's intent. ",
    "This tool transitions you into plan mode where you should explore the codebase and design an implementation approach before execution.\n\n",
    "## When to Use This Tool\n\n",
    "Prefer using EnterPlanMode for implementation tasks unless they're obviously simple. Use it when any of these conditions apply:\n\n",
    "1. New feature implementation that changes product behavior or user workflows\n",
    "2. Multiple valid technical approaches with meaningful trade-offs\n",
    "3. Code modifications that restructure existing behavior or architecture\n",
    "4. Multi-file changes that likely span more than two or three files\n",
    "5. Unclear requirements where exploration is needed before coding\n",
    "6. Tasks where user preferences materially affect the implementation approach\n\n",
    "## When NOT to Use This Tool\n\n",
    "Skip EnterPlanMode for straightforward tasks such as typos, isolated one-line fixes, obvious small bugs, or narrowly scoped edits with a single clear implementation path.\n\n",
    "## What Happens in Plan Mode\n\n",
    "In plan mode, you should:\n",
    "1. Thoroughly explore the codebase using read-only tools such as Glob, Grep, and Read\n",
    "2. Understand existing patterns, architecture, and neighboring implementations\n",
    "3. Consider multiple approaches and their trade-offs\n",
    "4. Use AskUserQuestion if you need clarification while shaping the approach\n",
    "5. Design a concrete implementation strategy\n",
    "6. Use VerifyPlanExecution and ExitPlanMode before moving into execution\n\n",
    "Remember: plan mode is a read-only exploration and planning phase. Do not write or edit files until the plan is ready to execute."
);

const ENTER_PLAN_MODE_MESSAGE: &str = "Entered plan mode. You should now focus on exploring the codebase and designing an implementation approach.";
const ENTER_PLAN_MODE_ALREADY_ACTIVE_MESSAGE: &str = "Plan mode is already active. Continue exploring the codebase and refining the implementation approach.";

const ENTER_PLAN_MODE_INSTRUCTIONS: [&str; 6] = [
    "Thoroughly explore the codebase to understand existing patterns.",
    "Identify similar features and architectural approaches.",
    "Consider multiple implementation approaches and their trade-offs.",
    "Use AskUserQuestion if you need to clarify the approach.",
    "Design a concrete implementation strategy.",
    "When ready, use VerifyPlanExecution and ExitPlanMode before execution.",
];

/// 返回进入计划模式时的状态消息。
///
/// 参数 `already_active` 表示会话在本次调用前是否已经处于计划模式。返回值为
/// 静态字符串；该函数不会失败。
pub(super) fn enter_plan_mode_message(already_active: bool) -> &'static str {
    if already_active { ENTER_PLAN_MODE_ALREADY_ACTIVE_MESSAGE } else { ENTER_PLAN_MODE_MESSAGE }
}

/// 返回计划模式中的步骤清单。
///
/// 返回值为静态切片，调用方只读使用；该函数不会失败。
pub(super) fn enter_plan_mode_instruction_lines() -> &'static [&'static str] {
    &ENTER_PLAN_MODE_INSTRUCTIONS
}

/// 生成面向模型的计划模式结果文本。
///
/// 参数 `already_active` 决定开头消息是否提示“已处于计划模式”。返回值包含
/// 步骤清单和只读提醒；该函数只做字符串拼接，不产生错误。
pub(super) fn enter_plan_mode_result_text(already_active: bool) -> String {
    let message = enter_plan_mode_message(already_active);
    let steps = ENTER_PLAN_MODE_INSTRUCTIONS
        .iter()
        .enumerate()
        .map(|(index, line)| format!("{}. {}", index + 1, line))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "{message}\n\nIn plan mode, you should:\n{steps}\n\nRemember: DO NOT write or edit any files yet. This is a read-only exploration and planning phase."
    )
}
#[cfg(test)]
#[path = "prompt_tests.rs"]
mod prompt_tests;
