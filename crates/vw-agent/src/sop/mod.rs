pub mod audit;
pub mod condition;
pub mod dispatch;
pub mod engine;
pub mod metrics;
pub mod types;

pub use audit::SopAuditLogger;
pub use engine::SopEngine;
pub use metrics::SopMetricsCollector;
#[allow(unused_imports)]
pub use types::{
    Sop, SopEvent, SopExecutionMode, SopPriority, SopRun, SopRunAction, SopRunStatus, SopStep,
    SopStepResult, SopStepStatus, SopTrigger, SopTriggerSource,
};

use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::warn;

use types::{SopManifest, SopMeta};

// ── SOP directory helpers ───────────────────────────────────────

/// Return the default SOPs directory: `<workspace>/sops`.
fn sops_dir(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join("sops")
}

/// Resolve the SOPs directory from config, falling back to workspace default.
pub fn resolve_sops_dir(workspace_dir: &Path, config_dir: Option<&str>) -> PathBuf {
    match config_dir {
        Some(dir) if !dir.is_empty() => {
            #[cfg(not(target_arch = "wasm32"))]
            let expanded = shellexpand::tilde(dir);
            #[cfg(target_arch = "wasm32")]
            let expanded = dir;
            PathBuf::from(<str as AsRef<str>>::as_ref(&expanded))
        }
        _ => sops_dir(workspace_dir),
    }
}

// ── SOP loading ─────────────────────────────────────────────────

/// Load all SOPs from the configured directory.
pub fn load_sops(
    workspace_dir: &Path,
    config_dir: Option<&str>,
    default_execution_mode: SopExecutionMode,
) -> Vec<Sop> {
    let dir = resolve_sops_dir(workspace_dir, config_dir);
    load_sops_from_directory(&dir, default_execution_mode)
}

/// Load SOPs from a specific directory. Each subdirectory may contain
/// `SOP.toml` (metadata + triggers) and `SOP.md` (procedure steps).
fn load_sops_from_directory(sops_dir: &Path, default_execution_mode: SopExecutionMode) -> Vec<Sop> {
    #[cfg(target_arch = "wasm32")]
    return Vec::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        if !sops_dir.exists() {
            return Vec::new();
        }

        let mut sops = Vec::new();

        let Ok(entries) = std::fs::read_dir(sops_dir) else {
            return sops;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let toml_path = path.join("SOP.toml");
            if !toml_path.exists() {
                continue;
            }

            match load_sop(&path, default_execution_mode) {
                Ok(sop) => sops.push(sop),
                Err(e) => {
                    warn!("Failed to load SOP from {}: {e}", path.display());
                }
            }
        }

        sops.sort_by(|a, b| a.name.cmp(&b.name));
        sops
    }
}

/// Load a single SOP from a directory containing SOP.toml and optionally SOP.md.
#[cfg(not(target_arch = "wasm32"))]
fn load_sop(sop_dir: &Path, default_execution_mode: SopExecutionMode) -> Result<Sop> {
    let toml_path = sop_dir.join("SOP.toml");
    let toml_content = std::fs::read_to_string(&toml_path)?;
    let manifest: SopManifest = toml::from_str(&toml_content)?;

    let md_path = sop_dir.join("SOP.md");
    let steps = if md_path.exists() {
        let md_content = std::fs::read_to_string(&md_path)?;
        parse_steps(&md_content)
    } else {
        Vec::new()
    };

    let SopMeta {
        name,
        description,
        version,
        priority,
        execution_mode,
        cooldown_secs,
        max_concurrent,
    } = manifest.sop;

    Ok(Sop {
        name,
        description,
        version,
        priority,
        execution_mode: execution_mode.unwrap_or(default_execution_mode),
        triggers: manifest.triggers,
        steps,
        cooldown_secs,
        max_concurrent,
        location: Some(sop_dir.to_path_buf()),
    })
}

// ── Markdown step parser ────────────────────────────────────────

