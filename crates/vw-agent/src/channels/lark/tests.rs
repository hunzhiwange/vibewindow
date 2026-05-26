//! 飞书/Lark 通道测试入口。

use super::*;

#[path = "tests/ack_tests.rs"]
mod ack_tests;
#[path = "tests/config_tests.rs"]
mod config_tests;
#[path = "tests/parsing_tests.rs"]
mod parsing_tests;
#[path = "tests/token_tests.rs"]
mod token_tests;
#[path = "tests/ws_tests.rs"]
mod ws_tests;

pub(super) fn with_bot_open_id(ch: LarkChannel, bot_open_id: &str) -> LarkChannel {
    ch.set_resolved_bot_open_id(Some(bot_open_id.to_string()));
    ch
}

pub(super) fn make_channel() -> LarkChannel {
    with_bot_open_id(
        LarkChannel::new(
            "cli_test_app_id".into(),
            "test_app_secret".into(),
            "test_verification_token".into(),
            None,
            vec!["ou_testuser123".into()],
            true,
        ),
        "ou_bot",
    )
}
