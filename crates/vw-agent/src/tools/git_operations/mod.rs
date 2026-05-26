//! Git 操作工具
//!
//! 提供结构化的 Git 仓库管理功能，输出 JSON 格式结果。
//! 支持常用的 Git 操作（status、diff、log、add、commit 等），具有参数安全检查。
//!
//! # 主要功能
//!
//! - **读取操作**：status（状态）、diff（差异）、log（日志）、show（显示）、branch（分支）
//! - **写入操作**：add（暂存）、commit（提交）、checkout（切换分支）、stash（储藏）
//! - **安全特性**：参数净化、权限检查、速率限制
//!
//! # 安全机制
//!
//! 1. 所有 Git 参数都经过净化处理，防止命令注入攻击
//! 2. 写入操作需要适当的自主权限级别
//! 3. 操作受到安全策略的速率限制约束
//!
//! # 示例
//!
//! ```ignore
//! use std::sync::Arc;
//! use vibe_window::app::agent::tools::git_operations::GitOperationsTool;
//! use vibe_window::app::agent::security::SecurityPolicy;
//!
//! let security = Arc::new(SecurityPolicy::default());
//! let tool = GitOperationsTool::new(security, std::path::PathBuf::from("."));
//!
//! // 执行 git status
//! let result = tool.execute(json!({"operation": "status"})).await?;
//! ```

use super::traits::{Tool, ToolResult};
use crate::app::agent::security::{AutonomyLevel, SecurityPolicy};
use crate::app::agent::shell::git_tokio_command;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

/// Git 操作工具
///
/// 提供安全、结构化的 Git 仓库管理功能，所有操作输出解析后的 JSON 格式结果。
///
/// # 架构设计
///
/// 该工具实现了 `Tool` trait，与安全策略深度集成：
/// - 读取操作（status、diff、log）可在只读模式下执行
/// - 写入操作（commit、add、checkout、stash）需要更高权限级别
/// - 所有操作都受速率限制约束
///
/// # 字段说明
///
/// - `security`：安全策略引用，控制操作权限和速率限制
/// - `workspace_dir`：工作目录路径，所有 Git 命令在此目录下执行
///
/// # 使用场景
///
/// - 代理系统需要自动管理 Git 仓库时
/// - 需要结构化的 Git 操作结果用于后续处理
/// - 需要在受限权限环境下执行 Git 操作
pub struct GitOperationsTool {
    security: Arc<SecurityPolicy>,
    workspace_dir: std::path::PathBuf,
}

impl GitOperationsTool {
    /// 创建新的 Git 操作工具实例
    ///
    /// # 参数
    ///
    /// - `security`：安全策略的原子引用，用于权限检查和速率限制
    /// - `workspace_dir`：Git 仓库的工作目录路径
    ///
    /// # 返回值
    ///
    /// 返回配置好的 `GitOperationsTool` 实例
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let security = Arc::new(SecurityPolicy::default());
    /// let tool = GitOperationsTool::new(security, PathBuf::from("/path/to/repo"));
    /// ```
    pub fn new(security: Arc<SecurityPolicy>, workspace_dir: std::path::PathBuf) -> Self {
        Self { security, workspace_dir }
    }

