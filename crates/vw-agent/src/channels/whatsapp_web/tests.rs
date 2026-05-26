//! WhatsApp Web 通道单元测试模块
//!
//! 本模块提供 `WhatsAppWebChannel` 的全面单元测试覆盖，验证：
//! - 通道基本属性（名称标识）
//! - 号码访问控制（允许列表、通配符、拒绝策略）
//! - 电话号码标准化（格式转换、JID 提取）
//! - QR 码配对渲染（输入验证、输出格式）
//! - 连接健康检查
//! - 消息附件标记解析（图像、文档、多附件）
//!
//! # 特性门控
//!
//! 所有测试均需要启用 `whatsapp-web` 特性才能编译和执行：
//!
//! ```ignore
//! cargo test --features whatsapp-web
//! ```
//!
//! # 测试隔离
//!
//! - 使用独立的临时数据库路径（`/tmp/test-whatsapp.db`）避免污染生产环境
//! - 允许列表使用测试专用号码（`+1234567890`）

use super::*;

/// 测试模块内部定义
///
/// 使用 `#[allow(dead_code)]` 标记以避免编译器警告，
/// 因为该模块内的测试函数仅在特性启用时才被使用。
#[allow(dead_code)]
mod tests {
    use super::*;

    /// 创建用于测试的 WhatsAppWebChannel 实例
    ///
    /// # 返回值
    ///
    /// 返回一个配置了以下参数的 `WhatsAppWebChannel`：
    /// - 数据库路径：`/tmp/test-whatsapp.db`（临时测试数据库）
    /// - 会话数据：`None`（无预加载会话）
    /// - 安全配置：`None`（使用默认安全策略）
    /// - 允许列表：仅包含 `+1234567890`（单一测试号码）
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let channel = make_channel();
    /// assert_eq!(channel.name(), "whatsapp");
    /// ```
    ///
    /// # 特性要求
    ///
    /// 仅在启用 `whatsapp-web` 特性时可用。
    #[cfg(feature = "whatsapp-web")]
    fn make_channel() -> WhatsAppWebChannel {
        WhatsAppWebChannel::new(
            "/tmp/test-whatsapp.db".into(),
            None,
            None,
            vec!["+1234567890".into()],
        )
    }

    /// 测试通道名称标识
    ///
    /// 验证 `WhatsAppWebChannel::name()` 返回预期的标识符 "whatsapp"，
    /// 该标识符用于通道注册、路由和日志记录。
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_channel_name() {
        let ch = make_channel();
        assert_eq!(ch.name(), "whatsapp");
    }

    /// 测试号码允许列表的精确匹配
    ///
    /// 验证当允许列表包含具体号码时：
    /// - 匹配的号码（`+1234567890`）应被允许
    /// - 不匹配的号码（`+9876543210`）应被拒绝
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_number_allowed_exact() {
        let ch = make_channel();
        assert!(ch.is_number_allowed("+1234567890"));
        assert!(!ch.is_number_allowed("+9876543210"));
    }

    /// 测试号码允许列表的通配符匹配
    ///
    /// 验证当允许列表包含通配符 "*" 时：
    /// - 所有号码均应被允许，无论具体数字如何
    ///
    /// # 场景
    ///
    /// - 输入：`+1234567890` -> 允许
    /// - 输入：`+9999999999` -> 允许
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_number_allowed_wildcard() {
        let ch = WhatsAppWebChannel::new("/tmp/test.db".into(), None, None, vec!["*".into()]);
        assert!(ch.is_number_allowed("+1234567890"));
        assert!(ch.is_number_allowed("+9999999999"));
    }

    /// 测试空允许列表的拒绝策略
    ///
    /// 验证当允许列表为空时的行为：所有号码均应被拒绝。
    /// 这遵循"默认拒绝"的安全原则，与通道级别的访问控制策略一致。
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_number_denied_empty() {
        let ch = WhatsAppWebChannel::new("/tmp/test.db".into(), None, None, vec![]);
        // 空允许列表表示"拒绝所有"，符合通道级安全策略
        assert!(!ch.is_number_allowed("+1234567890"));
    }

    /// 测试电话号码标准化：自动添加 "+" 前缀
    ///
    /// 验证 `normalize_phone` 方法能为不带前缀的号码添加 "+"。
    ///
    /// # 测试用例
    ///
    /// 输入：`"1234567890"` -> 输出：`"+1234567890"`
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_normalize_phone_adds_plus() {
        let ch = make_channel();
        assert_eq!(ch.normalize_phone("1234567890"), "+1234567890");
    }

    /// 测试电话号码标准化：保留已有的 "+" 前缀
    ///
    /// 验证 `normalize_phone` 方法不会重复添加 "+" 前缀。
    ///
    /// # 测试用例
    ///
    /// 输入：`"+1234567890"` -> 输出：`"+1234567890"`（保持不变）
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_normalize_phone_preserves_plus() {
        let ch = make_channel();
        assert_eq!(ch.normalize_phone("+1234567890"), "+1234567890");
    }

    /// 测试电话号码标准化：从 JID 格式提取号码
    ///
    /// 验证 `normalize_phone` 方法能从 WhatsApp JID 格式中提取并标准化号码。
    ///
    /// # 测试用例
    ///
    /// 输入：`"1234567890@s.whatsapp.net"` -> 输出：`"+1234567890"`
    ///
    /// # JID 格式说明
    ///
    /// WhatsApp JID 格式为 `<phone>@s.whatsapp.net`，需要移除域名后缀并添加 "+" 前缀。
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_normalize_phone_from_jid() {
        let ch = make_channel();
        assert_eq!(ch.normalize_phone("1234567890@s.whatsapp.net"), "+1234567890");
    }

