# Conversation-Scoped LLM Context & MCP View

目标：将“LLM 上下文（指令 / 工具视图等）”从 Session 下放到 Conversation，Session 仅负责 MCP 服务器生命周期与共享基础设施；每个 Conversation 拥有独立、可裁剪的 MCP 工具视图，以及独立的基础 / 用户指令；仍保留每个 turn 的临时 compact/override 能力。

**目录**

- 背景与现状
- 设计目标与边界
- 新的职责划分
- 数据模型与关键结构
- Prompt 构建与优先级
- MCP 工具视图裁剪（per‑conversation）
- 协议与 API 变更
- 兼容性与迁移路径
- 实施计划（编码指引）
- 测试要点
- 风险与缓解

## 背景与现状

- Session：当前持有 TurnContext（含 base_instructions/user_instructions、tools_config 等）、MCP 连接管理器、事件通道与根会话等。`run_turn` 构建 Prompt 时使用 `turn_context.base_instructions`（或特例如 compact 任务的覆盖）。
- Conversation：持有历史与 pending 输入，以及一个 `McpView`（目前是 manager 的快照，默认全量，不启用 allowlist）。
- MCP：`McpConnectionManager` 在 Session 启动时聚合所有 MCP 服务器与工具；`McpView` 目前未启用过滤能力（但留有 `with_allowlist`）。

问题：

- 需要“按 Conversation 下放与隔离”LLM 上下文（尤其 base/user 指令与 MCP 工具可见性），避免不同对话间的指令或工具暴露互相影响。
- 仍需保留“当回合 temporary override（如 compact）”的能力，不改变默认会话 / 对话配置。
- 现状冲突点：`TurnContext` 内仍持有 base/user/tools_config，属于待瘦身项，避免与对话级上下文“双写”。
- 指令自动附加逻辑：`Prompt::get_full_instructions()` 已实现“当 override 为空时按需附加 apply_patch 说明”，应保留。

## 设计目标与边界

**目标**

- 将 LLM Prompt 相关上下文（base_instructions、user_instructions、可见 MCP 工具集合）下放到 Conversation。
- Session 仅负责：MCP 客户端生命周期、共享资源与日志 / 事件中枢；不再承载各对话的默认 Prompt 上下文。
- 支持 per‑conversation 的 MCP 工具 allowlist（裁剪视图）。
- 保留 per‑turn 的 compact/override 能力（仅当回合生效，不回写对话默认值）。
- 强化策略边界：对话层只能“收窄”，不得突破 Session 级审批 / 沙箱上限（取最严）。

**非目标**

- 不改变“一次只跑一个任务”的执行模型与事件协议。
- 不改 `history + pending` 管理方式（仍由 Conversation 持有）。

## 新的职责划分

- Session（进程范围，现状保留）

  - 启动与管理所有 MCP 服务器（`McpConnectionManager`）。
  - 作为事件中枢，维持任务注册表，转发事件，处理审批请求等。
  - 不再保存“会话默认 base/user 指令”；不再决定 MCP 工具可见性。

- Conversation（用户显式创建的对话实例）

  - 新增：LLM Prompt 上下文（`base_instructions`、`user_instructions`）。
  - 新增：MCP 工具视图（`McpView`）的 allowlist（可选）；默认全量。
  - 仍旧：历史与 pending（`history`、`pending_input`）。
  - 生成 Prompt 时，从对话级上下文与历史构造输入。

- Turn（每次回合）
  - Prompt 的 `base_instructions_override` 在此确定：
    - 普通回合：使用 `conversation.base_instructions`（若为 None 则回退到内置 `prompt.md`）。
    - 特例回合（compact）：临时 override（不回写 conversation 的默认值）。
  - 工具集：由 `conversation.mcp_view.list_tools()` + 对话级 tools 配置（见下）决定，再经 `get_openai_tools(...)` 组装。

## 数据模型与关键结构

建议引入 / 调整以下结构，并尽可能保持最小改动：

- ConversationContext（新增）

  - `base_instructions: Option<String>`
  - `user_instructions: Option<String>`
  - `mcp_view: McpView`（内部含 `allowlist: Option<HashSet<String>>`）
  - `tools_prefs: ToolsPrefs`（见下）

- ToolsPrefs（新增，对话级工具偏好 / 开关）

  - `include_plan_tool: bool`
  - `include_apply_patch_tool: bool`
  - `web_search_request: bool`
  - `shell_mode`: 枚举（默认 / 带审批提示 /local/streamable），由审批 / 沙箱策略派生或显式设置
  - 合规合成：与 Session 的 `approval_policy` / `sandbox_policy` 合并时取“最严格”结果（不得越权）。

- Conversation（现有，新增字段）

  - `ctx: ConversationContext`
  - 历史与 pending 不变

- TurnContext（瘦身，仅保留“与 provider/ 模型相关且与对话正交”的信息）
  - `client: ModelClient`
  - `cwd: PathBuf`
  - `approval_policy: AskForApproval`
  - `sandbox_policy: SandboxPolicy`
  - `shell_environment_policy: ShellEnvironmentPolicy`
  - `storage_policy: StoragePolicy`
  - 去除：`base_instructions`、`user_instructions`、`tools_config`

注：`tools_config` 的组合逻辑迁移至每个 turn 的“工具拼装”路径，通过 `conversation.ctx.tools_prefs` 与 `conversation.mcp_view.list_tools()` 构建。

## Prompt 构建与优先级

每次 turn：

