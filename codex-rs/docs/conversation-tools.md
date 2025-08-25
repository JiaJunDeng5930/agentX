# Conversation Tools（内置工具）系统设计

本文定义一组“内置工具”（`task` 运行时模型可调用，与 `apply_patch`/`shell` 并列）用于在单一 `session` 中管理与协作多个 `conversation`，并满足“打断并串行切换”的语义。

- 创建新 `conversation`（可定制 `base_instruction` / `user_instruction` / `mcp_view` / `internal tools`）
- 向既有 `conversation` 发送消息
- 查看当前存在的 `conversation`
- 查看某 `conversation` 的历史消息（仅 message）
- 销毁某 `conversation`
  关键语义：系统不存在“子任务 / 暂停”。所有 `task` 原子且串行（同一时刻最多运行一个）。当“打断型工具”触发时：
- 当前运行的 `task` 立即结束（事件：`TurnAborted(Replaced)`）；
- 然后依次启动：目标 `conversation` 的 `task`，该 `task` 完成后，再启动承接原逻辑的 `task`，把“工具返回值”注入模型继续该 `turn`。

---

## 术语（统一使用代码标识）

- `session`：一次 CLI/TUI 运行期内的整体容器（已有）
- `conversation`：`session` 内的子对话上下文（独立历史 / 工具视图 / 指令）
- `task`：由用户输入或系统调度触发的一次流程，包含若干 `turn`（已有，原子、不暂停）
- `turn`：一次输入构造 + 模型输出的基本单位

---

## 工具清单

内置 5 个工具，默认始终随模型工具集暴露（与 `shell`、`apply_patch` 同级），在 `core/src/openai_tools.rs#get_openai_tools()` 中以固定顺序加入。

1. `conv.create`（打断型）

- 参数：
  - `base_instruction_text?: string`
  - `base_instruction_file?: string`（相对路径相对于 `turn_context.cwd`，与上互斥；均未提供则继承当前 `conversation` 的 base）
  - `user_instruction?: string`（与 `items` 至少一项，作为首轮用户输入之一）
  - `items?: InputItem[]`（与 `user_instruction` 至少一项，支持多模态首轮输入）
  - `mcp_allowlist?: string[]`（未提供则继承当前 `conversation` 的 `mcp_view`）
  - `internal_tools?: { include_plan_tool?: bool, include_apply_patch_tool?: bool, web_search_request?: bool, use_streamable_shell_tool?: bool }`（未提供则继承当前 `conversation` 的 `tools_prefs`）
- 行为：
  - 触发时：当前 `task` → `TurnAborted(Replaced)`
  - 随后依次入队运行：
    - T_new：在新 `conversation` 上启动 `task`，输入为 `user_instruction`；运行至完成
    - T_cont：在原 `conversation` 上启动承接 `task`，其输入为本工具的 `FunctionCallOutput`（详见“承接策略”）
- 返回：
  - 工具返回值（注入到 T_cont 的 `FunctionCallOutput.content`）为 T_new 的“最后一条助手消息”字符串
  - JSON 负载（字符串形式）：
    {
      "conversation_id": "<uuid>",
      "first_user_message": "<string, optional>",
      "first_items_count": <number, optional>,
      "last_assistant_message": "<string>"
    }

2. `conv.send`（打断型）

- 参数：
  - `conversation_id: string`
  - `text?: string`
  - `items?: InputItem[]`（二者至少一项）
- 行为：
  - 触发时：当前 `task` → `TurnAborted(Replaced)`
  - 随后依次入队运行：
    - T_target：在目标 `conversation` 上启动 `task`，输入为 `text/items`；运行至完成
    - T_cont：在原 `conversation` 上启动承接 `task`，其输入为本工具的 `FunctionCallOutput`
- 返回：
  - 工具返回值（注入到 T_cont 的 `FunctionCallOutput.content`）为 T_target 的“最后一条助手消息”字符串
  - JSON 负载：
    {
    "conversation_id": "<uuid>",
    "last_assistant_message": "<string>"
    }

