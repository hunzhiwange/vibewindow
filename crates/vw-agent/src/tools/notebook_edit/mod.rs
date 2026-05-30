//! Notebook 专用编辑工具。
//!
//! `notebook_edit` 只负责 `.ipynb` 文件的结构化 cell 编辑：
//! - `insert`: 插入新 cell
//! - `edit`: 替换现有 cell
//! - `delete`: 删除现有 cell
//!
//! 与普通文本 `edit` / `file_write` 不同，此工具只接受 notebook 路径，
//! 并显式按 cell id / cell number / insert position 工作，不提供伪文本支持。

use super::context::note_current_read_state;
use super::external_directory;
use super::file_edit::types::{FileDescriptor, StructuredPatch};
use super::file_edit::utils::{
    build_file_descriptor, build_patch_summary, display_path, ensure_read_state_is_fresh,
    read_state_metadata_for_path, require_read_state_for_existing_file, snapshot_from_text,
};
use super::traits::{Tool, ToolCallResult, ToolCallTelemetry, ToolRenderHint, ToolResult};
use crate::app::agent::file;
use crate::app::agent::file::watcher;
use crate::app::agent::security::SecurityPolicy;
use anyhow::anyhow;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use vw_api_types::tools::ToolResultContentDto;

/// Notebook 编辑工具。
pub struct NotebookEditTool {
    security: Arc<SecurityPolicy>,
}

