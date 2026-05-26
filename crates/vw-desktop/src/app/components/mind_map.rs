//! 思维导图数据模型。
//!
//! 本模块提供 Markdown 列表与树形节点之间的转换，并支持按路径编辑节点。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MindNode {
    // text 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
    pub text: String,
    // children 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
    pub children: Vec<MindNode>,
}

impl Default for MindNode {
    /// 处理 default 对应的局部职责。
    ///
    /// # 参数
    ///
    /// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
    ///
    /// # 返回值
    ///
    /// 返回值保持当前模块的领域语义，供相邻视图或测试继续使用。
    ///
    /// # 错误处理
    ///
    /// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
    fn default() -> Self {
        Self { text: "思维导图".to_string(), children: Vec::new() }
    }
}

/// 解析 heading root 的输入文本，返回后续视图可以直接消费的结构化结果。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// `None` 表示输入缺少必要字段、当前状态不匹配，或该视图片段不需要展示。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
fn parse_heading_root(md: &str) -> Option<String> {
    for line in md.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix('#') {
            let rest = rest.trim_start_matches('#').trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

/// 解析 list item 的输入文本，返回后续视图可以直接消费的结构化结果。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// `None` 表示输入缺少必要字段、当前状态不匹配，或该视图片段不需要展示。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
fn parse_list_item(line: &str) -> Option<(usize, String)> {
    let mut indent = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => indent += 1,
            '\t' => indent += 4,
            _ => break,
        }
    }
    let trimmed = line[indent.min(line.len())..].trim_start();
    let (after_marker, ok) = if let Some(rest) = trimmed.strip_prefix("- ") {
        (rest, true)
    } else if let Some(rest) = trimmed.strip_prefix("* ") {
        (rest, true)
    } else if let Some(rest) = trimmed.strip_prefix("+ ") {
        (rest, true)
    } else {
        let bytes = trimmed.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == 0 || i + 1 >= bytes.len() {
            ("", false)
        } else if (bytes[i] == b'.' || bytes[i] == b')') && bytes[i + 1] == b' ' {
            (&trimmed[i + 2..], true)
        } else {
            ("", false)
        }
    };

    if !ok {
        return None;
    }
    let text = after_marker.trim().to_string();
    if text.is_empty() {
        return None;
    }
    let depth = indent / 2 + 1;
    Some((depth, text))
}

/// 处理 parse 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 返回值保持当前模块的领域语义，供相邻视图或测试继续使用。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn parse(md: &str) -> MindNode {
    /// Flat 保存 mind_map 模块需要跨函数传递的状态。
    ///
    /// 字段保持贴近调用方的真实数据，避免在 UI 边界处隐藏额外转换。
    #[derive(Clone)]
    struct Flat {
        // text 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
        text: String,
        // children 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
        children: Vec<usize>,
    }

    let root_text = parse_heading_root(md).unwrap_or_else(|| "思维导图".to_string());
    let mut flat: Vec<Flat> = vec![Flat { text: root_text, children: Vec::new() }];
    let mut stack: Vec<(usize, usize)> = vec![(0, 0)];

    for line in md.lines() {
        let Some((mut depth, text)) = parse_list_item(line) else {
            continue;
        };

        let parent_depth = stack.last().map(|(d, _)| *d).unwrap_or(0);
        if depth > parent_depth + 1 {
            depth = parent_depth + 1;
        }

        while stack.last().map(|(d, _)| *d).unwrap_or(0) >= depth {
            stack.pop();
        }
        let parent = stack.last().map(|(_, i)| *i).unwrap_or(0);

        let idx = flat.len();
        flat.push(Flat { text, children: Vec::new() });
        flat[parent].children.push(idx);
        stack.push((depth, idx));
    }

    /// 处理 build 对应的局部职责。
    ///
    /// # 参数
    ///
    /// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
    ///
    /// # 返回值
    ///
    /// 返回值保持当前模块的领域语义，供相邻视图或测试继续使用。
    ///
    /// # 错误处理
    ///
    /// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
    fn build(idx: usize, flat: &[Flat]) -> MindNode {
        MindNode {
            // text 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            text: flat[idx].text.clone(),
            // children 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            children: flat[idx].children.iter().map(|&c| build(c, flat)).collect(),
        }
    }

    build(0, &flat)
}

/// 处理 to markdown 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 返回字符串已经按界面展示或比较需求做过必要整理。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn to_markdown(root: &MindNode) -> String {
    /// 渲染 children 对应的 diff 行、工具卡片或控件内容。
    ///
    /// # 参数
    ///
    /// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
    ///
    /// # 返回值
    ///
    /// 无返回值时，函数通过发布消息或更新局部状态完成交互。
    ///
    /// # 错误处理
    ///
    /// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
    fn render_children(out: &mut String, nodes: &[MindNode], depth: usize) {
        for n in nodes {
            let indent = "  ".repeat(depth);
            out.push_str(&indent);
            out.push_str("- ");
            out.push_str(n.text.trim());
            out.push('\n');
            if !n.children.is_empty() {
                render_children(out, &n.children, depth + 1);
            }
        }
    }

    let mut out = String::new();
    out.push_str("# ");
    out.push_str(root.text.trim());
    out.push('\n');
    if !root.children.is_empty() {
        out.push('\n');
        render_children(&mut out, &root.children, 0);
    }
    out
}