3) `conv.list`（非打断型）

- 参数：无
- 行为：读取 `session` 内全部 `conversation`
- 返回：
  - JSON 负载：{ "conversations": [ { "id": string, "message_count": number, "last_active_at": "<RFC3339 string>" } ] }

4) `conv.history`（非打断型）

- 参数：
  - `conversation_id: string`
  - `limit?: number`
- 行为：返回该 `conversation` 历史中的 `ResponseItem::Message`（不含 tool 调用与输出）
- 返回：
  - JSON 负载（含轻量多模态）：
    { "entries": [ { "role": "user" | "assistant", "text": "<string, optional>", "attachments": [ { "type": "image", "image_url": "<data:... or http(s)://...>" } ] } ] }

5. `conv.destroy`（非打断型）

- 参数：
  - `conversation_id: string`
- 行为：销毁指定 `conversation`；若正处于运行，则该 `conversation` 收到 `TurnAborted(Interrupted)`；不影响其他 `conversation`
- 返回：
  - { "ok": true } 或 { "ok": false, "reason": "<string>" }
  - 禁止销毁 root `conversation`

---

## 执行语义（打断与串行调度）

串行调度规则：任意时刻最多只有一个 `task` 在运行；新 `task` 启动前，前一个必须已结束。
当 `conv.create` / `conv.send` 被调用：

1. 当前运行的 `task` → `TurnAborted(Replaced)`
2. 调度器立即启动 T_new/T_target（目标 `conversation`），运行至完成（期间可产生 `TaskStarted`、`AgentMessageDelta`、`TokenCount`、`TaskComplete` 等事件）
3. 紧接启动 T_cont（原 `conversation`），向模型注入本工具的 `FunctionCallOutput`
   当 `conv.list` / `conv.history` / `conv.destroy` 被调用：

- 不打断当前 `task`；直接返回 `FunctionCallOutput`

---

## 承接策略（工具返回值如何交回模型）

- 对于 `conv.create`：T_cont 注入 `ResponseInputItem::FunctionCallOutput { call_id, output }`，其中 `output.content` 为 T_new 的“最后一条助手消息”
- 对于 `conv.send`：T_cont 注入 `ResponseInputItem::FunctionCallOutput { call_id, output }`，其中 `output.content` 为 T_target 的“最后一条助手消息”
- 实现提示：在 `handle_function_call()` 处理中捕获 `call_id`/ 工具名，并将其随 `TaskPlan` 传递，以便 T_cont 能闭合对应的 `FunctionCallOutput`

---

## 数据与结构（核心）

### 1) `ConversationContext` 扩展

- 继续承载：
  - `base_instructions: Option<String>`
  - `user_instructions: Option<String>`
  - `mcp_view: McpView`（开放 `with_allowlist`）
  - `tools_prefs: ToolsPrefs`（可按需扩展）

### 2) `conversation` 寻址

- 仅通过 `conversation_id: Uuid` 寻址；不引入 `index` 模式

### 3) `task` 调度器（串行执行）

- 增加 `pending_tasks: VecDeque<TaskPlan>`，确保“一个完成→启动下一个”
- 消费点：在 `submission_loop` 中消费 `pending_tasks`，当 `current_task` 完成时按 FIFO 启动下一个
- 提供：
  - `abort_current_and_enqueue(plans: Vec<TaskPlan>)`
  - `enqueue(plans: Vec<TaskPlan>)`
- `TaskPlan` 字段：
  - `conversation_id: Uuid`
  - `input_items: Vec<InputItem>`
  - `base_instructions_override: Option<String>`
  - `closure: Option<{ call_id: String, tool_name: String }>`（承接工具调用闭合信息）
  - 公平性与上限：可配置 `max_interrupt_chain_depth`，限制连续“打断型工具”的嵌套深度，避免长期饥饿

### 4) `conversation` 构造与计划生成

