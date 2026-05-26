//! `notebook_edit` 工具测试。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    use crate::app::agent::security::{AutonomyLevel, SecurityPolicy};
    use crate::app::agent::tools::FileSnapshot;
    use crate::app::agent::tools::ToolUseContext;
    use crate::app::agent::tools::context::scope_tool_use_context;
    use serde_json::json;
    use std::sync::Arc;
    use vw_api_types::tools::ToolResultContentDto;

    fn test_security(workspace: std::path::PathBuf) -> Arc<SecurityPolicy> {
        Arc::new(SecurityPolicy {
            autonomy: AutonomyLevel::Supervised,
            workspace_dir: workspace,
            ..SecurityPolicy::default()
        })
    }

    fn notebook_text(value: serde_json::Value) -> String {
        let mut text = serde_json::to_string_pretty(&value).unwrap();
        text.push('\n');
        text
    }

    fn seed_read_state(
        context: &Arc<ToolUseContext>,
        workspace: &std::path::Path,
        path: &str,
        content: &str,
    ) {
        let read_state = context.read_state_handle();
        read_state.lock().unwrap_or_else(|error| error.into_inner()).note_read(
            Some(workspace),
            path,
            content.len(),
            false,
            None,
            None,
            Some(FileSnapshot::from_text(content)),
        );
    }

    fn sample_notebook() -> serde_json::Value {
        json!({
            "cells": [
                {
                    "cell_type": "markdown",
                    "id": "cell-1",
                    "metadata": {"id": "cell-1", "language": "markdown"},
                    "source": ["# Title\n"]
                },
                {
                    "cell_type": "code",
                    "id": "cell-2",
                    "metadata": {"id": "cell-2", "language": "python"},
                    "source": ["print('old')\n"],
                    "outputs": [],
                    "execution_count": null
                }
            ],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        })
    }

    #[test]
    fn notebook_edit_name() {
        let tool = NotebookEditTool::new(test_security(std::env::temp_dir()));
        assert_eq!(tool.name(), "notebook_edit");
    }

    #[tokio::test]
    async fn notebook_edit_inserts_cell_at_end() {
        let dir = std::env::temp_dir().join("vibewindow_test_notebook_edit_insert");
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.unwrap();

        let original = notebook_text(sample_notebook());
        tokio::fs::write(dir.join("demo.ipynb"), &original).await.unwrap();

        let tool = NotebookEditTool::new(test_security(dir.clone()));
        let context = Arc::new(ToolUseContext::new(
            "notebook-edit-insert",
            Some(dir.to_string_lossy().to_string()),
        ));
        seed_read_state(&context, dir.as_path(), "demo.ipynb", &original);

        let result = scope_tool_use_context(
            context.clone(),
            tool.call(json!({
                "path": "demo.ipynb",
                "edit_type": "insert",
                "position": "end",
                "cell": {
                    "cell_type": "code",
                    "metadata": {"language": "python"},
                    "source": ["print('new')\n"],
                    "outputs": [],
                    "execution_count": null
                }
            })),
        )
        .await
        .unwrap();

        assert!(result.is_success(), "error: {:?}", result.error_text());
        assert_eq!(result.data["kind"], json!("insert"));
        assert_eq!(result.data["cell"]["cell_number"], json!(3));
        assert_eq!(
            result.render_hint.as_ref().and_then(|hint| hint.kind.as_deref()),
            Some("notebook_edit")
        );
        assert!(matches!(
            result.content_blocks.first(),
            Some(ToolResultContentDto::StructuredPatch { hunks }) if !hunks.is_empty()
        ));

        let written: serde_json::Value =
            serde_json::from_str(&tokio::fs::read_to_string(dir.join("demo.ipynb")).await.unwrap())
                .unwrap();
        assert_eq!(written["cells"].as_array().unwrap().len(), 3);
        assert!(written["cells"][2]["id"].as_str().is_some_and(|value| !value.is_empty()));

        let mut snapshot = context.read_state_snapshot();
        let entry = snapshot
            .get(Some(dir.as_path()), "demo.ipynb")
            .expect("read state missing");
        assert_eq!(
            entry.snapshot,
            Some(FileSnapshot::from_text(
                &tokio::fs::read_to_string(dir.join("demo.ipynb")).await.unwrap()
            ))
        );

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn notebook_edit_replaces_cell_by_id() {
        let dir = std::env::temp_dir().join("vibewindow_test_notebook_edit_replace");
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.unwrap();

        let original = notebook_text(sample_notebook());
        tokio::fs::write(dir.join("demo.ipynb"), &original).await.unwrap();

        let tool = NotebookEditTool::new(test_security(dir.clone()));
        let context = Arc::new(ToolUseContext::new(
            "notebook-edit-replace",
            Some(dir.to_string_lossy().to_string()),
        ));
        seed_read_state(&context, dir.as_path(), "demo.ipynb", &original);

        let result = scope_tool_use_context(
            context,
            tool.call(json!({
                "path": "demo.ipynb",
                "edit_type": "edit",
                "cell_id": "cell-2",
                "cell": {
                    "cell_type": "code",
                    "metadata": {"language": "python"},
                    "source": ["print('updated')\n"],
                    "outputs": [],
                    "execution_count": null
                }
            })),
        )
        .await
        .unwrap();

        assert!(result.is_success(), "error: {:?}", result.error_text());
        assert_eq!(result.data["kind"], json!("edit"));
        assert_eq!(result.data["cell"]["cell_id"], json!("cell-2"));

        let written: serde_json::Value =
            serde_json::from_str(&tokio::fs::read_to_string(dir.join("demo.ipynb")).await.unwrap())
                .unwrap();
        assert_eq!(written["cells"][1]["source"][0], json!("print('updated')\n"));
        assert_eq!(written["cells"][1]["id"], json!("cell-2"));

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn notebook_edit_deletes_cell_by_number() {
        let dir = std::env::temp_dir().join("vibewindow_test_notebook_edit_delete");
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.unwrap();

        let original = notebook_text(sample_notebook());
        tokio::fs::write(dir.join("demo.ipynb"), &original).await.unwrap();

        let tool = NotebookEditTool::new(test_security(dir.clone()));
        let context = Arc::new(ToolUseContext::new(
            "notebook-edit-delete",
            Some(dir.to_string_lossy().to_string()),
        ));
        seed_read_state(&context, dir.as_path(), "demo.ipynb", &original);

        let result = scope_tool_use_context(
            context,
            tool.call(json!({
                "path": "demo.ipynb",
                "edit_type": "delete",
                "cell_number": 1
            })),
        )
        .await
        .unwrap();

        assert!(result.is_success(), "error: {:?}", result.error_text());
        assert_eq!(result.data["kind"], json!("delete"));
        assert_eq!(result.data["total_cells"], json!(1));

        let written: serde_json::Value =
            serde_json::from_str(&tokio::fs::read_to_string(dir.join("demo.ipynb")).await.unwrap())
                .unwrap();
        assert_eq!(written["cells"].as_array().unwrap().len(), 1);
        assert_eq!(written["cells"][0]["id"], json!("cell-2"));

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn notebook_edit_rejects_non_notebook_paths() {
        let dir = std::env::temp_dir().join("vibewindow_test_notebook_edit_reject_plain");
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.unwrap();
        tokio::fs::write(dir.join("demo.txt"), "hello\n").await.unwrap();

        let tool = NotebookEditTool::new(test_security(dir.clone()));
        let context = Arc::new(ToolUseContext::new(
            "notebook-edit-reject-plain",
            Some(dir.to_string_lossy().to_string()),
        ));
        seed_read_state(&context, dir.as_path(), "demo.txt", "hello\n");

        let result = scope_tool_use_context(
            context,
            tool.call(json!({
                "path": "demo.txt",
                "edit_type": "delete",
                "cell_number": 1
            })),
        )
        .await
        .unwrap();

        assert!(!result.is_success());
        assert!(
            result
                .error_text()
                .unwrap_or_default()
                .contains("NotebookEdit only supports .ipynb files")
        );

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}