    /// 净化 Git 参数以防止注入攻击
    ///
    /// 对用户提供的 Git 参数进行安全检查，阻止可能导致命令注入或
    /// 绕过安全限制的危险选项。
    ///
    /// # 被阻止的危险模式
    ///
    /// - `--exec=`：允许执行任意命令
    /// - `--upload-pack=` / `--receive-pack=`：允许指定外部程序
    /// - `--pager=` / `--editor=`：允许指定外部程序
    /// - `--no-verify`：跳过 pre-commit 钩子（安全风险）
    /// - `-c` / `-c=`：Git 配置注入
    /// - Shell 元字符：`$(`、`` ` ``、`|`、`;`、`>`
    ///
    /// # 参数
    ///
    /// - `args`：需要净化的参数字符串（空格分隔）
    ///
    /// # 返回值
    ///
    /// - `Ok(Vec<String>)`：净化后的参数向量
    /// - `Err(anyhow::Error)`：发现危险参数时返回错误
    ///
    /// # 安全考虑
    ///
    /// 此方法采用白名单思维，只允许已知安全的参数模式通过。
    /// 任何可疑的模式都会被拒绝，即使可能存在误报。
    fn sanitize_git_args(&self, args: &str) -> anyhow::Result<Vec<String>> {
        let mut result = Vec::new();
        for arg in args.split_whitespace() {
            // 阻止可能导致命令注入的危险 Git 选项
            let arg_lower = arg.to_lowercase();
            if arg_lower.starts_with("--exec=")
                || arg_lower.starts_with("--upload-pack=")
                || arg_lower.starts_with("--receive-pack=")
                || arg_lower.starts_with("--pager=")
                || arg_lower.starts_with("--editor=")
                || arg_lower == "--no-verify"
                || arg_lower.contains("$(")
                || arg_lower.contains('`')
                || arg.contains('|')
                || arg.contains(';')
                || arg.contains('>')
            {
                anyhow::bail!("Blocked potentially dangerous git argument: {arg}");
            }
            // 阻止 `-c` 配置注入（精确匹配或 `-c=...` 前缀）
            // 这不会误报 `--cached` 或 `-cached`
            if arg_lower == "-c" || arg_lower.starts_with("-c=") {
                anyhow::bail!("Blocked potentially dangerous git argument: {arg}");
            }
            result.push(arg.to_string());
        }
        Ok(result)
    }

    /// 检查操作是否需要写入权限
    ///
    /// 某些 Git 操作会修改仓库状态，需要更高的自主权限级别才能执行。
    ///
    /// # 参数
    ///
    /// - `operation`：Git 操作名称（如 "commit"、"add" 等）
    ///
    /// # 返回值
    ///
    /// - `true`：该操作需要写入权限
    /// - `false`：该操作是只读的
    fn requires_write_access(&self, operation: &str) -> bool {
        matches!(operation, "commit" | "add" | "checkout" | "stash" | "reset" | "revert")
    }

    /// 检查操作是否为只读
    ///
    /// 只读操作不会修改仓库状态，可以在受限权限环境下执行。
    ///
    /// # 参数
    ///
    /// - `operation`：Git 操作名称
    ///
    /// # 返回值
    ///
    /// - `true`：该操作是只读的
    /// - `false`：该操作可能会修改仓库状态
    fn is_read_only(&self, operation: &str) -> bool {
        matches!(operation, "status" | "diff" | "log" | "show" | "branch" | "rev-parse")
    }

