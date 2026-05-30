//! # Workflow 核心状态操作
//!
//! 该模块提供核心状态读写、选择管理、脏状态判断、应用切换和浮层开关等通用逻辑。

use super::*;

impl WorkflowState {
    pub fn title(&self) -> &str {
        if self.source_name.trim().is_empty() { "Dify工作流" } else { self.source_name.as_str() }
    }

    pub fn active_app(&self) -> Option<&WorkflowAppEntry> {
        let active_id = self.active_app_id.as_deref()?;
        self.apps.iter().find(|app| app.id == active_id)
    }

    pub fn has_apps(&self) -> bool {
        !self.apps.is_empty()
    }

    pub fn has_start_node(&self) -> bool {
        self.document.nodes.iter().any(|node| node.block_type == "start")
    }

    pub fn active_meta(&self) -> Option<&WorkflowAppMeta> {
        self.active_app().map(|app| &app.meta)
    }

    pub fn selected_node(&self) -> Option<&WorkflowNode> {
        self.selected_node_id.as_deref().and_then(|id| self.document.node(id))
    }

    pub fn selected_edge(&self) -> Option<&WorkflowEdge> {
        self.selected_edge_id.as_deref().and_then(|id| self.document.edge(id))
    }

    pub fn environment_variable(&self, id: &str) -> Option<&WorkflowEnvironmentVariable> {
        self.environment_variables.iter().find(|variable| variable.id == id)
    }

    pub fn conversation_variable(&self, id: &str) -> Option<&WorkflowConversationVariable> {
        self.conversation_variables.iter().find(|variable| variable.id == id)
    }

    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error_message = Some(error.into());
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn select_node(&mut self, id: String) {
        self.selected_node_id = Some(id);
        self.selected_edge_id = None;
        self.context_menu = None;
        self.sync_selection_flags();
    }

    pub fn select_edge(&mut self, id: String) {
        self.selected_edge_id = Some(id);
        self.selected_node_id = None;
        self.context_menu = None;
        self.sync_selection_flags();
    }

    pub fn clear_selection(&mut self) {
        self.selected_node_id = None;
        self.selected_edge_id = None;
        self.context_menu = None;
        self.sync_selection_flags();
    }

    pub fn current_history_snapshot(&self) -> Option<WorkflowHistorySnapshot> {
        self.active_app_id.as_ref()?;

        Some(WorkflowHistorySnapshot {
            meta: WorkflowAppMeta {
                name: self.source_name.clone(),
                ..self.active_meta().cloned().unwrap_or_default()
            },
            document: self.document.clone(),
            environment_variables: self.environment_variables.clone(),
            conversation_variables: self.conversation_variables.clone(),
            pan: self.pan,
            zoom: self.zoom,
            selected_node_id: self.selected_node_id.clone(),
            selected_edge_id: self.selected_edge_id.clone(),
        })
    }

    pub(super) fn push_history_snapshot(&mut self, snapshot: WorkflowHistorySnapshot) {
        if self.undo_stack.last() != Some(&snapshot) {
            self.undo_stack.push(snapshot);
            if self.undo_stack.len() > WORKFLOW_HISTORY_LIMIT {
                self.undo_stack.remove(0);
            }
        }
        self.redo_stack.clear();
    }

    pub(super) fn push_undo_snapshot(&mut self) {
        let Some(snapshot) = self.current_history_snapshot() else {
            return;
        };

        self.push_history_snapshot(snapshot);
    }

    pub(super) fn refresh_dirty_state(&mut self) {
        let Some(current) = self.current_history_snapshot() else {
            self.active_is_dirty = false;
            return;
        };

        self.active_is_dirty = self.saved_snapshot.as_ref() != Some(&current);
    }

