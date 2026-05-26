//! # MQTT 通道测试模块
//!
//! 本模块包含 MQTT 通道实现的单元测试，主要覆盖以下功能：
//!
//! - **配置验证**：确保 MQTT 配置参数符合规范要求
//! - **URL 解析**：验证 broker 地址的解析功能正确性
//! - **TLS 一致性**：检查 TLS 标志与 URL scheme 的匹配关系
//!
//! ## 测试覆盖范围
//!
//! | 功能模块 | 测试场景 |
//! |---------|---------|
//! | QoS 验证 | 无效 QoS 值被拒绝 |
//! | URL 格式 | 非 MQTT scheme 被拒绝 |
//! | Topics | 空主题列表被拒绝 |
//! | Client ID | 空客户端 ID 被拒绝 |
//! | TLS 标志 | scheme 与 use_tls 不一致被拒绝 |
//! | Host 解析 | 从 URL 正确提取主机名 |
//! | Port 解析 | 从 URL 正确提取端口号，含默认值 |
//!
//! 所有测试均使用 `MqttConfig` 结构体的 `validate()` 方法进行配置校验。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    /// 测试 MQTT 配置验证拒绝无效的 QoS 值
    ///
    /// # 测试场景
    /// - 构造一个 QoS 值为 3 的配置（有效值为 0、1、2）
    /// - 调用 `validate()` 方法应返回错误
    /// - 错误消息应包含 "qos must be 0, 1, or 2"
    ///
    /// # 预期结果
    /// 配置验证失败，返回包含 QoS 提示的错误信息
    #[test]
    fn mqtt_config_validation_rejects_bad_qos() {
        let config = MqttConfig {
            broker_url: "mqtt://localhost:1883".into(),
            client_id: "vibewindow".into(),
            topics: vec!["test".into()],
            qos: 3,  // 无效值：MQTT QoS 仅支持 0、1、2
            username: None,
            password: None,
            use_tls: false,
            keep_alive_secs: 30,
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("qos must be 0, 1, or 2"));
    }

    /// 测试 MQTT 配置验证拒绝非 MQTT scheme 的 URL
    ///
    /// # 测试场景
    /// - 构造一个使用 `http://` scheme 的配置
    /// - 调用 `validate()` 方法应返回错误
    /// - 错误消息应提示需要 `mqtt://` scheme
    ///
    /// # 预期结果
    /// 配置验证失败，返回包含 scheme 提示的错误信息
    #[test]
    fn mqtt_config_validation_rejects_bad_url() {
        let config = MqttConfig {
            broker_url: "http://localhost:1883".into(),  // 无效 scheme
            client_id: "vibewindow".into(),
            topics: vec!["test".into()],
            qos: 1,
            username: None,
            password: None,
            use_tls: false,
            keep_alive_secs: 30,
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("mqtt://"));
    }

    /// 测试 MQTT 配置验证拒绝空的主题列表
    ///
    /// # 测试场景
    /// - 构造一个 `topics` 为空向量的配置
    /// - 调用 `validate()` 方法应返回错误
    /// - 错误消息应提示至少需要一个主题
    ///
    /// # 预期结果
    /// 配置验证失败，返回包含主题数量提示的错误信息
    #[test]
    fn mqtt_config_validation_rejects_empty_topics() {
        let config = MqttConfig {
            broker_url: "mqtt://localhost:1883".into(),
            client_id: "vibewindow".into(),
            topics: vec![],  // 空主题列表无效
            qos: 1,
            username: None,
            password: None,
            use_tls: false,
            keep_alive_secs: 30,
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("at least one topic"));
    }

    /// 测试 MQTT 配置验证拒绝空的客户端 ID
    ///
    /// # 测试场景
    /// - 构造一个 `client_id` 为空字符串的配置
    /// - 调用 `validate()` 方法应返回错误
    /// - 错误消息应提示客户端 ID 不能为空
    ///
    /// # 预期结果
    /// 配置验证失败，返回包含客户端 ID 提示的错误信息
    #[test]
    fn mqtt_config_validation_rejects_empty_client_id() {
        let config = MqttConfig {
            broker_url: "mqtt://localhost:1883".into(),
            client_id: String::new(),  // 空客户端 ID 无效
            topics: vec!["test".into()],
            qos: 1,
            username: None,
            password: None,
            use_tls: false,
            keep_alive_secs: 30,
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("client_id must not be empty"));
    }

    /// 测试 MQTT 配置验证接受所有参数均有效的配置
    ///
    /// # 测试场景
    /// - 构造一个所有参数均符合规范的配置
    /// - `broker_url` 使用 `mqtt://` scheme
    /// - `client_id` 非空
    /// - `topics` 包含至少一个主题
    /// - `qos` 为有效值
    /// - `use_tls` 与 scheme 一致
    ///
    /// # 预期结果
    /// 配置验证通过，`validate()` 返回 `Ok(())`
    #[test]
    fn mqtt_config_validation_accepts_valid() {
        let config = MqttConfig {
            broker_url: "mqtt://localhost:1883".into(),
            client_id: "vibewindow".into(),
            topics: vec!["sensors/#".into()],  // 使用通配符订阅
            qos: 1,
            username: None,
            password: None,
            use_tls: false,
            keep_alive_secs: 30,
        };
        assert!(config.validate().is_ok());
    }

    /// 测试 TLS 标志与 scheme 不一致：mqtt:// 搭配 use_tls=true
    ///
    /// # 测试场景
    /// - 构造一个 `broker_url` 使用 `mqtt://`（非加密）
    /// - 同时设置 `use_tls` 为 `true`
    /// - 这种配置存在矛盾：非加密 scheme 却启用 TLS
    ///
    /// # 预期结果
    /// 配置验证失败，返回包含 "use_tls is true" 提示的错误信息
    #[test]
    fn mqtt_tls_flag_rejects_mqtt_scheme_with_use_tls() {
        let config = MqttConfig {
            broker_url: "mqtt://localhost:1883".into(),  // 非 TLS scheme
            client_id: "vibewindow".into(),
            topics: vec!["test".into()],
            qos: 1,
            username: None,
            password: None,
            use_tls: true,  // 矛盾：非加密 URL 却启用 TLS
            keep_alive_secs: 30,
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("use_tls is true"));
    }

    /// 测试 TLS 标志与 scheme 不一致：mqtts:// 搭配 use_tls=false
    ///
    /// # 测试场景
    /// - 构造一个 `broker_url` 使用 `mqtts://`（加密）
    /// - 同时设置 `use_tls` 为 `false`
    /// - 这种配置存在矛盾：加密 scheme 却禁用 TLS
    ///
    /// # 预期结果
    /// 配置验证失败，返回包含 "mqtts://" 提示的错误信息
    #[test]
    fn mqtt_tls_flag_rejects_mqtts_scheme_without_use_tls() {
        let config = MqttConfig {
            broker_url: "mqtts://localhost:8883".into(),  // TLS scheme
            client_id: "vibewindow".into(),
            topics: vec!["test".into()],
            qos: 1,
            username: None,
            password: None,
            use_tls: false,  // 矛盾：加密 URL 却禁用 TLS
            keep_alive_secs: 30,
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("mqtts://"));
    }

    /// 测试 TLS 标志与 scheme 一致：mqtts:// 搭配 use_tls=true
    ///
    /// # 测试场景
    /// - 构造一个 `broker_url` 使用 `mqtts://`（加密）
    /// - 同时设置 `use_tls` 为 `true`
    /// - scheme 与 TLS 标志一致
    ///
    /// # 预期结果
    /// 配置验证通过，`validate()` 返回 `Ok(())`
    #[test]
    fn mqtt_tls_flag_accepts_mqtts_with_use_tls() {
        let config = MqttConfig {
            broker_url: "mqtts://localhost:8883".into(),  // TLS scheme
            client_id: "vibewindow".into(),
            topics: vec!["test".into()],
            qos: 1,
            username: None,
            password: None,
            use_tls: true,  // 一致：加密 URL 且启用 TLS
            keep_alive_secs: 30,
        };
        assert!(config.validate().is_ok());
    }

    /// 测试 `broker_host()` 函数从 URL 提取主机名
    ///
    /// # 测试场景
    /// - 输入 `mqtt://myhost:1883`，应返回 `myhost`
    /// - 输入 `mqtts://secure.example.com:8883`，应返回 `secure.example.com`
    ///
    /// # 预期结果
    /// 函数正确解析 URL 并返回主机名部分，不包含端口
    #[test]
    fn broker_host_extracts_host() {
        assert_eq!(broker_host("mqtt://myhost:1883"), "myhost");
        assert_eq!(
            broker_host("mqtts://secure.example.com:8883"),
            "secure.example.com"
        );
    }

    /// 测试 `broker_port()` 函数从 URL 提取端口号
    ///
    /// # 测试场景
    /// - 输入 `mqtt://localhost:1883`，应返回 `1883`
    /// - 输入 `mqtts://host:8883`，应返回 `8883`
    ///
    /// # 预期结果
    /// 函数正确解析 URL 并返回显式指定的端口号
    #[test]
    fn broker_port_extracts_port() {
        assert_eq!(broker_port("mqtt://localhost:1883"), 1883);
        assert_eq!(broker_port("mqtts://host:8883"), 8883);
    }

    /// 测试 `broker_port()` 函数对 mqtt:// 的默认端口处理
    ///
    /// # 测试场景
    /// - 输入 `mqtt://localhost`（未指定端口）
    /// - 函数应返回 MQTT 标准非加密端口 `1883`
    ///
    /// # 预期结果
    /// 当 URL 中未指定端口时，返回 MQTT 默认端口 1883
    #[test]
    fn broker_port_defaults_1883_for_mqtt() {
        assert_eq!(broker_port("mqtt://localhost"), 1883);
    }

    /// 测试 `broker_port()` 函数对 mqtts:// 的默认端口处理
    ///
    /// # 测试场景
    /// - 输入 `mqtts://secure.example.com`（未指定端口）
    /// - 函数应返回 MQTT 标准加密端口 `8883`
    ///
    /// # 预期结果
    /// 当 URL 中未指定端口时，返回 MQTTS 默认端口 8883
    #[test]
    fn broker_port_defaults_8883_for_mqtts() {
        assert_eq!(broker_port("mqtts://secure.example.com"), 8883);
    }
}
