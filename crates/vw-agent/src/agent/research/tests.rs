//! 研究阶段触发条件测试模块
//!
//! 本模块包含针对 `should_trigger` 函数的完整单元测试套件，
//! 验证研究阶段在不同触发策略下的行为是否正确。
//!
//! # 测试覆盖范围
//!
//! - `Never` 触发策略：永不触发研究阶段
//! - `Always` 触发策略：总是触发研究阶段
//! - `Keywords` 触发策略：基于关键词匹配触发
//! - `Length` 触发策略：基于消息长度触发
//! - `Question` 触发策略：基于问句检测触发
//! - 禁用状态：验证配置禁用时永不触发

use super::*;

/// 研究阶段触发条件测试套件
///
/// 包含所有触发策略的边界条件和正常路径测试。
#[allow(dead_code)]
mod tests {
    use super::*;

    /// 测试 `Never` 触发策略
    ///
    /// 验证当触发策略设置为 `Never` 时，无论输入什么消息，
    /// 研究阶段都不应被触发。
    ///
    /// # 预期行为
    ///
    /// - 输入："find something"（包含关键词）
    /// - 结果：不触发（返回 false）
    #[test]
    fn should_trigger_never() {
        // 配置：启用研究阶段，但触发策略为 Never
        let config = ResearchPhaseConfig {
            enabled: true,
            trigger: ResearchTrigger::Never,
            ..Default::default()
        };
        // 即使消息包含关键词，也不应触发
        assert!(!should_trigger(&config, "find something"));
    }

    /// 测试 `Always` 触发策略
    ///
    /// 验证当触发策略设置为 `Always` 时，任何消息都会触发研究阶段。
    ///
    /// # 预期行为
    ///
    /// - 输入："hello"（简单消息）
    /// - 结果：触发（返回 true）
    #[test]
    fn should_trigger_always() {
        // 配置：启用研究阶段，触发策略为 Always
        let config = ResearchPhaseConfig {
            enabled: true,
            trigger: ResearchTrigger::Always,
            ..Default::default()
        };
        // 任何消息都应触发
        assert!(should_trigger(&config, "hello"));
    }

    /// 测试 `Keywords` 触发策略
    ///
    /// 验证基于关键词列表的触发逻辑，包括大小写不敏感匹配。
    ///
    /// # 测试场景
    ///
    /// 1. 消息包含关键词 "find" -> 应触发
    /// 2. 消息包含关键词 "SEARCH"（大写）-> 应触发（大小写不敏感）
    /// 3. 消息不包含任何关键词 -> 不应触发
    #[test]
    fn should_trigger_keywords() {
        // 配置：启用研究阶段，触发策略为 Keywords，关键词列表为 ["find", "search"]
        let config = ResearchPhaseConfig {
            enabled: true,
            trigger: ResearchTrigger::Keywords,
            keywords: vec!["find".into(), "search".into()],
            ..Default::default()
        };
        // 场景1：消息包含 "find" 关键词
        assert!(should_trigger(&config, "please find the file"));
        // 场景2：消息包含 "SEARCH"（验证大小写不敏感）
        assert!(should_trigger(&config, "SEARCH for errors"));
        // 场景3：消息不包含任何关键词
        assert!(!should_trigger(&config, "hello world"));
    }

    /// 测试 `Length` 触发策略
    ///
    /// 验证基于消息长度的触发逻辑。
    ///
    /// # 测试场景
    ///
    /// 1. 消息长度小于阈值 -> 不应触发
    /// 2. 消息长度大于等于阈值 -> 应触发
    #[test]
    fn should_trigger_length() {
        // 配置：启用研究阶段，触发策略为 Length，最小长度为 20
        let config = ResearchPhaseConfig {
            enabled: true,
            trigger: ResearchTrigger::Length,
            min_message_length: 20,
            ..Default::default()
        };
        // 场景1：短消息（长度 < 20）
        assert!(!should_trigger(&config, "short"));
        // 场景2：长消息（长度 >= 20）
        assert!(should_trigger(&config, "this is a longer message that exceeds the minimum"));
    }

    /// 测试 `Question` 触发策略
    ///
    /// 验证基于问句检测的触发逻辑。
    ///
    /// # 测试场景
    ///
    /// 1. 消息以问号结尾 -> 应触发
    /// 2. 消息不以问号结尾 -> 不应触发
    #[test]
    fn should_trigger_question() {
        // 配置：启用研究阶段，触发策略为 Question
        let config = ResearchPhaseConfig {
            enabled: true,
            trigger: ResearchTrigger::Question,
            ..Default::default()
        };
        // 场景1：问句（以 ? 结尾）
        assert!(should_trigger(&config, "what is this?"));
        // 场景2：非问句
        assert!(!should_trigger(&config, "do this now"));
    }

    /// 测试禁用状态下的触发行为
    ///
    /// 验证当研究阶段被禁用时（`enabled: false`），
    /// 无论触发策略如何设置，都不应触发研究阶段。
    ///
    /// # 测试场景
    ///
    /// - 配置：禁用研究阶段，但触发策略为 Always
    /// - 输入："anything"
    /// - 结果：不触发（返回 false）
    #[test]
    fn disabled_never_triggers() {
        // 配置：禁用研究阶段，即使触发策略为 Always
        let config = ResearchPhaseConfig {
            enabled: false,
            trigger: ResearchTrigger::Always,
            ..Default::default()
        };
        // 即使触发策略为 Always，禁用状态下也不应触发
        assert!(!should_trigger(&config, "anything"));
    }
}
