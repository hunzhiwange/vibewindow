/// 最终 JSON 输出的转换过程
///
/// 该模块包含应用于
/// 初始解析和 blob 替换后的 JSON 文档：
///
/// - `image_hash`：将图像哈希数组转换为文件名字符串
/// - `blobs_removal`：从最终输出中删除根级 blob 数组
/// - `matrix_to_css`：将 2D 仿射变换矩阵转换为 CSS 定位属性
/// - `color_to_css`：将 RGBA 颜色对象转换为 CSS 十六进制颜色字符串
/// - `text_glyphs_removal`：从文本对象中删除字形矢量数据
/// - `guid_removal`：删除内部Figma GUID标识符
/// - `edit_info_removal`：删除版本控制编辑信息元数据
/// - `phase_removal`：删除 Figma 内部phase状态
/// - `geometry_removal`：删除详细的几何路径命令
/// - `text_layout_removal`：删除详细的文本布局数据
/// - `text_metadata_removal`：删除文本配置元数据
/// - `text_line_defaults_removal`：从行数组中删除默认文本行属性
/// - `stroke_properties_removal`：删除与 CSS 不兼容的笔画属性
/// - `frame_properties_removal`：删除特定于帧的元数据
/// - `image_metadata_removal`：删除图像元数据字段
/// - `document_properties_removal`：删除文档级属性
/// - `enum_simplification`：将详细的枚举对象简化为简单的字符串
/// - `default_text_properties_removal`：删除默认文本属性值
/// - `empty_font_postscript_removal`：从 fontName 中删除空postscript
/// - `border_weights_removal`：删除单个边框权重字段
/// - `default_blend_mode_removal`：删除默认的混合模式值
/// - `background_properties_removal`：删除后台元数据字段
/// - `internal_only_nodes_removal`：过滤掉仅限内部的节点
/// - `derived_text_layout_size_removal`：从衍生文本数据中删除多余的布局大小
/// - `empty_derived_text_data_removal`：删除空的derivedTextData对象
/// - `empty_objects_removal`：从 JSON 树中删除空对象 {}
/// - `default_opacity_removal`：删除默认不透明度值 (1.0)
/// - `default_visible_removal`：删除默认可见值(true)
/// - `default_rotation_removal`：删除默认旋转值 (0.0)
/// - `root_metadata_removal`：删除根级别版本和文件type 字段
/// - `guid_path_removal`：删除内部 Figma guidPath 引用
/// - `user_facing_version_removal`：删除 Figma 版本字符串
/// - `style_id_removal`：删除 Figma 共享样式引用
/// - `export_settings_removal`：删除资产导出配置
/// - `plugin_data_removal`：删除 Figma 插件存储数据
/// - `rectangle_corner_radii_independent_removal`：删除角半径独立标志
/// - `constraint_properties_removal`：删除 Figma 自动布局约束属性
/// - `scroll_resize_properties_removal`：删除 Figma 滚动和调整大小行为属性
/// - `layout_aids_removal`：删除设计时布局辅助工具(指南、layoutGrids)
/// - `detached_symbol_id_removal`：删除 Figma 组件实例元数据
/// - `redundant_corner_radii_removal`：当通用cornerRadius存在时删除单个角半径字段
/// - `corner_smoothing_removal`：删除 Figma 的角平滑属性
/// - `invisible_paints_removal`：从 fillPaints 和 StrokePaints 数组中删除不可见的paint
/// - `stack_child_properties_removal`：删除 Figma 自动布局子属性(stackChildAlignSelf、stackChildPrimaryGrow)
/// - `redundant_padding_removal`：当存在通用基于轴的填充时删除冗余填充属性
/// - `stack_sizing_properties_removal`：删除 Figma 自动布局大小调整属性(stackCounterSizing、stackPrimarySizing)
/// - `stack_align_items_removal`：删除 Figma 自动布局对齐属性(stackCounterAlignItems、stackPrimaryAlignItems)
/// - `text_properties_simplification`：将冗长的 letterSpacing/lineHeight 结构简化为 CSS 就绪字符串
/// - `type_removal`：从所有节点中删除type 字段
/// - `empty_paint_arrays_removal`：删除空的 fillPaints 和 strokePaints 数组
/// - `overridden_symbol_id_removal`：从数组中删除独立的 overridedenSymbolID 对象
/// - `symbol_id_removal`：删除仅包含 localID 和/或 sessionID 的 symbolID 对象
/// - `visible_only_objects_removal`：删除仅包含visible 属性的对象
/// - `uniform_scale_factor_removal`：删除默认的uniformScaleFactor值(1.0)
pub mod background_properties_removal;
pub mod blobs_removal;
pub mod border_weights_removal;
pub mod color_to_css;
pub mod constraint_properties_removal;
pub mod corner_smoothing_removal;
pub mod default_blend_mode_removal;
pub mod default_opacity_removal;
pub mod default_rotation_removal;
pub mod default_text_properties_removal;
pub mod default_visible_removal;
pub mod derived_text_layout_size_removal;
pub mod detached_symbol_id_removal;
pub mod document_properties_removal;
pub mod edit_info_removal;
pub mod empty_derived_text_data_removal;
pub mod empty_font_postscript_removal;
pub mod empty_objects_removal;
pub mod empty_paint_arrays_removal;
pub mod enum_simplification;
pub mod export_settings_removal;
pub mod frame_properties_removal;
pub mod geometry_removal;
pub mod guid_path_removal;
pub mod guid_removal;
pub mod image_hash;
pub mod image_metadata_removal;
pub mod internal_only_nodes_removal;
pub mod invisible_paints_removal;
pub mod layout_aids_removal;
pub mod matrix_to_css;
pub mod overridden_symbol_id_removal;
pub mod phase_removal;
pub mod plugin_data_removal;
pub mod rectangle_corner_radii_independent_removal;
pub mod redundant_corner_radii_removal;
pub mod redundant_padding_removal;
pub mod root_metadata_removal;
pub mod scroll_resize_properties_removal;
pub mod stack_align_items_removal;
pub mod stack_child_properties_removal;
pub mod stack_sizing_properties_removal;
pub mod stroke_properties_removal;
pub mod style_id_removal;
pub mod symbol_id_removal;
pub mod text_glyphs_removal;
pub mod text_layout_removal;
pub mod text_line_defaults_removal;
pub mod text_metadata_removal;
pub mod text_properties_simplification;
pub mod type_removal;
pub mod uniform_scale_factor_removal;
pub mod user_facing_version_removal;
pub mod visible_only_objects_removal;

