## 1. 设计目标与边界

**目标**

- 在单一 Session 内提供多个**相互隔离**的子会话（Conversation）：独立的历史、工具 /MCP 视图、模型客户端与存储策略。
- 复用会话级能力：事件总线、审批 / 沙箱 / 执行、回放记录、通知。
- 维持默认行为不变：root-conversation 等价于现有实现。
- 以最小改动修复：强引用环、事件可观测性缺口、锁与异步安全、工具顺序不稳定、历史拷贝膨胀、审批作用域与 TTL、MCP 资源策略、禁存语义不清等问题。

**非目标（后续演进）**

- 多 Conversation 并行调度与事件多路复用（当前保持**全局单飞**策略）。
- UI/ 协议层面的重大改版（事件新增字段**向后兼容**提供）。

---

## 2. 总体架构（瘦 Session + 多 Conversation + 会话级 TaskRegistry）

```
Session
├─ EventBus
├─ Exec/Approval（审批 / 沙箱 / 执行）
├─ RolloutRecorder（回放）
├─ Notifier
├─ McpConnectionPool（会话级连接池）
├─ TaskRegistry（会话级，防环，统一取消 / 观测）
├─ State (RwLock)
│  ├─ approvals: HashSet<CommandKey>（带 TTL/ 作用域）
│  ├─ current_running: Option<TaskId>（单飞）
│  └─ conversations: HashMap<ConversationId, Arc<Conversation>>
└─ Compatibility: sub_id（保留）

Conversation (N)
├─ State (Mutex)
│  ├─ history: HistoryStore（有上限 + 可选摘要）
│  ├─ pending_input: VecDeque<ResponseInputItem>
│  └─ current_task: Option<TaskHandle>（轻量句柄，无环）
├─ tools_config: ToolsConfig
├─ mcp_view: McpView（隔离视图，复用会话池）
├─ model_client: ModelClient
└─ storage_policy: StoragePolicy
```

---

## 3. 数据模型与所有权（Rust 伪代码）

### 3.1 强类型 ID 与错误边界

```rust
struct SessionId(Uuid);
struct ConversationId(Uuid);
struct TaskId(Uuid);

#[derive(Debug)]
enum AgentError {
    Cancelled, Timeout, ApprovalDenied, SandboxError, McpError, ToolConflict,
    StoragePolicyViolation, Internal(String),
}
```

### 3.2 SessionState（读多写少，RwLock）

```rust
struct SessionState {
    approvals: HashSet<CommandKey>,              // 带 TTL/ 作用域
    current_running: Option<TaskId>,             // 全局单飞
    conversations: HashMap<ConversationId, Arc<Conversation>>,
}
```

### 3.3 Conversation 与无环任务句柄

```rust
struct TaskHandle {
    task_id: TaskId,
    started_at: Instant,
    abort: AbortHandle,
    status: AtomicU8, // 0=Running,1=Done,2=Cancelled
}

struct ConversationState {
    history: HistoryStore,                       // 有上限，支持摘要
    pending_input: VecDeque<ResponseInputItem>,  // drain 消费
    current_task: Option<TaskHandle>,            // 不持有 Conversation 的 Arc
}

struct Conversation {
    id: ConversationId,
    state: Mutex<ConversationState>,
    tools_config: ToolsConfig,
    mcp_view: McpView,                           // 会话池上的隔离视图
    model_client: ModelClient,
    storage_policy: StoragePolicy,               // 替代 bool
}
```

### 3.4 会话级 TaskRegistry（打破强引用环）

```rust
struct AgentTask {
    task_id: TaskId,
    conv: Weak<Conversation>,                    // Weak 打环
    sub_id: String,                              // 兼容旧消费者
    handle: AbortHandle,
}

struct TaskRegistry {
    inner: RwLock<HashMap<TaskId, AgentTask>>,
}
```

### 3.5 审批作用域与 TTL（结构化键）

