//! 换行注入 validator。
//!
//! shell 工具一次应执行一条明确命令；嵌入换行或回车可能隐藏第二条命令，因此阻断。

use super::{SecurityCategory, SecurityValidator, block, raw};
use crate::tools::shell::ast::ParsedCommand;

pub(super) struct NewlineInjectionValidator;

impl SecurityValidator for NewlineInjectionValidator {
    fn name(&self) -> &str {
        "newline_injection"
    }

    fn validate(&self, cmd: &ParsedCommand) -> Vec<crate::tools::shell::security::SecurityFinding> {
        let command = raw(cmd);
        if command.contains('\n') || command.contains('\r') {
            return vec![block(
                SecurityCategory::Injection,
                "Embedded newlines or carriage returns can inject additional shell commands",
                Some("Send one command per tool invocation"),
            )];
        }
        Vec::new()
    }
}
