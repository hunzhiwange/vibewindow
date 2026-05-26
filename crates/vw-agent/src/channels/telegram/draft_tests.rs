use super::TelegramChannel;

#[test]
fn new_channel_has_no_draft_edit_timestamps() {
    let channel = TelegramChannel::new("token".to_string(), vec![], true);

    assert!(channel.last_draft_edit.lock().is_empty());
}
