//! # 域名匹配器测试模块
//!
//! 本模块包含 `DomainMatcher` 的单元测试，用于验证域名白名单匹配功能的正确性。
//!
//! ## 测试覆盖范围
//!
//! - **精确匹配**：验证完整域名的精确匹配功能
//! - **通配符匹配**：验证通配符模式的域名匹配功能
//! - **预设类别**：验证银行类等预定义域名类别的扩展与匹配
//! - **错误处理**：验证对无效域名模式和未知类别的拒绝处理
//!
//! ## 运行测试
//!
//! ```bash
//! cargo test domain_matcher::tests
//! ```

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    /// 测试精确域名匹配功能
    ///
    /// # 测试场景
    ///
    /// - 配置白名单为 `accounts.google.com`
    ///
    /// # 预期结果
    ///
    /// - ✅ `accounts.google.com` 应被匹配（精确匹配）
    /// - ✅ `https://accounts.google.com/login` 应被匹配（URL 格式，自动提取域名）
    /// - ❌ `mail.google.com` 不应被匹配（不同子域名）
    #[test]
    fn exact_match_works() {
        let matcher =
            DomainMatcher::new(&["accounts.google.com".to_string()], &[] as &[String]).unwrap();
        assert!(matcher.is_gated("accounts.google.com"));
        assert!(matcher.is_gated("https://accounts.google.com/login"));
        assert!(!matcher.is_gated("mail.google.com"));
    }

    /// 测试通配符域名匹配功能
    ///
    /// # 测试场景
    ///
    /// - 配置白名单为 `*.chase.com`（通配符匹配任意子域名）
    ///
    /// # 预期结果
    ///
    /// - ✅ `www.chase.com` 应被匹配（单级子域名）
    /// - ✅ `secure.chase.com` 应被匹配（单级子域名）
    /// - ❌ `chase.com` 不应被匹配（通配符不匹配根域名）
    #[test]
    fn wildcard_match_works() {
        let matcher = DomainMatcher::new(&["*.chase.com".to_string()], &[] as &[String]).unwrap();
        assert!(matcher.is_gated("www.chase.com"));
        assert!(matcher.is_gated("secure.chase.com"));
        assert!(!matcher.is_gated("chase.com"));
    }

    /// 测试预设类别扩展与匹配功能
    ///
    /// # 测试场景
    ///
    /// - 配置类别预设为 `banking`（银行业务类别）
    /// - 系统应自动扩展为该类别下所有银行相关域名
    ///
    /// # 预期结果
    ///
    /// - ✅ `login.paypal.com` 应被匹配（PayPal 属于银行类别）
    /// - ✅ `api.coinbase.com` 应被匹配（Coinbase 属于银行类别）
    /// - ❌ `developer.mozilla.org` 不应被匹配（非银行类别）
    #[test]
    fn category_preset_expands_and_matches() {
        let matcher = DomainMatcher::new(&[] as &[String], &["banking".to_string()]).unwrap();
        assert!(matcher.is_gated("login.paypal.com"));
        assert!(matcher.is_gated("api.coinbase.com"));
        assert!(!matcher.is_gated("developer.mozilla.org"));
    }

    /// 测试非匹配域名返回 false
    ///
    /// # 测试场景
    ///
    /// - 配置白名单为 `accounts.google.com`
    /// - 测试不在白名单中的域名
    ///
    /// # 预期结果
    ///
    /// - ❌ `example.com` 不应被匹配
    #[test]
    fn non_matching_domain_returns_false() {
        let matcher =
            DomainMatcher::new(&["accounts.google.com".to_string()], &[] as &[String]).unwrap();
        assert!(!matcher.is_gated("example.com"));
    }

    /// 测试拒绝格式错误的域名模式
    ///
    /// # 测试场景
    ///
    /// - 尝试使用包含非法字符（空格）的域名模式创建匹配器
    ///
    /// # 预期结果
    ///
    /// - ❌ 创建应失败并返回错误
    /// - 错误消息应包含 "invalid characters" 提示
    #[test]
    fn malformed_domain_pattern_is_rejected() {
        let err = DomainMatcher::new(&["bad domain.com".to_string()], &[] as &[String])
            .expect_err("expected invalid pattern");
        assert!(err.to_string().contains("invalid characters"));
    }

    /// 测试拒绝未知的域名类别
    ///
    /// # 测试场景
    ///
    /// - 尝试使用不存在的类别名称 `unknown` 创建匹配器
    ///
    /// # 预期结果
    ///
    /// - ❌ 创建应失败并返回错误
    /// - 错误消息应包含 "Unknown OTP domain category" 提示
    #[test]
    fn unknown_category_is_rejected() {
        let err = DomainMatcher::new(&[] as &[String], &["unknown".to_string()])
            .expect_err("expected unknown category rejection");
        assert!(err.to_string().contains("Unknown OTP domain category"));
    }
}
