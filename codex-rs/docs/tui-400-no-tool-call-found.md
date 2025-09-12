# TUI 报错 400：No tool call found for function call output（问题分析与定位）
+
本文记录在 TUI 使用过程中出现的错误：
+
🖐 unexpected status 400 Bad Request: {"error":{"message":"No tool call found for function call output with call_id …","type":"invalid_request_error","param":"input","code":null}}
+
核心含义：我们在向 Responses API 发起下一轮请求时，`input` 中包含了某个 `function_call_output`（携带 `call_id`），但这一轮请求的 `input` 里找不到与之匹配的工具调用（`function_call` / `local_shell_call` / `custom_tool_call`）项，因此服务端返回 400。
+
---
+
## 症状
- TUI 界面出现一条红色错误气泡（来源于 `new_error_event`）：
  - 文件：`codex-rs/tui/src/history_cell.rs`
  - 位置：`new_error_event()`（约 L840 附近）会以“🖐 …”渲染错误文本。
- 错误文本由核心层抛出的 `CodexErr::UnexpectedStatus(StatusCode, body)` 直接传至 UI。
  - 入口：`codex-rs/core/src/client.rs` 的 `stream_responses()` 在非 2xx 且非 429/401/5xx 时直接返回 `UnexpectedStatus(status, body)`（约 L220–L420 一带）。
+
---
+
## 根因（高概率）
Responses API 要求：本轮请求的 `input` 中，任一 `function_call_output` 的 `call_id` 必须能在“同一轮请求的 `input`”中找到对应的工具调用项（例如 `function_call`，或 `local_shell_call`，或 `custom_tool_call`）。
当历史裁剪或会话分叉后，可能出现“输出还在，但触发它的调用项不在”的不一致，导致 400。
+
---
+
## 触发路径与时序（常见）
1) 自动 compact 历史导致配对丢失
   - 代码：
     - `codex-rs/core/src/codex.rs`
       - `ensure_auto_compact_pre_turn()`（约 L3388 起）
       - `ensure_auto_compact_post_turn()`（约 L3433 起）
       - `inline_compact()` → 最终 `keep_last_messages(1)`（约 L3500），仅保留最后一条 assistant 消息。
     - `codex-rs/core/src/conversation_history.rs`
       - `keep_last_messages(n)`（约 L94 起）、`record_items()`（约 L33 起）
   - 现象：在工具调用刚刚完成或尚未在下一轮完全闭合时发生自动 compact，可能把较早的 `function_call`（或 `local_shell_call`/`custom_tool_call`）裁掉，但其对应的 `function_call_output` 仍在，导致新的请求里出现“有输出、无调用”的失配。
+
2) 会话回溯 / 分叉（Backtrack/Fork）后的初始历史不自洽
   - 代码：
     - `codex-rs/core/src/conversation_manager.rs`
       - `fork_conversation()`（约 L104 起）
       - `truncate_after_dropping_last_messages()`（约 L210 起）：仅以“用户消息”为单位裁剪，可能留下“输出条目但没有对应调用条目”的尾巴。
   - 现象：用这段历史作为新会话首次请求的 `input`，服务端校验发现 `function_call_output` 没有对应的调用项 → 400。
+
3) 早期的重复转发问题（已修复，留档）
   - 代码：
     - `codex-rs/core/src/client.rs` 的 SSE 解析 `process_sse()` 中明确只转发单条 `response.output_item.done`，避免再从 `response.completed` 里的 `output` 数组重放一次（约 L480–L680，一段长注释解释了重复导致的 call 链错乱问题）。
   - 说明：当前代码已避免“重复 output 导致的 previous_response_not_found/No tool call found”类问题，本次问题是另一类（历史不一致）。
+
---
+
## 请求输入构造与历史写入（数据面）
- 每一轮 turn 的输入构造：
  - 代码：`codex-rs/core/src/codex.rs`
    - `run_task()` → 组装 `turn_input`（约 L1700–L1900 / L1900–L2420 段）
    - `try_run_turn()`（约 L2040–L2410 / L1950–L2160 段）：最终调用 `turn_context.client.stream(&prompt)`。
    - `turn_input_with_history_for()`（约 L1120 起）：`history.contents()` + `pending_input` 拼接成这轮的 `input`。
  - 重要：`try_run_turn()` 内部有一段“缺失调用的合成处理”（`missing_calls`），当检测到“存在调用但没有对应输出”时，注入 `CustomToolCallOutput { output: "aborted" }`，以闭合“未响应”的调用。但这只解决“调用缺输出”的情况，不解决“输出缺调用”的情况（本 bug 的根因正是后者）。
- 工具调用与输出的写入：
  - 代码：`codex-rs/core/src/codex.rs`
    - `handle_response_item()`（约 L2200 起、前后两处同名实现）：遇到 `FunctionCall`/`LocalShellCall`/`CustomToolCall` 时，执行工具并把 `FunctionCallOutput`（或 `CustomToolCallOutput`）写入 `items_to_record_in_conversation_history`，随后 `record_conversation_items_for()` 持久化到历史（约 L2000–L2100 流程）。
  - 历史层：
    - `codex-rs/core/src/conversation_history.rs` 的 `record_items()` 控制保留 / 合并策略，`keep_last_messages()` 在 compact 时进行极限裁剪。
- 序列化层（避免另一类 400）：
  - `codex-rs/protocol/src/models.rs`
    - `FunctionCallOutputPayload` 的 `Serialize` 实现强制序列化为纯字符串（约 L190 起），与上游 JS CLI 行为一致，避免发送 `{content,success}` 形态导致 400。
