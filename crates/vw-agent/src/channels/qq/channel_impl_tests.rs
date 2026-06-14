use super::*;
use crate::app::agent::channels::traits::Channel;

#[test]
fn qq_channel_trait_name_is_static() {
    let channel = QQChannel::new("app".to_string(), "secret".to_string(), vec![]);
    let as_trait: &dyn Channel = &channel;

    assert_eq!(as_trait.name(), "qq");
}
