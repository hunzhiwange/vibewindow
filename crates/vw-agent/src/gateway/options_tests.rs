use super::*;

#[test]
fn default_options_keep_gateway_disabled_by_default() {
    let options = ServeOptions::default();

    assert_eq!(options.hostname, "127.0.0.1");
    assert_eq!(options.port, 4099);
    assert!(options.cors.is_empty());
}
