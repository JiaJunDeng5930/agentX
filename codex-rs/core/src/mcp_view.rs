use std::collections::HashMap;
use std::collections::HashSet;

use mcp_types::Tool;

use crate::mcp_connection_manager::McpConnectionManager;

/// A per-conversation view over the session-wide MCP connection manager.
///
/// By default, exposes all tools. Optionally, can restrict to an allowlist of
/// fully-qualified tool names (e.g., "server__tool").
#[derive(Clone, Debug)]
pub(crate) struct McpView {
    base_tools: HashMap<String, Tool>,
    allowlist: Option<HashSet<String>>,
}

impl McpView {
    pub fn new_from_manager(manager: &McpConnectionManager) -> Self {
        Self {
            base_tools: manager.list_all_tools(),
            allowlist: None,
        }
    }

    /// Restrict the view to a specific set of fully-qualified tool names.
    #[allow(dead_code)]
    pub fn with_allowlist(mut self, names: HashSet<String>) -> Self {
        self.allowlist = Some(names);
        self
    }

    /// Returns the tools visible in this view as a map of fully-qualified
    /// tool name to Tool definition.
    pub fn list_tools(&self) -> HashMap<String, Tool> {
        let all = self.base_tools.clone();
        match &self.allowlist {
            None => all,
            Some(allowed) => all
                .into_iter()
                .filter(|(k, _)| allowed.contains(k))
                .collect(),
        }
    }
}
