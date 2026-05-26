use super::TelegramChannel;

#[test]
fn file_api_url_keeps_method_path() {
    let channel = TelegramChannel::new("123:ABC".to_string(), vec![], false);

    assert_eq!(channel.api_url("getFile"), "https://api.telegram.org/bot123:ABC/getFile");
}
