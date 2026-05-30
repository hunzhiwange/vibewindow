use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 文档中删除根级元数据字段(版本和文件类型)
///
/// 这些字段是 Figma 特定的元数据，HTML/CSS 渲染不需要这些元数据。
/// 此函数仅从根级别删除它们，以保持输出干净和
/// 专注于可渲染内容。
///
/// 删除的字段：
/// - "version" - Figma 文件格式版本号
/// - "fileType" - 文件类型标识符(例如 "figma"、"figjam")
///
/// # 参数
/// * `json` - 根 JSON 对象(通常包含版本、文件类型、文档)
///
/// # 返回值
/// * `Ok(())` - 成功删除元数据字段(或者它们不存在)
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_root_metadata;
/// use serde_json::json;
///
/// let mut output = json!({
///     "version": 101,
///     "fileType": "figma",
///     "document": {"name": "Root"}
/// });
/// remove_root_metadata(&mut output).unwrap();
/// // 输出现在只有文档字段，版本和文件类型已删除
/// ```
pub fn remove_root_metadata(json: &mut JsonValue) -> Result<()> {
    if let Some(obj) = json.as_object_mut() {
        obj.remove("version");
        obj.remove("fileType");
    }
    Ok(())
}

#[cfg(test)]
#[path = "root_metadata_removal_tests.rs"]
mod root_metadata_removal_tests;