    pub(super) fn restore_history_snapshot(&mut self, snapshot: WorkflowHistorySnapshot) {
        self.source_name = snapshot.meta.name.clone();
        self.document = snapshot.document;
        self.environment_variables = snapshot.environment_variables;
        self.conversation_variables = snapshot.conversation_variables;
        self.pan = snapshot.pan;
        self.zoom = snapshot.zoom;
        self.selected_node_id = snapshot.selected_node_id;
        self.selected_edge_id = snapshot.selected_edge_id;
        self.connection_draft = None;
        self.context_menu = None;
        self.quick_insert_panel_open = false;
        self.action_menu_open = false;
        self.zoom_menu_open = false;
        self.dragging_node_id = None;
        self.drag_pending_snapshot = None;
        self.app_editor = None;
        self.node_editor = None;
        self.variable_panel = None;
        self.variable_editor = None;

        if let Some(active_id) = self.active_app_id.clone()
            && let Some(app) = self.apps.iter_mut().find(|app| app.id == active_id)
        {
            app.meta = WorkflowAppMeta { name: self.source_name.clone(), ..snapshot.meta };
        }

        self.sync_selection_flags();
        self.refresh_dirty_state();
    }

    pub fn apply_loaded(&mut self, loaded: LoadedWorkflow, window_size: (f32, f32)) {
        self.persist_active_snapshot();
        let app_id = generate_app_id();
        let selected_node_id =
            loaded.document.nodes.iter().find(|node| node.selected).map(|node| node.id.clone());
        let selected_edge_id =
            loaded.document.edges.iter().find(|edge| edge.selected).map(|edge| edge.id.clone());

        let (pan, zoom) = if loaded.had_viewport {
            (
                Vector::new(loaded.document.viewport.x, loaded.document.viewport.y),
                loaded.document.viewport.zoom.clamp(0.1, 4.0),
            )
        } else {
            fitted_viewport(&loaded.document, window_size)
        };
        let saved_snapshot = WorkflowHistorySnapshot {
            meta: loaded.app_meta.clone(),
            document: loaded.document.clone(),
            environment_variables: loaded.environment_variables.clone(),
            conversation_variables: loaded.conversation_variables.clone(),
            pan,
            zoom,
            selected_node_id: selected_node_id.clone(),
            selected_edge_id: selected_edge_id.clone(),
        };

        self.apps.push(WorkflowAppEntry {
            id: app_id.clone(),
            meta: loaded.app_meta.clone(),
            source_path: loaded.source_path.clone(),
            raw_root: loaded.raw_root.clone(),
            document: loaded.document.clone(),
            environment_variables: loaded.environment_variables.clone(),
            conversation_variables: loaded.conversation_variables.clone(),
            pan,
            zoom,
            selected_node_id: selected_node_id.clone(),
            selected_edge_id: selected_edge_id.clone(),
            connection_draft: None,
            is_dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            saved_snapshot: saved_snapshot.clone(),
        });

        self.active_app_id = Some(app_id);
        self.active_is_dirty = false;

        self.source_name = loaded.source_name.clone();
        self.source_path = loaded.source_path.clone();
        self.document = loaded.document;
        self.environment_variables = loaded.environment_variables;
        self.conversation_variables = loaded.conversation_variables;
        self.selected_node_id = selected_node_id;
        self.selected_edge_id = selected_edge_id;
        self.connection_draft = None;
        self.context_menu = None;
        self.quick_insert_panel_open = false;
        self.action_menu_open = false;
        self.zoom_menu_open = false;
        self.variable_panel = None;
        self.variable_editor = None;
        self.error_message = None;

        self.pan = pan;
        self.zoom = zoom;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.saved_snapshot = Some(saved_snapshot);
        self.dragging_node_id = None;
        self.drag_pending_snapshot = None;

        self.status_message = Some(match self.source_path.as_deref() {
            Some(path) => format!("已加载 {}", path),
            None => format!("已加载内置示例: {}", self.title()),
        });
        self.sync_selection_flags();
    }