#[derive(Debug, Clone, Deserialize)]
struct Args {
    #[serde(alias = "filePath", alias = "file_path")]
    path: String,
    #[serde(alias = "editType", alias = "operation")]
    edit_type: NotebookEditOperation,
    #[serde(default, alias = "cellId")]
    cell_id: Option<String>,
    #[serde(default)]
    cell_number: Option<usize>,
    #[serde(default)]
    position: Option<InsertPosition>,
    #[serde(default)]
    cell: Option<NotebookCell>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NotebookEditOperation {
    Insert,
    Edit,
    Delete,
}

impl NotebookEditOperation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Insert => "insert",
            Self::Edit => "edit",
            Self::Delete => "delete",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InsertPosition {
    Before,
    After,
    Start,
    End,
}

impl InsertPosition {
    fn as_str(self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::After => "after",
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct NotebookDocument {
    #[serde(default)]
    cells: Vec<NotebookCell>,
    #[serde(default)]
    metadata: Value,
    #[serde(default)]
    nbformat: Option<u32>,
    #[serde(default)]
    nbformat_minor: Option<u32>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct NotebookCell {
    #[serde(default)]
    cell_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(default)]
    metadata: Value,
    #[serde(default)]
    source: Value,
    #[serde(default)]
    outputs: Vec<Value>,
    #[serde(default)]
    execution_count: Value,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl NotebookCell {
    fn resolved_id(&self) -> Option<String> {
        self.id
            .clone()
            .or_else(|| self.metadata.get("id").and_then(Value::as_str).map(ToOwned::to_owned))
    }
}

#[derive(Debug, Clone, Serialize)]
struct NotebookCellDescriptor {
    cell_number: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    cell_id: Option<String>,
    cell_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum NotebookEditPayload {
    Insert {
        notebook: FileDescriptor,
        cell: NotebookCellDescriptor,
        position: String,
        total_cells: usize,
        structured_patch: StructuredPatch,
        #[serde(skip_serializing_if = "Option::is_none")]
        read_state: Option<Value>,
    },
    Edit {
        notebook: FileDescriptor,
        cell: NotebookCellDescriptor,
        changed: bool,
        total_cells: usize,
        structured_patch: StructuredPatch,
        #[serde(skip_serializing_if = "Option::is_none")]
        read_state: Option<Value>,
    },
    Delete {
        notebook: FileDescriptor,
        cell: NotebookCellDescriptor,
        total_cells: usize,
        structured_patch: StructuredPatch,
        #[serde(skip_serializing_if = "Option::is_none")]
        read_state: Option<Value>,
    },
}

#[derive(Debug, Clone)]
struct NotebookEditResponse {
    model_text: String,
    payload: NotebookEditPayload,
    render_hint: ToolRenderHint,
    patch_hunks: Vec<vw_api_types::tools::StructuredPatchHunkDto>,
}

impl NotebookEditResponse {
    fn into_tool_call_result(self) -> ToolCallResult {
        let content_blocks = if self.patch_hunks.is_empty() {
            Vec::new()
        } else {
            vec![ToolResultContentDto::StructuredPatch { hunks: self.patch_hunks }]
        };

        ToolCallResult {
            data: serde_json::to_value(&self.payload).unwrap_or(Value::Null),
            model_result: Value::String(self.model_text),
            content_blocks,
            render_hint: Some(self.render_hint),
            telemetry: Some(ToolCallTelemetry { success: true, ..ToolCallTelemetry::default() }),
            ..ToolCallResult::default()
        }
    }
}

impl NotebookEditTool {
    fn failure(msg: impl Into<String>) -> ToolResult {
        let msg = msg.into();
        ToolResult { success: false, output: msg.clone(), error: Some(msg) }
    }

    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self { security }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let candidate = PathBuf::from(path);
        if candidate.is_absolute() {
            candidate
        } else {
            self.security.workspace_dir.join(candidate)
        }
    }

    async fn ensure_requested_path_allowed(&self, path: &str) -> anyhow::Result<()> {
        if !self.security.is_path_allowed(path) {
            anyhow::bail!("Path not allowed by security policy: {path}")
        }

        external_directory::assert_external_directory(
            &self.security,
            Some(path),
            Some(external_directory::Options {
                bypass: false,
                kind: external_directory::Kind::File,
            }),
        )
        .await
        .map_err(anyhow::Error::msg)
    }

    async fn ensure_existing_file_allowed(&self, path: &Path) -> anyhow::Result<PathBuf> {
        if !path.exists() {
            anyhow::bail!("NotebookEdit requires an existing notebook file: {}", path.display());
        }
        let meta = tokio::fs::symlink_metadata(path)
            .await
            .map_err(|error| anyhow!("Failed to read file metadata {}: {error}", path.display()))?;
        if meta.file_type().is_symlink() {
            anyhow::bail!("Refusing to edit through symlink: {}", path.display());
        }
        if !meta.is_file() {
            anyhow::bail!("Path is a directory, not a file: {}", path.display());
        }

        let resolved = tokio::fs::canonicalize(path)
            .await
            .map_err(|error| anyhow!("Failed to resolve file path {}: {error}", path.display()))?;
        if !self.security.is_resolved_path_allowed(&resolved) {
            anyhow::bail!(self.security.resolved_path_violation_message(&resolved));
        }
        Ok(resolved)
    }

    fn is_notebook_path(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ipynb"))
    }

    fn normalize_document(document: &mut NotebookDocument) -> anyhow::Result<()> {
        if document.metadata.is_null() {
            document.metadata = Value::Object(Map::new());
        }
        if !document.metadata.is_object() {
            anyhow::bail!("Notebook metadata must be a JSON object");
        }
        Ok(())
    }

    fn notebook_cell_language(metadata: &Value) -> Option<String> {
        metadata
            .get("language")
            .and_then(Value::as_str)
            .or_else(|| {
                metadata
                    .get("language_info")
                    .and_then(|value| value.get("name"))
                    .and_then(Value::as_str)
            })
            .or_else(|| {
                metadata
                    .get("vscode")
                    .and_then(|value| value.get("languageId"))
                    .and_then(Value::as_str)
            })
            .map(ToOwned::to_owned)
    }

    fn cell_descriptor(cell: &NotebookCell, index: usize) -> NotebookCellDescriptor {
        NotebookCellDescriptor {
            cell_number: index + 1,
            cell_id: cell.resolved_id(),
            cell_type: cell.cell_type.clone(),
            language: Self::notebook_cell_language(&cell.metadata),
        }
    }

    fn generate_cell_id() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        format!("cell-{nanos:x}")
    }

    fn prepare_cell(
        mut cell: NotebookCell,
        forced_id: Option<String>,
    ) -> anyhow::Result<NotebookCell> {
        if cell.cell_type.trim().is_empty() {
            anyhow::bail!("cell.cell_type must not be empty");
        }

        if cell.metadata.is_null() {
            cell.metadata = Value::Object(Map::new());
        }
        if !cell.metadata.is_object() {
            anyhow::bail!("cell.metadata must be a JSON object");
        }

        match &cell.source {
            Value::String(_) => {}
            Value::Array(items) if items.iter().all(Value::is_string) => {}
            _ => anyhow::bail!("cell.source must be a string or an array of strings"),
        }

        let resolved_id =
            forced_id.or_else(|| cell.resolved_id()).unwrap_or_else(Self::generate_cell_id);
        cell.id = Some(resolved_id.clone());
        if let Some(metadata) = cell.metadata.as_object_mut() {
            metadata.insert("id".to_string(), Value::String(resolved_id));
        }

        if cell.cell_type != "code" {
            cell.outputs.clear();
            cell.execution_count = Value::Null;
        }

        Ok(cell)
    }

    fn resolve_cell_index(
        cells: &[NotebookCell],
        cell_id: Option<&str>,
        cell_number: Option<usize>,
    ) -> anyhow::Result<usize> {
        let index_by_id = cell_id
            .map(|target_id| {
                cells
                    .iter()
                    .position(|cell| cell.resolved_id().as_deref() == Some(target_id))
                    .ok_or_else(|| anyhow!("Notebook cell id not found: {target_id}"))
            })
            .transpose()?;

        let index_by_number = cell_number
            .map(|number| {
                if number == 0 || number > cells.len() {
                    anyhow::bail!("Notebook cell number out of range: {number}");
                }
                Ok(number - 1)
            })
            .transpose()?;

        match (index_by_id, index_by_number) {
            (Some(left), Some(right)) if left != right => {
                anyhow::bail!("cell_id and cell_number refer to different notebook cells")
            }
            (Some(index), _) | (_, Some(index)) => Ok(index),
            (None, None) => {
                anyhow::bail!("NotebookEdit requires cell_id or cell_number for this operation")
            }
        }
    }

    fn resolve_insert_index(
        cells: &[NotebookCell],
        cell_id: Option<&str>,
        cell_number: Option<usize>,
        position: Option<InsertPosition>,
    ) -> anyhow::Result<(usize, InsertPosition)> {
        let has_selector = cell_id.is_some() || cell_number.is_some();
        let position = position.unwrap_or(if has_selector {
            InsertPosition::After
        } else {
            InsertPosition::End
        });

        match position {
            InsertPosition::Start => {
                if has_selector {
                    anyhow::bail!("position=start cannot be combined with cell_id or cell_number");
                }
                Ok((0, position))
            }
            InsertPosition::End => {
                if has_selector {
                    anyhow::bail!("position=end cannot be combined with cell_id or cell_number");
                }
                Ok((cells.len(), position))
            }
            InsertPosition::Before => {
                let target = Self::resolve_cell_index(cells, cell_id, cell_number)?;
                Ok((target, position))
            }
            InsertPosition::After => {
                let target = Self::resolve_cell_index(cells, cell_id, cell_number)?;
                Ok((target + 1, position))
            }
        }
    }

    fn serialize_document(document: &NotebookDocument) -> anyhow::Result<String> {
        let mut text = serde_json::to_string_pretty(document)
            .map_err(|error| anyhow!("Failed to serialize notebook JSON: {error}"))?;
        text.push('\n');
        Ok(text)
    }

    async fn execute_internal(&self, args: Args) -> anyhow::Result<NotebookEditResponse> {
        if args.path.trim().is_empty() {
            anyhow::bail!("Missing path");
        }
        if !self.security.can_act() {
            anyhow::bail!("Action blocked: autonomy is read-only");
        }
        if self.security.is_rate_limited() {
            anyhow::bail!("Rate limit exceeded: too many actions in the last hour");
        }

        self.ensure_requested_path_allowed(&args.path).await?;
        let requested_path = self.resolve_path(&args.path);
        if !Self::is_notebook_path(&requested_path) {
            anyhow::bail!(
                "NotebookEdit only supports .ipynb files; use edit/file_write for plain text files"
            );
        }

        let resolved = self.ensure_existing_file_allowed(&requested_path).await?;
        let display = display_path(&self.security.workspace_dir, &resolved);
        let current_text = tokio::fs::read_to_string(&resolved)
            .await
            .map_err(|error| anyhow!("Failed to read notebook file: {error}"))?;
        let read_state =
            require_read_state_for_existing_file(&resolved, &display, "notebook_edit")?;
        ensure_read_state_is_fresh(&read_state, &current_text, &display, "notebook_edit")?;
        let read_state_metadata =
            read_state_metadata_for_path(&self.security.workspace_dir, &resolved, "notebook_edit");

        let mut document: NotebookDocument = serde_json::from_str(&current_text)
            .map_err(|error| anyhow!("Failed to parse notebook JSON: {error}"))?;
        Self::normalize_document(&mut document)?;

        let (payload, model_text, updated_text) = match args.edit_type {
            NotebookEditOperation::Insert => {
                let mut cell = args
                    .cell
                    .ok_or_else(|| anyhow!("NotebookEdit insert requires a cell payload"))?;
                cell = Self::prepare_cell(cell, None)?;
                let (insert_index, position) = Self::resolve_insert_index(
                    &document.cells,
                    args.cell_id.as_deref(),
                    args.cell_number,
                    args.position,
                )?;
                document.cells.insert(insert_index, cell.clone());
                let descriptor = Self::cell_descriptor(&cell, insert_index);
                let updated_text = Self::serialize_document(&document)?;
                let patch_summary = build_patch_summary(&display, &current_text, &updated_text);
                let notebook = build_file_descriptor(
                    &self.security.workspace_dir,
                    &resolved,
                    updated_text.len() as u64,
                );
                let payload = NotebookEditPayload::Insert {
                    notebook: notebook.clone(),
                    cell: descriptor.clone(),
                    position: position.as_str().to_string(),
                    total_cells: document.cells.len(),
                    structured_patch: StructuredPatch { hunks: patch_summary.hunks.clone() },
                    read_state: read_state_metadata.clone(),
                };
                let model_text =
                    format!("Inserted cell {} into {}.", descriptor.cell_number, notebook.path);
                (payload, model_text, (updated_text, patch_summary.hunks, true))
            }
            NotebookEditOperation::Edit => {
                let index = Self::resolve_cell_index(
                    &document.cells,
                    args.cell_id.as_deref(),
                    args.cell_number,
                )?;
                let existing_id = document.cells[index].resolved_id();
                let mut replacement = args
                    .cell
                    .ok_or_else(|| anyhow!("NotebookEdit edit requires a cell payload"))?;
                replacement = Self::prepare_cell(replacement, existing_id)?;
                document.cells[index] = replacement.clone();
                let descriptor = Self::cell_descriptor(&replacement, index);
                let updated_text = Self::serialize_document(&document)?;
                let patch_summary = build_patch_summary(&display, &current_text, &updated_text);
                let changed = updated_text != current_text;
                let notebook = build_file_descriptor(
                    &self.security.workspace_dir,
                    &resolved,
                    updated_text.len() as u64,
                );
                let payload = NotebookEditPayload::Edit {
                    notebook: notebook.clone(),
                    cell: descriptor.clone(),
                    changed,
                    total_cells: document.cells.len(),
                    structured_patch: StructuredPatch { hunks: patch_summary.hunks.clone() },
                    read_state: read_state_metadata.clone(),
                };
                let model_text = if changed {
                    format!("Updated cell {} in {}.", descriptor.cell_number, notebook.path)
                } else {
                    format!(
                        "Matched cell {} in {}, but the replacement did not change notebook contents.",
                        descriptor.cell_number, notebook.path
                    )
                };
                (payload, model_text, (updated_text, patch_summary.hunks, changed))
            }
            NotebookEditOperation::Delete => {
                let index = Self::resolve_cell_index(
                    &document.cells,
                    args.cell_id.as_deref(),
                    args.cell_number,
                )?;
                let removed = document.cells.remove(index);
                let descriptor = Self::cell_descriptor(&removed, index);
                let updated_text = Self::serialize_document(&document)?;
                let patch_summary = build_patch_summary(&display, &current_text, &updated_text);
                let notebook = build_file_descriptor(
                    &self.security.workspace_dir,
                    &resolved,
                    updated_text.len() as u64,
                );
                let payload = NotebookEditPayload::Delete {
                    notebook: notebook.clone(),
                    cell: descriptor.clone(),
                    total_cells: document.cells.len(),
                    structured_patch: StructuredPatch { hunks: patch_summary.hunks.clone() },
                    read_state: read_state_metadata.clone(),
                };
                let model_text =
                    format!("Deleted cell {} from {}.", descriptor.cell_number, notebook.path);
                (payload, model_text, (updated_text, patch_summary.hunks, true))
            }
        };

        let (updated_text, patch_hunks, changed) = updated_text;
        if changed {
            if !self.security.record_action() {
                anyhow::bail!("Rate limit exceeded: action budget exhausted");
            }
            tokio::fs::write(&resolved, &updated_text)
                .await
                .map_err(|error| anyhow!("Failed to write notebook file: {error}"))?;
            file::publish_edited(display.clone());
            watcher::publish_updated(display.clone(), "change");
        }

        let _ = note_current_read_state(
            &resolved,
            updated_text.len(),
            false,
            None,
            None,
            Some(snapshot_from_text(&updated_text)),
        );

        let (operation, cell_number, cell_id, total_cells) = match &payload {
            NotebookEditPayload::Insert { cell, total_cells, .. }
            | NotebookEditPayload::Edit { cell, total_cells, .. }
            | NotebookEditPayload::Delete { cell, total_cells, .. } => {
                (args.edit_type.as_str(), cell.cell_number, cell.cell_id.clone(), *total_cells)
            }
        };

        Ok(NotebookEditResponse {
            model_text,
            payload,
            render_hint: ToolRenderHint {
                title: Some(format!("Edited notebook {}", display)),
                kind: Some("notebook_edit".to_string()),
                summary: Some(format!("{} cell {}", operation, cell_number)),
                metadata: json!({
                    "path": display,
                    "operation": operation,
                    "cellNumber": cell_number,
                    "cellId": cell_id,
                    "totalCells": total_cells,
                }),
            },
            patch_hunks,
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for NotebookEditTool {
    fn name(&self) -> &str {
        "notebook_edit"
    }

    fn description(&self) -> &str {
        "Edit existing .ipynb notebooks by inserting, replacing, or deleting cells. NotebookEdit only supports notebook paths and works by cell id, cell number, and explicit insert positions. Existing notebooks must be read first with file_read in the current tool context."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "目标 notebook 路径，必须是现有 .ipynb 文件。"
                },
                "filePath": {
                    "type": "string",
                    "description": "path 的兼容别名。"
                },
                "file_path": {
                    "type": "string",
                    "description": "path 的兼容别名。"
                },
                "edit_type": {
                    "type": "string",
                    "enum": ["insert", "edit", "delete"],
                    "description": "执行的 notebook 编辑操作。"
                },
                "editType": {
                    "type": "string",
                    "description": "edit_type 的兼容别名。"
                },
                "cell_id": {
                    "type": "string",
                    "description": "目标 cell 的 id。edit/delete 必须提供 cell_id 或 cell_number。"
                },
                "cellId": {
                    "type": "string",
                    "description": "cell_id 的兼容别名。"
                },
                "cell_number": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "目标 cell 编号，从 1 开始。edit/delete 必须提供 cell_id 或 cell_number。"
                },
                "position": {
                    "type": "string",
                    "enum": ["before", "after", "start", "end"],
                    "description": "insert 操作的插入位置。默认：有 selector 时 after，否则 end。"
                },
                "cell": {
                    "type": "object",
                    "description": "insert/edit 操作使用的完整 cell JSON。缺失 id 时会生成或复用现有 id。"
                }
            },
            "required": ["path", "edit_type"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let args: Args = serde_json::from_value(args)
            .map_err(|error| anyhow!("Missing or invalid parameters: {error}"))?;

        match self.execute_internal(args).await {
            Ok(response) => {
                Ok(ToolResult { success: true, output: response.model_text, error: None })
            }
            Err(error) => Ok(Self::failure(error.to_string())),
        }
    }

    async fn call(&self, input: Value) -> anyhow::Result<ToolCallResult> {
        let args: Args = serde_json::from_value(input)
            .map_err(|error| anyhow!("Missing or invalid parameters: {error}"))?;

        match self.execute_internal(args).await {
            Ok(response) => Ok(response.into_tool_call_result()),
            Err(error) => Ok(ToolCallResult::from_legacy_result(Self::failure(error.to_string()))),
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
