    /// 重新导出 use super::tool_inline_summary，让上层模块通过稳定路径访问。
    use super::tool_inline_summary;

    /// 生成 ls summary uses path，用于工具卡片或状态行的简短说明。
    ///
    /// # 参数
    ///
    /// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
    ///
    /// # 返回值
    ///
    /// 无返回值时，函数通过发布消息或更新局部状态完成交互。
    ///
    /// # 错误处理
    ///
    /// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
    #[test]
    fn ls_summary_uses_path() {
        let input = serde_json::json!({
            "path": "docs/agents"
        })
        .to_string();

        assert_eq!(tool_inline_summary("ls", &input).as_deref(), Some("docs/agents"));
    }

    /// 生成 image info summary uses path，用于工具卡片或状态行的简短说明。
    ///
    /// # 参数
    ///
    /// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
    ///
    /// # 返回值
    ///
    /// 无返回值时，函数通过发布消息或更新局部状态完成交互。
    ///
    /// # 错误处理
    ///
    /// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
    #[test]
    fn image_info_summary_uses_path() {
        let input = serde_json::json!({
            "path": "assets/demo.png"
        })
        .to_string();

        assert_eq!(
            tool_inline_summary("image_info", &input).as_deref(),
            Some("assets/demo.png")
        );
    }
