use super::{ChannelsConfig, decrypt_channel_secrets, encrypt_channel_secrets};
use crate::app::agent::security::SecretStore;

#[test]
fn channel_secret_round_trip_is_noop_when_encryption_disabled() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SecretStore::new(tmp.path(), false);
    let mut channels = ChannelsConfig::default();

    encrypt_channel_secrets(&store, &mut channels).unwrap();
    decrypt_channel_secrets(&store, &mut channels).unwrap();

    assert!(channels.telegram.is_none());
}
