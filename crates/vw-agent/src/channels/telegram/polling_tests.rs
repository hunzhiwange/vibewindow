use super::TelegramChannel;

#[test]
fn polling_channel_starts_with_empty_voice_cache() {
    let channel = TelegramChannel::new("token".to_string(), vec![], false);

    assert!(channel.voice_transcriptions.lock().is_empty());
}
