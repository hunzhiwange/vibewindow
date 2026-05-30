use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除与几何相关的字段
///
/// 递归遍历 JSON 树并删除几何字段：
/// - "fillGeometry" - 填充形状的路径命令(M、L、Q、Z 等)
/// - "strokeGeometry" - 笔划形状的路径命令
/// - "windingRule" - SVG 缠绕规则属性
/// - "styleID" - 内部样式参考
///
/// 这些字段包含详细的路径几何形状，简单的 HTML/CSS 形状渲染不需要它们。
///
/// **例外**：为图标和图像保留几何形状，这些图标和图像通过以下方式标识：
/// - 在 symbolData.symbolOverrides 中使用 imageType (SVG/PNG) 进行导出设置
/// - 节点名称以 "icon/" 或 "arrows/" 开头
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有几何字段(图标/图像除外)
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_geometry_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "fillGeometry": [
///         {
///             "commands": ["M", 0.0, 0.0, "L", 100.0, 0.0, "Z"],
///             "styleID": 0,
///             "windingRule": {
///                 "__enum__": "WindingRule",
///                 "value": "NONZERO"
///             }
///         }
///     ],
///     "size": {"x": 100.0, "y": 100.0}
/// });
/// remove_geometry_fields(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "size" 字段
/// ```
pub fn remove_geometry_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 确定是否应为此节点保留几何数据
///
/// 为图标和图像保留几何形状，这些图标和图像通过以下方式标识：
/// 1. 在顶层设置带有 imageType 字段的导出设置(表示 SVG/PNG 导出)
/// 2. 名称以 "icon/" 或 "arrows/" 开头(常见图标命名模式)
///
/// 注意：我们只检查节点自己的名称，而不检查 symbolOverrides 中代表的名称
/// 子组件。
///
/// # 参数
/// * `value` - 要检查的 JSON 节点
///
/// # 返回值
/// * `true` 如果应保留几何图形，`false` 如果应删除几何图形
fn should_preserve_geometry(value: &JsonValue) -> bool {
    if let Some(obj) = value.as_object() {
        // 检查 1：查找图标/图像名称模式(此节点的名称，而不是子节点的名称)
        if let Some(name) = obj.get("name")
            && let Some(name_str) = name.as_str()
            && (name_str.starts_with("icon/") || name_str.starts_with("arrows/"))
        {
            return true; // Icon or arrow, preserve geometry
        }

        // 检查2：在symbolData.symbolOverrides中查找带有imageType的exportSettings
        // 这检查此特定节点是否具有导出设置(不是子覆盖)
        if let Some(symbol_data) = obj.get("symbolData")
            && let Some(overrides) = symbol_data.get("symbolOverrides")
            && let Some(overrides_array) = overrides.as_array()
        {
            for override_item in overrides_array {
                if let Some(export_settings) = override_item.get("exportSettings")
                    && let Some(settings_array) = export_settings.as_array()
                {
                    for setting in settings_array {
                        if setting.get("imageType").is_some() {
                            return true; // Has imageType, preserve geometry
                        }
                    }
                }
            }
        }
    }

    false // Not an icon/image, remove geometry
}

/// 从 JSON 值中递归删除几何字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    // 检查我们是否应该在匹配之前保留几何图形(以避免借用检查器问题)
    let preserve = should_preserve_geometry(value);

    match value {
        JsonValue::Object(map) => {
            // 仅删除不是图标/图像节点的几何图形
            if !preserve {
                map.remove("fillGeometry");
                map.remove("strokeGeometry");
                map.remove("windingRule");
                map.remove("styleID");
            }

            // 递归到所有剩余值
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if let Some(val) = map.get_mut(&key) {
                    transform_recursive(val)?;
                }
            }
        }
        JsonValue::Array(arr) => {
            // 递归到数组元素
            for val in arr.iter_mut() {
                transform_recursive(val)?;
            }
        }
        _ => {
            // 原始值，无需处理
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "geometry_removal_tests.rs"]
mod geometry_removal_tests;
