use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 文档中删除根级 "blobs" 字段
///
/// 在blob替换之后，blob已经被整合到文档树中，
/// 因此不再需要单独的 blob 数组。该函数将其从
/// 根级 JSON 对象。
///
/// # 参数
/// * `json` - 根 JSON 对象(通常包含版本、文件类型、文档、blob)
///
/// # 返回值
/// * `Ok(())` - 成功删除 blob 字段(或者它不存在)
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_root_blobs;
/// use serde_json::json;
///
/// let mut output = json!({
///     "version": 48,
///     "fileType": "figma",
///     "document": {"name": "Root"},
///     "blobs": [{"bytes": "..."}]
/// });
/// remove_root_blobs(&mut output).unwrap();
/// // 输出现在只有版本、文件类型和文档
/// ```
pub fn remove_root_blobs(json: &mut JsonValue) -> Result<()> {
    if let Some(obj) = json.as_object_mut() {
        obj.remove("blobs");
    }
    Ok(())
}

#[cfg(test)]
#[path = "blobs_removal_tests.rs"]
mod blobs_removal_tests;