/// 处理 node text 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// `None` 表示输入缺少必要字段、当前状态不匹配，或该视图片段不需要展示。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn node_text<'a>(root: &'a MindNode, path: &[usize]) -> Option<&'a str> {
    let mut cur = root;
    for &i in path {
        cur = cur.children.get(i)?;
    }
    Some(cur.text.as_str())
}

/// 处理 node text mut 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// `None` 表示输入缺少必要字段、当前状态不匹配，或该视图片段不需要展示。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn node_text_mut<'a>(root: &'a mut MindNode, path: &[usize]) -> Option<&'a mut String> {
    let mut cur = root;
    for &i in path {
        cur = cur.children.get_mut(i)?;
    }
    Some(&mut cur.text)
}

/// 处理 node 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// `None` 表示输入缺少必要字段、当前状态不匹配，或该视图片段不需要展示。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn node<'a>(root: &'a MindNode, path: &[usize]) -> Option<&'a MindNode> {
    let mut cur = root;
    for &i in path {
        cur = cur.children.get(i)?;
    }
    Some(cur)
}

/// 处理 node mut 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// `None` 表示输入缺少必要字段、当前状态不匹配，或该视图片段不需要展示。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn node_mut<'a>(root: &'a mut MindNode, path: &[usize]) -> Option<&'a mut MindNode> {
    let mut cur = root;
    for &i in path {
        cur = cur.children.get_mut(i)?;
    }
    Some(cur)
}

/// 处理 path exists 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// `true` 表示当前输入满足该辅助函数描述的条件。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn path_exists(root: &MindNode, path: &[usize]) -> bool {
    node_text(root, path).is_some()
}

/// 处理 add child 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// `None` 表示输入缺少必要字段、当前状态不匹配，或该视图片段不需要展示。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn add_child(root: &mut MindNode, path: &[usize], text: String) -> Option<Vec<usize>> {
    let mut cur = root;
    for &i in path {
        cur = cur.children.get_mut(i)?;
    }
    cur.children.push(MindNode { text, children: Vec::new() });
    let mut new_path = path.to_vec();
    new_path.push(cur.children.len() - 1);
    Some(new_path)
}

/// 处理 insert child node 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn insert_child_node(
    root: &mut MindNode,
    // path 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
    path: &[usize],
    // node 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
    node: MindNode,
) -> Option<Vec<usize>> {
    let cur = node_mut(root, path)?;
    cur.children.push(node);
    let mut new_path = path.to_vec();
    new_path.push(cur.children.len() - 1);
    Some(new_path)
}

/// 处理 add sibling 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// `None` 表示输入缺少必要字段、当前状态不匹配，或该视图片段不需要展示。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn add_sibling(root: &mut MindNode, path: &[usize], text: String) -> Option<Vec<usize>> {
    if path.is_empty() {
        return add_child(root, &[], text);
    }
    let mut parent = root;
    for &i in &path[..path.len() - 1] {
        parent = parent.children.get_mut(i)?;
    }
    let idx = *path.last()?;
    let insert_at = (idx + 1).min(parent.children.len());
    parent.children.insert(insert_at, MindNode { text, children: Vec::new() });
    let mut new_path = path.to_vec();
    *new_path.last_mut().unwrap() = insert_at;
    Some(new_path)
}

/// 处理 insert sibling node 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn insert_sibling_node(
    root: &mut MindNode,
    // path 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
    path: &[usize],
    // node 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
    node: MindNode,
) -> Option<Vec<usize>> {
    if path.is_empty() {
        return insert_child_node(root, &[], node);
    }
    let parent_path = &path[..path.len() - 1];
    let idx = *path.last()?;
    let parent = node_mut(root, parent_path)?;
    let insert_at = (idx + 1).min(parent.children.len());
    parent.children.insert(insert_at, node);
    let mut new_path = path.to_vec();
    *new_path.last_mut().unwrap() = insert_at;
    Some(new_path)
}

/// 处理 take node 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// `None` 表示输入缺少必要字段、当前状态不匹配，或该视图片段不需要展示。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn take_node(root: &mut MindNode, path: &[usize]) -> Option<MindNode> {
    if path.is_empty() {
        return None;
    }
    let parent_path = &path[..path.len() - 1];
    let idx = *path.last()?;
    let parent = node_mut(root, parent_path)?;
    if idx >= parent.children.len() {
        return None;
    }
    Some(parent.children.remove(idx))
}

/// 处理 delete node 对应的局部职责。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// `None` 表示输入缺少必要字段、当前状态不匹配，或该视图片段不需要展示。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
pub fn delete_node(root: &mut MindNode, path: &[usize]) -> Option<Vec<usize>> {
    if path.is_empty() {
        return None;
    }
    let mut parent = root;
    for &i in &path[..path.len() - 1] {
        parent = parent.children.get_mut(i)?;
    }
    let idx = *path.last()?;
    if idx >= parent.children.len() {
        return None;
    }
    parent.children.remove(idx);
    Some(path[..path.len() - 1].to_vec())
}