+
---
+
## 错误抛出与 UI 展示
- 抛出位置：
  - `codex-rs/core/src/client.rs::stream_responses()`：非 429/401/5xx 的 4xx/5xx 会直接读取响应 body 并返回 `CodexErr::UnexpectedStatus(status, body)`（约 L220–L420）。
- UI 展示：
  - `codex-rs/tui/src/chatwidget.rs` → `handle_codex_event()` → `on_error(message)`；
  - `codex-rs/tui/src/history_cell.rs` → `new_error_event(message)`（约 L840），以“🖐 …”显示。
+
---
+
## 可能的修复方向（建议）
为避免“输出缺调用”导致的 400，在发请求前做一次“成对性校验 / 清洗”：
1) 在构造最终 `Prompt` 后（或在进入 `ResponsesApiRequest` 前）：
   - 从 `input` 中收集“本轮可见的调用 ID 集合”：
     - `FunctionCall.call_id`
     - `LocalShellCall.call_id`（或缺省时使用 `id` 作为有效 ID）
     - `CustomToolCall.call_id`
   - 过滤掉所有“其 `call_id` 不在上述集合内”的 `FunctionCallOutput` / `CustomToolCallOutput`。
   - 落点建议：`codex-rs/core/src/codex.rs::try_run_turn()` 里，`prompt` 就绪后、调用 `turn_context.client.stream(&prompt)` 之前进行一次纯内存过滤（最小侵入）。
2) 会话分叉前的历史自检（可选增强）：
   - 在 `ConversationManager::fork_conversation()`（`codex-rs/core/src/conversation_manager.rs`）中，对 `entries` 做“调用 - 输出成对性检查”，剔除尾部失配片段，确保 `initial_history` 自洽。
3) 自动 compact 的边界处理（可选增强）：
   - 在触发 `inline_compact()` 之前，如发现本轮存在刚闭合 / 尚未闭合的工具调用，优先保证配对项成对保留或一并剔除，避免只留一端。
+
权衡：过滤失配输出会丢失模型上下文中的一段“工具结果”文本，但能避免 400 直接中断对话；同时该过滤仅影响“已不自洽”的历史 / 分叉场景，是合理的兜底策略。
+
---
+
## 临时规避（无需改代码）
- 避免在“模型刚触发工具调用但尚未拿到对应输出”时立刻 backtrack/fork 或触发自动 compact。
- 若频繁命中，暂时关闭自动 compact（配置里有开关），观察是否消除 400。
- 报错发生后，再发送一条消息或重开一次会话，通常可恢复（因为失配输出不会再次注入）。
+
---
+
## 复现与验证建议
1) 打开详细日志，观察请求 payload 与历史：
   - 运行前设置：`RUST_LOG=trace`；
   - `codex-rs/core/src/client.rs` 会 `trace!` 打印 POST 与 payload；
   - 对照本轮 `input` 中的 `ResponseItem` 列表，核对是否存在 `FunctionCallOutput` 的 `call_id` 在同一 `input` 中没有对应 `FunctionCall/LocalShellCall/CustomToolCall`。
2) 人为制造分叉 / 裁剪（在工具调用后立即 backtrack 或触发 compact），确认 400 可复现；随后验证预过滤方案能消除 400。
+
---
+
## 相关代码位置（便于检索）
- 请求与错误处理
  - `codex-rs/core/src/client.rs`
    - `stream_responses()`：非 2xx 的错误返回 `UnexpectedStatus`（约 L220–L420）
    - `process_sse()`：只转发 `response.output_item.done`，避免重复（约 L480–L680，含长注释）
- 回合 / 历史拼接
  - `codex-rs/core/src/codex.rs`
    - `run_task()`（约 L1700–L2420）：组装 turn input、记录历史
    - `try_run_turn()`（约 L2040–L2410 / L1950–L2160）：构造 `Prompt` 并请求；内含 `missing_calls`（只兜底“调用缺输出”）
    - `handle_response_item()`：把工具调用与输出写回历史
    - `turn_input_with_history_for()`（约 L1120 起）：历史 + 本轮输入组合
    - 自动 compact：`ensure_auto_compact_pre_turn()` / `ensure_auto_compact_post_turn()` / `inline_compact()`（约 L3388、L3433、L3468 起）
- 历史存储 / 裁剪
  - `codex-rs/core/src/conversation_history.rs`
    - `record_items()` / `keep_last_messages()`（约 L33、L94 起）
- 会话分叉
  - `codex-rs/core/src/conversation_manager.rs`
    - `fork_conversation()` / `truncate_after_dropping_last_messages()`（约 L104、L210 起）
- 序列化（避免另一类 400）
  - `codex-rs/protocol/src/models.rs`
    - `FunctionCallOutputPayload` 的 `Serialize` 强制输出纯字符串（约 L190 起）
- TUI 错误展示
  - `codex-rs/tui/src/history_cell.rs`
    - `new_error_event()`（约 L840）：以“🖐 …”显示
  - `codex-rs/tui/src/chatwidget.rs`
    - `handle_codex_event()` → `on_error(message)`（约 L884 起）
+
---
+
## 总结
问题本质是“请求输入中 `function_call_output` 与其对应调用不成对”，常由“自动 compact 的极限裁剪或会话分叉的历史截断”导致。
最小化修复方案是在发起请求前做“配对校验 / 清洗”，过滤掉失配输出，既能避免 Responses API 400，又不会影响正常路径；同时配合在分叉 /compact 边界做成对性保障，可进一步降低失配概率。
+
