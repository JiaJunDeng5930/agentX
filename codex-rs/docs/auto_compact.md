# 自动 Compact（内联总结与剪枝）设计说明

本文档定义“自动 compact”的核心方案：在允许破坏性修改的前提下，只要会话的“context left”低于阈值，核心逻辑立即触发一次真正的“总结 + 剪枝”，随后继续当前任务，无需用户输入“continue”。

## 目标与范围

- 目标：
  - 只要“context left”低于阈值，就自动触发一次真正的 `/compact` 等效操作（总结 + 剪枝）。
  - 自动流程对所有前端统一生效（TUI、CLI 等）。
  - 无需用户手动“continue”，兼容保留手动 `/compact` 命令。
- 作用域：
  - 粒度：单个会话（conversation）。
  - context left：基于当前会话对模型构造 Prompt 时，所占用的上下文窗口（context window）与模型最大上下文容量的比例，反映“剩余可用比例”。
  - /compact：将当前会话历史“总结为一条消息”并裁剪，释放上下文空间（保留一条总结消息）。

## 概念与术语

- 会话（conversation）：核心 `Session` 内的独立对话上下文，历史记录、工具调用等都按会话隔离管理。
- turn：与模型的一次请求-响应（streaming）过程。一个任务（task）可能跨多个 turn。
- 真实 compact：使用 `prompt_for_compact_command.md` 的“总结指令”发起一轮模型请求，生成总结消息，并将历史裁剪为“只保留一条总结消息”。
- 内联 compact：compact 作为当前任务的内部步骤执行，不创建新 task，也不需要“continue”。

## 配置

新增 core 级别的配置项（适用于所有前端）：

```toml
[auto_compact]
# 是否启用自动 compact（默认 false）
enabled = false
# 触发阈值，单位百分比（默认 15）
threshold_percent = 15

# 可选：是否允许中断进行中的 turn 以抢占式触发（默认 false，见“中断式触发（可选）”）
# interrupt_mid_turn = false
```

说明：
- 该配置添加在 core 的 Config 下（`ConfigToml`/`Config`），所有客户端共享。
- TUI 的手动 `/compact` 命令保留，不受该配置影响。

## 总体方案

在核心 turn 执行管线（`run_task()`）中加入“自动 compact 守卫”，按如下时机检测并触发：

1) 预检（pre-turn check）：
- 在每次构造 Prompt 并调用模型前，调用 `ensure_auto_compact_if_needed()`：
  - 估算“本次 turn 的 context left%”。
  - 若低于阈值，则先执行一次“内联 compact”（见后文），紧接着重新构造 Prompt 并发起本次 turn。

2) 事后（post-turn check）：
- 在收到本 turn 的 `response.completed` 与 `TokenUsage` 后，再计算一次 context left%。
- 若仍低于阈值，则立即执行一次“内联 compact”（为下一轮 turn 腾挪空间）。

注意：采用“内联 compact”意味着该流程在同一个任务内部进行，不会产生独立的 Task，也不需要用户输入“continue”。

## 内联 compact 实现细节

触发时，执行如下步骤：

1) 构造“总结指令”的 Prompt：
- 使用现有 `prompt_for_compact_command.md` 作为 `base_instructions_override`。
- 输入为“当前会话历史 + 一个空/占位输入项”即可，不需要注入 `Start Summarization` 文本（避免把 compact 视作普通用户消息）。

2) 调用模型执行一轮总结：
- 通过与 `run_turn()` 相同的 streaming 管道执行（可沿用 `drain_to_completed()` 逻辑），产生一条 assistant 总结消息。

3) 记录并裁剪历史：
- 将该条总结消息追加到会话历史。
- 立即执行 `history.keep_last_messages(1)`，将历史裁剪为“仅保留最后一条 assistant（总结）”。

4) 事件与用户感知：
- 推荐直接保留 assistant 文本流式输出（用户可看到“自动 compact 正在进行”与“总结输出”）。
- 或者，追加一条 `BackgroundEvent` 作为轻提示（可选）。

5) 不发送 TaskComplete：
- 整个 compact 过程视为当前任务的内部步骤，不创建新 Task，也不需要“continue”。

## context left 计算

- post-turn（精准）：
  - 使用上报的 `TokenUsage`（`tokens_in_context_window()` 考虑了 reasoning token 的回收），结合 `model_context_window` 得到已用/剩余比例。
  - 基线（baseline_used_tokens）：用于扣除固定前置内容（系统指令、工具说明等）影响，优先取首轮的 `cached_input_tokens`；否则取 0。
  - 采用既有 `percent_of_context_window_remaining(context_window, baseline)` 计算，确保 UI 与 core 一致。

- pre-turn（估算）：
  - 在发起请求前，需要基于“当前历史 + 指令文案”估算 token 占用。
  - 简化策略：字符数/4 近似 token；或引入对应模型 tokenizer 做更精确估算（允许破坏性引入依赖）。
  - 同样扣除 baseline 后计算剩余百分比，与 post-turn 一致。

## 中断式触发（可选）

如果希望“在单个 turn 的 streaming 过程中，一旦估算低于阈值就抢占触发”，可以增加开关：

