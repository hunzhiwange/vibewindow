use super::{ApprovalRequest, ApprovalResponse, summarize_args};
use std::io::{self, BufRead, Write};

/// 显示审批提示并从标准输入读取用户响应。
pub(super) fn prompt_cli_interactive(request: &ApprovalRequest) -> ApprovalResponse {
    let summary = summarize_args(&request.arguments);

    eprintln!();
    eprintln!("🔧 Agent wants to execute: {}", request.tool_name);
    eprintln!("   {summary}");
    eprint!("   [Y]es / [N]o / [A]lways for {}: ", request.tool_name);
    let _ = io::stderr().flush();

    let stdin = io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_err() {
        return ApprovalResponse::No;
    }

    parse_cli_response(&line)
}

pub(super) fn parse_cli_response(line: &str) -> ApprovalResponse {
    match line.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => ApprovalResponse::Yes,
        "a" | "always" => ApprovalResponse::Always,
        _ => ApprovalResponse::No,
    }
}