/// Parse procedure steps from SOP.md content.
///
/// Expects a `## Steps` heading followed by numbered items (`1.`, `2.`, …).
/// Each item's first bold text (`**...**`) is the step title; the rest is body.
/// Sub-bullets `- tools:` and `- requires_confirmation: true` are parsed.
pub fn parse_steps(md: &str) -> Vec<SopStep> {
    let mut steps = Vec::new();
    let mut in_steps_section = false;
    let mut current_number: Option<u32> = None;
    let mut current_title = String::new();
    let mut current_body = String::new();
    let mut current_tools: Vec<String> = Vec::new();
    let mut current_requires_confirmation = false;

    for line in md.lines() {
        let trimmed = line.trim();

        // Detect ## Steps heading
        if trimmed.starts_with("## ") {
            if trimmed.eq_ignore_ascii_case("## steps") || trimmed.eq_ignore_ascii_case("## Steps")
            {
                in_steps_section = true;
                continue;
            }
            // Any other ## heading ends the steps section
            if in_steps_section {
                // Flush pending step
                flush_step(
                    &mut steps,
                    &mut current_number,
                    &mut current_title,
                    &mut current_body,
                    &mut current_tools,
                    &mut current_requires_confirmation,
                );
                in_steps_section = false;
            }
            continue;
        }

        if !in_steps_section {
            continue;
        }

        // Check for numbered item: `1.`, `2.`, etc.
        if let Some(rest) = parse_numbered_item(trimmed) {
            // Flush previous step
            flush_step(
                &mut steps,
                &mut current_number,
                &mut current_title,
                &mut current_body,
                &mut current_tools,
                &mut current_requires_confirmation,
            );

            let step_num = u32::try_from(steps.len()).unwrap_or(u32::MAX).saturating_add(1);
            current_number = Some(step_num);

            // Extract title from bold text: **title** — body
            if let Some((title, body)) = extract_bold_title(rest) {
                current_title = title;
                current_body = body;
            } else {
                current_title = rest.to_string();
                current_body = String::new();
            }
            current_tools = Vec::new();
            current_requires_confirmation = false;
            continue;
        }

        // Sub-bullet parsing (only when inside a step)
        if current_number.is_some() && trimmed.starts_with("- ") {
            let bullet = trimmed.trim_start_matches("- ").trim();
            if let Some(tools_str) = bullet.strip_prefix("tools:") {
                current_tools = tools_str
                    .split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
            } else if bullet.starts_with("requires_confirmation:") {
                if let Some(val) = bullet.strip_prefix("requires_confirmation:") {
                    current_requires_confirmation = val.trim().eq_ignore_ascii_case("true");
                }
            } else {
                // Continuation body line
                if !current_body.is_empty() {
                    current_body.push('\n');
                }
                current_body.push_str(trimmed);
            }
            continue;
        }

        // Continuation line for step body
        if current_number.is_some() && !trimmed.is_empty() {
            if !current_body.is_empty() {
                current_body.push('\n');
            }
            current_body.push_str(trimmed);
        }
    }

    // Flush final step
    flush_step(
        &mut steps,
        &mut current_number,
        &mut current_title,
        &mut current_body,
        &mut current_tools,
        &mut current_requires_confirmation,
    );

    steps
}

/// Flush accumulated step state into the steps vector.
fn flush_step(
    steps: &mut Vec<SopStep>,
    number: &mut Option<u32>,
    title: &mut String,
    body: &mut String,
    tools: &mut Vec<String>,
    requires_confirmation: &mut bool,
) {
    if let Some(n) = number.take() {
        steps.push(SopStep {
            number: n,
            title: std::mem::take(title),
            body: body.trim().to_string(),
            suggested_tools: std::mem::take(tools),
            requires_confirmation: *requires_confirmation,
        });
        *body = String::new();
        *requires_confirmation = false;
    }
}

/// Try to parse `N. rest` from a line, returning `rest` if successful.
fn parse_numbered_item(line: &str) -> Option<&str> {
    let dot_pos = line.find(". ")?;
    let prefix = &line[..dot_pos];
    if prefix.chars().all(|c| c.is_ascii_digit()) && !prefix.is_empty() {
        Some(line[dot_pos + 2..].trim())
    } else {
        None
    }
}

/// Extract `**title**` from the beginning of text, returning (title, rest).
fn extract_bold_title(text: &str) -> Option<(String, String)> {
    let start = text.find("**")?;
    let after_start = start + 2;
    let end = text[after_start..].find("**")?;
    let title = text[after_start..after_start + end].to_string();

    // Rest is everything after the closing ** and any separator (— or -)
    let rest_start = after_start + end + 2;
    let rest = text[rest_start..].trim();
    let rest = rest
        .strip_prefix("—")
        .or_else(|| rest.strip_prefix("–"))
        .or_else(|| rest.strip_prefix("-"))
        .unwrap_or(rest)
        .trim();

    Some((title, rest.to_string()))
}

// ── Validation ──────────────────────────────────────────────────

/// Validate a loaded SOP and return a list of warnings.
pub fn validate_sop(sop: &Sop) -> Vec<String> {
    let mut warnings = Vec::new();

    if sop.name.is_empty() {
        warnings.push("SOP name is empty".into());
    }
    if sop.description.is_empty() {
        warnings.push("SOP description is empty".into());
    }
    if sop.triggers.is_empty() {
        warnings.push("SOP has no triggers defined".into());
    }
    if sop.steps.is_empty() {
        warnings.push("SOP has no steps (missing or empty SOP.md)".into());
    }

    // Check step numbering continuity
    for (i, step) in sop.steps.iter().enumerate() {
        let expected = u32::try_from(i).unwrap_or(u32::MAX).saturating_add(1);
        if step.number != expected {
            warnings.push(format!("Step numbering gap: expected {expected}, got {}", step.number));
        }
        if step.title.is_empty() {
            warnings.push(format!("Step {} has an empty title", step.number));
        }
    }

    warnings
}

