//! SkillForge 模块单元测试
//!
//! 本模块包含对 SkillForge 技能锻造系统的测试用例，主要验证：
//! - 禁用状态下的锻造行为
//! - 默认配置值的正确性
//!
//! 测试覆盖核心配置和运行时行为的边界条件

use super::*;

/// SkillForge 测试套件
///
/// 包含针对 SkillForge 核心功能的单元测试，验证配置加载、
/// 锻造流程和边界条件处理的正确性
#[allow(dead_code)]
mod tests {
    use super::*;

    /// 测试禁用状态下的锻造行为
    ///
    /// 当 SkillForge 被禁用时（enabled = false），调用 forge() 方法
    /// 应返回空的锻造报告，不执行任何实际的发现或集成操作。
    ///
    /// # 验证点
    /// - 发现的技能数量应为 0
    /// - 自动集成的技能数量应为 0
    /// - 不应发生错误
    #[tokio::test]
    async fn disabled_forge_returns_empty_report() {
        // 创建禁用状态的配置
        let cfg = SkillForgeConfig { enabled: false, ..Default::default() };

        // 实例化 SkillForge
        let forge = SkillForge::new(cfg);

        // 执行锻造操作
        let report = forge.forge().await.unwrap();

        // 验证返回空报告
        assert_eq!(report.discovered, 0);
        assert_eq!(report.auto_integrated, 0);
    }

    /// 测试默认配置值
    ///
    /// 验证 SkillForgeConfig 的默认值符合预期设计：
    /// - 默认禁用（enabled = false）
    /// - 默认启用自动集成（auto_integrate = true）
    /// - 默认扫描间隔为 24 小时
    /// - 默认最低质量分数为 0.7
    /// - 默认技能源为 ["github", "clawhub"]
    #[test]
    fn default_config_values() {
        // 获取默认配置
        let cfg = SkillForgeConfig::default();

        // 验证默认禁用状态
        assert!(!cfg.enabled);

        // 验证默认启用自动集成
        assert!(cfg.auto_integrate);

        // 验证默认扫描间隔（24 小时）
        assert_eq!(cfg.scan_interval_hours, 24);

        // 验证默认最低质量分数（0.7，使用浮点数容差比较）
        assert!((cfg.min_score - 0.7).abs() < f64::EPSILON);

        // 验证默认技能源列表
        assert_eq!(cfg.sources, vec!["github", "clawhub"]);
    }
}

#[test]
fn debug_redacts_github_token() {
    let cfg =
        SkillForgeConfig { github_token: Some("ghp_secret".to_string()), ..Default::default() };
    let rendered = format!("{cfg:?}");

    assert!(rendered.contains("***"));
    assert!(!rendered.contains("ghp_secret"));
}

#[test]
fn config_deserialization_fills_defaults() {
    let cfg: SkillForgeConfig =
        serde_json::from_str(r#"{"enabled":true,"github_token":"token"}"#).unwrap();

    assert!(cfg.enabled);
    assert!(cfg.auto_integrate);
    assert_eq!(cfg.sources, vec!["github", "clawhub"]);
    assert_eq!(cfg.scan_interval_hours, 24);
    assert!((cfg.min_score - 0.7).abs() < f64::EPSILON);
    assert_eq!(cfg.output_dir, "./skills");
    assert_eq!(cfg.github_token.as_deref(), Some("token"));
}

#[tokio::test]
async fn enabled_unimplemented_sources_return_empty_report_without_network() {
    let dir = tempfile::tempdir().expect("temp dir");
    let cfg = SkillForgeConfig {
        enabled: true,
        sources: vec!["clawhub".to_string(), "huggingface".to_string()],
        output_dir: dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let report = SkillForge::new(cfg).forge().await.unwrap();

    assert_eq!(report.discovered, 0);
    assert_eq!(report.evaluated, 0);
    assert_eq!(report.auto_integrated, 0);
    assert_eq!(report.manual_review, 0);
    assert_eq!(report.skipped, 0);
}
