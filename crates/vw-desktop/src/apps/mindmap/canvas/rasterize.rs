/// 将 SVG 数据渲染为 PNG 图像
///
/// # 参数
///
/// * `svg_data` - SVG XML 字符串
///
/// # 返回值
///
/// 成功时返回 `Some(Vec<u8>)`，包含 PNG 图像的二进制数据；
/// 解析或渲染失败时返回 `None`。
///
/// # 功能说明
///
/// 此函数执行以下步骤：
/// 1. 配置 SVG 解析选项，加载系统字体
/// 2. 将 SVG 字符串解析为 SVG 树
/// 3. 创建与 SVG 尺寸匹配的像素图
/// 4. 将 SVG 渲染到像素图
/// 5. 将像素图编码为 PNG 格式
///
/// # 平台限制
///
/// 此函数仅在非 WASM 平台可用，依赖 `resvg` 和 `tiny_skia` 库。
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn render_svg_to_png(svg_data: &str) -> Option<Vec<u8>> {
    use resvg::usvg::{self, Tree};
    use tiny_skia::{Pixmap, Transform};

    let mut opt = usvg::Options::default();

    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    opt.fontdb = std::sync::Arc::new(fontdb);

    let tree = Tree::from_str(svg_data, &opt).ok()?;
    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height())?;

    let mut pm = pixmap.as_mut();
    resvg::render(&tree, Transform::default(), &mut pm);
    pixmap.encode_png().ok()
}