// ── CLI handler ─────────────────────────────────────────────────

/// Handle the `sop` CLI subcommand.
pub fn handle_command(
    command: SopCommands,
    config: &crate::app::agent::config::Config,
) -> Result<()> {
    let sops_dir_override = config.sop.sops_dir.as_deref();

    match command {
        SopCommands::List => {
            let sops = load_sops(
                &config.workspace_dir,
                sops_dir_override,
                config.sop.default_execution_mode,
            );
            if sops.is_empty() {
                println!("No SOPs found.");
                println!();
                println!(
                    "  Create one: mkdir -p {}",
                    config.workspace_dir.join("sops").join("my-sop").display()
                );
                println!("              # Add SOP.toml and SOP.md");
                println!();
                println!(
                    "  SOPs directory: {}",
                    resolve_sops_dir(&config.workspace_dir, sops_dir_override).display()
                );
            } else {
                println!("SOPs ({}):", sops.len());
                println!();
                for sop in &sops {
                    let triggers: Vec<String> =
                        sop.triggers.iter().map(ToString::to_string).collect();
                    println!(
                        "  {} {} [{}] — {}",
                        console::style(&sop.name).white().bold(),
                        console::style(format!("v{}", sop.version)).dim(),
                        console::style(&sop.priority).cyan(),
                        sop.description
                    );
                    println!(
                        "    Mode: {}  Steps: {}  Triggers: {}",
                        sop.execution_mode,
                        sop.steps.len(),
                        triggers.join(", ")
                    );
                    if sop.cooldown_secs > 0 {
                        println!("    Cooldown: {}s", sop.cooldown_secs);
                    }
                }
            }
            println!();
            Ok(())
        }

        SopCommands::Validate { name } => {
            let sops = load_sops(
                &config.workspace_dir,
                sops_dir_override,
                config.sop.default_execution_mode,
            );
            let matching: Vec<&Sop> = if let Some(ref name) = name {
                sops.iter().filter(|s| s.name == *name).collect()
            } else {
                sops.iter().collect()
            };

            if matching.is_empty() {
                if let Some(name) = name {
                    anyhow::bail!("SOP not found: {name}");
                }
                println!("No SOPs to validate.");
                return Ok(());
            }

            let mut any_warnings = false;
            for sop in &matching {
                let warnings = validate_sop(sop);
                if warnings.is_empty() {
                    println!("  {} {} — valid", console::style("✓").green().bold(), sop.name);
                } else {
                    any_warnings = true;
                    println!(
                        "  {} {} — {} warning(s):",
                        console::style("!").yellow().bold(),
                        sop.name,
                        warnings.len()
                    );
                    for w in &warnings {
                        println!("      {w}");
                    }
                }
            }
            println!();

            if any_warnings {
                anyhow::bail!("Validation completed with warnings");
            }
            Ok(())
        }

        SopCommands::Show { name } => {
            let sops = load_sops(
                &config.workspace_dir,
                sops_dir_override,
                config.sop.default_execution_mode,
            );
            let sop = sops
                .iter()
                .find(|s| s.name == name)
                .ok_or_else(|| anyhow::anyhow!("SOP not found: {name}"))?;

            println!("{} v{}", console::style(&sop.name).white().bold(), sop.version);
            println!("{}", sop.description);
            println!();
            println!("Priority:       {}", sop.priority);
            println!("Execution mode: {}", sop.execution_mode);
            println!("Cooldown:       {}s", sop.cooldown_secs);
            println!("Max concurrent: {}", sop.max_concurrent);
            println!();

            if !sop.triggers.is_empty() {
                println!("Triggers:");
                for trigger in &sop.triggers {
                    println!("  - {trigger}");
                }
                println!();
            }

            if !sop.steps.is_empty() {
                println!("Steps:");
                for step in &sop.steps {
                    let confirm_tag =
                        if step.requires_confirmation { " [requires confirmation]" } else { "" };
                    println!(
                        "  {}. {}{}",
                        step.number,
                        console::style(&step.title).bold(),
                        confirm_tag
                    );
                    if !step.body.is_empty() {
                        for line in step.body.lines() {
                            println!("     {line}");
                        }
                    }
                    if !step.suggested_tools.is_empty() {
                        println!("     Tools: {}", step.suggested_tools.join(", "));
                    }
                }
            }
            println!();
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SopCommands {
    List,
    Validate { name: Option<String> },
    Show { name: String },
}
