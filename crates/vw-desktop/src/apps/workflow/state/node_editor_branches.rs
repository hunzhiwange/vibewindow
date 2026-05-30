//! # Workflow 分支节点编辑
//!
//! 该模块处理条件分支与知识检索节点的可视化草稿字段编辑更新。

use super::*;

impl WorkflowState {
    pub fn add_node_editor_if_else_case(&mut self) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::IfElse { cases } = draft {
                cases.push(default_if_else_case_draft());
                true
            } else {
                false
            }
        });
    }

    pub fn set_node_editor_if_else_case_logical_operator(&mut self, index: usize, value: String) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::IfElse { cases } = draft {
                if let Some(case) = cases.get_mut(index) {
                    case.logical_operator = value;
                    return true;
                }
            }
            false
        });
    }

    pub fn add_node_editor_if_else_condition(&mut self, case_index: usize) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::IfElse { cases } = draft {
                if let Some(case) = cases.get_mut(case_index) {
                    case.conditions.push(default_if_else_condition_draft());
                    return true;
                }
            }
            false
        });
    }

    pub fn remove_node_editor_if_else_condition(
        &mut self,
        case_index: usize,
        condition_index: usize,
    ) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::IfElse { cases } = draft {
                if let Some(case) = cases.get_mut(case_index) {
                    if condition_index < case.conditions.len() {
                        case.conditions.remove(condition_index);
                        return true;
                    }
                }
            }
            false
        });
    }

    pub fn set_node_editor_if_else_condition_selector(
        &mut self,
        case_index: usize,
        condition_index: usize,
        value: String,
    ) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::IfElse { cases } = draft {
                if let Some(condition) = cases
                    .get_mut(case_index)
                    .and_then(|case| case.conditions.get_mut(condition_index))
                {
                    condition.variable_selector_input = value;
                    return true;
                }
            }
            false
        });
    }

    pub fn set_node_editor_if_else_condition_operator(
        &mut self,
        case_index: usize,
        condition_index: usize,
        value: String,
    ) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::IfElse { cases } = draft {
                if let Some(condition) = cases
                    .get_mut(case_index)
                    .and_then(|case| case.conditions.get_mut(condition_index))
                {
                    condition.comparison_operator = value;
                    return true;
                }
            }
            false
        });
    }

    pub fn set_node_editor_if_else_condition_value(
        &mut self,
        case_index: usize,
        condition_index: usize,
        value: String,
    ) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::IfElse { cases } = draft {
                if let Some(condition) = cases
                    .get_mut(case_index)
                    .and_then(|case| case.conditions.get_mut(condition_index))
                {
                    condition.compare_value = value;
                    return true;
                }
            }
            false
        });
    }

    pub fn set_node_editor_if_else_condition_var_type(
        &mut self,
        case_index: usize,
        condition_index: usize,
        value: String,
    ) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::IfElse { cases } = draft {
                if let Some(condition) = cases
                    .get_mut(case_index)
                    .and_then(|case| case.conditions.get_mut(condition_index))
                {
                    condition.var_type = value;
                    return true;
                }
            }
            false
        });
    }

    pub fn set_node_editor_knowledge_query_selector(&mut self, value: String) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::KnowledgeRetrieval { query_selector_input, .. } = draft
            {
                *query_selector_input = value;
                true
            } else {
                false
            }
        });
    }

    pub fn set_node_editor_knowledge_query_attachment_selector(&mut self, value: String) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::KnowledgeRetrieval {
                query_attachment_selector_input,
                ..
            } = draft
            {
                *query_attachment_selector_input = value;
                true
            } else {
                false
            }
        });
    }

    pub fn set_node_editor_knowledge_dataset_ids(&mut self, value: String) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::KnowledgeRetrieval { dataset_ids_input, .. } = draft {
                *dataset_ids_input = value;
                true
            } else {
                false
            }
        });
    }

    pub fn set_node_editor_knowledge_retrieval_mode(&mut self, value: String) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::KnowledgeRetrieval { retrieval_mode, .. } = draft {
                *retrieval_mode = value;
                true
            } else {
                false
            }
        });
    }

    pub fn set_node_editor_knowledge_top_k(&mut self, value: String) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::KnowledgeRetrieval { top_k_input, .. } = draft {
                *top_k_input = value;
                true
            } else {
                false
            }
        });
    }

    pub fn set_node_editor_knowledge_score_threshold_enabled(&mut self, value: bool) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::KnowledgeRetrieval { score_threshold_enabled, .. } =
                draft
            {
                *score_threshold_enabled = value;
                true
            } else {
                false
            }
        });
    }

    pub fn set_node_editor_knowledge_score_threshold(&mut self, value: String) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::KnowledgeRetrieval { score_threshold_input, .. } = draft
            {
                *score_threshold_input = value;
                true
            } else {
                false
            }
        });
    }

    pub fn set_node_editor_knowledge_reranking_enabled(&mut self, value: bool) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::KnowledgeRetrieval { reranking_enable, .. } = draft {
                *reranking_enable = value;
                true
            } else {
                false
            }
        });
    }

    pub fn set_node_editor_knowledge_single_model_provider(&mut self, value: String) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::KnowledgeRetrieval { single_model_provider, .. } = draft
            {
                *single_model_provider = value;
                true
            } else {
                false
            }
        });
    }

    pub fn set_node_editor_knowledge_single_model_name(&mut self, value: String) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::KnowledgeRetrieval { single_model_name, .. } = draft {
                *single_model_name = value;
                true
            } else {
                false
            }
        });
    }

    pub fn set_node_editor_knowledge_single_model_mode(&mut self, value: String) {
        self.update_node_visual_draft(|draft| {
            if let WorkflowNodeVisualDraft::KnowledgeRetrieval { single_model_mode, .. } = draft {
                *single_model_mode = value;
                true
            } else {
                false
            }
        });
    }
}
