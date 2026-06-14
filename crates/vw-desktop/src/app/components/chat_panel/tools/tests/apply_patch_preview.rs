//! apply_patch_preview.rs 测试模块。
//!
//! 这些测试固定相邻解析器、视图辅助函数或状态计算的行为，防止后续 UI 重排时破坏边界契约。

use super::{
    ChangeFile, apply_patch_paths_match, collect_apply_patch_changes, find_apply_patch_change,
    parse_apply_patch_change_files, parse_unified_diff_change_files,
};

/// 验证 apply patch paths match accepts relative absolute and diff prefixed paths 这一行为，确保对应解析或视图契约稳定。
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
#[test]
fn apply_patch_paths_match_accepts_relative_absolute_and_diff_prefixed_paths() {
    assert!(apply_patch_paths_match("a/src/main.rs", "src/main.rs"));
    assert!(apply_patch_paths_match("/Users/demo/project/src/main.rs", "src/main.rs"));
    assert!(apply_patch_paths_match("./src/lib.rs", "b/src/lib.rs"));
}

/// 验证 find apply patch change prefers preview with full file bodies 这一行为，确保对应解析或视图契约稳定。
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
#[test]
fn find_apply_patch_change_prefers_preview_with_full_file_bodies() {
    let changes = vec![
        ChangeFile {
            // path 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            path: "/Users/demo/project/src/main.rs".into(),
            // additions 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            additions: 2,
            // deletions 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            deletions: 1,
            // before 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            before: String::new(),
            // after 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            after: String::new(),
        },
        ChangeFile {
            // path 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            path: "src/main.rs".into(),
            // additions 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            additions: 2,
            // deletions 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            deletions: 1,
            // before 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            before: "fn old_main() {}".into(),
            // after 保存该结构在渲染、解析或测试断言中需要直接访问的数据。
            after: "fn new_main() {}".into(),
        },
    ];

    let selected = find_apply_patch_change(&changes, "src/main.rs").expect("matched change");
    assert_eq!(selected.before, "fn old_main() {}");
    assert_eq!(selected.after, "fn new_main() {}");
}

/// 解析 apply patch change files builds preview from patch input 的输入文本，返回后续视图可以直接消费的结构化结果。
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
#[test]
fn parse_apply_patch_change_files_builds_preview_from_patch_input() {
    let patch = concat!(
        "*** Begin Patch\n",
        "*** Update File: src/main.rs\n",
        "@@ fn main() {\n",
        " fn main() {\n",
        "-    println!(\"old\");\n",
        "+    println!(\"new\");\n",
        " }\n",
        "*** End Patch"
    );

    let changes = parse_apply_patch_change_files(patch);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "src/main.rs");
    assert!(changes[0].before.contains("println!(\"old\")"));
    assert!(changes[0].after.contains("println!(\"new\")"));
    assert!(changes[0].additions >= 1);
    assert!(changes[0].deletions >= 1);
}

/// 验证 collect apply patch changes falls back to patch input when changes block is missing 这一行为，确保对应解析或视图契约稳定。
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
#[test]
fn collect_apply_patch_changes_falls_back_to_patch_input_when_changes_block_is_missing() {
    let input = concat!(
        "*** Begin Patch\n",
        "*** Add File: src/new.rs\n",
        "+fn main() {}\n",
        "*** End Patch"
    );

    let changes =
        collect_apply_patch_changes("Success. Updated the following files:\nA src/new.rs", input);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "src/new.rs");
    assert_eq!(changes[0].after, "fn main() {}");
}

/// 验证 collect apply patch changes returns deleted entry without preview body 这一行为，确保对应解析或视图契约稳定。
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
#[test]
fn collect_apply_patch_changes_returns_deleted_entry_without_preview_body() {
    let input = concat!("*** Begin Patch\n", "*** Delete File: src/old.rs\n", "*** End Patch");

    let changes =
        collect_apply_patch_changes("Success. Updated the following files:\nD src/old.rs", input);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "src/old.rs");
    assert!(changes[0].before.is_empty());
    assert!(changes[0].after.is_empty());
}

/// 验证 collect apply patch changes prefers diff body over empty changes entry for deleted file 这一行为，确保对应解析或视图契约稳定。
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
#[test]
fn collect_apply_patch_changes_prefers_diff_body_over_empty_changes_entry_for_deleted_file() {
    let output = concat!(
        "Success. Updated the following files:\n",
        "D src/old.rs\n\n",
        "<diff>\n",
        "--- a/src/old.rs\n",
        "+++ /dev/null\n",
        "@@\n",
        "-old line\n",
        "-another line\n",
        "</diff>\n\n",
        "<changes>\n",
        "{\"files\":[{\"path\":\"src/old.rs\",\"additions\":0,\"deletions\":2,\"before\":\"\",\"after\":\"\"}]}\n",
        "</changes>"
    );

    let changes = collect_apply_patch_changes(output, "");
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "src/old.rs");
    assert!(changes[0].before.contains("old line"));
    assert!(changes[0].before.contains("another line"));
    assert!(changes[0].after.is_empty());
}

#[test]
fn parse_unified_diff_change_files_handles_add_modify_delete_and_metadata() {
    let diff = concat!(
        "diff --git a/src/new.rs b/src/new.rs\n",
        "new file mode 100644\n",
        "--- /dev/null\n",
        "+++ b/src/new.rs\n",
        "@@ -0,0 +1 @@\n",
        "+new\n",
        "diff --git a/src/mod.rs b/src/mod.rs\n",
        "index 111..222 100644\n",
        "--- a/src/mod.rs\n",
        "+++ b/src/mod.rs\n",
        "@@ -1 +1 @@\n",
        "-old\n",
        "+new\n",
        "\\ No newline at end of file\n",
        "diff --git a/src/old.rs b/src/old.rs\n",
        "deleted file mode 100644\n",
        "--- a/src/old.rs\n",
        "+++ /dev/null\n",
        "@@ -1 +0,0 @@\n",
        "-old\n"
    );

    let changes = parse_unified_diff_change_files(diff);
    assert_eq!(changes.len(), 3);
    assert_eq!(changes[0].path, "src/new.rs");
    assert_eq!(changes[0].additions, 1);
    assert_eq!(changes[1].deletions, 1);
    assert_eq!(changes[2].path, "src/old.rs");
    assert!(changes[2].after.is_empty());
}

#[test]
fn collect_apply_patch_changes_prefers_changes_block_then_fills_missing_diff_fields() {
    let output = concat!(
        "<diff>\n",
        "--- a/src/main.rs\n",
        "+++ b/src/main.rs\n",
        "@@\n",
        "-old\n",
        "+new\n",
        "</diff>\n",
        "<changes>\n",
        "{\"files\":[{\"path\":\"src/main.rs\",\"additions\":1,\"deletions\":1,\"before\":\"\",\"after\":\"\"}]}\n",
        "</changes>"
    );

    let changes = collect_apply_patch_changes(output, "");
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "src/main.rs");
    assert_eq!(changes[0].before, "old");
    assert_eq!(changes[0].after, "new");
}

#[test]
fn parse_apply_patch_change_files_returns_empty_for_invalid_patch() {
    assert!(parse_apply_patch_change_files("not a patch").is_empty());
}
