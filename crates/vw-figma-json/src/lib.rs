//! # fig2json
//!
//! 用于解析 Figma `.fig` 文件并将其转换为 JSON 的库。
//!
//! ## 示例
//!
//! ```no_run
//! use fig2json::parser::{is_zip_container, extract_from_zip, detect_file_type, extract_chunks};
//!
//! let bytes = std::fs::read("example.fig").unwrap();
//!
//! // 检查是否是 ZIP 容器
//! let bytes = if is_zip_container(&bytes) {
//!     extract_from_zip(&bytes).unwrap()
//! } else {
//!     bytes
//! };
//!
//! // 检测文件类型
//! let file_type = detect_file_type(&bytes).unwrap();
//! println!("File type: {:?}", file_type);
//!
//! // 提取块
//! let parsed = extract_chunks(&bytes).unwrap();
//! println!("Version: {}", parsed.version);
//! println!("Number of chunks: {}", parsed.chunks.len());
//! ```

pub mod blobs;
pub mod error;
pub mod parser;
pub mod schema;
pub mod types;

// 重新导出常用项
pub use error::{FigError, Result};
pub use types::{FileType, ParsedFile};

/// 将 .fig 文件转换为 JSON
///
/// 这是将 Figma .fig 文件转换为 JSON 格式的主要入口点。
/// 它处理转换的所有阶段：
/// 1. ZIP 提取(如果需要)
/// 2. 文件类型检测
/// 3. 块提取
/// 4. 解压
/// 5. Kiwi schema解码
/// 6. 从nodeChanges构建树
/// 7. Blob base64 编码
/// 8. Blob 替换(用解析的内容替换 Blob 索引)
/// 9. 图像哈希转换(将哈希数组转换为文件名字符串)
/// 10. 矩阵到CSS转换(将2D仿射矩阵转换为CSS属性)
/// 11. 颜色到CSS转换(将RGBA颜色对象转换为CSS十六进制字符串)
/// 12. 文本字形去除(从文本对象中去除字形矢量数据)
/// 13. 枚举简化(将详细的枚举对象转换为简单的字符串)
/// 14. GUID 删除(删除内部 Figma 标识符)
/// 15. 编辑信息删除(删除版本控制元数据)
/// 16. phase去除(去除Figma内部状态)
/// 17. 几何去除(去除详细路径命令)
/// 18. 文本布局去除(去除详细的文本布局数据)
/// 19. 文本元数据移除(移除文本配置元数据)
/// 20. 默认文本行属性删除(从textData.lines数组中删除默认值)
/// 21. 默认文本属性删除(删除默认的 letterSpacing/lineHeight 值)
/// 22. 文本属性简化(将详细的 letterSpacing/lineHeight 转换为 CSS 字符串)
/// 23. 空字体postscript去除(从fontName中去除空postscript)
/// 24. 删除笔画属性(删除CSS不兼容的笔画属性)
/// 25. 边框权重去除(去除单个边框权重字段)
/// 26. 帧属性删除(删除特定于帧的元数据)
/// 27. 背景属性去除(去除backgroundEnabled、backgroundOpacity)
/// 28. 图像元数据去除(去除图像元数据字段)
/// 29. Internal-only节点移除(过滤掉internalOnly: true节点)
/// 30. 默认不透明度去除(去除不透明度：1.0)
/// 31. 默认可见删除(删除可见：true)
/// 32. 默认旋转移除(移除旋转：0.0)
/// 33. 默认uniformScaleFactor删除(删除uniformScaleFactor：1.0)
/// 34. 文档属性去除(去除文档级属性)
/// 35. 根元数据删除(删除版本和文件type 字段)
/// 36. 根 blob 删除(从输出中删除现在不需要的 blob 数组)
/// 37. GUID路径删除(删除内部Figma guidPath引用)
/// 38. 面向用户的版本删除(删除 Figma 版本字符串)
/// 39. 样式ID删除(删除Figma共享样式引用)
/// 40. 导出设置删除(删除资产导出配置)
/// 41. 插件数据删除(删除Figma插件存储数据)
/// 42. 矩形角半径独立去除(去除角半径独立标志)
/// 43. 约束属性移除(移除Figma自动布局约束属性)
/// 44. 滚动/调整大小属性删除(删除Figma滚动和调整大小行为属性)
/// 45. 布局辅助工具删除(删除设计时布局辅助工具，如指南和布局网格)
/// 46. 分离符号ID删除(删除Figma组件实例元数据)
/// 47. 覆盖符号 ID 删除(从数组中删除独立的 overriddenSymbolID 对象)
/// 48. 冗余角半径去除(当通用角半径存在时，去除单独的角半径字段)
/// 49. 角平滑去除(去掉Figma的角平滑属性)
/// 50. 不可见paint去除(从 fillPaints 和 strokePaints 数组中去除不可见paint)
/// 51. 空paint数组移除(移除空fillPaints和strokePaints数组)
/// 52. 冗余填充去除(当存在基于轴的填充时去除 stackPaddingRight/stackPaddingBottom)
/// 53. Stack子属性移除(移除stackChildAlignSelf和stackChildPrimaryGrow)
/// 54. 堆栈大小属性删除(删除 stackCounterSizing 和 stackPrimarySizing)
/// 55. 堆栈对齐属性移除(移除stackCounterAlignItems和stackPrimaryAlignItems)
/// 56. 符号ID删除(删除仅包含localID和/或sessionID的symbolID对象)
/// 57. 类型删除(从所有节点中删除type 字段)
/// 58. 仅可见对象删除(删除仅包含visible 属性的对象)
/// 59. 空对象移除(从JSON树中移除空对象{})
///
/// # 参数
/// * `bytes` - .fig 文件中的原始字节
/// * `base_dir` - 图像文件所在的可选基本目录(用于使用扩展名重命名)
///
/// # 返回值
/// * `Ok(serde_json::Value)` - 带有文档树和元数据的 JSON 表示
/// * `Err(FigError)` - 如果转换在任何阶段失败
///
/// # 示例
/// ```no_run
/// use fig2json::convert;
/// use std::path::Path;
///
/// let bytes = std::fs::read("example.fig").unwrap();
/// let json = convert(&bytes, Some(Path::new("/output/dir"))).unwrap();
/// println!("{}", serde_json::to_string_pretty(&json).unwrap());
/// ```
pub fn convert(bytes: &[u8], base_dir: Option<&std::path::Path>) -> Result<serde_json::Value> {
    // 1. 如果需要，检测并从 ZIP 中提取
    let bytes = if parser::is_zip_container(bytes) {
        parser::extract_from_zip(bytes)?
    } else {
        bytes.to_vec()
    };

    // 2.检测文件类型(figma vs FigJam)
    let file_type = parser::detect_file_type(&bytes)?;

    // 3.提取块(版本格式)
    let parsed = parser::extract_chunks(&bytes)?;

    // 4.解压缩块
    let schema_bytes = parser::decompress_chunk(
        parsed.schema_chunk().ok_or(FigError::NotEnoughChunks { expected: 1, actual: 0 })?,
    )?;
    let data_bytes = parser::decompress_chunk(
        parsed
            .data_chunk()
            .ok_or(FigError::NotEnoughChunks { expected: 2, actual: parsed.chunks.len() })?,
    )?;

    // 5. 使用 Kiwi schema解码
    let json = schema::decode_fig_to_json(&schema_bytes, &data_bytes)?;

    // 6.提取nodeChanges并构建树结构
    let node_changes = json
        .get("nodeChanges")
        .and_then(|v| v.as_array())
        .ok_or_else(|| FigError::ZipError("No nodeChanges found in decoded data".to_string()))?
        .clone();

    let mut document = schema::build_tree(node_changes)?;

    // 7.提取和处理blob(转换为base64)
    let blobs = json
        .get("blobs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| FigError::ZipError("No blobs found in decoded data".to_string()))?
        .clone();

    let processed_blobs = blobs::process_blobs(blobs)?;

    // 8. 用解析的 blob 内容替换文档树中的 blob 引用
    // 这会将 "commandsBlob: 5" 等字段替换为 "commands: [parsed array]"
    blobs::substitute_blobs(&mut document, processed_blobs.as_array().unwrap())?;

    // 9. 将图像哈希数组转换为带扩展名的文件名字符串
    // 这会将 "image.hash: [96, 73, ...]" 转换为 "image.filename: images/6049.jpg"
    // 如果提供了 base_dir，还可以检测格式并重命名物理文件
    if let Some(dir) = base_dir {
        schema::transform_image_hashes(&mut document, dir)?;
    } else {
        // 如果没有提供base_dir，则使用当前目录作为后备
        schema::transform_image_hashes(&mut document, std::path::Path::new("."))?;
    }

    // 10.将2D仿射变换矩阵转换为CSS属性
    // 这会将 "transform: {m00, m01, m02, m10, m11, m12}" 转换为 "transform: {x, y, rotation, scaleX, scaleY, skewX}"
    schema::transform_matrix_to_css(&mut document)?;

    // 11.将RGBA颜色对象转换为CSS十六进制字符串
    // 这会将 "color: {r, g, b, a}" 转换为 "color: #rrggbb" 或 "color: #rrggbbaa"
    schema::transform_colors_to_css(&mut document)?;

    // 12.删除文本字形矢量数据
    // 这将从 "derivedTextData" 对象中删除 "glyphs" 数组以减少输出大小
    schema::remove_text_glyphs(&mut document)?;

    // 13. 将枚举对象简化为简单字符串
    // 这会将 {"__enum__": "NodeType", "value": "FRAME"} 转换为 "FRAME"
    schema::simplify_enums(&mut document)?;

    // 14. "NORMAL" 时删除默认的混合模式(必须在枚举简化后运行)
    // 这会删除具有默认 "NORMAL" 值的 blendMode 字段以减少输出大小
    schema::remove_default_blend_mode(&mut document)?;

    // 15.删除GUID字段(内部Figma标识符)
    schema::remove_guid_fields(&mut document)?;

    // 16.删除editInfo字段(版本控制元数据)
    schema::remove_edit_info_fields(&mut document)?;

    // 17.移除相场(Figma内部状态)
    schema::remove_phase_fields(&mut document)?;

    // 18.删除几何字段(详细路径命令)
    schema::remove_geometry_fields(&mut document)?;

    // 19.删除文本布局字段(详细文本布局数据)
    schema::remove_text_layout_fields(&mut document)?;

    // 20.从衍生文本数据中删除layoutSize(与节点大小冗余)
    schema::remove_derived_text_layout_size(&mut document)?;

    // 21. 删除空的 derivedTextData 对象(对于 HTML/CSS 没有有用的信息)
    schema::remove_empty_derived_text_data(&mut document)?;

    // 22.删除文本元数据字段(文本配置元数据)
    schema::remove_text_metadata_fields(&mut document)?;

    // 23.删除默认文本行属性(来自textData.lines数组的默认值)
    schema::remove_default_text_line_properties(&mut document)?;

    // 24.删除默认文本属性(letterSpacing 0%，lineHeight 100%)
    schema::remove_default_text_properties(&mut document)?;

    // 25.简化文本属性(将冗长的 letterSpacing/lineHeight 转换为 CSS 字符串)
    schema::simplify_text_properties(&mut document)?;

    // 26. 从 fontName 对象中删除空postscript
    schema::remove_empty_font_postscript(&mut document)?;

    // 27.删除笔划属性(CSS不兼容的笔划属性)
    schema::remove_stroke_properties(&mut document)?;

    // 28.删除边框权重字段(CSS不兼容的单独边框权重)
    schema::remove_border_weights(&mut document)?;

    // 29.删除帧属性(特定于帧的元数据)
    schema::remove_frame_properties(&mut document)?;

    // 30.删除背景属性(backgroundEnabled、backgroundOpacity)
    schema::remove_background_properties(&mut document)?;

    // 31.删除图像元数据字段(图像元数据，包括imageThumbnail)
    schema::remove_image_metadata_fields(&mut document)?;

    // 32.移除internal-only节点(过滤掉internalOnly: true节点)
    schema::remove_internal_only_nodes(&mut document)?;

    // 33.删除默认不透明度值(默认为1.0)
    schema::remove_default_opacity(&mut document)?;

    // 34.删除默认可见值(true为默认值)
    schema::remove_default_visible(&mut document)?;

    // 35.删除默认旋转值(默认为0.0)
    schema::remove_default_rotation(&mut document)?;

    // 36.删除默认的uniformScaleFactor值(默认为1.0)
    schema::remove_default_uniform_scale_factor(&mut document)?;

    // 构建最终 JSON 输出
    let mut output = serde_json::json!({
        "version": parsed.version,
        "fileType": match file_type {
            FileType::Figma => "figma",
            FileType::FigJam => "figjam",
        },
        "document": document,
        "blobs": processed_blobs,
    });

    // 37.删除文档属性(文档级属性)
    schema::remove_document_properties(&mut output)?;

    // 38.删除根级元数据字段(版本和文件类型)
    schema::remove_root_metadata(&mut output)?;

    // 39.删除根级blobs数组(替换后不再需要)
    schema::remove_root_blobs(&mut output)?;

    // 40. 删除 guid 路径(Figma 内部 guidPath 引用)
    schema::remove_guid_paths(&mut output)?;

    // 41.删除面向用户的版本(Figma 版本字符串)
    schema::remove_user_facing_versions(&mut output)?;

    // 42.删除样式ID(Figma共享样式引用)
    schema::remove_style_ids(&mut output)?;

    // 43.删除导出设置(资产导出配置)
    schema::remove_export_settings(&mut output)?;

    // 44.删除插件数据(Figma插件存储数据)
    schema::remove_plugin_data(&mut output)?;

    // 45.去除矩形角半径独立(角半径独立标志)
    schema::remove_rectangle_corner_radii_independent(&mut output)?;

    // 46.删除约束属性(horizontalConstraint、verticalConstraint)
    schema::remove_constraint_properties(&mut output)?;

    // 47.删除滚动/调整大小属性(scrollBehavior，resizeToFit)
    schema::remove_scroll_resize_properties(&mut output)?;

    // 48.删除布局辅助工具(指南、layoutGrids)
    schema::remove_layout_aids(&mut output)?;

    // 49.删除分离的符号ID(Figma组件实例元数据)
    schema::remove_detached_symbol_id(&mut output)?;

    // 50.删除独立的overriddenSymbolID对象(Figma组件交换元数据)
    schema::remove_overridden_symbol_id(&mut output)?;

    // 51.去除多余的角半径(cornerRadius存在时的各个角半径字段)
    schema::remove_redundant_corner_radii(&mut output)?;

    // 52.去除角平滑(Figma的角平滑属性)
    schema::remove_corner_smoothing(&mut output)?;

    // 53.去除不可见的paint(过滤掉可见的paint：false)
    schema::remove_invisible_paints(&mut output)?;

    // 54.移除空的paint数组(移除空的fillPaints和strokePaints数组)
    schema::remove_empty_paint_arrays(&mut output)?;

    // 55.删除多余的padding属性(当存在基于轴的padding时为stackPaddingRight/stackPaddingBottom)
    schema::remove_redundant_padding(&mut output)?;

    // 56.删除堆栈子属性(stackChildAlignSelf、stackChildPrimaryGrow)
    schema::remove_stack_child_properties(&mut output)?;

    // 57. 删除堆栈大小属性(stackCounterSizing、stackPrimarySizing)
    schema::remove_stack_sizing_properties(&mut output)?;

    // 58.删除堆栈对齐属性(stackCounterAlignItems、stackPrimaryAlignItems)
    schema::remove_stack_align_items(&mut output)?;

    // 59.删除仅包含localID和/或sessionID的symbolID字段
    schema::remove_symbol_id_fields(&mut output)?;

    // 60.从所有节点中删除type 字段
    schema::remove_type(&mut output)?;

    // 61. 删除仅包含visible 属性的对象
    schema::remove_visible_only_objects(&mut output)?;

    // 62. 从 JSON 树中删除空对象 {}
    schema::remove_empty_objects(&mut output)?;

    Ok(output)
}

/// 将 .fig 文件转换为原始 JSON，无需转换
///
/// 此函数与 `convert()` 类似，但在应用任何转换之前停止。
/// 它提供原始 Figma 数据结构，未针对 HTML/CSS 转换进行优化。
///
/// 原始输出包括所有 Figma 特定字段和内部数据结构
/// 通常在标准转换过程中被删除或简化。
///
/// # 参数
/// * `bytes` - .fig 文件中的原始字节
///
/// # 返回值
/// * `Ok(serde_json::Value)` - 带有完整 Figma 数据的原始 JSON 表示
/// * `Err(FigError)` - 如果转换在任何阶段失败
///
/// # 示例
/// ```no_run
/// use fig2json::convert_raw;
///
/// let bytes = std::fs::read("example.fig").unwrap();
/// let json = convert_raw(&bytes).unwrap();
/// println!("{}", serde_json::to_string_pretty(&json).unwrap());
/// ```
pub fn convert_raw(bytes: &[u8]) -> Result<serde_json::Value> {
    // 1. 如果需要，检测并从 ZIP 中提取
    let bytes = if parser::is_zip_container(bytes) {
        parser::extract_from_zip(bytes)?
    } else {
        bytes.to_vec()
    };

    // 2.检测文件类型(figma vs FigJam)
    let file_type = parser::detect_file_type(&bytes)?;

    // 3.提取块(版本格式)
    let parsed = parser::extract_chunks(&bytes)?;

    // 4.解压缩块
    let schema_bytes = parser::decompress_chunk(
        parsed.schema_chunk().ok_or(FigError::NotEnoughChunks { expected: 1, actual: 0 })?,
    )?;
    let data_bytes = parser::decompress_chunk(
        parsed
            .data_chunk()
            .ok_or(FigError::NotEnoughChunks { expected: 2, actual: parsed.chunks.len() })?,
    )?;

    // 5. 使用 Kiwi schema解码
    let json = schema::decode_fig_to_json(&schema_bytes, &data_bytes)?;

    // 6.提取nodeChanges并构建树结构
    let node_changes = json
        .get("nodeChanges")
        .and_then(|v| v.as_array())
        .ok_or_else(|| FigError::ZipError("No nodeChanges found in decoded data".to_string()))?
        .clone();

    let mut document = schema::build_tree(node_changes)?;

    // 7.提取和处理blob(转换为base64)
    let blobs = json
        .get("blobs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| FigError::ZipError("No blobs found in decoded data".to_string()))?
        .clone();

    let processed_blobs = blobs::process_blobs(blobs)?;

    // 8. 用解析的 blob 内容替换文档树中的 blob 引用
    // 这会将 "commandsBlob: 5" 等字段替换为 "commands: [parsed array]"
    blobs::substitute_blobs(&mut document, processed_blobs.as_array().unwrap())?;

    // 构建最终的 JSON 输出而不进行转换
    let output = serde_json::json!({
        "version": parsed.version,
        "fileType": match file_type {
            FileType::Figma => "figma",
            FileType::FigJam => "figjam",
        },
        "document": document,
        "blobs": processed_blobs,
    });

    Ok(output)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