```toml
[auto_compact]
interrupt_mid_turn = true
```

启用后：
- 在 streaming 过程中，按增量输出（或工具调用）累计进行粗略 token 估算；
- 一旦估算的 context left% 低于阈值，立即发送 `Op::Interrupt` 中断当前 turn；
- 然后执行“内联 compact”，再继续后续对话。

权衡：此模式能更“激进”释放上下文，但输出会更碎，用户可能看到被中断的 turn。默认关闭。

## 与现有行为的差异（破坏性变更）

- 自动 compact 不再创建独立 Task；compact 流程在当前任务内部内联执行。
- 自动 compact 完成后，不再需要发送“continue”。
- 手动 `/compact` 命令保留原行为：
  - 若当前无 Task 运行：创建 summarization 任务，应用总结指令并剪枝。
  - 若当前有 Task 运行：原有实现会注入“Start Summarization”作为普通用户输入（不更换总结指令，不剪枝）。
  - 注：手动行为与自动行为的差异是有意保留，方便用户显式控制。

## 失败与重试

- 内联 compact turn 采用与正常 turn 一致的重试/退避策略（按 provider 配置的 stream/request retry 策略）。
- 重试用尽后，会上报 `ErrorEvent`；但不终止主任务循环，用户可继续对话或手动重试。

## 并发与工具调用

- 若在工具/函数调用链过程中触发 post-turn 检测：先执行内联 compact，再继续后续 turn，避免下轮被上下文挤爆。
- 预检在“发送前”执行，能保证“下一次请求”至少在阈值之上。

## 兼容性与迁移

- 默认关闭（`enabled=false`），不改变现有用户体验。
- 开启后，自动 compact 的流式总结内容会写入历史，且随后立即裁剪到 1 条消息（总结）。
- 对于依赖旧式“compact 后需要 continue”的脚本/工具，此设计改变了默认行为（破坏性），但从交互体验角度更符合“自动化”诉求。

## 代码改动点（指引）

1) 配置层（core）：
   - `core/src/config_types.rs`：为 `ConfigToml` 增加 `auto_compact` 字段块（`enabled: Option<bool>`, `threshold_percent: Option<u8>`, `interrupt_mid_turn: Option<bool>`）。
   - `core/src/config.rs`：在 `Config` 中承载上述字段并设定默认值（`enabled=false`、`threshold_percent=15`、`interrupt_mid_turn=false`）。

2) 执行管线（core）：
   - `core/src/codex.rs`：
     - 在 `run_task()` 的 turn 循环中，加入 pre-turn 与 post-turn 检查：`ensure_auto_compact_if_needed(sess, conv, turn_context)`。
     - 新增内联 compact 入口函数（例如 `inline_compact(sess, turn_context)`）以“总结指令 + drain_to_completed”获取一条总结，再 `history.keep_last_messages(1)`。
     - 在 streaming 边上（如工具调用、输出增量聚合处）预留可选的 `interrupt_mid_turn` 启发式中断。

3) 指令文案：
   - 复用 `core/src/prompt_for_compact_command.md`（保持与手动 `/compact` 一致）。

4) 事件：
   - 保持 `AgentMessageDelta/AgentMessage` 流式输出用于用户可感；必要时可追加一条 `BackgroundEvent` 标明“自动 compact 已触发（阈值 X%）”。

5) 手动命令保留：
   - `tui/src/slash_command.rs` 与 `tui/src/chatwidget.rs` 保留 `/compact` 原路径，不移除、不重写语义。

## 测试策略

- 单元测试（core）：
  - 构造多 turn 对话，模拟 `TokenUsage` 演进；当 pre/post 检测到低于阈值，验证触发了内联 compact，历史被裁剪为 1 条总结消息。
  - 验证 compact 后能继续运行下一 turn，不需要“continue”。
  - 模拟工具调用链场景，验证 post-turn 触发路径。

- 集成测试：
  - 类似 `core/tests/compact.rs` 的思路，检查请求体在 compact turn 使用了“总结指令”，且下一次请求的历史只包含总结 + 新输入。
  - 开关关闭时行为不变；阈值可配置。

- TUI/UI：
  - 不新增 UI 元素；如保留流式总结输出，应评估对 snapshot 的影响，必要时更新 snapshot。

## 性能与安全

- 紧急触发 compact 会引入一次额外请求；但只在临界情况下触发，阈值可配置，整体收益（释放上下文）大于开销。
- 若引入 tokenizer，注意依赖体积与平台兼容性；默认可先用近似估算法。

## FAQ

- Q：为什么不把手动 `/compact` 也改成“内联”？  
  A：手动命令保留原行为，避免破坏用户已有工作流；自动 compact 走“更自动”的新路径即可。

- Q：是否支持跨会话共享阈值/统计？  
  A：否。语义与操作都在“单个会话”内进行。

- Q：mid-turn 真的能精确判断“耗尽”吗？  
  A：streaming 过程中只能估算；因此默认不抢占中断，只有在开启 `interrupt_mid_turn` 时采用启发式中断。

---

实施本设计后，默认保持关闭；开启后对所有前端统一生效，达到“只要低于阈值就触发自动 compact”的效果，同时保留手动 `/compact` 的原有能力。

