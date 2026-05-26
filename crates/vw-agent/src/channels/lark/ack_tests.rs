use super::*;

#[test]
fn map_locale_tag_handles_supported_locale_variants() {
    assert_eq!(map_locale_tag("zh-CN"), Some(LarkAckLocale::ZhCn));
    assert_eq!(map_locale_tag("zh-Hant"), Some(LarkAckLocale::ZhTw));
    assert_eq!(map_locale_tag("en-US"), Some(LarkAckLocale::En));
    assert_eq!(map_locale_tag("ja_JP"), Some(LarkAckLocale::Ja));
    assert_eq!(map_locale_tag(""), None);
}

#[test]
fn lark_ack_pool_is_non_empty_for_every_locale() {
    for locale in [LarkAckLocale::ZhCn, LarkAckLocale::ZhTw, LarkAckLocale::En, LarkAckLocale::Ja] {
        assert!(!lark_ack_pool(locale).is_empty());
    }
}
