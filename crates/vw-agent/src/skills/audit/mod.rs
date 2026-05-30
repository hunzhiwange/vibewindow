//! 技能审计模块
//!
//! 本模块提供技能包的安全审计功能，用于在安装和加载技能前检测潜在的安全风险。
//!
//! # 核心功能
//!
//! - **技能目录审计**：扫描技能目录中的所有文件，检测不安全的模式和配置
//! - **Markdown 文件审计**：检查 Markdown 文件中的链接、命令片段和危险模式
//! - **TOML 清单审计**：验证技能清单文件中的工具定义和配置项
//! - **高风险模式检测**：识别提示注入、凭证收集、恶意命令等安全威胁
//!
//! # 安全策略
//!
//! 审计模块执行以下安全检查：
//!
//! - 禁止符号链接（防止路径遍历攻击）
//! - 禁止脚本文件（.sh、.bat、.ps1 等）
//! - 限制文件大小（防止 DoS 攻击）
//! - 检测 Shell 链接操作符（&&、||、; 等）
//! - 识别远程 Markdown 链接
//! - 防止路径逃逸
//! - 检测提示注入和钓鱼模式
//!
//! # 使用示例
//!
//! ```ignore
//! use std::path::Path;
//! use vibe_agent::skills::audit::audit_skill_directory;
//!
//! let skill_dir = Path::new("./skills/my-skill");
//! let report = audit_skill_directory(skill_dir)?;
//!
//! if report.is_clean() {
//!     println!("技能审计通过");
//! } else {
//!     println!("发现问题: {}", report.summary());
//! }
//! ```

use anyhow::{Context, Result, bail};
use std::path::Path;

mod manifest;
mod markdown;
mod report;
mod risk;
mod scan;
mod support;

pub use report::SkillAuditReport;

use self::markdown::audit_markdown_file;
use self::scan::{audit_path, collect_paths_depth_first};

/// 文本文件的最大允许字节数
///
/// 超过此大小的 Markdown 和 TOML 文件将被标记为过大，无法进行静态审计。
/// 默认值为 512 KB，平衡了审计精度和性能开销。
pub(crate) const MAX_TEXT_FILE_BYTES: u64 = 512 * 1024;

/// 审计技能目录
///
/// 对技能目录执行全面的安全审计，包括清单文件检查、文件类型验证、
/// 路径遍历检测和内容模式匹配。
///
/// # 参数
///
/// - `skill_dir`：技能目录的路径，必须是一个已存在的目录
///
/// # 返回值
///
/// 返回 `Result<SkillAuditReport>`，包含扫描结果和发现的问题列表。
///
/// # 错误
///
/// 在以下情况会返回错误：
/// - 技能目录不存在
/// - 路径不是目录
/// - 无法获取规范路径（权限问题或路径无效）
/// - 无法读取目录内容
///
/// # 审计规则
///
/// 1. **清单文件检查**：技能根目录必须包含 `SKILL.md` 或 `SKILL.toml`
/// 2. **符号链接检测**：禁止任何符号链接文件
/// 3. **脚本文件阻止**：禁止 .sh、.bat、.ps1 等脚本文件
/// 4. **文件大小限制**：Markdown 和 TOML 文件不能超过 512 KB
/// 5. **内容模式检测**：检查高风险命令模式和提示注入
/// 6. **链接验证**：验证 Markdown 中的本地链接和远程链接
///
/// # 示例
///
/// ```ignore
/// use std::path::Path;
///
/// let skill_path = Path::new("./skills/data-analysis");
/// match audit_skill_directory(skill_path) {
///     Ok(report) => {
///         if report.is_clean() {
///             println!("✓ 技能审计通过 (扫描 {} 个文件)", report.files_scanned);
///         } else {
///             eprintln!("✗ 技能审计失败:");
///             for finding in &report.findings {
///                 eprintln!("  - {}", finding);
///             }
///         }
///     }
///     Err(e) => eprintln!("审计过程出错: {}", e),
/// }
/// ```
pub fn audit_skill_directory(skill_dir: &Path) -> Result<SkillAuditReport> {
    if !skill_dir.exists() {
        bail!("Skill source does not exist: {}", skill_dir.display());
    }
    if !skill_dir.is_dir() {
        bail!("Skill source must be a directory: {}", skill_dir.display());
    }

    let canonical_root = skill_dir
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", skill_dir.display()))?;
    let mut report = SkillAuditReport::default();

    let has_manifest =
        canonical_root.join("SKILL.md").is_file() || canonical_root.join("SKILL.toml").is_file();
    if !has_manifest {
        report.findings.push(
            "Skill root must include SKILL.md or SKILL.toml for deterministic auditing."
                .to_string(),
        );
    }

    for path in collect_paths_depth_first(&canonical_root)? {
        report.files_scanned += 1;
        audit_path(&canonical_root, &path, &mut report)?;
    }

    Ok(report)
}

/// 审计开放式技能 Markdown 文件
///
/// 对仓库中的单个 Markdown 技能文件执行安全审计。
/// 主要用于检查以 Markdown 格式定义的开放式技能。
///
/// # 参数
///
/// - `path`：Markdown 技能文件的路径
/// - `repo_root`：仓库根目录路径，用于防止路径逃逸
///
/// # 返回值
///
/// 返回 `Result<SkillAuditReport>`，包含审计结果。
///
/// # 错误
///
/// 在以下情况会返回错误：
/// - Markdown 文件不存在
/// - 文件路径位于仓库根目录之外（路径逃逸）
/// - 无法获取规范路径
///
/// # 安全检查
///
/// - 验证文件路径在仓库范围内
/// - 检测高风险命令模式和提示注入
/// - 验证 Markdown 链接的安全性
///
/// # 示例
///
/// ```ignore
/// use std::path::Path;
///
/// let repo = Path::new("./my-repo");
/// let skill_file = repo.join("docs/skills/analysis.md");
///
/// let report = audit_open_skill_markdown(&skill_file, repo)?;
/// if !report.is_clean() {
///     println!("发现安全问题: {}", report.summary());
/// }
/// ```
pub fn audit_open_skill_markdown(path: &Path, repo_root: &Path) -> Result<SkillAuditReport> {
    if !path.exists() {
        bail!("Open-skill markdown not found: {}", path.display());
    }
    let canonical_repo = repo_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", repo_root.display()))?;
    let canonical_path = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", path.display()))?;
    if !canonical_path.starts_with(&canonical_repo) {
        bail!("Open-skill markdown escapes repository root: {}", path.display());
    }

    let mut report = SkillAuditReport { files_scanned: 1, findings: Vec::new() };
    audit_markdown_file(&canonical_repo, &canonical_path, &mut report)?;
    Ok(report)
}

#[cfg(test)]
use self::markdown::is_cross_skill_reference;

/// 单元测试模块
///
/// 测试代码位于同目录的 tests.rs 文件中。
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