// 重新导出常用函数
pub use background_properties_removal::remove_background_properties;
pub use blobs_removal::remove_root_blobs;
pub use border_weights_removal::remove_border_weights;
pub use color_to_css::transform_colors_to_css;
pub use constraint_properties_removal::remove_constraint_properties;
pub use corner_smoothing_removal::remove_corner_smoothing;
pub use default_blend_mode_removal::remove_default_blend_mode;
pub use default_opacity_removal::remove_default_opacity;
pub use default_rotation_removal::remove_default_rotation;
pub use default_text_properties_removal::remove_default_text_properties;
pub use default_visible_removal::remove_default_visible;
pub use derived_text_layout_size_removal::remove_derived_text_layout_size;
pub use detached_symbol_id_removal::remove_detached_symbol_id;
pub use document_properties_removal::remove_document_properties;
pub use edit_info_removal::remove_edit_info_fields;
pub use empty_derived_text_data_removal::remove_empty_derived_text_data;
pub use empty_font_postscript_removal::remove_empty_font_postscript;
pub use empty_objects_removal::remove_empty_objects;
pub use empty_paint_arrays_removal::remove_empty_paint_arrays;
pub use enum_simplification::simplify_enums;
pub use export_settings_removal::remove_export_settings;
pub use frame_properties_removal::remove_frame_properties;
pub use geometry_removal::remove_geometry_fields;
pub use guid_path_removal::remove_guid_paths;
pub use guid_removal::remove_guid_fields;
pub use image_hash::transform_image_hashes;
pub use image_metadata_removal::remove_image_metadata_fields;
pub use internal_only_nodes_removal::remove_internal_only_nodes;
pub use invisible_paints_removal::remove_invisible_paints;
pub use layout_aids_removal::remove_layout_aids;
pub use matrix_to_css::transform_matrix_to_css;
pub use overridden_symbol_id_removal::remove_overridden_symbol_id;
pub use phase_removal::remove_phase_fields;
pub use plugin_data_removal::remove_plugin_data;
pub use rectangle_corner_radii_independent_removal::remove_rectangle_corner_radii_independent;
pub use redundant_corner_radii_removal::remove_redundant_corner_radii;
pub use redundant_padding_removal::remove_redundant_padding;
pub use root_metadata_removal::remove_root_metadata;
pub use scroll_resize_properties_removal::remove_scroll_resize_properties;
pub use stack_align_items_removal::remove_stack_align_items;
pub use stack_child_properties_removal::remove_stack_child_properties;
pub use stack_sizing_properties_removal::remove_stack_sizing_properties;
pub use stroke_properties_removal::remove_stroke_properties;
pub use style_id_removal::remove_style_ids;
pub use symbol_id_removal::remove_symbol_id_fields;
pub use text_glyphs_removal::remove_text_glyphs;
pub use text_layout_removal::remove_text_layout_fields;
pub use text_line_defaults_removal::remove_default_text_line_properties;
pub use text_metadata_removal::remove_text_metadata_fields;
pub use text_properties_simplification::simplify_text_properties;
pub use type_removal::remove_type;
pub use uniform_scale_factor_removal::remove_default_uniform_scale_factor;
pub use user_facing_version_removal::remove_user_facing_versions;
pub use visible_only_objects_removal::remove_visible_only_objects;
