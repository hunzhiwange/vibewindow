use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct ComposioToolsResponse {
    #[serde(default)]
    pub items: Vec<ComposioV3Tool>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ComposioConnectedAccountsResponse {
    #[serde(default)]
    pub items: Vec<ComposioConnectedAccount>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ComposioConnectedAccount {
    pub id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub toolkit: Option<ComposioToolkitRef>,
}

impl ComposioConnectedAccount {
    pub fn is_usable(&self) -> bool {
        self.status.eq_ignore_ascii_case("INITIALIZING")
            || self.status.eq_ignore_ascii_case("ACTIVE")
            || self.status.eq_ignore_ascii_case("INITIATED")
    }

    pub fn toolkit_slug(&self) -> Option<&str> {
        self.toolkit.as_ref().and_then(|toolkit| toolkit.slug.as_deref())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ComposioV3Tool {
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "appName", default)]
    pub app_name: Option<String>,
    #[serde(default)]
    pub toolkit: Option<ComposioToolkitRef>,
    /// Full JSON Schema for the tool's input parameters (returned by v3 API).
    #[serde(default)]
    pub input_parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ComposioToolkitRef {
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ComposioAuthConfigsResponse {
    #[serde(default)]
    pub items: Vec<ComposioAuthConfig>,
}

#[derive(Debug, Clone)]
pub struct ComposioConnectionLink {
    pub redirect_url: String,
    pub connected_account_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ComposioAuthConfig {
    pub id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

impl ComposioAuthConfig {
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(false)
            || self.status.as_deref().is_some_and(|v| v.eq_ignore_ascii_case("enabled"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposioAction {
    pub name: String,
    #[serde(rename = "appName")]
    pub app_name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    /// Input parameter schema returned by the v3 API (absent from v2 responses).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_parameters: Option<serde_json::Value>,
}
#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
