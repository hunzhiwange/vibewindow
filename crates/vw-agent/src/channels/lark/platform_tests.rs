use super::*;

#[test]
fn lark_platform_maps_endpoints_and_labels() {
    assert!(LarkPlatform::Lark.api_base().contains("larksuite.com"));
    assert!(LarkPlatform::Feishu.api_base().contains("feishu.cn"));
    assert_eq!(LarkPlatform::Lark.locale_header(), "en");
    assert_eq!(LarkPlatform::Feishu.locale_header(), "zh");
    assert_eq!(LarkPlatform::Lark.proxy_service_key(), "channel.lark");
    assert_eq!(LarkPlatform::Feishu.channel_name(), "feishu");
}
