# 在 TUI 显示当前运行 Task 的 Conversation ID（conv_id）

本文档描述在 Codex Rust TUI 中显示“当前运行的 task 对应的 conversation id（conv_id）”的方案。该设计遵循最小变更原则，不改变现有业务语义，不引入并发模型变化。

## 目标与非目标

- 目标
  - 在 TUI 界面实时显示“当前运行中的 Task 所属 Conversation 的 ID”。
  - 当没有 Task 运行时不显示 / 置灰该信息（可配置）。
  - 默认以短格式（8 位或 12 位）呈现，保持状态行紧凑。
- 非目标
  - 不实现会话（Conversation）切换 UI；仍然沿用当前“/new 创建新会话”的交互。
  - 不改动 Core 中的单任务调度模型（同一 Session 同时仅 1 个运行 Task）。
  - 不变更协议与事件语义。

## 背景与现状

- 事件协议 `Event` 带有 `conversation_id: Option<Uuid>`；`TaskStarted/TaskComplete`、`Exec*`、`Patch*` 等事件都会携带。
- 当前架构中 Task 与 Conversation 同级，Task 通过 `conv_id` 绑定 Conversation；TUI 仅使用根会话（`root_conversation`）。
- TUI `ChatWidget` 接收事件时没有保存 `conversation_id`，也未在 UI 展示。

## 核心思路

- 在 `ChatWidget` 维护一个 `active_conv_id: Option<Uuid>`：
  - 在收到 `TaskStarted` 事件时，从该事件的 `conversation_id` 取值并设置。
  - 在收到 `TaskComplete` 或 `TurnAborted` 时清空。
  - 若出现 `Exec*`/`Patch*` 事件提前到达（极少数次序问题），也可以按需覆盖为 `event.conversation_id`，但以 `TaskStarted/TaskComplete` 为主。
- 在底部状态行（Status Indicator）新增一个轻量只读区域显示 `conv_id`：
  - 格式：`conv <short-id>`。
  - 样式：`dim()`（与现有状态 / 提示风格一致，低打扰）。
  - 宽度不足时可进一步截断（例如 8 位）并在超窄终端下自动隐藏（不换行，不影响主要信息）。

## UI 展示位置与格式

- 位置：底部状态行（复用现有 `status_indicator_widget`/`status_indicator_view` 渲染线路）。
- 默认格式：短 UUID（建议 8 位）
  - 例如：`conv 8f7c4ac2`
- 空闲（无 Task）时：
  - 默认不显示；可配置开启“空闲也显示最后一次 conv_id”。

## 配置项（建议）

在 `config.md` 的 `[tui]` 段新增：

- `show_conv_id`（bool，默认：true）
  - true：有 Task 运行时显示 conv_id。
- `conv_id_idle_mode`（string，默认："hide"）
  - 可选：`hide` | `dim`（空闲显示上次 conv_id、置灰）
- `conv_id_format`（string，默认："short8"）
  - 可选：`short8` | `short12` | `full`

（实现时若需要最小化改动，可仅实现 `show_conv_id=true` + `short8`，其余留作扩展。）

## 代码改动点（不立即实现，仅设计）

1. `tui/src/chatwidget.rs`
   - 新增字段：`active_conv_id: Option<Uuid>`。
   - 在 `handle_codex_event` 中保留 `Event` 的 `conversation_id`：
     - `TaskStarted`：`self.active_conv_id = event.conversation_id;` 并 `bottom_pane.set_active_conv_id(self.active_conv_id)`。
     - `TaskComplete`/`TurnAborted`：`self.active_conv_id = None;` 并同步更新底部状态。
     - 可选：对 `ExecCommandBegin/PatchApplyBegin` 若 `active_conv_id.is_none()` 时补充设置（容错）。

2. `tui/src/bottom_pane/mod.rs` 与 / 或 `status_indicator_view.rs`
   - 在 `BottomPane` 增加只读状态：`active_conv_id_short: Option<String>`，提供 `set_active_conv_id(Option<Uuid>)`，内部按配置做格式化并请求重绘。
   - 在 `status_indicator_view.rs` 渲染时，如果配置启用且有值，按顺序加入一段 `"conv "+short_id` 的 `Line`，使用 `.dim()`。

3. UUID 短格式化工具（可内联实现）
   - 简单实现：`fn short_uuid(id: &Uuid, n: usize) -> String { id.as_simple().to_string()[0..n].to_string() }`
   - 注意边界检查与 `n` 限制（8 或 12）。

4. 配置读取（可选最小实现）
   - 在 `codex_core::config` 或 `tui` 层 `Config` 中扩展 `[tui]` 配置字段；默认值如上。
   - 若追求最小变更，第一版可直接内置 `show_conv_id = true`、`short8`，后续再增配置。

## 行为细节

- 有 Task：显示当前 Task 所在 `conv_id`（来自最近的 `TaskStarted`）。
- 无 Task：按 `conv_id_idle_mode` 显示 / 隐藏；默认隐藏。
- 新建会话（`/new`）：
  - 旧 `ChatWidget` 被替换，新 Widget 初始 `active_conv_id=None`，直到收到新的 `TaskStarted`。

## 兼容性与风险

- 仅新增 UI 信息，不改变事件流与核心逻辑；风险极低。
- 超窄终端可能空间紧张，故设置自动隐藏优先保证现有关键信息（token、状态）。

## 测试方案

- 单元 / 快照测试（`codex-rs/tui` 使用 `insta`）：
  - 为 `TaskStarted`→消息流→`TaskComplete` 的典型路径更新快照，断言出现 `conv <short-id>` 文本（或在无 Task 时不出现）。
  - 宽度边界下的渲染（可以通过现有测试辅助）验证截断 / 隐藏逻辑。
- 手动验证：
  - 启动 `codex tui`，发起一次交互，观察底部状态行出现 `conv <id>`；任务结束后按配置隐藏或置灰。

## 发布与回滚

- 增量发布：先最小功能（始终 `short8`）、默认开启；若反馈占用过多空间，再引入配置开关。
- 回滚：UI 仅增不改，移除渲染分支即可；不影响核心逻辑。

## 后续可选项

- 提供 `/status` 或快捷键显示完整 `conv_id` 的弹窗详情。
- 提供“复制 conv_id 到剪贴板”的快捷键（跨平台实现需评估）。
