//! 终端交互式权限确认提示。

use std::io::{self, IsTerminal as _, Write as _};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionPromptOptions {
    pub prompt: String,
    pub header: Option<String>,
    pub details: Option<String>,
}

pub fn can_prompt_for_permission() -> bool {
    io::stdin().is_terminal() && io::stderr().is_terminal()
}

pub fn prompt_for_permission(options: &PermissionPromptOptions) -> io::Result<bool> {
    if !can_prompt_for_permission() {
        return Ok(false);
    }

    let mut stderr = io::stderr().lock();
    if let Some(header) = options.header.as_deref() {
        writeln!(stderr)?;
        writeln!(stderr, "{header}")?;
    }
    if let Some(details) = options.details.as_deref()
        && !details.trim().is_empty()
    {
        writeln!(stderr, "{details}")?;
    }
    write!(stderr, "{}", options.prompt)?;
    stderr.flush()?;
    drop(stderr);

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let normalized = answer.trim().to_ascii_lowercase();
    Ok(matches!(normalized.as_str(), "y" | "yes"))
}

#[cfg(test)]
#[path = "permission_prompt_tests.rs"]
mod permission_prompt_tests;
