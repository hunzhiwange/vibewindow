use super::email::EmailConfig;
use crate::app::agent::config::traits::ChannelConfig;

#[test]
fn email_channel_metadata_is_stable() {
    assert_eq!(EmailConfig::name(), "Email");
    assert_eq!(EmailConfig::desc(), "Email over IMAP/SMTP");
}