- `open_conversation_with(spec)`：从当前 `conversation` 继承并覆盖上下文；记录 `user_instruction` 为首条 message
- `plan_for_conv_create(spec, call_ctx)` → `[T_new, T_cont]`
- `plan_for_conv_send(ref, input, call_ctx)` → `[T_target, T_cont]`
 
### 5) `conversation` 元数据
- 新增并维护 `last_active_at: Instant`：
  - 在写入历史（record_items）与 `TaskComplete` 时刷新
  - `conv.list` 对外返回时序列化为 RFC3339 字符串

---

## 实现挂点

### openai_tools.rs

- 在 `get_openai_tools()` 中新增 5 个工具定义（`JsonSchema::Object`）
- 描述字段明确：`conv.create`/`conv.send` 为打断型、其余为非打断型

### codex.rs（handle_function_call）

- 新增分支：
  - `conv.create`：
    - 解析 `base_instruction_text` / `base_instruction_file`
    - `open_conversation_with(spec)` → `conversation_id`
    - `abort_current_and_enqueue([T_new(user_instruction), T_cont(return=last_assistant_message_of_T_new)])`
    - 返回 JSON（包含 `conversation_id`/`first_user_message`/`last_assistant_message`），`FunctionCallOutput` 在 T_cont 注入
  - `conv.send`：
    - 解析 `conversation_id`
    - `abort_current_and_enqueue([T_target(text/items), T_cont(return=last_assistant_message_of_T_target)])`
    - 返回 JSON（包含 `conversation_id`/`last_assistant_message`），`FunctionCallOutput` 在 T_cont 注入
  - `conv.list` / `conv.history` / `conv.destroy`：直接读取 / 修改内存并 `FunctionCallOutput` 返回

---

## 事件策略
 
- 打断型工具事件顺序保证：
  - 原 `conversation`：`TurnAborted(Replaced)`
  - 目标 `conversation`（T_new/T_target）：`TaskStarted` → `AgentMessageDelta`（可选）→ `TokenCount`（可选）→ `TaskComplete`
  - 原 `conversation`（T_cont）：`TaskStarted` → `AgentMessageDelta`（可选）→ `TaskComplete`
- 前端以 `Event.conversation_id` 路由 UI；在替换发生时提示“当前 `task` 被 `conv.*` 替换，正在执行目标 `conversation`…”
- 非打断工具：仅返回 `FunctionCallOutput`

---

## 错误处理

- `conversation` 不存在 / 编号无效：{ ok: false, reason: "conversation not found" } 或 `success: false`
- 读取 `base_instruction_file` 失败：`success: false`
- `conv.send` 参数校验：`text` 与 `items` 至少一项
- 禁止销毁 root `conversation`

---

## TUI/CLI

- 列表按 `conversation_id` 展示与渲染事件
- 打断型工具触发时：可提示“当前 `task` 被 `conv.*` 工具替换，正在运行目标 `conversation`，随后承接原逻辑”

---

## 返回值与模型交互

- 所有工具的 `FunctionCallOutputPayload.content` 使用 JSON 字符串：
  - `conv.create`：主字段 `last_assistant_message`（返回值）；附带 `conversation_id`、`first_user_message`
  - `conv.send`：主字段 `last_assistant_message`；附带 `conversation_id`
  - `conv.list`：`conversations`
  - `conv.history`：`entries`
  - `conv.destroy`：`{ ok }`、`reason?`

---

## 测试要点

- create：验证 `TurnAborted(Replaced)` → T_new（`TaskStarted`→`TaskComplete`）→ T_cont 注入 `last_assistant_message`；`conversation_id` 合法
- send：同上，T_cont 注入 T_target 的 `last_assistant_message`
- list/history/destroy：不中断；destroy 失败用例（root/ 不存在）
- 历史过滤仅保留 message 类型

---

## 版本与发布

- 破坏性修改（新增内置工具与行为），版本号建议 MAJOR+1（例如 `3.0.0`）