```rust
struct CommandKey {
    path: Vec<String>,
    conversation: Option<ConversationId>,        // 默认 None 表示会话级共享
    expires_at: Option<Instant>,
}
```

### 3.6 存储策略（替代 disable_response_storage: bool）

```rust
enum StoragePolicy {
    Full,
    HeadersOnly,
    None,
    Ttl(std::time::Duration),
}
```

---

## 4. 历史与 Prompt 视图（避免全量拷贝）

### 4.1 HistoryStore（环形 + 可选摘要）

- **上限**：按“条目数 + 近似 tokens/ 字节”双阈值触发裁剪。
- **裁剪策略**：优先移除最早区段；若达到阈值再写入**摘要消息**（由模型 / 本地摘要器生成）。
- **记录**：只记录“新产生 items”，避免重复写入。

```rust
struct HistoryStore {
    segments: VecDeque<Arc<HistorySegment>>, // 分段存储，复用 Arc 降复制
    approx_tokens: usize,
    max_tokens: usize,
    max_items: usize,
}
```

### 4.2 PromptView 构建（零拷贝优先）

- `build_prompt_view(extra_pending: &[ResponseItem]) -> PromptInputView`
- 返回**迭代器 / 视图**，按顺序遍历历史分段与本轮 pending，不做整体 concat。
- **小历史优化**：当历史极短时，允许一次性 `SmallVec` 合并，减少分支代价。

---

## 5. 工具系统：确定性、去重与冲突策略

### 5.1 合并顺序（稳定）

- 顺序规则：**内置工具**（固定序）→ **MCP 工具**（名称字典序）。
- 冲突检测：同名工具且签名不一致 → **错误并告警**；签名一致 → **去重**保留一份。
- 输出稳定性：`list_all_tools()` 必须返回**确定性顺序**，确保 prompt 稳定。

```rust
fn list_all_tools(conv: &Conversation) -> Vec<ToolSpec> {
    let mut tools = Vec::new();
    tools.extend(get_builtin_tools(&conv.tools_config));    // 固定序
    let mut mcp = conv.mcp_view.list_all_tools();           // 可能动态
    mcp.sort_by(|a,b| a.name.cmp(&b.name));
    dedup_and_validate(&mut tools, &mcp)?;                  // 冲突检测
    tools
}
```

---

## 6. 并发与锁策略（无 await 持锁）

- `Session.state: RwLock`，查找对话走读锁，创建 / 删除走写锁。
- `Conversation.state: Mutex`，**只在短临界区内读 / 写**；持锁期间**不执行 await**。
- 模式：**先快照后异步**。从 `Conversation.state` 复制必要数据到栈上，释放锁，再进入异步调用。
- 全局单飞：以 `current_running: Option<TaskId>` 保证“同一时刻仅一个运行中的 Task”。占用 / 释放使用**原子检查 + 写锁**确保一致性。

---

## 7. MCP 连接与资源策略（池化 + 视图隔离）

- 会话级 `McpConnectionPool` 维护到各远端的**长连接 / 会话**（按 server 配置键控）。
- 每个 Conversation 持有 `McpView`：

  - 定义**可见工具集合**与**授权范围**（按对话隔离）。
  - 复用底层连接，减少 FD/ 握手成本。

- 资源护栏：每 Conversation 限制**并发调用数**与**速率**；会话级限额作为硬上限。
- 关闭时，若无其他对话使用同一连接，池可按空闲策略回收。

---

## 8. 审批与安全（作用域 + TTL）

- 默认维持**会话级共享白名单**（KISS）；但 `CommandKey` 支持可选的 `conversation` 与 `expires_at`，使之能被按对话 / 时间限定。
- 审批通过时写入结构化键；回收协程周期性清理过期项。
- 执行路径与现有 `Session::run_exec_with_events()` 保持不变。

---

## 9. 事件与可观测性（显式 conversation_id，向后兼容）

