//! GrepTool 单元测试模块
//!
//! 本模块提供了对 `GrepTool` 工具的全面测试覆盖，包括：
//! - 工具名称和参数 schema 验证
//! - 基本搜索功能测试
//! - 文件过滤器测试（include 模式）
//! - 错误处理测试（无效正则表达式）
//! - 速率限制测试
//!
//! 所有测试均在临时目录中运行，确保测试隔离性和可重复性。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    use crate::app::agent::security::{AutonomyLevel, SecurityPolicy};
    use tempfile::TempDir;

    /// 创建测试用的安全策略配置
    ///
    /// 返回一个具有以下特性的 `SecurityPolicy`：
    /// - 自主级别为 `Supervised`（监督模式）
    /// - 工作区目录设置为指定的 `workspace` 路径
    /// - 其他配置使用默认值
    ///
    /// # 参数
    ///
    /// * `workspace` - 测试工作区的路径，通常是临时目录
    ///
    /// # 返回值
    ///
    /// 返回一个 `Arc<SecurityPolicy>`，可在多线程间共享的安全策略实例
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let policy = test_security(PathBuf::from("/tmp/test"));
    /// ```
    fn test_security(workspace: PathBuf) -> Arc<SecurityPolicy> {
        Arc::new(SecurityPolicy {
            autonomy: AutonomyLevel::Supervised,
            workspace_dir: workspace,
            ..SecurityPolicy::default()
        })
    }

    /// 创建带有速率限制的测试用安全策略配置
    ///
    /// 返回一个具有以下特性的 `SecurityPolicy`：
    /// - 自主级别为 `Supervised`（监督模式）
    /// - 工作区目录设置为指定的 `workspace` 路径
    /// - 每小时最大操作数限制为 `max_actions_per_hour`
    /// - 其他配置使用默认值
    ///
    /// # 参数
    ///
    /// * `workspace` - 测试工作区的路径，通常是临时目录
    /// * `max_actions_per_hour` - 每小时允许的最大操作数，用于测试速率限制功能
    ///
    /// # 返回值
    ///
    /// 返回一个 `Arc<SecurityPolicy>`，可在多线程间共享的安全策略实例
    ///
    /// # 示例
    ///
    /// ```ignore
    /// // 创建不允许任何操作的策略（用于测试速率限制）
    /// let policy = test_security_with_limit(PathBuf::from("/tmp/test"), 0);
    /// ```
    fn test_security_with_limit(
        workspace: PathBuf,
        max_actions_per_hour: u32,
    ) -> Arc<SecurityPolicy> {
        Arc::new(SecurityPolicy {
            autonomy: AutonomyLevel::Supervised,
            workspace_dir: workspace,
            max_actions_per_hour,
            ..SecurityPolicy::default()
        })
    }

    /// 测试 GrepTool 的基本属性和参数 schema
    ///
    /// 验证以下内容：
    /// 1. 工具名称是否正确设置为 "grep"
    /// 2. V2 规格是否暴露 Claude 兼容 canonical 名
    /// 3. 参数 schema 中是否包含必需的字段：
    ///    - `pattern`：搜索模式
    ///    - `path`：搜索路径
    ///    - `include`：文件过滤模式
    ///    - `output_mode`：输出模式
    #[test]
    fn grep_name_and_schema() {
        // 创建 GrepTool 实例
        let tool = GrepTool::new(test_security(std::env::temp_dir()));

        // 验证工具名称
        assert_eq!(tool.name(), "grep");
        assert_eq!(tool.spec().id, "grep");
        assert!(tool.spec().aliases.is_empty());

        // 获取并验证参数 schema
        let schema = tool.parameters_schema();

        // 检查必需的参数字段是否存在于 schema 中
        assert!(schema["properties"]["pattern"].is_object());
        assert!(schema["properties"]["path"].is_object());
        assert!(schema["properties"]["include"].is_object());
        assert!(schema["properties"]["output_mode"].is_object());
    }

    /// 测试 GrepTool 使用 include 过滤器查找匹配内容
    ///
    /// 测试场景：
    /// 1. 创建一个临时目录，包含两个文件：
    ///    - `a.rs`：Rust 源文件，包含 "hello" 字符串
    ///    - `b.txt`：文本文件，包含 "hello" 字符串
    /// 2. 使用 `include: "*.rs"` 过滤器搜索 "hello"
    /// 3. 验证：
    ///    - 操作成功
    ///    - 只找到 1 处匹配
    ///    - 结果包含 `a.rs`
    ///    - 结果不包含 `b.txt`（被过滤器排除）
    #[tokio::test]
    async fn grep_finds_matches_with_include() {
        // 创建临时目录
        let dir = TempDir::new().unwrap();

        // 创建测试文件：Rust 源文件
        std::fs::write(dir.path().join("a.rs"), "fn hello() {}\nlet x = 1;\n").unwrap();

        // 创建测试文件：普通文本文件
        std::fs::write(dir.path().join("b.txt"), "hello from txt\n").unwrap();

        // 创建 GrepTool 实例，使用临时目录作为工作区
        let tool = GrepTool::new(test_security(dir.path().to_path_buf()));

        // 执行搜索，只搜索 .rs 文件
        let result = tool
            .execute(json!({
                "pattern": "hello",
                "include": "*.rs",
                "output_mode": "content"
            }))
            .await
            .unwrap();

        // 验证搜索成功
        assert!(result.success);

        // 验证找到了 1 处匹配
        assert!(result.output.contains("找到 1 处匹配"));

        // 验证结果包含 a.rs 文件
        assert!(result.output.contains("a.rs"));

        // 验证结果不包含 b.txt 文件（被过滤器排除）
        assert!(!result.output.contains("b.txt"));
    }

    /// 测试 GrepTool 拒绝无效的正则表达式
    ///
    /// 测试场景：
    /// 1. 创建一个临时目录，包含一个文本文件
    /// 2. 使用无效的正则表达式 `(`（未闭合的括号）进行搜索
    /// 3. 验证：
    ///    - 操作失败（success = false）
    ///    - 错误信息包含"正则表达式无效"的提示
    #[tokio::test]
    async fn grep_rejects_invalid_regex() {
        // 创建临时目录
        let dir = TempDir::new().unwrap();

        // 创建测试文件
        std::fs::write(dir.path().join("a.txt"), "hello\n").unwrap();

        // 创建 GrepTool 实例
        let tool = GrepTool::new(test_security(dir.path().to_path_buf()));

        // 使用无效的正则表达式执行搜索
        let result = tool.execute(json!({ "pattern": "(" })).await.unwrap();

        // 验证操作失败
        assert!(!result.success);

        // 验证错误信息包含正则表达式无效的提示
        assert!(result.error.as_ref().unwrap().contains("正则表达式无效"));
    }

    /// 测试 GrepTool 的速率限制功能
    ///
    /// 测试场景：
    /// 1. 创建一个临时目录
    /// 2. 创建一个每小时最大操作数为 0 的安全策略
    /// 3. 执行搜索操作
    /// 4. 验证：
    ///    - 操作失败（success = false）
    ///    - 错误信息包含"Rate limit"的提示
    ///
    /// 这个测试确保速率限制机制能够正确阻止超出限制的操作
    #[tokio::test]
    async fn grep_rate_limited() {
        // 创建临时目录
        let dir = TempDir::new().unwrap();

        // 创建 GrepTool 实例，设置每小时最大操作数为 0（不允许任何操作）
        let tool = GrepTool::new(test_security_with_limit(dir.path().to_path_buf(), 0));

        // 尝试执行搜索操作
        let result = tool.execute(json!({ "pattern": "hello" })).await.unwrap();

        // 验证操作因速率限制而失败
        assert!(!result.success);

        // 验证错误信息包含速率限制的提示
        assert!(result.error.as_ref().unwrap().contains("Rate limit"));
    }
}