    /// 执行 Git 命令
    ///
    /// 在工作目录中异步执行指定的 Git 命令，捕获标准输出和错误。
    ///
    /// # 参数
    ///
    /// - `args`：Git 命令参数数组
    ///
    /// # 返回值
    ///
    /// - `Ok(String)`：命令的标准输出内容
    /// - `Err(anyhow::Error)`：命令执行失败，错误消息来自标准错误输出
    ///
    /// # 错误处理
    ///
    /// 当 Git 命令返回非零退出码时，会将标准错误输出作为错误消息返回。
    async fn run_git_command(&self, args: &[&str]) -> anyhow::Result<String> {
        let output =
            git_tokio_command().args(args).current_dir(&self.workspace_dir).output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git command failed: {stderr}");
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 执行 Git status 操作
    ///
    /// 获取仓库当前状态，包括已暂存的更改、未暂存的更改和未跟踪的文件。
    /// 使用 `--porcelain=2` 格式以获得机器可解析的输出。
    ///
    /// # 参数
    ///
    /// - `_args`：未使用的参数（保留用于未来扩展）
    ///
    /// # 返回值
    ///
    /// 返回包含以下字段的 JSON 对象：
    /// - `branch`：当前分支名称
    /// - `staged`：已暂存的更改数组，每项包含 `path` 和 `status`
    /// - `unstaged`：未暂存的更改数组
    /// - `untracked`：未跟踪的文件路径数组
    /// - `clean`：布尔值，指示工作目录是否干净
    ///
    /// # 输出格式说明
    ///
    /// `--porcelain=2` 格式：
    /// - `# branch.head <name>`：分支信息行
    /// - `1 <xy> <sub> <mH> <mI> <mW> <hH> <hI> <path>`：普通更改条目
    /// - `? <path>`：未跟踪文件
    async fn git_status(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let output = self.run_git_command(&["status", "--porcelain=2", "--branch"]).await?;

        // 解析 git status 输出为结构化格式
        let mut result = serde_json::Map::new();
        let mut branch = String::new();
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();

        for line in output.lines() {
            if line.starts_with("# branch.head ") {
                // 解析分支名称
                branch = line.trim_start_matches("# branch.head ").to_string();
            } else if let Some(rest) = line.strip_prefix("1 ") {
                // 普通更改条目：1 <xy> <sub> <mH> <mI> <mW> <hH> <hI> <path>
                let mut parts = rest.splitn(3, ' ');
                if let (Some(staging), Some(path)) = (parts.next(), parts.next()) {
                    if !staging.is_empty() {
                        // 第一个字符表示暂存区状态
                        let status_char = staging.chars().next().unwrap_or(' ');
                        if status_char != '.' && status_char != ' ' {
                            staged.push(json!({"path": path, "status": status_char}));
                        }
                        // 第二个字符表示工作目录状态
                        let status_char = staging.chars().nth(1).unwrap_or(' ');
                        if status_char != '.' && status_char != ' ' {
                            unstaged.push(json!({"path": path, "status": status_char}));
                        }
                    }
                }
            } else if let Some(rest) = line.strip_prefix("? ") {
                // 未跟踪文件
                untracked.push(rest.to_string());
            }
        }

        result.insert("branch".to_string(), json!(branch));
        result.insert("staged".to_string(), json!(staged));
        result.insert("unstaged".to_string(), json!(unstaged));
        result.insert("untracked".to_string(), json!(untracked));
        result.insert(
            "clean".to_string(),
            json!(staged.is_empty() && unstaged.is_empty() && untracked.is_empty()),
        );

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&result).unwrap_or_default(),
            error: None,
        })
    }

    /// 执行 Git diff 操作
    ///
    /// 显示文件差异，将输出解析为结构化的代码块（hunks）格式。
    ///
    /// # 参数
    ///
    /// - `args`：JSON 对象，包含：
    ///   - `files`（可选）：要对比的文件路径，默认为 "."（所有文件）
    ///   - `cached`（可选）：布尔值，是否显示暂存的更改
    ///
    /// # 返回值
    ///
    /// 返回包含以下字段的 JSON 对象：
    /// - `hunks`：差异块数组，每块包含文件名、头部信息和行列表
    /// - `file_count`：涉及的文件数量
    ///
    /// # 行类型说明
    ///
    /// 每行包含 `type` 字段：
    /// - `add`：新增行（以 `+` 开头）
    /// - `delete`：删除行（以 `-` 开头）
    /// - `context`：上下文行
    async fn git_diff(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let files = args.get("files").and_then(|v| v.as_str()).unwrap_or(".");
        let cached = args.get("cached").and_then(|v| v.as_bool()).unwrap_or(false);

        // 对文件参数进行注入模式验证
        self.sanitize_git_args(files)?;

        let mut git_args = vec!["diff", "--unified=3"];
        if cached {
            git_args.push("--cached");
        }
        git_args.push("--");
        git_args.push(files);

        let output = self.run_git_command(&git_args).await?;

        // 将 diff 输出解析为结构化的代码块
        let mut result = serde_json::Map::new();
        let mut hunks = Vec::new();
        let mut current_file = String::new();
        let mut current_hunk = serde_json::Map::new();
        let mut lines = Vec::new();

        for line in output.lines() {
            if line.starts_with("diff --git ") {
                // 新文件的差异开始，保存之前的块
                if !lines.is_empty() {
                    current_hunk.insert("lines".to_string(), json!(lines));
                    if !current_hunk.is_empty() {
                        hunks.push(serde_json::Value::Object(current_hunk.clone()));
                    }
                    lines = Vec::new();
                    current_hunk = serde_json::Map::new();
                }
                // 解析文件路径：diff --git a/path b/path
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    current_file = parts[3].trim_start_matches("b/").to_string();
                    current_hunk.insert("file".to_string(), json!(current_file));
                }
            } else if line.starts_with("@@ ") {
                // 新的差异块开始，保存之前的行
                if !lines.is_empty() {
                    current_hunk.insert("lines".to_string(), json!(lines));
                    if !current_hunk.is_empty() {
                        hunks.push(serde_json::Value::Object(current_hunk.clone()));
                    }
                    lines = Vec::new();
                    current_hunk = serde_json::Map::new();
                    current_hunk.insert("file".to_string(), json!(current_file));
                }
                // 记录差异块头部信息
                current_hunk.insert("header".to_string(), json!(line));
            } else if !line.is_empty() {
                // 将行分类为添加、删除或上下文
                lines.push(json!({
                    "text": line,
                    "type": if line.starts_with('+') { "add" }
                           else if line.starts_with('-') { "delete" }
                           else { "context" }
                }));
            }
        }

        // 保存最后一个块
        if !lines.is_empty() {
            current_hunk.insert("lines".to_string(), json!(lines));
            if !current_hunk.is_empty() {
                hunks.push(serde_json::Value::Object(current_hunk));
            }
        }

        result.insert("hunks".to_string(), json!(hunks));
        result.insert("file_count".to_string(), json!(hunks.len()));

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&result).unwrap_or_default(),
            error: None,
        })
    }

    /// 执行 Git log 操作
    ///
    /// 获取提交历史记录，返回结构化的提交信息列表。
    ///
    /// # 参数
    ///
    /// - `args`：JSON 对象，包含：
    ///   - `limit`（可选）：返回的提交数量，默认 10，最大 1000
    ///
    /// # 返回值
    ///
    /// 返回包含 `commits` 数组的 JSON 对象，每个提交包含：
    /// - `hash`：完整提交哈希
    /// - `author`：作者名称
    /// - `email`：作者邮箱
    /// - `date`：提交日期（ISO 格式）
    /// - `message`：提交消息
    ///
    /// # 安全限制
    ///
    /// 最大限制为 1000 条记录，防止资源耗尽。
    async fn git_log(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let limit_raw = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10);
        // 限制最大值为 1000，防止资源耗尽
        let limit = usize::try_from(limit_raw).unwrap_or(usize::MAX).min(1000);
        let limit_str = limit.to_string();

        let output = self
            .run_git_command(&[
                "log",
                &format!("-{limit_str}"),
                "--pretty=format:%H|%an|%ae|%ad|%s",
                "--date=iso",
            ])
            .await?;

        let mut commits = Vec::new();

        // 解析格式化的日志输出
        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                commits.push(json!({
                    "hash": parts[0],
                    "author": parts[1],
                    "email": parts[2],
                    "date": parts[3],
                    "message": parts[4]
                }));
            }
        }

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&json!({ "commits": commits }))
                .unwrap_or_default(),
            error: None,
        })
    }

    /// 执行 Git branch 操作
    ///
    /// 列出所有本地分支，标识当前分支。
    ///
    /// # 参数
    ///
    /// - `_args`：未使用的参数（保留用于未来扩展）
    ///
    /// # 返回值
    ///
    /// 返回包含以下字段的 JSON 对象：
    /// - `current`：当前分支名称
    /// - `branches`：分支数组，每项包含 `name` 和 `current` 布尔值
    async fn git_branch(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let output = self.run_git_command(&["branch", "--format=%(refname:short)|%(HEAD)"]).await?;

        let mut branches = Vec::new();
        let mut current = String::new();

        // 解析分支列表输出
        for line in output.lines() {
            if let Some((name, head)) = line.split_once('|') {
                let is_current = head == "*";
                if is_current {
                    current = name.to_string();
                }
                branches.push(json!({
                    "name": name,
                    "current": is_current
                }));
            }
        }

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&json!({
                "current": current,
                "branches": branches
            }))
            .unwrap_or_default(),
            error: None,
        })
    }

    /// 截断提交消息
    ///
    /// 将过长的提交消息截断到 2000 个字符以内，添加省略号后缀。
    ///
    /// # 参数
    ///
    /// - `message`：原始提交消息
    ///
    /// # 返回值
    ///
    /// 如果消息超过 2000 字符，返回前 1997 字符加 "..."；否则返回原消息。
    fn truncate_commit_message(message: &str) -> String {
        if message.chars().count() > 2000 {
            format!("{}...", message.chars().take(1997).collect::<String>())
        } else {
            message.to_string()
        }
    }

    /// 执行 Git commit 操作
    ///
    /// 创建新的提交，包含当前暂存的更改。
    ///
    /// # 参数
    ///
    /// - `args`：JSON 对象，包含：
    ///   - `message`（必需）：提交消息
    ///
    /// # 返回值
    ///
    /// - 成功：`success` 为 true，`output` 包含提交消息
    /// - 失败：`success` 为 false，`error` 包含错误信息
    ///
    /// # 安全处理
    ///
    /// 1. 移除空行和首尾空白
    /// 2. 验证消息非空
    /// 3. 限制消息长度（最大 2000 字符）
    async fn git_commit(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'message' parameter"))?;

        // 净化提交消息：移除空行和首尾空白
        let sanitized = message
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        if sanitized.is_empty() {
            anyhow::bail!("Commit message cannot be empty");
        }

        // 限制消息长度
        let message = Self::truncate_commit_message(&sanitized);

        let output = self.run_git_command(&["commit", "-m", &message]).await;

        match output {
            Ok(_) => Ok(ToolResult {
                success: true,
                output: format!("Committed: {message}"),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Commit failed: {e}")),
            }),
        }
    }

    /// 执行 Git add 操作
    ///
    /// 将指定的文件或路径添加到暂存区。
    ///
    /// # 参数
    ///
    /// - `args`：JSON 对象，包含：
    ///   - `paths`（必需）：要暂存的文件路径（支持通配符）
    ///
    /// # 返回值
    ///
    /// - 成功：`success` 为 true，`output` 包含已暂存的路径
    /// - 失败：`success` 为 false，`error` 包含错误信息
    ///
    /// # 安全检查
    ///
    /// 路径参数会经过注入模式验证。
    async fn git_add(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let paths = args
            .get("paths")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'paths' parameter"))?;

        // 对路径参数进行注入模式验证
        self.sanitize_git_args(paths)?;

        let output = self.run_git_command(&["add", "--", paths]).await;

        match output {
            Ok(_) => {
                Ok(ToolResult { success: true, output: format!("Staged: {paths}"), error: None })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Add failed: {e}")),
            }),
        }
    }

    /// 执行 Git checkout 操作
    ///
    /// 切换到指定的分支。
    ///
    /// # 参数
    ///
    /// - `args`：JSON 对象，包含：
    ///   - `branch`（必需）：目标分支名称
    ///
    /// # 返回值
    ///
    /// - 成功：`success` 为 true，`output` 包含切换信息
    /// - 失败：`success` 为 false，`error` 包含错误信息
    ///
    /// # 安全限制
    ///
    /// 分支名称不能包含以下字符：
    /// - `@`：可能引用特殊引用
    /// - `^`：父引用语法
    /// - `~`：祖先引用语法
    async fn git_checkout(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let branch = args
            .get("branch")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'branch' parameter"))?;

        // 净化分支名称
        let sanitized = self.sanitize_git_args(branch)?;

        // 验证只有一个参数（分支名称）
        if sanitized.is_empty() || sanitized.len() > 1 {
            anyhow::bail!("Invalid branch specification");
        }

        let branch_name = &sanitized[0];

        // 阻止危险的分支名称模式
        if branch_name.contains('@') || branch_name.contains('^') || branch_name.contains('~') {
            anyhow::bail!("Branch name contains invalid characters");
        }

        let output = self.run_git_command(&["checkout", branch_name]).await;

        match output {
            Ok(_) => Ok(ToolResult {
                success: true,
                output: format!("Switched to branch: {branch_name}"),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Checkout failed: {e}")),
            }),
        }
    }

    /// 执行 Git stash 操作
    ///
    /// 管理工作目录的临时储藏。
    ///
    /// # 参数
    ///
    /// - `args`：JSON 对象，包含：
    ///   - `action`（可选）：储藏操作类型，默认为 "push"
    ///     - `push` / `save`：创建新储藏
    ///     - `pop`：应用并删除最近的储藏
    ///     - `list`：列出所有储藏
    ///     - `drop`：删除指定储藏
    ///   - `index`（可选）：用于 `drop` 操作的储藏索引
    ///
    /// # 返回值
    ///
    /// - 成功：`success` 为 true，`output` 包含操作结果
    /// - 失败：`success` 为 false，`error` 包含错误信息
    ///
    /// # 注意事项
    ///
    /// `push` 操作会使用 "auto-stash" 作为默认消息。
    async fn git_stash(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("push");

        let output = match action {
            "push" | "save" => self.run_git_command(&["stash", "push", "-m", "auto-stash"]).await,
            "pop" => self.run_git_command(&["stash", "pop"]).await,
            "list" => self.run_git_command(&["stash", "list"]).await,
            "drop" => {
                // 解析并验证储藏索引
                let index_raw = args.get("index").and_then(|v| v.as_u64()).unwrap_or(0);
                let index = i32::try_from(index_raw)
                    .map_err(|_| anyhow::anyhow!("stash index too large: {index_raw}"))?;
                self.run_git_command(&["stash", "drop", &format!("stash@{{{index}}}")]).await
            }
            _ => anyhow::bail!("Unknown stash action: {action}. Use: push, pop, list, drop"),
        };

        match output {
            Ok(out) => Ok(ToolResult { success: true, output: out, error: None }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Stash {action} failed: {e}")),
            }),
        }
    }
}

