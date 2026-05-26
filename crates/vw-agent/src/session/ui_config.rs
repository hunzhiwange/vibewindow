//! 桌面 UI 配置读写入口，负责同步读取和更新持久化的偏好字段。

use serde_json::Value;

const UI_CONFIG_KEY: &[&str] = &["desktop", "preferences"];

/// 执行 load_app_config 操作，并返回调用方需要的结果。
pub fn load_app_config() -> Value {
    let cfg = match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| {
            handle.block_on(async { crate::storage::read::<Value>(UI_CONFIG_KEY).await })
        }),
        Err(_) => match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => {
                runtime.block_on(async { crate::storage::read::<Value>(UI_CONFIG_KEY).await })
            }
            Err(err) => Err(crate::storage::Error::Io(std::io::Error::other(err.to_string()))),
        },
    }
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

    let _ = match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| {
            handle.block_on(async { crate::storage::write(UI_CONFIG_KEY, &cfg).await })
        }),
        Err(_) => match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => {
                runtime.block_on(async { crate::storage::write(UI_CONFIG_KEY, &cfg).await })
            }
            Err(err) => Err(crate::storage::Error::Io(std::io::Error::other(err.to_string()))),
        },
    };
}
#[cfg(test)]
#[path = "ui_config_tests.rs"]
mod ui_config_tests;
