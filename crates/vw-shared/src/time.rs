//! WASM 兼容的时间工具模块
//!
//! 在 WASM 环境中不能直接依赖标准库的完整时间实现，因此这里统一封装时间访问，
//! 自动在原生平台与 WASM 平台之间切换到底层实现。
//!
//! # 设计目标
//!
//! - 对调用方暴露统一接口，避免上层到处分支判断
//! - 在需要时间戳时直接返回毫秒值，减少重复样板代码
//! - 在需要原始 `SystemTime` 时保留底层能力

/// 获取当前时间的毫秒时间戳
///
/// 在非 WASM 环境下使用 `std::time::SystemTime`，在 WASM 环境下使用
/// `web_time::SystemTime`，对调用方隐藏平台差异。
pub fn now_ms() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time error")
            .as_millis() as u64
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_time::SystemTime::now()
            .duration_since(web_time::SystemTime::UNIX_EPOCH)
            .expect("system time error")
            .as_millis() as u64
    }
}

/// 获取当前时间的 SystemTime
///
/// 该函数主要用于仍需要保留原始时间对象的场景。
#[cfg(not(target_arch = "wasm32"))]
pub fn now() -> std::time::SystemTime {
    std::time::SystemTime::now()
}

/// 获取当前时间的 `web_time::SystemTime`。
#[cfg(target_arch = "wasm32")]
pub fn now() -> web_time::SystemTime {
    web_time::SystemTime::now()
}

#[cfg(test)]
#[path = "time_tests.rs"]
mod time_tests;
