//! VibeWindow 全局路径与缓存版本定义。
//!
//! 该模块集中计算应用数据、配置、状态、日志、二进制和缓存目录。非 wasm 目标
//! 使用系统目录约定并支持测试覆盖 home 目录；wasm 目标使用固定虚拟路径。

use std::sync::LazyLock;
use std::path::PathBuf;

/// 应用目录名。
pub const APP: &str = "vibewindow";
/// 缓存结构版本；变更后会清理旧缓存内容。
pub const CACHE_VERSION: &str = "21";

#[derive(Debug, Clone)]
/// 应用运行所需的全局路径集合。
pub struct GlobalPaths {
    /// 用户 home 目录或测试覆盖目录。
    pub home: PathBuf,
    /// 应用数据目录，存放持久数据和派生资源。
    pub data: PathBuf,
    /// 应用管理的可执行文件目录。
    pub bin: PathBuf,
    /// 日志输出目录。
    pub log: PathBuf,
    /// 缓存目录，可在版本变化时安全重建。
    pub cache: PathBuf,
    /// 配置文件目录。
    pub config: PathBuf,
    /// 状态文件目录，优先使用平台 state 目录。
    pub state: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
static PATHS: LazyLock<GlobalPaths> = LazyLock::new(|| {
    let base = directories::BaseDirs::new();
    // 测试可以通过环境变量隔离 home，避免污染真实用户目录。
    let home = std::env::var_os("VIBEWINDOW_TEST_HOME")
        .map(PathBuf::from)
        .or_else(|| base.as_ref().map(|b| b.home_dir().to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let data = base
        .as_ref()
        .map(|b| b.data_dir().join(APP))
        .unwrap_or_else(|| home.join(".local").join("share").join(APP));
    let cache = base
        .as_ref()
        .map(|b| b.cache_dir().join(APP))
        .unwrap_or_else(|| home.join(".cache").join(APP));
    let config = base
        .as_ref()
        .map(|b| b.config_dir().join(APP))
        .unwrap_or_else(|| home.join(".config").join(APP));
    let state = base
        .as_ref()
        .map(|b| {
            #[allow(deprecated)]
            {
                if let Some(p) = b.state_dir() {
                    return p.join(APP);
                }
            }
            b.data_dir().join(APP)
        })
        .unwrap_or_else(|| home.join(".local").join("state").join(APP));

    let bin = data.join("bin");
    let log = data.join("log");

    // 全局目录在首次访问时创建，调用方可以直接使用返回路径。
    let _ = std::fs::create_dir_all(&data);
    let _ = std::fs::create_dir_all(&config);
    let _ = std::fs::create_dir_all(&state);
    let _ = std::fs::create_dir_all(&log);
    let _ = std::fs::create_dir_all(&bin);
    let _ = std::fs::create_dir_all(&cache);

    let version_path = cache.join("version");
    let version = std::fs::read_to_string(&version_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "0".to_string());

    if version != CACHE_VERSION {
        // 缓存可重建，版本不匹配时清空目录，避免旧格式数据被误读。
        if let Ok(entries) = std::fs::read_dir(&cache) {
            for entry in entries.flatten() {
                let p = entry.path();
                let _ =
                    if p.is_dir() { std::fs::remove_dir_all(&p) } else { std::fs::remove_file(&p) };
            }
        }
        let _ = std::fs::write(&version_path, CACHE_VERSION);
    }

    GlobalPaths { home, data, bin, log, cache, config, state }
});

#[cfg(target_arch = "wasm32")]
static PATHS: LazyLock<GlobalPaths> = LazyLock::new(|| {
    let home = PathBuf::from("/");
    let data = home.join("data").join(APP);
    let cache = home.join("cache").join(APP);
    let config = home.join("config").join(APP);
    let state = home.join("state").join(APP);
    let bin = data.join("bin");
    let log = data.join("log");

    GlobalPaths { home, data, bin, log, cache, config, state }
});

/// 返回进程级全局路径配置。
///
/// # 返回值
///
/// 返回惰性初始化后的 `GlobalPaths` 静态引用。
pub fn paths() -> &'static GlobalPaths {
    &PATHS
}

#[cfg(test)]
mod tests;