- **新增字段**（向后兼容，立即生效）：

  - `conversation_id: Option<ConversationId>`
  - `task_id: Option<TaskId>`
  - `trace_id/span_id: Option<Uuid>`（可选）

- 仍保留 `sub_id`，并继续填充，保证旧消费者可用。
- 事件生产方**双写**：新字段 + 原 `sub_id`。
- 指标按 Conversation 维度统计：tokens、工具次数、MCP 时延、取消率、失败率。

---

## 10. 生命周期与关闭语义（幂等、可取消、可超时）

- `conversation.open`：创建对话，初始化 `HistoryStore`、`McpView`、`StoragePolicy`、`model_client`；返回 `conversation_id`。
- `conversation.message`：串行执行一个 Task，返回最终回复与统计信息（tokens、工具调用、耗时、trace）。
- `conversation.close`：

  - 若有运行中任务：发出取消，等待**可配置超时**；超时后强制中止并记录事件。
  - 释放 `McpView` 引用；触发连接池引用计数回收。
  - 幂等：重复 close 安全无害。

---

## 11. 关键路径与方法（伪代码）

```rust
async fn conversation_message(sess: &Session, cid: ConversationId, input: Vec<InputItem>) -> Result<FinalOutput, AgentError> {
    // 1) 获取对话与快照（无 await 持锁）
    let conv = sess.get_conversation(cid)?;                    // RwLock 读锁
    let (history_view, tools, model_client, storage) = {
        let st = conv.state.lock().unwrap();
        (
            st.history.build_view(),                           // 视图，不拷贝
            list_all_tools(&conv),                             // 稳定顺序
            conv.model_client.clone(),
            conv.storage_policy.clone(),
        )
    };

    // 2) 全局单飞 - 占位
    let task_id = TaskId(Uuid::new_v4());
    sess.scheduler_acquire(task_id).await?;                    // 内部基于写锁 /CAS

    // 3) 任务注册（防环）
    let (sub_id, abort) = (new_sub_id(cid, task_id), AbortHandle::new_pair().0);
    sess.task_registry.insert(task_id, AgentTask { task_id, conv: Arc::downgrade(&conv), sub_id: sub_id.clone(), handle: abort.clone() });

    // 4) 记录输入（遵循 StoragePolicy）
    sess.rollout.record_input(cid, &input, storage)?;

    // 5) 回合循环
    let mut ctx = TurnContext::from(conv, model_client, tools, storage, /*覆盖项*/);
    let final_output = run_task(&conv, &sess, &mut ctx, &sub_id, input, history_view).await;

    // 6) 清理与释放
    sess.task_registry.remove(task_id);
    sess.scheduler_release(task_id);
    final_output
}

async fn run_task(conv: &Conversation, sess: &Session, ctx: &mut TurnContext, sub_id: &str, input: Vec<InputItem>, history_view: PromptInputView) -> Result<FinalOutput, AgentError> {
    // 任务开始事件（带 conversation_id/task_id）
    sess.events.task_started(conv.id, ctx.task_id, sub_id);

    // 消费 pending_input（drain）
    let pending: Vec<_> = {
        let mut st = conv.state.lock().unwrap();
        st.pending_input.drain(..).map(ResponseItem::from).collect()
    };

    // 构建本轮 prompt 视图（历史视图 + pending + input）
    let prompt = Prompt::from_view(history_view.chain(pending.iter()).chain(input.iter()), ctx.tools(), ctx.storage());

    // turn 执行（工具调用仍委托 Session 执行层）
    let out = try_run_turn(conv, sess, ctx, sub_id, &prompt).await?;

    // 记录输出到历史（可能触发裁剪 / 摘要）
    {
        let mut st = conv.state.lock().unwrap();
        st.history.append(out.items());
    }
    // 返回最终结果或继续下一轮（按 out.responses）
    // ...
    Ok(out.finalize())
}
```

> 以上伪代码只展示“先快照、后异步；不在持锁区 await；任务注册在 Session；StoragePolicy 与 Rollout 同步”的关键要点。