1. 指令（系统顶部）

   - 选择 `Prompt.base_instructions_override`：
     - 若是 compact 任务：使用 compact 的 override 内容。
     - 否则：使用 `conversation.ctx.base_instructions`；若为 None，`Prompt::get_full_instructions()` 回退到内置 `prompt.md`。
   - `Prompt::get_full_instructions()` 的“自动附加 APPLY_PATCH_TOOL_INSTRUCTIONS”逻辑保留（仅当 override 为 None 且工具集中无 apply_patch 或模型需要特殊说明）。

2. 历史 + 当轮输入

   - `conversation.history.contents()`：包含在创建对话时注入的一条 `user_instructions`（若有）与环境上下文等。
   - 与当轮 pending 合并：`sess.turn_input_with_history_for(&conv, extra)`（保留现有形态）。

3. 工具集合
   - 来自 `conversation.ctx.tools_prefs` + `conversation.mcp_view.list_tools()`，通过 `get_openai_tools(...)` 组装为 OpenAI 工具列表；不再使用 `turn_context.tools_config`。
   - 组装时进行“最严格策略”合成（对话声明 ≤ 会话允许），并在越权时发出显式事件 / 日志告警。

## MCP 工具视图裁剪（per‑conversation）

- Session 启动所有 MCP，并在 `McpConnectionManager` 内聚合（与现状一致）。
- Conversation 创建时：
  - 构造 `McpView::new_from_manager(&manager)`。
  - 若传入 allowlist，则 `mcp_view = mcp_view.with_allowlist(names)`；否则默认全量。
  - `conversation.ctx.mcp_view = mcp_view`。

> 说明：`McpView` 仍是快照语义（当前实现即如此）。如需动态刷新可在后续扩展（不是本次目标）。

## 命名规范与视图一致性

- 规范：Fully‑Qualified Tool Name（FQN）统一采用下划线分隔的形式 `server__tool`（双下划线）。理由：OpenAI 工具名限制为 `^[a-zA-Z0-9_-]+$`，不允许斜杠 `/`。
- 兼容：对话传入的 allowlist 若使用 `server/tool` 形式，需在入口归一化为 `server__tool`，并记录一次 deprecation 日志。
- 冲突与排序：工具列表应以归一化后的 FQN 进行稳定字典序排序，并启用签名冲突检测 / 去重（已有实现应作为契约固化）。

## 协议与 API 变更

扩展 MCP `NewConversationParams`：

- `base_instructions?: string`（已存在，沿用）
- `user_instructions?: string`（新增）
- `mcp_tool_allowlist?: string[]`（新增；元素为 fully‑qualified 工具名，允许 `server/tool` 输入并归一化为 `server__tool`）
- `include_plan_tool?: boolean`（已存在）
- `include_apply_patch_tool?: boolean`（已存在）
- `tools_web_search_request?: boolean`（新增，若需要端到端控制）

可选：新增 `SetConversationContext` 请求以在对话存续期间更新上述字段（不影响历史记录顺序）。

服务器侧（mcp-server）：

- `CodexMessageProcessor::process_new_conversation` 从 `NewConversationParams` 读取以上参数，构造 `ConversationContext`，并在 Conversation 初始化时：
  - 将 `user_instructions` 作为一条最早的 `user` 消息写入历史（沿用现有 `Prompt::format_user_instructions_message`）。
  - 按 allowlist 构造 `McpView`。
  - 将 `base_instructions` 放入 `conversation.ctx.base_instructions`（不写历史）。
  - 组装 `tools_prefs`。

核心层（core）：

- `TurnContext` 瘦身（移除 base/user 指令与 tools_config）。
- `run_turn` 与 `try_run_turn` 构建 Prompt 时，改为从 `conversation.ctx` 读取指令与工具集。
- compact 任务路径（`run_compact_task`）保留 override 行为，仅对当轮生效。

## 测试要点

- 对话级指令隔离：创建两个对话，分别提供不同 `base/user instructions`，验证各自 Prompt 指令不同且互不影响。
- MCP 工具视图裁剪：对话 A 指定 allowlist（例如仅 `server__foo`），对话 B 默认全量；验证 `ListTools` 与实际工具调用可见性一致。
- compact 覆盖：同一对话，普通回合与 compact 回合的 `base_instructions_override` 不同，且 compact 不回写默认。
- 回退逻辑：未指定 `base_instructions` 时回退到内置 `prompt.md`；当工具缺少 `apply_patch` 且 override 为 None 时自动附加说明。
- 兼容行为：未提供新字段时，行为与旧版一致（历史中仅有环境上下文与用户输入，MCP 工具全量）。
- 无效 allowlist：传入未知或非法 FQN（含 `/` 等）时触发前置校验错误；`server/tool` 被归一化并提示弃用。
- 稳定性：工具清单排序与冲突处理的快照测试（同输入多次运行顺序一致）。
- 并发压力：多对话快速 open / message / close 的资源回收测试（TaskRegistry / MCP 客户端引用计数）。

## 风险与缓解

- 快照 vs 热更新：`McpView` 为快照语义。短期提供“刷新视图”开关（可通过 `SetConversationContext{ refresh_mcp_view:true }`）；长期考虑 `McpConnectionManager` 发布工具变更事件，按需重建视图。
- 工具名冲突与稳定排序：坚持以 FQN 稳定排序（字典序），并做签名冲突检测 / 去重；将此作为契约与测试断言固化。
- 权限收敛：对话层策略不得扩权（仅可收窄）。当出现越权声明时，以 Session 策略为准并记录事件 / 告警。

## 可观测性与性能

- Prompt 缓存键：建议组合“对话级基础指令哈希 + 工具名列表哈希 + 模型家族”形成稳定 cache key，减少重复序列化开销。
- 事件标注：持续双写 `conversation_id` / `task_id`，支撑按对话观测与追踪；在工具越权收敛时发出 `BackgroundEvent` 说明收敛原因。
