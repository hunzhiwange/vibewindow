//! 覆盖密码工具消息处理行为，验证生成参数和结果状态。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::{
    DIGITS_CHARSET, LOWERCASE_CHARSET, SPECIAL_CHARSET, UPPERCASE_CHARSET, build_pool,
    generate_one, selected_charsets,
};
use rand::{SeedableRng, rngs::StdRng};

#[test]
fn build_pool_combines_selected_charsets() {
    let charsets = selected_charsets(true, false, true, false);
    let pool = build_pool(&charsets);

    assert_eq!(pool, [DIGITS_CHARSET.as_bytes(), UPPERCASE_CHARSET.as_bytes()].concat());
}

#[test]
fn generate_one_contains_each_selected_charset() {
    let charsets = selected_charsets(true, true, true, true);
    let pool = build_pool(&charsets);
    let mut rng = StdRng::seed_from_u64(7);

    let password = generate_one(16, &pool, &charsets, &mut rng).expect("password should generate");

    assert_eq!(password.len(), 16);
    assert!(password.chars().any(|ch| DIGITS_CHARSET.contains(ch)));
    assert!(password.chars().any(|ch| LOWERCASE_CHARSET.contains(ch)));
    assert!(password.chars().any(|ch| UPPERCASE_CHARSET.contains(ch)));
    assert!(password.chars().any(|ch| SPECIAL_CHARSET.contains(ch)));
    assert!(password.bytes().all(|byte| pool.contains(&byte)));
}

#[test]
fn selected_charsets_is_empty_when_nothing_enabled() {
    assert!(selected_charsets(false, false, false, false).is_empty());
}
