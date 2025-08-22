//! Experimental scaffolding for the multi‑conversation Session design described in
//! `codex-rs/design.md`. This module is intentionally unused by the current
//! runtime to preserve backwards compatibility while enabling incremental
//! adoption in follow‑up PRs.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::sync::atomic::AtomicU8;
use std::time::{Duration, Instant};

use tokio::task::AbortHandle;
use uuid::Uuid;

use crate::models::{ResponseInputItem, ResponseItem};
use crate::openai_tools::ToolsConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SessionId(pub Uuid);
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConversationId(pub Uuid);
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(pub Uuid);

#[derive(Debug)]
pub enum AgentError {
    Cancelled,
    Timeout,
    ApprovalDenied,
    SandboxError,
    McpError,
    ToolConflict,
    StoragePolicyViolation,
    Internal(String),
}

// -------- Session state (RwLock) ---------

pub struct SessionState {
    pub approvals: HashSet<CommandKey>,
    pub current_running: Option<TaskId>,
    pub conversations: HashMap<ConversationId, Arc<Conversation>>, // weak refs are held from tasks
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            approvals: HashSet::new(),
            current_running: None,
            conversations: HashMap::new(),
        }
    }
}

// -------- Conversation and task handle (no strong cycles) ---------

pub struct TaskHandle {
    pub task_id: TaskId,
    pub started_at: Instant,
    pub abort: AbortHandle,
    pub status: AtomicU8, // 0=Running,1=Done,2=Cancelled
}

pub struct ConversationState {
    pub history: HistoryStore,
    pub pending_input: VecDeque<ResponseInputItem>,
    pub current_task: Option<TaskHandle>,
}

impl Default for ConversationState {
    fn default() -> Self {
        Self {
            history: HistoryStore::new(),
            pending_input: VecDeque::new(),
            current_task: None,
        }
    }
}

pub struct Conversation {
    pub id: ConversationId,
    pub state: Mutex<ConversationState>,
    pub tools_config: ToolsConfig,
    pub mcp_view: McpView,
    pub model_client: ModelClient,
    pub storage_policy: StoragePolicy,
}

// Placeholder types for compatibility. These are intentionally minimal and will
// be replaced/connected to the existing runtime incrementally.
#[derive(Clone)]
pub struct ModelClient; // stub

impl ModelClient {
    #[allow(dead_code)]
    pub fn clone(&self) -> Self { Self }
}

pub struct McpView; // stub

impl McpView {
    #[allow(dead_code)]
    pub fn list_all_tools(&self) -> Vec<ToolSpec> { Vec::new() }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolSpec {
    pub name: String,
}

// -------- Task registry (break strong cycles) ---------

pub struct AgentTask {
    pub task_id: TaskId,
    pub conv: Weak<Conversation>,
    pub sub_id: String,
    pub handle: AbortHandle,
}

#[derive(Default)]
pub struct TaskRegistry {
    inner: RwLock<HashMap<TaskId, AgentTask>>,
}

impl TaskRegistry {
    #[allow(dead_code)]
    pub fn insert(&self, id: TaskId, task: AgentTask) {
        self.inner.write().unwrap().insert(id, task);
    }
    #[allow(dead_code)]
    pub fn remove(&self, id: TaskId) {
        self.inner.write().unwrap().remove(&id);
    }
}

// -------- Approval scope + TTL ---------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandKey {
    pub path: Vec<String>,
    pub conversation: Option<ConversationId>,
    pub expires_at: Option<Instant>,
}

// -------- Storage policy ---------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoragePolicy {
    Full,
    HeadersOnly,
    None,
    Ttl(Duration),
}

// -------- History store (ring + optional summarization placeholder) ---------

pub struct HistoryStore {
    segments: VecDeque<Arc<HistorySegment>>,
    approx_tokens: usize,
    max_tokens: usize,
    max_items: usize,
}

impl HistoryStore {
    pub fn new() -> Self {
        Self {
            segments: VecDeque::new(),
            approx_tokens: 0,
            max_tokens: 0,
            max_items: usize::MAX,
        }
    }

    #[allow(dead_code)]
    pub fn build_view(&self) -> PromptInputView {
        PromptInputView {}
    }

    #[allow(dead_code)]
    pub fn append(&mut self, items: &[ResponseItem]) {
        if items.is_empty() { return; }
        let seg = Arc::new(HistorySegment { items: items.to_vec() });
        self.segments.push_back(seg);
        // accounting intentionally omitted; this is a scaffold
    }
}

pub struct HistorySegment {
    items: Vec<ResponseItem>,
}

pub struct PromptInputView; // zero‑copy iterator placeholder

// -------- Tool merging (deterministic, conflict detection) ---------

#[allow(dead_code)]
pub fn list_all_tools(conv: &Conversation) -> Vec<ToolSpec> {
    let mut out: Vec<ToolSpec> = Vec::new();

    // Built‑ins: fixed order. Currently determined by ToolsConfig; placeholder here.
    // In a later integration, this should mirror core::openai_tools ordering.
    let builtin = get_builtin_tools(&conv.tools_config);
    out.extend(builtin);

    // MCP tools: collect and sort by name for stability
    let mut mcp = conv.mcp_view.list_all_tools();
    mcp.sort_by(|a, b| a.name.cmp(&b.name));

    // De‑dup + conflict detection
    let mut seen: HashMap<String, ToolSpec> = HashMap::new();
    for t in out.iter().cloned().chain(mcp.into_iter()) {
        if let Some(existing) = seen.get(&t.name) {
            if existing != &t {
                // conflict: same name, different signature – log in real integration
                // e.g., tracing::error!("tool conflict: {}", t.name);
            }
            continue;
        }
        seen.insert(t.name.clone(), t.clone());
    }

    // emit in deterministic order: built‑ins first (preserve their order), then MCP (sorted)
    let mut merged: Vec<ToolSpec> = Vec::new();
    // built‑ins (without duplicates)
    for t in get_builtin_tools(&conv.tools_config) {
        if let Some(orig) = seen.remove(&t.name) { merged.push(orig); }
    }
    // remaining MCP entries in name order
    let mut rest: Vec<ToolSpec> = seen.into_values().collect();
    rest.sort_by(|a, b| a.name.cmp(&b.name));
    merged.extend(rest);

    merged
}

fn get_builtin_tools(_cfg: &ToolsConfig) -> Vec<ToolSpec> {
    // Minimal placeholder to make the scaffolding compile; real ordering is implemented
    // for OpenAI tools in core::openai_tools.
    Vec::new()
}
