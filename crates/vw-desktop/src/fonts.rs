//! 桌面端字体资源模块，集中暴露内置字体字节并按应用启动需求加载字体。

use std::borrow::Cow;

/// JETBRAINS_MONO_REGULAR 提供跨模块复用的稳定常量。
pub const JETBRAINS_MONO_REGULAR: &[u8] =
    include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf");

/// NOTO_SANS_CJK_SC_REGULAR 提供跨模块复用的稳定常量。
pub const NOTO_SANS_CJK_SC_REGULAR: &[u8] =
    include_bytes!("../../../assets/fonts/NotoSansCJKsc-Regular.otf");

/// 加载 all 数据。
///
/// 读取失败时按调用方约定回退为空值或默认值，避免 UI 因局部数据缺失而中断。
pub fn load_all() -> Vec<Cow<'static, [u8]>> {
    vec![Cow::Borrowed(JETBRAINS_MONO_REGULAR), Cow::Borrowed(NOTO_SANS_CJK_SC_REGULAR)]
}

#[cfg(test)]
#[path = "fonts_tests.rs"]
mod fonts_tests;