    pub fn replace_active_loaded(&mut self, loaded: LoadedWorkflow, window_size: (f32, f32)) {
        let Some(active_id) = self.active_app_id.clone() else {
            self.apply_loaded(loaded, window_size);
            return;
        };

        let selected_node_id =
            loaded.document.nodes.iter().find(|node| node.selected).map(|node| node.id.clone());
        let selected_edge_id =
            loaded.document.edges.iter().find(|edge| edge.selected).map(|edge| edge.id.clone());
        let (pan, zoom) = if loaded.had_viewport {
            (
                Vector::new(loaded.document.viewport.x, loaded.document.viewport.y),
                loaded.document.viewport.zoom.clamp(0.1, 4.0),
            )
        } else {
            fitted_viewport(&loaded.document, window_size)
        };
        let saved_snapshot = WorkflowHistorySnapshot {
            meta: loaded.app_meta.clone(),
            document: loaded.document.clone(),
            environment_variables: loaded.environment_variables.clone(),
            conversation_variables: loaded.conversation_variables.clone(),
            pan,
            zoom,
            selected_node_id: selected_node_id.clone(),
            selected_edge_id: selected_edge_id.clone(),
        };

        if let Some(app) = self.apps.iter_mut().find(|app| app.id == active_id) {
            app.meta = loaded.app_meta.clone();
            app.source_path = loaded.source_path.clone();
            app.raw_root = loaded.raw_root.clone();
            app.document = loaded.document.clone();
            app.environment_variables = loaded.environment_variables.clone();
            app.conversation_variables = loaded.conversation_variables.clone();
            app.pan = pan;
            app.zoom = zoom;
            app.selected_node_id = selected_node_id.clone();
            app.selected_edge_id = selected_edge_id.clone();
            app.connection_draft = None;
            app.is_dirty = false;
            app.undo_stack.clear();
            app.redo_stack.clear();
            app.saved_snapshot = saved_snapshot.clone();
        }

        self.active_is_dirty = false;
        self.source_name = loaded.source_name.clone();
        self.source_path = loaded.source_path.clone();
        self.document = loaded.document;
        self.environment_variables = loaded.environment_variables;
        self.conversation_variables = loaded.conversation_variables;
        self.pan = pan;
        self.zoom = zoom;
        self.selected_node_id = selected_node_id;
        self.selected_edge_id = selected_edge_id;
        self.connection_draft = None;
        self.app_editor = None;
        self.node_editor = None;
        self.app_editor = None;
        self.node_editor = None;
        self.context_menu = None;
        self.quick_insert_panel_open = false;
        self.action_menu_open = false;
        self.zoom_menu_open = false;
        self.variable_panel = None;
        self.variable_editor = None;
        self.error_message = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.saved_snapshot = Some(saved_snapshot);
        self.dragging_node_id = None;
        self.drag_pending_snapshot = None;
        self.status_message = Some(format!("已重新载入 {}", self.title()));
        self.sync_selection_flags();
    }

    pub fn select_app(&mut self, id: &str) -> bool {
        if self.active_app_id.as_deref() == Some(id) {
            return false;
        }

        self.persist_active_snapshot();
        let Some(app) = self.apps.iter().find(|app| app.id == id).cloned() else {
            return false;
        };

        self.active_app_id = Some(app.id.clone());
        self.load_entry_into_current(&app);
        self.status_message = Some(format!("已切换到 {}", self.title()));
        true
    }

    pub fn open_create_editor(&mut self) {
        self.node_editor = None;
        self.variable_editor = None;
        self.variable_panel = None;
        self.context_menu = None;
        self.quick_insert_panel_open = false;
        self.action_menu_open = false;
        self.zoom_menu_open = false;
        self.app_editor = Some(WorkflowAppEditorDraft {
            mode: WorkflowAppEditorMode::Create,
            name: self.next_app_name(),
            description: String::new(),
            icon: "🤖".to_string(),
            use_icon_as_answer_icon: false,
            max_active_requests_input: "0".to_string(),
        });
    }

    pub fn open_edit_editor(&mut self, id: Option<&str>) {
        let target_id = id.map(|value| value.to_string()).or_else(|| self.active_app_id.clone());
        let Some(target_id) = target_id else {
            return;
        };

        if let Some(app) = self.apps.iter().find(|app| app.id == target_id) {
            self.node_editor = None;
            self.variable_editor = None;
            self.variable_panel = None;
            self.context_menu = None;
            self.quick_insert_panel_open = false;
            self.action_menu_open = false;
            self.zoom_menu_open = false;
            self.app_editor = Some(WorkflowAppEditorDraft {
                mode: WorkflowAppEditorMode::Edit(target_id),
                name: app.meta.name.clone(),
                description: app.meta.description.clone(),
                icon: app.meta.icon.clone(),
                use_icon_as_answer_icon: app.meta.use_icon_as_answer_icon,
                max_active_requests_input: app.meta.max_active_requests.to_string(),
            });
        }
    }