/// Tool trait 实现
///
/// 为 `GitOperationsTool` 实现 `Tool` trait，使其可以被工具系统调用。
/// 该实现包含条件编译属性以支持 WASM 目标平台。
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for GitOperationsTool {
    /// 返回工具名称
    ///
    /// 工具名称用于在系统中标识和调用此工具。
    fn name(&self) -> &str {
        "git_operations"
    }

    /// 返回工具描述
    ///
    /// 描述说明了工具的功能、支持的操作和安全特性。
    fn description(&self) -> &str {
        "执行结构化 Git 操作（status、diff、log、branch、commit、add、checkout、stash）。提供解析后的 JSON 输出，并与安全策略集成实现自主控制。"
    }

    /// 返回工具参数的 JSON Schema
    ///
    /// 定义了工具接受的所有参数及其类型和约束。
    ///
    /// # 参数说明
    ///
    /// - `operation`（必需）：Git 操作类型
    /// - `message`：提交消息（commit 操作必需）
    /// - `paths`：文件路径（add 操作必需）
    /// - `branch`：分支名称（checkout 操作必需）
    /// - `files`：差异对比路径（diff 操作可选）
    /// - `cached`：是否显示暂存差异（diff 操作可选）
    /// - `limit`：日志条数限制（log 操作可选）
    /// - `action`：储藏操作类型（stash 操作可选）
    /// - `index`：储藏索引（stash drop 操作可选）
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["status", "diff", "log", "branch", "commit", "add", "checkout", "stash"],
                    "description": "要执行的 Git 操作"
                },
                "message": {
                    "type": "string",
                    "description": "提交消息（用于 'commit' 操作）"
                },
                "paths": {
                    "type": "string",
                    "description": "要暂存的文件路径（用于 'add' 操作）"
                },
                "branch": {
                    "type": "string",
                    "description": "分支名称（用于 'checkout' 操作）"
                },
                "files": {
                    "type": "string",
                    "description": "要对比的文件或路径（用于 'diff' 操作，默认：'.'）"
                },
                "cached": {
                    "type": "boolean",
                    "description": "显示暂存的更改（用于 'diff' 操作）"
                },
                "limit": {
                    "type": "integer",
                    "description": "日志条目数量（用于 'log' 操作，默认：10）"
                },
                "action": {
                    "type": "string",
                    "enum": ["push", "pop", "list", "drop"],
                    "description": "Stash 操作（用于 'stash' 操作）"
                },
                "index": {
                    "type": "integer",
                    "description": "Stash 索引（用于带 'drop' 操作的 'stash'）"
                }
            },
            "required": ["operation"]
        })
    }

    /// 执行 Git 操作
    ///
    /// 根据提供的参数执行相应的 Git 命令，返回结构化结果。
    ///
    /// # 参数
    ///
    /// - `args`：JSON 对象，必须包含 `operation` 字段
    ///
    /// # 返回值
    ///
    /// 返回 `ToolResult`，包含：
    /// - `success`：操作是否成功
    /// - `output`：JSON 格式的结果数据
    /// - `error`：错误消息（如果失败）
    ///
    /// # 执行流程
    ///
    /// 1. 验证操作参数是否存在
    /// 2. 检查当前目录是否在 Git 仓库中
    /// 3. 对于写入操作，检查自主权限级别
    /// 4. 记录操作以进行速率限制
    /// 5. 调用相应的 Git 操作方法
    ///
    /// # 安全检查
    ///
    /// - 只读模式（`AutonomyLevel::ReadOnly`）会阻止所有写入操作
    /// - 速率限制超限时操作会被阻止
    /// - 不在 Git 仓库中时操作会被拒绝
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 提取并验证操作参数
        let operation = match args.get("operation").and_then(|v| v.as_str()) {
            Some(op) => op,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'operation' parameter".into()),
                });
            }
        };

        // 检查是否在 Git 仓库中
        if !self.workspace_dir.join(".git").exists() {
            // 尝试在父目录中查找 .git
            let mut current_dir = self.workspace_dir.as_path();
            let mut found_git = false;
            while current_dir.parent().is_some() {
                if current_dir.join(".git").exists() {
                    found_git = true;
                    break;
                }
                current_dir = current_dir.parent().unwrap();
            }

            if !found_git {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Not in a git repository".into()),
                });
            }
        }

        // 检查写入操作的自主权限级别
        if self.requires_write_access(operation) {
            // 检查是否允许执行操作
            if !self.security.can_act() {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(
                        "Action blocked: git write operations require higher autonomy level".into(),
                    ),
                });
            }

            // 根据自主级别决定是否允许操作
            match self.security.autonomy {
                AutonomyLevel::ReadOnly => {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some("Action blocked: read-only mode".into()),
                    });
                }
                AutonomyLevel::Supervised | AutonomyLevel::Full => {}
            }
        }

        // 记录操作用于速率限制
        if !self.security.record_action() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Action blocked: rate limit exceeded".into()),
            });
        }

        // 根据操作类型调用相应的方法
        match operation {
            "status" => self.git_status(args).await,
            "diff" => self.git_diff(args).await,
            "log" => self.git_log(args).await,
            "branch" => self.git_branch(args).await,
            "commit" => self.git_commit(args).await,
            "add" => self.git_add(args).await,
            "checkout" => self.git_checkout(args).await,
            "stash" => self.git_stash(args).await,
            _ => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Unknown operation: {operation}")),
            }),
        }
    }
}

/// 测试模块
///
/// 测试代码位于 `tests/git_operations.rs` 文件中。
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
