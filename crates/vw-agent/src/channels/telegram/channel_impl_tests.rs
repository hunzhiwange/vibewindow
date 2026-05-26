use super::TelegramChannel;
use crate::app::agent::channels::traits::Channel;

#[test]
fn channel_name_and_draft_support_are_stable() {
    let channel = TelegramChannel::new("token".to_string(), vec![], false);

    assert_eq!(channel.name(), "telegram");
    assert!(channel.supports_draft_updates());
}
