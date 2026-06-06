//! 桌面 UI 配置读写入口，负责同步读取和更新持久化的偏好字段。

use serde_json::Value;
use std::future::Future;

const UI_CONFIG_KEY: &[&str] = &["desktop", "preferences"];

/// 执行 load_app_config 操作，并返回调用方需要的结果。
pub fn load_app_config() -> Value {
    let cfg = block_on_storage(async { crate::storage::read::<Value>(UI_CONFIG_KEY).await })
        .unwrap_or_else(|_| serde_json::json!({}));

    if cfg.is_object() { cfg } else { serde_json::json!({}) }
}

/// 执行 set_config_field 操作，并返回调用方需要的结果。
pub fn set_config_field(key: &str, value: Value) {
    let mut cfg = load_app_config();
    if let Some(obj) = cfg.as_object_mut() {
        obj.insert(key.to_string(), value);
    } else {
        cfg = serde_json::json!({ key: value });
    }

    let _ = block_on_storage(async move { crate::storage::write(UI_CONFIG_KEY, &cfg).await });
}

fn block_on_storage<F, T>(future: F) -> Result<T, crate::storage::Error>
where
    F: Future<Output = Result<T, crate::storage::Error>> + Send + 'static,
    T: Send + 'static,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current()
        && handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread
    {
        return tokio::task::block_in_place(|| handle.block_on(future));
    }

    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| crate::storage::Error::Io(std::io::Error::other(err.to_string())))?
            .block_on(future)
    })
    .join()
    .map_err(|_| crate::storage::Error::Io(std::io::Error::other("storage runtime panicked")))?
}
#[cfg(test)]
#[path = "ui_config_tests.rs"]
mod ui_config_tests;
