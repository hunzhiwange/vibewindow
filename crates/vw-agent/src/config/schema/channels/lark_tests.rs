use super::lark::{FeishuConfig, LarkConfig};
use crate::app::agent::config::traits::ChannelConfig;

#[test]
fn lark_and_feishu_metadata_are_distinct() {
    assert_eq!(LarkConfig::name(), "Lark");
    assert_eq!(FeishuConfig::name(), "Feishu");
    assert_eq!(LarkConfig::desc(), "Lark Bot");
    assert_eq!(FeishuConfig::desc(), "Feishu Bot");
}
