//! Workflow 变量池。

use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub(crate) struct VariablePool {
    values: BTreeMap<String, Value>,
}

impl VariablePool {
    pub(crate) fn insert_selector(&mut self, selector: &[String], value: Value) {
        if selector.is_empty() {
            return;
        }
        self.values.insert(selector_key(selector), value);
    }

    pub(crate) fn insert_node_output(&mut self, node_id: &str, key: &str, value: Value) {
        self.values.insert(format!("{node_id}.{key}"), value);
    }

    pub(crate) fn get_selector(&self, selector: &[String]) -> Option<&Value> {
        if selector.is_empty() {
            return None;
        }
        self.values.get(&selector_key(selector))
    }

    pub(crate) fn node_outputs(&self, node_id: &str) -> BTreeMap<String, Value> {
        let prefix = format!("{node_id}.");
        self.values
            .iter()
            .filter_map(|(key, value)| {
                key.strip_prefix(&prefix).map(|name| (name.to_string(), value.clone()))
            })
            .collect()
    }
}

pub(crate) fn selector_from_value(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| items.iter().filter_map(|item| item.as_str().map(ToOwned::to_owned)).collect())
        .unwrap_or_default()
}

pub(crate) fn selector_key(selector: &[String]) -> String {
    selector.join(".")
}