    pub fn close_editor(&mut self) {
        self.app_editor = None;
    }

    pub fn toggle_action_menu(&mut self) {
        self.context_menu = None;
        self.quick_insert_panel_open = false;
        self.zoom_menu_open = false;
        self.action_menu_open = !self.action_menu_open;
    }

    pub fn toggle_zoom_menu(&mut self) {
        self.context_menu = None;
        self.quick_insert_panel_open = false;
        self.action_menu_open = false;
        self.zoom_menu_open = !self.zoom_menu_open;
    }

    pub fn close_floating_panels(&mut self) {
        self.action_menu_open = false;
        self.zoom_menu_open = false;
        self.quick_insert_panel_open = false;
    }

    pub fn toggle_quick_insert_panel(&mut self) {
        self.context_menu = None;
        self.action_menu_open = false;
        self.zoom_menu_open = false;
        self.quick_insert_panel_open = !self.quick_insert_panel_open;
    }

    pub fn close_quick_insert_panel(&mut self) {
        self.quick_insert_panel_open = false;
    }

    pub fn update_active_source_path(&mut self, path: String) {
        self.source_path = Some(path.clone());

        if let Some(active_id) = self.active_app_id.clone()
            && let Some(app) = self.apps.iter_mut().find(|app| app.id == active_id)
        {
            app.source_path = Some(path);
        }

        if let Some(snapshot) = self.current_history_snapshot() {
            self.saved_snapshot = Some(snapshot.clone());
            self.active_is_dirty = false;

            if let Some(active_id) = self.active_app_id.clone()
                && let Some(app) = self.apps.iter_mut().find(|app| app.id == active_id)
            {
                app.saved_snapshot = snapshot;
                app.is_dirty = false;
            }
        }
    }

    pub(super) fn sync_selection_flags(&mut self) {
        let selected_node_id = self.selected_node_id.as_deref();
        for node in &mut self.document.nodes {
            node.selected = Some(node.id.as_str()) == selected_node_id;
        }

        let selected_edge_id = self.selected_edge_id.as_deref();
        for edge in &mut self.document.edges {
            edge.selected = Some(edge.id.as_str()) == selected_edge_id;
        }
    }

    fn load_entry_into_current(&mut self, app: &WorkflowAppEntry) {
        self.source_name = app.meta.name.clone();
        self.source_path = app.source_path.clone();
        self.document = app.document.clone();
        self.environment_variables = app.environment_variables.clone();
        self.conversation_variables = app.conversation_variables.clone();
        self.pan = app.pan;
        self.zoom = app.zoom;
        self.selected_node_id = app.selected_node_id.clone();
        self.selected_edge_id = app.selected_edge_id.clone();
        self.connection_draft = app.connection_draft.clone();
        self.undo_stack = app.undo_stack.clone();
        self.redo_stack = app.redo_stack.clone();
        self.saved_snapshot = Some(app.saved_snapshot.clone());
        self.active_is_dirty = app.is_dirty;
        self.app_editor = None;
        self.node_editor = None;
        self.variable_panel = None;
        self.variable_editor = None;
        self.context_menu = None;
        self.quick_insert_panel_open = false;
        self.action_menu_open = false;
        self.zoom_menu_open = false;
        self.dragging_node_id = None;
        self.drag_pending_snapshot = None;
        self.error_message = None;
        self.sync_selection_flags();
    }

    fn next_app_name(&self) -> String {
        let base = "未命名应用";
        if !self.apps.iter().any(|app| app.meta.name == base) {
            return base.to_string();
        }

        let mut index = 2;
        loop {
            let candidate = format!("{} {}", base, index);
            if !self.apps.iter().any(|app| app.meta.name == candidate) {
                return candidate;
            }
            index += 1;
        }
    }
}
