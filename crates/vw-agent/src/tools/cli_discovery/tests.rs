//! CLI 工具发现功能的测试模块
//!
//! 本模块包含对 CLI 工具自动发现功能的单元测试，主要测试：
//! - 工具发现功能的基本行为
//! - 工具排除列表的过滤功能
//! - CLI 工具分类的字符串表示
//!
//! 这些测试确保系统能够正确识别和过滤系统中可用的 CLI 工具。

use super::{CliCategory, DiscoveredCli, discover_cli_tools};

/// 测试工具发现功能返回非空结果
///
/// 验证 `discover_cli_tools` 函数能够返回一个包含已发现 CLI 工具的向量，
/// 且每个工具的名称字段都不为空。
///
/// # 测试场景
/// - 不提供任何包含列表和排除列表
/// - 检查返回结果中每个工具的名称字段有效性
#[test]
fn discover_returns_vec() {
    // 使用空的包含列表和排除列表调用发现函数
    let results = discover_cli_tools(&[], &[]);

    // 验证每个发现的 CLI 工具都有非空的名称
    for cli in &results {
        assert!(!cli.name.is_empty());
    }
}

/// 测试排除列表能够正确过滤工具
///
/// 验证当工具被添加到排除列表时，该工具不会出现在发现结果中。
///
/// # 测试场景
/// - 将 "git" 添加到排除列表
/// - 验证结果中不包含 "git" 工具
#[test]
fn excluded_tools_are_skipped() {
    // 调用发现函数，排除 git 工具
    let results = discover_cli_tools(&[], &["git".to_string()]);

    // 断言结果中不存在 git 工具
    assert!(!results.iter().any(|r| r.name == "git"));
}

/// 测试 CLI 工具分类的字符串显示
///
/// 验证各个 CLI 工具分类枚举值能够正确转换为人类可读的字符串表示。
///
/// # 测试场景
/// - 测试所有预定义的 CLI 工具分类
/// - 验证每个分类的字符串表示符合预期格式
#[test]
fn category_display() {
    // 验证版本控制分类的显示字符串
    assert_eq!(CliCategory::VersionControl.to_string(), "Version Control");

    // 验证编程语言分类的显示字符串
    assert_eq!(CliCategory::Language.to_string(), "Language");

    // 验证包管理器分类的显示字符串
    assert_eq!(CliCategory::PackageManager.to_string(), "Package Manager");

    // 验证容器分类的显示字符串
    assert_eq!(CliCategory::Container.to_string(), "Container");

    // 验证构建工具分类的显示字符串
    assert_eq!(CliCategory::Build.to_string(), "Build");

    // 验证云服务分类的显示字符串
    assert_eq!(CliCategory::Cloud.to_string(), "Cloud");
}