    /// 测试 QR 码配对渲染：拒绝空载荷
    ///
    /// 验证 `render_pairing_qr` 方法在输入为空白字符时返回错误，
    /// 错误消息应包含 "empty" 关键字。
    ///
    /// # 测试用例
    ///
    /// 输入：`"   "`（仅包含空格）-> 预期：错误，包含 "empty"
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_render_pairing_qr_rejects_empty_payload() {
        let err = WhatsAppWebChannel::render_pairing_qr("   ").expect_err("empty payload");
        assert!(err.to_string().contains("empty"));
    }

    /// 测试 QR 码配对渲染：输出多行文本
    ///
    /// 验证 `render_pairing_qr` 方法能将配对 URL 转换为多行 ASCII 艺术格式的 QR 码。
    ///
    /// # 输出验证
    ///
    /// - 行数应大于 10（确保 QR 码有足够的高度）
    /// - 去除首尾空白后的长度应大于 64 字节（确保 QR 码有足够的内容）
    ///
    /// # 示例
    ///
    /// 输入：`"https://example.com/whatsapp-pairing"` -> 输出：多行 ASCII QR 码
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_render_pairing_qr_outputs_multiline_text() {
        let rendered =
            WhatsAppWebChannel::render_pairing_qr("https://example.com/whatsapp-pairing")
                .expect("rendered QR");
        assert!(rendered.lines().count() > 10);
        assert!(rendered.trim().len() > 64);
    }

    /// 测试健康检查：未连接状态
    ///
    /// 验证在未建立 WhatsApp 连接时，`health_check` 返回 `false`。
    /// 这是异步测试，使用 `#[tokio::test]` 标记。
    ///
    /// # 预期行为
    ///
    /// 新创建的通道实例应返回 `false`，表示未连接到 WhatsApp 服务。
    #[tokio::test]
    #[cfg(feature = "whatsapp-web")]
    async fn whatsapp_web_health_check_disconnected() {
        let ch = make_channel();
        assert!(!ch.health_check().await);
    }

    /// 测试解析 WhatsApp 附件标记：单个图像
    ///
    /// 验证 `parse_wa_attachment_markers` 能正确解析单个图像标记。
    ///
    /// # 测试用例
    ///
    /// 输入：`"Here is the timeline [IMAGE:/tmp/chart.png]"`
    ///
    /// # 预期输出
    ///
    /// - 文本：`"Here is the timeline"`（移除了附件标记）
    /// - 附件数量：1
    /// - 附件路径：`"/tmp/chart.png"`
    /// - 附件类型：`WaAttachmentKind::Image`
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn parse_wa_markers_image() {
        let msg = "Here is the timeline [IMAGE:/tmp/chart.png]";
        let (text, attachments) = parse_wa_attachment_markers(msg);
        assert_eq!(text, "Here is the timeline");
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].target, "/tmp/chart.png");
        assert!(matches!(attachments[0].kind, WaAttachmentKind::Image));
    }

    /// 测试解析 WhatsApp 附件标记：多个附件
    ///
    /// 验证 `parse_wa_attachment_markers` 能正确解析混合类型的多个附件标记。
    ///
    /// # 测试用例
    ///
    /// 输入：`"Text [IMAGE:/a.png] more [DOCUMENT:/b.pdf]"`
    ///
    /// # 预期输出
    ///
    /// - 文本：`"Text  more"`（保留了中间的空格）
    /// - 附件数量：2
    /// - 第一个附件类型：`WaAttachmentKind::Image`，路径：`/a.png`
    /// - 第二个附件类型：`WaAttachmentKind::Document`，路径：`/b.pdf`
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn parse_wa_markers_multiple() {
        let msg = "Text [IMAGE:/a.png] more [DOCUMENT:/b.pdf]";
        let (text, attachments) = parse_wa_attachment_markers(msg);
        assert_eq!(text, "Text  more");
        assert_eq!(attachments.len(), 2);
        assert!(matches!(attachments[0].kind, WaAttachmentKind::Image));
        assert!(matches!(attachments[1].kind, WaAttachmentKind::Document));
    }

    /// 测试解析 WhatsApp 附件标记：无标记的纯文本
    ///
    /// 验证 `parse_wa_attachment_markers` 对不含附件标记的消息的处理。
    ///
    /// # 测试用例
    ///
    /// 输入：`"Just regular text"`
    ///
    /// # 预期输出
    ///
    /// - 文本：`"Just regular text"`（原样返回）
    /// - 附件数量：0（空列表）
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn parse_wa_markers_no_markers() {
        let msg = "Just regular text";
        let (text, attachments) = parse_wa_attachment_markers(msg);
        assert_eq!(text, "Just regular text");
        assert!(attachments.is_empty());
    }

    /// 测试解析 WhatsApp 附件标记：未知类型标记保留原样
    ///
    /// 验证对于无法识别的附件类型标记，解析器不进行处理，
    /// 将原始文本保留，且不提取任何附件。
    ///
    /// # 测试用例
    ///
    /// 输入：`"Check [UNKNOWN:/foo] out"`
    ///
    /// # 预期输出
    ///
    /// - 文本：`"Check [UNKNOWN:/foo] out"`（标记保留在文本中）
    /// - 附件数量：0（不提取未知类型的附件）
    ///
    /// # 设计意图
    ///
    /// 这种设计避免了意外丢弃用户消息内容，同时确保只有已知类型的
    /// 附件才会被处理和发送。
    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn parse_wa_markers_unknown_kind_preserved() {
        let msg = "Check [UNKNOWN:/foo] out";
        let (text, attachments) = parse_wa_attachment_markers(msg);
        assert_eq!(text, "Check [UNKNOWN:/foo] out");
        assert!(attachments.is_empty());
    }
}