---

## 12. 回放与存储策略（与 StoragePolicy 一致）

- `Full`：记录全部消息 / 工具 / 补丁；可回放全文。
- `HeadersOnly`：记录结构化元数据（时间、工具名、计数、摘要指针）；不含正文。
- `None`：仅记录最小事件（开始 / 结束 / 错误），无内容。
- `Ttl(d)`：`Full` 记录，超时后由异步清理任务降级为 `HeadersOnly` 或删除正文（实现可配置）。

RolloutRecorder 按策略执行，**不会出现禁存却仍能看到正文**的矛盾。

---

## 13. 兼容性与迁移

- 事件**双写**：新增 `conversation_id`/`task_id` 字段，同时保留 `sub_id`。
- 默认仅创建 root-conversation，行为与旧版等价。
- 版本提升：**MINOR**。
- 迁移步骤（低风险）：

  1. 引入 `StoragePolicy`、`CommandKey`、`TaskRegistry` 与 `McpConnectionPool`；
  2. 将 `conversations` 与 `ConversationState` 下沉；`current_task` 改为 `TaskHandle`；
  3. `AgentTask.conv` 替换为 `Weak<Conversation>`，注册到 `TaskRegistry`；
  4. `list_all_tools()` 实施稳定排序与冲突检测；
  5. `HistoryStore` 替换原历史结构，引入上限与摘要；
  6. 事件结构体新增字段（双写），消费者平滑过渡；
  7. 回归测试与快照确认。

---

## 14. 测试清单（覆盖功能与非功能）

- **内存与泄漏**：创建 / 关闭对话 + 任务取消的压力测试，确认无 `Arc` 环泄漏。
- **确定性**：同输入多次运行工具清单顺序一致、输出一致（统计波动除外）。
- **并发安全**：快速连续 `open/message/close`，验证全局单飞与状态释放的原子性。
- **历史护栏**：长对话不 OOM；到阈值触发裁剪 / 摘要；PromptView 构建不做全量 concat。
- **审批 TTL/ 作用域**：到期失效、按对话限制生效；会话级共享仍可配置。
- **回放一致性**：四种 `StoragePolicy` 下能看到预期数据。
- **MCP 限流 / 池化**：压测连接与调用并发，验证池复用与视图隔离。
- **关闭幂等与超时**：运行中 close、重复 close、MCP 释放失败的重试 / 告警。
- **异常注入**：工具冲突、审批拒绝、沙箱错误、超时取消、事件通道背压。

---

## 15. 问题修复对照表（“问题 → 设计修复”）

- **强引用环** → `AgentTask.conv: Weak<Conversation>` + `Conversation.current_task: TaskHandle` + 会话级 `TaskRegistry`。
- **事件关联缺口** → 事件新增 `conversation_id`/`task_id`（双写保兼容）。
- **锁与异步安全** → “先快照后异步”，不在持锁区 `await`；`Session.state` 用 `RwLock`。
- **工具顺序不稳定 / 冲突** → 固定合并顺序 + 名称排序 + 冲突检测与去重。
- **历史拷贝膨胀** → `HistoryStore` 分段 + 上限 + 摘要；`PromptView` 视图化拼装。
- **审批作用域与 TTL** → `CommandKey` 带 `conversation` 与 `expires_at`，定期清理。
- **MCP 资源** → 会话级连接池 + 每对话 `McpView` 隔离 + 并发 / 速率限额。
- **禁存语义不清** → `StoragePolicy` 枚举与 Rollout 一致生效，避免歧义。
- **关闭语义不明** → `conversation.close` 幂等 + 取消并等待 + 超时强制中止。
- **全局单飞的原子性** → `current_running: Option<TaskId>` 原子占用 / 释放，调度简单可验证。
- **可观测性不足** → 指标 / 事件按 Conversation 维度统计；保留 `sub_id` 兼容旧消费方。
