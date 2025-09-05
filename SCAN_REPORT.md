# 杂乱扫描报告

## 批次 33（会话自动）

- codex-rs/tui/tests/suite/status_indicator.rs
  - 无明显问题。

- codex-rs/tui/tests/suite/vt100_history.rs
  - 回退兜底/断言放宽：注释期望“插入两行至少产生两个 RI (ESC M)”，但断言仅要求 `>= 1`。这会降低回归敏感度，把应当失败的部分行为当作“勉强可接受”。
  - 默认值掩盖错误：`screen_rows_from_bytes` 里当 `cell` 缺失或 `contents()` 为空时以空格 `' '` 代替，并在末尾 `trim_end()`。这会把渲染缺口伪装成空白，掩盖真实缺失来源（例如某些单元格根本未写入）。测试中建议显式区分并断言 `None` 分支。
  - 轻微复杂分支：多层 `if let` 嵌套拼接字符，可读性一般；可通过 `and_then`/`map`/`unwrap_or` 或匹配表达式合并分支（非必须）。

- codex-rs/tui/tests/suite/vt100_live_commit.rs
  - 默认值掩盖错误：读取屏幕并拼接 `joined` 时，对空内容与缺失单元格同样以空格兜底，存在与上文相同的“把异常当空白”的风险，降低问题可观测性。
  - 其余：构造终端时对错误直接 `panic!`，未吞错；整体无其他明显问题。

- codex-rs/tui/tests/suite/vt100_streaming_no_dup.rs
  - 无明显问题。

## 批次 32（会话自动）

- codex-rs/execpolicy/tests/suite/bad.rs
  - 无明显问题。

- codex-rs/execpolicy/tests/suite/cp.rs
  - 无明显问题。

- codex-rs/execpolicy/tests/suite/good.rs
  - 无明显问题。

- codex-rs/execpolicy/tests/suite/head.rs
  - 参数解析歧义：`-n -1` 被解析为 `-n` 后跟一个新的选项 `-1`，从而触发 `OptionFollowedByOptionInsteadOfValue`，而非更直观的“负数非法”错误。这种“凡以 `-` 开头即视为选项”的隐式规则会使负数参数难以表达，也让错误语义不够聚焦。

- codex-rs/execpolicy/tests/suite/literal.rs
  - 空 system_path：`ValidExec::new(..., &[])` 允许可执行路径集合为空。在测试中无害，但在真实策略中易把“未配置路径”和“明确禁止执行”混淆，属于表达上的潜在杂讯点。

- codex-rs/execpolicy/tests/suite/ls.rs
  - 设计缺口/标志位置：`test_flags_after_file_args` 允许在文件参数之后继续出现标志（注释已标 TODO 与真实 `ls` 行为不符）。这种“宽松默认”在更多命令上可能演化为回退兜底路径。
  - 选项捆绑未实现：`-al` 被判定为 `UnknownOption`，等待 `option_bundling=True` 支持。当前会把常见捆绑写法当作错误，属于策略能力空洞导致的非直观失败。

- codex-rs/execpolicy/tests/suite/parse_sed_command.rs
  - 无明显问题。

- codex-rs/execpolicy/tests/suite/pwd.rs
  - 细节：`use std::vec;` 为冗余导入；测试中无需显式导入即可使用 `vec![]` 宏。虽不影响行为，但属轻微杂项。

- codex-rs/execpolicy/tests/suite/sed.rs
  - 无明显问题。

## 批次 31（会话自动）

- codex-rs/mcp-server/tests/suite/create_conversation.rs
  - 轮询兜底导致错误被稀释：`server.received_requests().await.unwrap_or_default()` 在读取失败时返回空列表，外层循环仅以“是否为空”作为停止条件，可能把底层 I/O/解析错误伪装成“暂时还没有请求”，直到超时才暴露。建议显式区分“无请求”与“获取失败”，对后者立即 fail 或记录原因。
  - 读取结果处理不一致：上一处用 `unwrap_or_default()` 忽略错误，但随后 `server.received_requests().await.unwrap()[0]` 又用 `unwrap()`，两处语义不对齐，易出现“前面吞错→后面索引 panic”的脆弱点。
  - 断言布尔取值的兜底损失诊断：`body["stream"].as_bool().unwrap_or(false)` 虽会在缺失/类型不符时触发断言失败，但错误信息只表现为布尔断言失败，隐藏了真实原因（字段缺失/类型错误）。可改为 `expect` 带上结构性提示。

- codex-rs/mcp-server/tests/suite/login.rs
  - 取消登录的事件观测被降级为告警：对 `login_chat_gpt_complete` 的通知采用 `timeout(...).await` 后 `if maybe_note.is_err() { eprintln!(...) }`，测试不失败。这会掩盖“取消后仍发送完成通知”或“通知管道断开”等回归。若仅为缓解竞态，建议最少在调试构建下 fail 或统计重试次数。
  - 其余流程以 `expect`/`timeout` 明确失败，暂无明显问题。

- codex-rs/mcp-server/tests/suite/interrupt.rs
  - 无明显问题。

- codex-rs/mcp-server/tests/suite/send_message.rs
  - 无明显问题。

## 批次 30（会话自动）

- codex-rs/core/tests/all.rs
  - 无明显问题。

- codex-rs/core/tests/common/lib.rs
  - 默认值掩盖/入口未校验结构：在构建 SSE 文本的多处逻辑中使用 `e.as_object().map(|o| o.len() == 1).unwrap_or(false)` 判定是否仅含 `type` 字段（`load_sse_fixture*` 系列）。被破坏不变量：测试夹具事件应为“对象且至少包含 `type`”，否则应在入口处直接报错；当前对非对象分支返回 `false` 相当于将坏结构当作“含数据”的正常路径处理。

- codex-rs/core/tests/suite/client.rs
  - 兜底默认值导致来源不一致风险：`write_auth_json` 中 `account_id.unwrap_or("acc-123")` 会在未提供时伪造一个看似有效的账户 ID 写入 JWT 负载，但外层 `tokens["account_id"]` 仅在 `Some` 时写入。被破坏不变量：同一身份信息应有单一真实来源并保持一致；首次坏点为该 `unwrap_or`。

- codex-rs/core/tests/suite/cli_stream.rs
  - 消费点兜底：从请求体读取 `instructions` 时使用 `body.get("instructions").and_then(|v| v.as_str()).unwrap_or_default()`。被破坏不变量：`instructions` 应当存在且为字符串；首次坏点是消费点以空串兜底而非在入口断言结构。虽随后使用 `contains(marker)` 会使缺失用例失败，但入口判定更清晰。
  - 异常吞噬（搜索/发现流程）：扫描 session 文件时多处 `Err(_) => continue`（遍历目录、读文件、逐行反序列化）。虽最终若找不到目标会 `panic!`，不构成“看似成功”，但建议专一化仅忽略“非目标/不可读”这类预期错误，避免误吞结构性问题。

- codex-rs/core/tests/suite/compact.rs
  - 默认值掩盖输入坏值：匹配器中多处 `std::str::from_utf8(&req.body).unwrap_or("")`。被破坏不变量：HTTP 请求体应为有效 UTF‑8；首次坏点用空串兜底可能让匹配误判为“不包含某片段”。
  - 消费点兜底：收集第三次请求消息时对 `role`、`text` 使用 `as_str().unwrap_or_default()`，会将缺失字段转为空字符串继续统计。被破坏不变量：消息项必须包含 `role` 与文本内容；应在入口断言结构。
  - 同 common 中的 SSE 帮助逻辑：`ev.as_object().map(|o| o.len() == 1).unwrap_or(false)` 的“非对象→false”判定同样掩盖了夹具事件的坏结构。

- codex-rs/core/tests/suite/exec.rs
  - 无明显问题。

- codex-rs/core/tests/suite/exec_stream_events.rs
  - 回退兜底目录：构造 `ExecParams` 时使用 `std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))`。被破坏不变量：工作目录不可未知；首次坏点在参数构建时以 `"."` 兜底，掩盖环境错误。应在入口显式报错或在测试前确保上下文。

- codex-rs/core/tests/suite/live_cli.rs
  - 异常吞噬：`tee` 中对向父进程 stdout/stderr 写入的 I/O 错误使用 `.ok()` 全部忽略（`write_all`/`flush`）。被破坏不变量：I/O 失败应被处理或记录；首次坏点为该 catch‑all 忽略。虽为测试便捷，但可能掩盖真实失败。

- codex-rs/core/tests/suite/mod.rs
  - 无明显问题。

- codex-rs/core/tests/suite/prompt_caching.rs
  - 无明显问题。

## 批次 29（会话自动）

- codex-rs/execpolicy/build.rs
  - 无明显问题。

- codex-rs/execpolicy/Cargo.toml
  - 无明显问题。

- codex-rs/execpolicy/src/error.rs
  - 异常信息降级：`CannotCanonicalizePath` 仅保留 `std::io::ErrorKind`，丢失底层 `io::Error` 细节（路径权限/不存在/环等）。虽未吞噬异常，但削弱入口处的可观测性，不利于定位首个坏值来源。

- codex-rs/execpolicy/src/exec_call.rs
  - 无明显问题。

- codex-rs/execpolicy/src/lib.rs
  - 无明显问题。

- codex-rs/execpolicy/src/main.rs
  - 进程层“看似成功”（异常吞噬/默认值掩盖）：`check_command` 将 `Forbidden` 与任意 `Err` 在 `require_safe == false` 时统一映射为退出码 `0`，仅在 JSON 里表征失败。首个破坏点：`match policy.check(&exec_call)` 分支中 `Ok(MatchedExec::Forbidden { .. })` 与 `Err(err)` 两路均在 `check == false` 返回 `0`。被破坏不变量：退出码应与结果的成败一致，除非调用方显式声明可降级；否则上游仅看退出码会误判“成功”。

- codex-rs/execpolicy/src/program.rs
  - 正负例校验逻辑混淆（复杂分支语义不清）：
    - `verify_should_match_list` 把所有 `Ok(_)`（包括 `Ok(MatchedExec::Forbidden)`）当作“通过”，破坏“不变量：正例应当得到允许执行的匹配（Match），而非 Forbidden”。首次坏点：`match self.check(&exec_call) { Ok(_) => {} Err(error) => violations.push(...) }` 未区分枚举变体。
    - 代码注释“可能是 `--option=value` 风格”但未实现解析，随即返回 `UnknownOption`。这是入口处未完成的分支，容易诱发“在下游补 if 兜底”的冲动；应在入口完善结构而非下游加分支。

- codex-rs/execpolicy/src/sed_command.rs
  - 无明显问题。

- codex-rs/execpolicy/tests/all.rs
  - 无明显问题。

- codex-rs/execpolicy/tests/suite/mod.rs
  - 无明显问题。

## 批次 28（会话自动）

- codex-rs/tui/src/status_indicator_widget.rs
  - 无明显问题。

- codex-rs/tui/src/streaming/controller.rs
  - 头部不变量被破坏：应仅在有实际内容时渲染头部。首次坏点：`finalize(false, ...)` 分支在 `remaining.is_empty()` 时仍无条件 `enqueue(vec![Line::from("")])` 触发动画；随后 `on_commit_tick` 会对仅含空行的 `history` 调用 `emit_header_if_needed`，导致“只有头部+空行”被插入历史。建议在入口分支处仅在确有待排队内容时才加入空行/触发动画，而非在下游消费点兜底。
  - 状态清理重复/分支复杂：`allow_reemit_in_turn` 与 `reset_for_stream` 在 `finalize(true)` 与 `on_commit_tick` 的收尾路径各处重复设置，增加漂移风险。可提取统一的“完成清理”入口，减少分支。
  - 重入风险：`begin` 在 `active == true` 时不重置头部流状态；若误用导致重入，可能与 `finishing_after_drain` 交互产生意外状态。建议在入口约束调用序或在 `begin` 做更强约束/重置。

- codex-rs/tui/src/streaming/mod.rs
  - 无明显问题。

- codex-rs/tui/src/text_formatting.rs
  - 复杂分支/手写解析器风险：`format_json_compact` 通过字符级状态机处理转义与引号，`escape_next` 在每次遇到 `\\` 时“取反”而非“针对下一字符一次性置位并清除”，对诸如 `"\\\\"`、`\uXXXX` 等序列易出错，破坏“不在字符串外改变结构/不在字符串内改变引号语义”的不变量。首次坏点：`'\\' if in_string => { escape_next = !escape_next; ... }`。不增分支优先：建议在数据来源处采用成熟的 JSON 紧凑化/最小化方案（如 serde 配置/第三方格式化器），而非在下游追加分支修正。
  - 异常吞噬/默认值掩盖：`to_string_pretty(&json).unwrap_or_else(|_| json.to_string())` 将序列化失败静默降级为普通 `to_string`，调用方无法察觉。虽对 `Value` 几乎不触发，但建议最少记录/区分失败原因。

- codex-rs/tui/src/tui.rs
  - 广泛静默忽略错误（异常专一化缺失）：多处使用 `let _ = execute!(...)` 与无条件忽略返回值（如 `PushKeyboardEnhancementFlags`、`PopKeyboardEnhancementFlags`、`Enter/LeaveAlternateScreen`、`Enable/DisableAlternateScroll`、`cursor::Show`、`terminal.clear`）。首次坏点：`set_modes` 对增强键盘标志的 `execute!` 直接丢弃错误；同类模式在进入/离开备选屏及事件路径反复出现。应区分“功能不支持/能力探测失败”与“真实 I/O 异常”，避免把失败当成“可继续”。
  - 入口兜底过宽：Ctrl+V 路径 `paste_image_to_temp_png()` 的 `Err(_) => yield TuiEvent::Key(key_event)` 将所有错误等同于“无图像可粘贴”，掩盖诸如权限/临时目录异常。首次坏点：该 `match` 的 catch‑all `Err(_)`。建议细分错误类型：无图像→回退；系统错误→记录/提示。
  - 备注：后台调度使用远未来时间作为哨兵无功能性问题，但需留意跨长时间睡眠的单调时钟语义。

- codex-rs/tui/src/updates.rs
  - 默认值掩盖破坏语义：`is_newer` 返回 `Option<bool>`，调用方在 `get_upgrade_version` 中以 `unwrap_or(false)` 将“版本无法比较（如预发布 tag 或格式异常）”等同为“不是新版本”。首次坏点：`is_newer(&info.latest_version, current_version).unwrap_or(false)`。被破坏不变量：版本比较应对接受的输入域全定义或显式失败，而非静默为否。建议在入口过滤/规范化 tag（仅稳定语义化版本），或在比较失败时记录并清理缓存，而非回落为 `false`。
  - 入口错误专一化不足：`read_version_info(...).ok()` 抹平“文件不存在/解析失败/权限错误”的差异，导致后续逻辑无法区分“首次运行”与“缓存损坏”。建议仅对“未找到”宽容，其余记录并重建。

- codex-rs/tui/src/user_approval_widget.rs
  - 无明显问题。

- codex-rs/tui/Cargo.toml
  - 无明显问题。

- codex-rs/tui/tests/all.rs
  - 无明显问题。

- codex-rs/tui/tests/suite/mod.rs
  - 无明显问题。

## 批次 23（会话自动）

- codex-rs/tui/src/app_backtrack.rs
  - 异常吞噬：`open_transcript_overlay` 与 `close_transcript_overlay` 对 `tui.enter_alt_screen()`/`tui.leave_alt_screen()` 以 `let _ = ...` 静默丢弃错误。首次坏点：上述两处调用。被破坏不变量：终端状态切换应“要么成功、要么可观测失败”，否则 UI 状态与真实终端状态可能漂移。
  - 默认值掩盖：确认回退时取预填 `prefill` 使用 `nth_last_user_text(...).unwrap_or_default()`，在计算/竞态导致找不到目标用户消息时回退为空串并继续分叉。首次坏点：`overlay_confirm_backtrack` 和 `confirm_backtrack_from_main`。被破坏不变量：当所选回退目标不存在时，应显式放弃/告警，而非提交“看似成功”的空文本。

- codex-rs/tui/src/app_event.rs
  - 无明显问题。

- codex-rs/tui/src/app_event_sender.rs
  - 异常吞噬：`send(&self, event)` 仅在通道关闭时 `tracing::error!` 记录，但调用方拿不到失败信号，后续逻辑仍按“已发送”推进。首次坏点：`if let Err(e) = self.app_event_tx.send(event)` 分支。被破坏不变量：事件投递应具备可感知的成败边界，以免上层状态机在消息丢失时继续运行。

- codex-rs/tui/src/app.rs
  - 异常吞噬：多处进入备选屏 `tui.enter_alt_screen()` 以 `let _ = ...` 静默失败（TUI Draw/Transcript/Diff 路径）。首次坏点：`handle_key_event` 与 `AppEvent::DiffResult` 分支。被破坏不变量：屏幕模式切换失败应可见化，避免 UI 逻辑与终端实际状态脱节。
  - 后台线程噪声/泄漏风险：提交动画线程在 `StartCommitAnimation` 中 spawn 后，循环仅受 `commit_anim_running` 控制；当接收端关闭时，`AppEventSender::send` 会在内部记录错误但不会让线程退出，导致每 50ms 重复报错（并持续占用资源）。首次坏点：提交动画发送循环与 `send` 的错误吞噬叠加。被破坏不变量：后台任务应受生命周期/通道状态驱动停止；发送失败应被视为“应当收敛”的终止信号，而非继续重试。
  - 轻微：`supports_keyboard_enhancement().unwrap_or(false)` 将探测失败回退为 `false`，属无害默认，但依然会掩盖环境/驱动异常；若该能力对交互契约关键，建议分辨“不可用”与“探测失败”。

- codex-rs/tui/src/backtrack_helpers.rs
  - 回退兜底：`normalize_backtrack_n` 当 `n` 超出范围时回绕为 `1`，当无用户消息时回落为 `0`。这把“越界请求”在入口处修正为“看似有效”，下游很难分辨是用户选择失效还是确有最近一条。首次坏点：`normalize_backtrack_n`。被破坏不变量：回退目标选择应具备显式越界信号（`None/Err`），而非静默改值。
  - 脆弱匹配：`find_nth_last_user_header_index` 依赖拼接 spans 后的纯文本等于字面量 `"user"` 判定消息头；一旦样式/本地化改变会静默找不到目标，进而与上游 `unwrap_or_default` 叠加造成空文本回退。入口坏值：消息头判定策略过于脆弱。

- codex-rs/tui/src/bottom_pane/approval_modal_view.rs
  - 队列语义异常：`maybe_advance` 使用 `self.queue.pop()`，导致审批请求按 LIFO 处理；若预期是“到达顺序处理”，则该实现与“队列”命名不符。首次坏点：`maybe_advance`。被破坏不变量：审批请求处理顺序通常为 FIFO，应在入口结构（如 `VecDeque::pop_front`）修正，而非下游分支补丁。

- codex-rs/tui/src/bottom_pane/bottom_pane_view.rs
  - 无明显问题。

- codex-rs/tui/src/bottom_pane/chat_composer_history.rs
  - 异常吞噬：`on_entry_response` 对 `entry: None` 静默返回 `None`，UI 侧仅拿到 `false`，用户看起来“没有反应”。首次坏点：`on_entry_response`。被破坏不变量：当后端不存在对应 offset 或出现错误，应区分“空历史条目”与“拉取失败”，并可见化给 UI。
  - 异常吞噬（间接）：`populate_history_at_index` 发送 `AppEvent::CodexOp` 失败由 `AppEventSender::send` 内部记录，但调用方不知情；导航状态机会在无请求实际发出的情况下保持“正在浏览”。入口坏点：发送失败不可见。

- codex-rs/tui/src/bottom_pane/chat_composer.rs
  - 默认值掩盖：`set_token_usage` 在缺少 `initial_prompt_tokens` 与 `cached_input_tokens` 时回退为 `0`，会把“未知基线”当作“零开销”，导致“剩余上下文百分比”过于乐观。首次坏点：`set_token_usage`。被破坏不变量：容量占用应区分“未知/不可测”与“零”。
  - 复杂分支/隐式状态机：Enter 行为受多个时间窗与计数器（`PASTE_BURST_CHAR_INTERVAL`、`PASTE_ENTER_SUPPRESS_WINDOW`、`paste_burst_buffer`、`in_paste_burst_mode` 等）共同驱动，存在误判时“吃掉提交”或“误插入换行”的可能。首次坏点：`handle_key_event_without_popup` 的 Enter 分支。被破坏不变量：在非粘贴场景下，Enter 应稳定触发提交；粘贴检测建议在入口专一化（如 bracketed paste/显式粘贴事件），避免时间启发式的跨分支耦合。
  - 异常吞噬（可用性）：`handle_paste_image_path` 仅 `tracing::info!` 记录 `image_dimensions` 错误并返回 `false`，对用户无反馈。首次坏点：`handle_paste_image_path`。被破坏不变量：用户粘贴无效图片路径时应得到一次性提示，而非静默失败。

- codex-rs/tui/src/bottom_pane/command_popup.rs
  - 无明显问题。

## 批次 18（会话自动）
- codex-rs/mcp-client/src/lib.rs
  - 无明显问题。

## 批次 22（会话自动）

- codex-rs/protocol-ts/src/lib.rs
  - 索引构建静默去重掩盖命名冲突：`generate_index_ts` 对文件基名 `stems` 先 `sort` 再 `dedup`，当存在同名 stem（如 `foo.ts` 与 `foo.d.ts` 或上游命名冲突）时，`index.ts` 仅导出一次，悄然丢弃重复，掩盖上游冲突。
    - 首次坏状态：`ts_files_in(out_dir)` 返回包含重复 stem 的列表。
    - 被破坏不变量/契约：导出的每个名称应唯一对应单一模块；若出现重复应显式报错/上报，而非静默去重。
  - 输出目录污染风险导致“误标生成文件”：`prepend_header_if_missing` 会给目录下所有扩展为 `.ts` 的文件添加“GENERATED CODE”头，包括 `.d.ts` 与可能的手写 `.ts`。若 `out_dir` 混入非本工具生成的文件，将被重写并被标注为“生成产物”。
    - 首次坏状态：`out_dir` 中预先存在非本次生成的 `.ts/.d.ts` 文件。
    - 被破坏不变量/契约：`out_dir` 应仅包含由该工具生成/管理的文件；否则应先校验/隔离，避免修改外部文件。

- codex-rs/protocol-ts/src/main.rs
  - 无明显问题。

- codex-rs/protocol-ts/Cargo.toml
  - 无明显问题。

- codex-rs/mcp-client/src/main.rs
  - 异常吞噬：日志初始化 `tracing_subscriber::fmt().try_init()` 的结果被忽略（`let _ = ...`），当初始化失败（如重复初始化）时静默继续，破坏“入口优先/异常专一化”，首次坏点：`try_init` 返回 Err 被丢弃。应显式区分“已初始化”与“真正失败”，至少记录一次告警。
  - 回退兜底堆叠：`EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(default_level)).unwrap_or_else(|_| EnvFilter::new(default_level))` 形成多层回退。对常量 `debug`，第二层几乎不可能失败，第三层属于冗余兜底，会掩盖无效环境变量配置的可见性。首次坏点：对无效 env 的处理采用“回退为默认”而非早失败+提示。

- codex-rs/mcp-client/src/mcp_client.rs
  - 异常吞噬（可观测性丢失）：子进程 stderr 被定向到 `Stdio::null()`，服务器启动/运行期的错误输出被完全丢弃，破坏“错误必须可见”的不变量。首次坏点：`Command` 构建时 `.stderr(std::process::Stdio::null())`。
  - 异常吞噬（I/O 读错误）：读取循环使用 `while let Ok(Some(line)) = lines.next_line().await`，当出现 `Err(e)` 时循环直接结束且无任何日志，导致读通道错误被静默掩盖。首次坏点：`while let Ok(Some(...))` 条件式本身吞掉了错误分支。应在 Err 分支记录错误并进行清理/关闭。
  - 协议违例未记录：`dispatch_error` 对 `RequestId::String(_)` 直接 `return`，未记录任何日志；与 `dispatch_response` 中相同情况会 `error!` 形成不一致。这会让服务端返回字符串 ID 的错误被静默丢弃。首次坏点：`dispatch_error` 的 `RequestId::String(_) => return` 分支。遵循“异常专一化”，应记录并丢弃，而非无声返回。
  - 资源清理提示：`timeout` 分支会从 `pending` 移除条目，之后迟到的响应被 reader 记录为 `warn!(id, "no pending request found")` 后丢弃，逻辑一致；此处无兜底伪成功。

- codex-rs/mcp-client/Cargo.toml
  - 无明显问题（纯依赖与元数据）。

## 批次 13（会话自动）
- codex-rs/common/src/approval_mode_cli_arg.rs
  - 无明显问题。

- codex-rs/common/src/approval_presets.rs
  - 无明显问题。

- codex-rs/common/src/config_override.rs
  - 入口坏值与异常吞噬：`parse_overrides` 在调用 `parse_toml_value` 失败时直接将值回退为 `Value::String`（`Err(_) => { ... Value::String(...) }`）。这会掩盖原本应为布尔/数值/数组等类型的解析错误，导致“看似成功”的覆盖，破坏“不变量优先/异常专一化”原则。首次坏值注入点：`parse_overrides` 中的回退分支。
  - 契约破坏与静默修复：`apply_single_override` 在遍历路径时，若当前节点不是 `Value::Table`，会无条件将其替换为一个新表并继续（`*current = Value::Table(Table::new())`）。这会悄然丢弃原有标量/数组值以满足下游结构，属于“在消费点兜底修结构”，而不是在入口校验路径/类型兼容性。首个破坏点：`apply_single_override` 里匹配分支的 `_ => { *current = Value::Table(...) }`。
  - 错误传播缺失：`apply_on_value` 返回 `Result<(), String>`，但内部的 `apply_single_override` 不返回错误，类型/路径冲突被吞没，调用方永远拿到 `Ok(())`。这会让“失败”被报告为“成功”。
  - 文档与实现不一致：模块文档声称“右值按 JSON 解析，并施加到 serde_json::Value”，实现实际使用 `toml::Value` 与 TOML 解析。这种错配会误导调用方选择错误的数据格式，属于入口契约不一致。
  - 次要问题：对未加引号字符串的“修剪引号”逻辑（`trim().trim_matches`）是单端字符剥离，可能误删仅一侧的引号；点分路径不支持转义会限制含点键名的覆盖能力。

- codex-rs/common/src/config_summary.rs
  - 无明显问题。

- codex-rs/common/src/elapsed.rs
  - 整型截断与潜在负值：`format_duration` 将 `duration.as_millis()`（`u128`）强制转换为 `i64`，超大持续时间会发生截断/回绕，可能产生负数，进而误入 `< 1000` 或秒/分分支生成荒谬输出。首次坏值注入点：`let millis = duration.as_millis() as i64;`。不变量：毫秒数应为非负且能精确表达 `Duration`；被截断破坏。
  - 边界展示不一致（提示）：`59_999ms` 被四舍五入显示为 `60.00s`，`60_000ms` 显示为 `1m00s`，虽与测试一致，但在 UX 上存在跨阈值的表现跳变。

- codex-rs/common/src/fuzzy_match.rs
  - 无明显问题。匹配失败返回 `None`，未见 catch-all 兜底为“成功”。分数计算对前缀有强奖励，索引经排序去重，行为一致可预期。

- codex-rs/common/src/lib.rs
  - 无明显问题。

- codex-rs/common/src/model_presets.rs
  - 无明显问题。

- codex-rs/common/src/sandbox_mode_cli_arg.rs
  - 无明显问题。

- codex-rs/common/src/sandbox_summary.rs
  - 轻微可见性问题：为摘要展示将路径用 `to_string_lossy()` 转换，若包含非 UTF-8 字节将以替代符显示，可能掩盖真实路径字节序列。但该处仅为 UI 摘要，不影响策略本身。

- codex-rs/common/Cargo.toml
  - 无明显问题（功能性配置）。

## 批次 6（会话自动）

- [文件]：codex-rs/mcp-server/src/codex_message_processor.rs
  - [定位]：
    - 函数 apply_bespoke_event_handling（第 646–693 行、第 672–709 行，rg:663/685）
    - 函数 on_exec_approval_response（第 826–836 行）
  - [类别]：[默认值掩盖] / [兜底] / [异常吞噬] / [状态扩散]
  - [不变量]：
    - 发往客户端的审批请求参数必须可序列化且完整；序列化失败应当使事务失败或被拒，而非以默认值继续。
    - 每次审批请求必须产生一次明确的决策（同意/拒绝）或明确错误事件；不能出现“仅记录日志后无响应”。
  - [首次坏点]：对审批参数 `serde_json::to_value(&params)` 使用 `unwrap_or_default()`，当序列化失败时以默认 JSON 值（Null/Object 空壳）继续发起请求，掩盖上游错误，使下游收到“看似有效”的请求。
  - [建议]：在 apply_bespoke_event_handling 源头判断参数序列化是否成功；失败则：
    - 对 Patch/Exec 审批直接向会话提交“拒绝”或发送服务端错误响应；
    - 不要使用默认值继续发起请求。
    同时在 on_exec_approval_response 的 `receiver.await` 失败分支，改为向 Codex 提交一个明确的 `Denied` 或上报错误事件，避免悬挂。
  - [片段]：
    let value = serde_json::to_value(&params).unwrap_or_default();
    ...
    Err(err) => { tracing::error!("request failed: {err:?}"); return; }

- [文件]：codex-rs/mcp-server/src/codex_tool_config.rs —— 无明显问题

- [文件]：codex-rs/mcp-server/src/codex_tool_runner.rs
  - [定位]：
    - 函数 run_codex_tool_session 提交初始提交失败分支（第 100–108 行，rg:104）
    - 函数 run_codex_tool_session_reply 提交失败分支（第 132–142 行，rg:137）
    - 函数 run_codex_tool_session_inner TaskComplete 分支（第 226–242 行，rg:226）
  - [类别]：[异常吞噬] / [状态扩散] / [默认值掩盖]
  - [不变量]：MCP `tools/call` 必须“一请求一响应”；任何提交失败都应转换为一次明确的错误响应返回给调用方。
  - [首次坏点]：提交初始提示/用户输入失败时，仅 `tracing::error!` 后 `return`，未向调用方发送任何响应，导致调用方悬挂。
  - [建议]：在上述错误分支构造 `CallToolResult{ is_error: Some(true), ... }` 并通过 `send_response` 返回；同时在 TaskComplete 中对 `last_agent_message == None` 不应默认空串，可显式为 None/结构化标记或带提示，避免信息丢失。
  - [片段]：
    if let Err(e) = conversation.submit_with_id(submission).await { tracing::error!(...); /* 无响应即返回 */ }

- [文件]：codex-rs/mcp-server/src/error_code.rs —— 无明显问题

- [文件]：codex-rs/mcp-server/src/exec_approval.rs
  - [定位]：函数 on_exec_approval_response（第 116–131 行，rg:125）
  - [类别]：[异常吞噬] / [状态扩散]
  - [不变量]：每次 Exec 审批请求必须落地为一次明确的决策事件或明确失败；通道错误不能被视作“无需处理”。
  - [首次坏点]：`receiver.await` 出错仅记录 `error!(...)` 然后 `return`，未向 Codex 提交拒绝或错误，导致流程悬挂。
  - [建议]：在此入口统一做保守拒绝（Denied）或发出错误事件，确保状态闭合；避免在消费端再加兜底。
  - [片段]：
    Err(err) => { error!("request failed: {err:?}"); return; }

- [文件]：codex-rs/mcp-server/src/json_to_toml.rs
  - [定位]：函数 json_to_toml（第 5–22 行）
  - [类别]：[默认值掩盖]
  - [不变量]：配置覆盖中 JSON 的 Null 与 TOML 的空字符串语义不同；Null 应表示“缺失/不设置”，不应被转换为空字符串从而改变含义。
  - [首次坏点]：将 `JsonValue::Null` 映射为 `TomlValue::String(String::new())`，使“缺失”被误解为“设置为空串”。
  - [建议]：入口处区分 Null：
    - 若作为 CLI 覆盖：遇 Null 应“不写入该键”或报错；
    - 若必须有值：在更靠前的验证层拒绝 Null。
  - [片段]：
    JsonValue::Null => TomlValue::String(String::new()),

- [文件]：codex-rs/mcp-server/src/lib.rs
  - [定位]：stdin 读取循环（第 70–86 行，rg:72/80）
  - [类别]：[异常吞噬]
  - [不变量]：I/O 错误与 EOF 必须区分；读取错误不应被静默当作“输入结束”。
  - [首次坏点]：`lines.next_line().await.unwrap_or_default()` 将读取错误劣化为 `None`，导致循环无声退出，破坏错误专一化与可观测性。
  - [建议]：明确匹配 `Ok(Some(line)) / Ok(None) / Err(e)`，对 `Err(e)` 记录并选择退出或上报，同步关闭流程而非静默。
  - [片段]：
    while let Some(line) = lines.next_line().await.unwrap_or_default() { ... }

- [文件]：codex-rs/mcp-server/src/main.rs —— 无明显问题

- [文件]：codex-rs/mcp-server/src/message_processor.rs
  - [定位]：process_request 将 `JSONRPCRequest` → `McpClientRequest` 失败分支（第 86–98 行，rg:94）
  - [类别]：[异常吞噬] / [状态扩散]
  - [不变量]：JSON-RPC 请求失败必须收到 JSON-RPC 错误响应；不能仅记录 warn 即返回，导致调用方等待超时。
  - [首次坏点]：`McpClientRequest::try_from(request)` 失败后仅 `tracing::warn!` 并 `return`，未回复错误。
  - [建议]：在此入口直接使用 `send_error` 返回 `INVALID_REQUEST`，并携带最小上下文，避免在后续链路补丁式兜底。
  - [片段]：
    Err(e) => { tracing::warn!(...); return; }

- [文件]：codex-rs/mcp-server/src/outgoing_message.rs —— 无明显问题

## 批次 3（会话自动）

- [文件]：codex-rs/core/src/exec.rs
  - [定位]：
    - 函数 consume_truncated_output（第 301–318 行）+ synthetic_exit_status（第 394–405 行）与调用方 process_exec_tool_call（第 135–146 行）
    - 函数 is_likely_sandbox_denied（第 174–187 行）
    - 函数 read_capped（第 359–382 行）
  - [类别]：[默认值掩盖] / [状态扩散] / [兜底] / [异常吞噬]
  - [不变量]：
    - 超时与信号来源必须被精确区分并以明确的错误通道返回，禁止通过“伪造的 ExitStatus”混入常规退出码路径。
    - 沙箱拒绝应由明确的来源信号/返回约定识别，未知退出码不可一概当作“沙箱拒绝”。
    - 事件/聚合通道的发送失败需要被特化处理（如通道关闭）而非静默忽略。
  - [首次坏点]：在 consume_truncated_output 超时分支伪造退出状态：`synthetic_exit_status(EXIT_CODE_SIGNAL_BASE + TIMEOUT_CODE)`（第 310 行）。随后在上层 process_exec_tool_call 通过 `exit_status.signal() == Some(TIMEOUT_CODE)` 企图识别超时（第 135–141 行）。但 `from_raw(128+64)` 并非标准的“由信号终止”编码，导致 `signal()` 很可能为 None，进而落入 `code()` 与 `is_likely_sandbox_denied()` 的兜底路径，错误地把超时或其它失败标记为“沙箱拒绝”。
  - [建议]：
    - 在入口修正“进程结果”的数据结构：用显式枚举区分 {正常退出(码), 被信号杀死(信号), 超时, Ctrl-C}，或直接在超时分支返回 Err(CodexErr::Sandbox(Timeout))，避免伪造 ExitStatus。
    - `is_likely_sandbox_denied` 仅匹配明确的沙箱约定码/标记（由 seatbelt/landlock wrapper 明示传回），不要以“非 127 则当沙箱拒绝”的默认返回 true（第 185–186 行）。
    - 对 `tx_event.send` 与聚合通道 `tx.send` 的错误进行特化：区分通道关闭与临时背压，至少记录 warn 并携带上下文，而非 `let _ = ...` 静默吞掉（第 376–382 行）。
  - [片段]：
    // timeout
    child.start_kill()?;
    synthetic_exit_status(EXIT_CODE_SIGNAL_BASE + TIMEOUT_CODE)
    ...
    match raw_output.exit_status.signal() { Some(TIMEOUT_CODE) => ... }

- [文件]：codex-rs/core/src/git_info.rs
  - [定位]：
    - 函数 run_git_command_with_timeout（第 112–124 行）
    - 函数 get_default_branch 使用 unwrap_or_default（第 151 行）
    - 函数 branch_ancestry 使用 unwrap_or_default（第 247 行）
  - [类别]：[异常吞噬] / [默认值掩盖] / [状态扩散]
  - [不变量]：
    - “超时/进程失败/非零退出”应被区分并向上游显式呈现；不可在入口将所有错误压扁为 None，导致调用方误将“出错”当作“无数据”。
  - [首次坏点]：`run_git_command_with_timeout` 将“超时或错误”一律返回 None（第 120–123 行），丢失错误类型与上下文，破坏了“错误专一化”。
  - [建议]：
    - 将返回类型改为 `Result<Output, GitErr>`，区分 Timeout、SpawnError、NonZeroExit(含码+stderr) 等；
    - 上层去除 `unwrap_or_default()`（第 151、247 行）这类把错误劣化为空集合/空字符串的兜底，改为在入口分支：无远端/无默认分支与“查询失败”走不同通道，并在最终接口按需降级或显式返回 None/Err。
  - [片段]：
    match result {
        Ok(Ok(output)) => Some(output),
        _ => None, // Timeout or error

## 批次 9（会话自动）

- [文件]：codex-rs/chatgpt/src/apply_command.rs —— 无明显问题

- [文件]：codex-rs/chatgpt/src/chatgpt_client.rs
  - [定位]：函数 chatgpt_get_request（第 10–50 行），错误分支（第 45–49 行）
  - [类别]：[默认值掩盖] / [异常吞噬]
  - [不变量]：错误响应应保留真实的失败语义；读取响应体失败与“空响应体”不可等同，错误信息不应被降级为空串。
  - [首次坏点]：在错误分支使用 `response.text().await.unwrap_or_default()`，将“读取 body 出错”掩盖为“空 body”，破坏错误专一化并降低可观测性。
  - [建议]：在此入口对 body 读取使用 `context` 并显式传播错误，例如 `let body = response.text().await.context("Failed to read error body")?;`；或在无法读取时保留底层错误而非返回空串。
  - [片段]：
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    anyhow::bail!("Request failed with status {}: {}", status, body)

- [文件]：codex-rs/chatgpt/src/chatgpt_token.rs
  - [定位]：函数 get_chatgpt_token_data（第 11–13 行），函数 set_chatgpt_token_data（第 15–18 行）
  - [类别]：[异常吞噬] / [兜底] / [状态扩散]
  - [不变量]：全局令牌存取必须具备一致性与可观测性；锁中毒或加锁失败应视为错误并显式上抛，而非退化为“无令牌”。
  - [首次坏点]：`CHATGPT_TOKEN.read().ok()?.clone()` 将 RwLock 读取失败静默转化为 None，令“并发错误/锁中毒”在上游被误判为“未登录”；`set_chatgpt_token_data` 也在写锁失败时静默丢弃更新，导致状态不一致。
  - [建议]：将 get/set 返回 `Result<_, LockErr>` 并在调用源头（如初始化或请求发起前）显式处理；至少对锁中毒进行记录并中止操作，避免以 None 兜底扩散到后续业务路径。
  - [片段]：
    pub fn get_chatgpt_token_data() -> Option<TokenData> {
        CHATGPT_TOKEN.read().ok()?.clone()
    }

- [文件]：codex-rs/chatgpt/src/get_task.rs
  - [定位]：枚举 OutputItem 反序列化（第 17–25 行）
  - [类别]：[兜底] / [状态扩散]
  - [不变量]：后端返回的 `output_items` 类型集合应受控且可枚举；出现未知类型应被明确识别并上报，而非被吞并为“其它”继续下游流程。
  - [首次坏点]：`#[serde(other)] Other` 将未知变体在入口处吞并为“成功可忽略”的分支，直到消费点再以“找不到 PR”失败，错误语义被延迟且失真。
  - [建议]：去除 `#[serde(other)]` 或改为显式 `Unknown { raw: Value }` 并在入口记录/返回错误；优先在解码层面特化异常，避免在消费层补 if。
  - [片段]：
    #[derive(Debug, Deserialize)]
    #[serde(tag = "type")]
    pub enum OutputItem {
        #[serde(rename = "pr")] Pr(PrOutputItem),
        #[serde(other)] Other,
    }

- [文件]：codex-rs/chatgpt/src/lib.rs —— 无明显问题

- [文件]：codex-rs/chatgpt/tests/all.rs —— 无明显问题

- [文件]：codex-rs/chatgpt/tests/suite/apply_command_e2e.rs
  - [定位]：
    - 函数 create_temp_git_repo 中多处外部命令（第 30–42 行、46–51 行）
    - 测试 test_apply_command_with_merge_conflicts 中的外部命令（第 145–157 行）
  - [类别]：[异常吞噬] / [默认值掩盖] / [状态扩散]
  - [不变量]：测试前置的每一步 git 命令必须成功，否则应立即失败；“子进程成功启动”不等于“命令成功执行”。
  - [首次坏点]：对 `git config`、`git add`、`git commit` 等调用仅 `output().await?` 而未校验 `status.success()`，使非零退出被当作“成功执行”，错因扩散到后续断言。
  - [建议]：对每次外部命令调用增加 `ensure!(output.status.success(), "...: {}", String::from_utf8_lossy(&output.stderr))`；或封装辅助函数统一执行并断言成功，保证失败“就地暴露”。
  - [片段]：
    Command::new("git").args(["config", "user.email", "test@example.com"])...
        .output().await?; // 未检查 status.success()

- [文件]：codex-rs/chatgpt/tests/suite/mod.rs —— 无明显问题

- [文件]：codex-rs/chatgpt/tests/task_turn_fixture.json —— 无明显问题

- [文件]：codex-rs/chatgpt/Cargo.toml —— 无明显问题
    }

- [文件] codex-rs/core/src/exec_env.rs —— 无明显问题

- [文件] codex-rs/core/src/flags.rs —— 无明显问题

- [文件] codex-rs/core/src/is_safe_command.rs —— 无明显问题

- [文件] codex-rs/core/src/landlock.rs —— 无明显问题

- [文件] codex-rs/core/src/lib.rs —— 无明显问题

- [文件] codex-rs/core/src/mcp_connection_manager.rs —— 无明显问题

- [文件] codex-rs/core/src/mcp_tool_call.rs —— 无明显问题

- [文件] codex-rs/core/src/mcp_view.rs —— 无明显问题

## 批次 1（会话自动）

- [文件]：codex-cli/bin/codex.js
  - [定位]：函数 tryImport（第 68–75 行）
  - [类别]：[异常吞噬] / [默认值掩盖]
  - [不变量]：仅在可接受的“模块缺失”情况下降级行为；对其它导入错误（语法/运行时错误）应显式失败而非静默视为“未安装”。
  - [首次坏点]：catch-all 捕获任何错误并返回 null，导致无法区分 “未安装 @vscode/ripgrep” 与 “包本身错误/环境错误”。
  - [建议]：在 tryImport 中仅对错误码为 `ERR_MODULE_NOT_FOUND` 的导入失败进行降级返回 null；其它错误应向上抛出或至少记录为致命，从入口修正依赖与环境，而非在下游兜底。
  - [片段]：
    try {
      return await import(moduleName);
    } catch (err) {
      return null;
    }

- [文件]：codex-cli/bin/codex.js
  - [定位]：函数 forwardSignal（第 120–129 行）
  - [类别]：[异常吞噬]
  - [不变量]：信号转发失败应是异常路径（如进程不存在/无权限），需要精确处理或记录；不可一概忽略。
  - [首次坏点]：`child.kill(signal)` 的异常被空 catch 吞掉，可能隐藏真实的 IPC/权限问题。
  - [建议]：仅针对已知的可忽略错误（如 `ESRCH` 进程不存在）进行特化处理；其它错误应向上抛出或至少打印到 stderr 并以非零退出，避免在消费处额外加分支。
  - [片段]：
    try {
      child.kill(signal);
    } catch {
      /* ignore */
    }

- [文件]：codex-cli/scripts/run_in_container.sh —— 无明显问题

- [文件]：codex-cli/scripts/install_native_deps.sh
  - [定位]：模块顶层常量 WORKFLOW_URL（第 23 行）与参数解析（第 25–33 行）
  - [类别]：[默认值掩盖] / [状态扩散]
  - [不变量]：安装的二进制版本应与待发布版本严格一致，且来源可追溯；不应依赖过期/硬编码的工作流 URL。
  - [首次坏点]：设置了硬编码的 `WORKFLOW_URL` 作为兜底，若调用方未显式传入，将下载固定工作流产物，可能与当前发布版本不一致而污染构件。
  - [建议]：取消硬编码兜底，要求显式传入 `--workflow-url`（或由上游通过版本解析出对应工作流）；在入口校验 URL 与版本匹配关系，避免在下游靠 if/默认值兜底。
  - [片段]：
    WORKFLOW_URL="https://github.com/openai/codex/actions/runs/16840150768"

- [文件]：codex-cli/scripts/build_container.sh —— 无明显问题

- [文件]：codex-cli/scripts/stage_rust_release.py —— 无明显问题

- [文件]：codex-cli/scripts/stage_release.sh
  - [定位]：复制 README（第 97–99 行）
  - [类别]：[异常吞噬] / [默认值掩盖]
  - [不变量]：发布包的文档文件应完整；缺失 README 属于构件不完整，属于需失败的入口条件。
  - [首次坏点]：`cp ../README.md "$TMPDIR" || true` 将错误吞掉，使构建“看似成功”但产物不完整。
  - [建议]：在入口显式检查 README 是否存在，缺失即失败；或将文档改为明确的可选项并在包内元数据中声明，而非静默忽略。
  - [片段]：
    cp ../README.md "$TMPDIR" || true

- [文件]：codex-cli/scripts/stage_release.sh
  - [定位]：版本与工作流参数（第 33–55、103–109 行）
  - [类别]：[状态扩散] / [默认值掩盖]
  - [不变量]：NPM 包版本与内置原生二进制版本需保持一致；发布时必须提供可追溯的工作流 URL。
  - [首次坏点]：默认 `VERSION` 使用时间戳生成，而 `WORKFLOW_URL` 可以为空并继续调用下游安装脚本（其又有硬编码兜底）。这会让“包版本”与“二进制来源”解耦，首次破坏一致性约束。
  - [建议]：要求显式提供 `--version` 与 `--workflow-url`（或从 `--version` 推导工作流），在入口交叉校验两者对应关系，避免依赖下游脚本的兜底默认值。
  - [片段]：
    VERSION="$(printf '0.1.%d' "$(date +%y%m%d%H%M)")"
    ./scripts/install_native_deps.sh --workflow-url "$WORKFLOW_URL" "$TMPDIR"

- [文件]：codex-cli/scripts/init_firewall.sh
  - [定位]：允许域名读取的兜底（第 7–17 行）
  - [类别]：[默认值掩盖] / [入口优先缺失]
  - [不变量]：允许域名清单应由上游明确提供；若缺失，应作为致命错误而非静默回落到默认域名。
  - [首次坏点]：当配置文件不存在时直接回退为 `api.openai.com` 并继续执行，掩盖了上游未成功写入配置的错误。
  - [建议]：去除静默回落；在入口要求配置文件存在且非空，否则失败退出。若确需默认值，应通过显式开关启用并在日志中标注。
  - [片段]：
    if [ -f "$ALLOWED_DOMAINS_FILE" ]; then ... else
      ALLOWED_DOMAINS=("api.openai.com")
    fi

- [文件]：codex-cli/scripts/init_firewall.sh
  - [定位]：连通性校验（第 101–115 行）
  - [类别]：[状态扩散] / [复杂分支]
  - [不变量]：验证逻辑应针对“允许清单”进行一致性校验，而非硬编码特定域名。
  - [首次坏点]：验证阶段硬编码校验 `api.openai.com` 的可达性，与入口读取到的 `ALLOWED_DOMAINS` 可能不一致，导致错误通过或误报。
  - [建议]：将校验逻辑改为：对 `ALLOWED_DOMAINS` 中的每个域名逐一验证可达性；对非允许域名抽样验证不可达，避免硬编码域。
  - [片段]：
    if ! curl --connect-timeout 5 https://api.openai.com >/dev/null 2>&1; then
      echo "ERROR: Firewall verification failed - unable to reach https://api.openai.com"

- [文件]：scripts/publish_to_npm.py —— 无明显问题

## 批次 2（会话自动）

- [文件]：codex-rs/core/src/apply_patch.rs
  - [定位]：函数 apply_patch（第 71–99 行，重点第 81 行）
  - [类别]：[默认值掩盖] / [异常吞噬]
  - [不变量]：当进入“需要用户审批”的分支时，必须获得一次明确的用户决策；通道取消/错误与“用户拒绝”是不同语义，不能被默认值掩盖。
  - [首次坏点]：`rx_approve.await.unwrap_or_default()` 将 oneshot 取消等错误静默降级为 `ReviewDecision::Denied`，使“系统错误/中断”被误解为“用户拒绝”。
  - [建议]：在 `apply_patch` 的审批等待处专门区分 `Ok(decision)` 与 `Err(Canceled)`；对后者发出错误事件并中止该次事务，或将其规范化为明确的 `Abort`，避免下游再加 if 兜底。
  - [片段]：
    match rx_approve.await.unwrap_or_default() {
      ReviewDecision::Approved | ReviewDecision::ApprovedForSession => { ... }
    }

- [文件] codex-rs/core/src/bash.rs —— 无明显问题

- [文件]：codex-rs/core/src/chat_completions.rs
  - [定位]：
    - 构造 messages：`LocalShellCall` 分支（第 91–96 行）
    - SSE 处理：`finish_reason == "tool_calls"`（第 404–409 行）与 JSON 解析（第 306–309 行）
  - [类别]：[默认值掩盖] / [异常吞噬]
  - [不变量]：
    - 工具调用必须具备稳定的标识（id、name、call_id 不可为空）。
    - 上游 SSE 分片必须是可解析 JSON；供应方协议错误不应被静默忽略。
  - [首次坏点]：在构造 Chat Completions messages 时对 `id` 采用 `unwrap_or_else(|| "".to_string())`，将缺失的 id 回落为空串，后续难以区分、追踪该调用。
  - [建议]：
    - 在构造与转发工具调用时，若关键字段缺失应视为协议错误：记录告警并跳过该调用或中止，而非填充空字符串。
    - SSE JSON 解析失败应至少计数/告警（或在阈值后终止），避免整流为“看似正常”的空输出。
  - [片段]：
    "id": id.clone().unwrap_or_else(|| "".to_string()),
    ...
    name: fn_call_state.name.clone().unwrap_or_else(|| "".to_string()),
    call_id: fn_call_state.call_id.clone().unwrap_or_else(String::new),

- [文件] codex-rs/core/src/client_common.rs —— 无明显问题

- [文件]：codex-rs/core/src/client.rs
  - [定位]：
    - SSE 适配：`response.output_item.added` → WebSearchCallBegin（第 618–633 行）
    - HTTP 错误体回传（第 309–311 行）
  - [类别]：[默认值掩盖] / [异常吞噬]
  - [不变量]：
    - 上游事件中的 `id` 应为稳定标识；缺失时应显式报错而非置空。
    - 当返回 `UnexpectedStatus` 时，错误信息应反映真实的 I/O/解码失败，而非被空字符串掩盖。
  - [首次坏点]：`call_id = item.get("id").and_then(...).unwrap_or("")` 将缺失的 id 置空并继续发出 `WebSearchCallBegin`，造成状态扩散且后续难以定位该次调用。
  - [建议]：
    - 对缺失/非法 `id` 的事件，记录并丢弃该事件或以明确错误上报；不要构造空 id 的“成功事件”。
    - 在读取错误响应体失败时，返回“读取错误体失败”的特化信息，避免 `unwrap_or_default()` 变成空体掩盖真实原因。
  - [片段]：
    .and_then(|v| v.as_str()).unwrap_or("")

- [文件]：codex-rs/core/src/codex_conversation.rs —— 无明显问题

- [文件]：codex-rs/core/src/codex.rs
  - [定位]：
    - 执行命令审批：`exec` 安全检查 AskUser 分支（第 3850–3874 行，重点第 3860 行）

## 批次 4（会话自动）

- [文件]：codex-rs/core/src/openai_model_info.rs
  - [定位]：fn get_model_info 86-91
  - [类别]：[默认值掩盖] / [未知分支当成功]
  - [不变量]：仅已知模型应有可信的上下文/输出上限元数据；未知型号不得默认赋予“看似合理”的额度。
  - [首次坏点]：对任意以“codex-”开头的 slug 统一赋固定窗口与上限，导致未知型号被当作成功路径并携带错误额度。
  - [建议]：在模型来源处建立白名单或从权威源拉取精确配置；对未知 slug 返回 None 并由上游显式处理或要求配置文件覆盖。
  - [片段]：
    _ if slug.starts_with("codex-") => Some(ModelInfo {
      context_window: 400_000,
      max_output_tokens: 128_000,
    }),

- [文件]：codex-rs/core/src/openai_tools.rs
  - [定位]：fn sanitize_json_schema 446-472（重点 469-471）
  - [类别]：[兜底] / [默认值掩盖]
  - [不变量]：工具入参 JSON‑Schema 必须具备明确类型；无法推断时应作为错误回报，而非静默转为 string。
  - [首次坏点]：未能推断 type 时使用 unwrap_or_else 默认 "string"，掩盖上游 Schema 缺陷，可能放宽校验导致后续序列化/调用歧义。
# 杂乱扫描报告
  - [建议]：在 mcp_tool_to_openai_tool 源头严格校验并返回结构化错误；必要时通过显式“宽松模式”开关才允许降级推断，且记录告警。
  - [片段]：
    // If we still couldn't infer, default to string
    let ty = ty.unwrap_or_else(|| "string".to_string());
    map.insert("type".to_string(), JsonValue::String(ty.to_string()));

- [文件]：codex-rs/core/src/parse_command.rs
  - [定位]：fn shlex_join 62-65
  - [类别]：[默认值掩盖]
  - [不变量]：命令应能被可靠转义/拼接；包含非法 NUL 时为输入错误，应显式标注失败或回传可判定状态。
  - [首次坏点]：unwrap_or_else 返回占位文本 "<command included NUL byte>"，把失败伪装成“成功有结果的字符串”，易在下游被当正常命令展示/处理。
  - [建议]：返回 Result 或在上层带错误标记；展示层再决定如何提示用户。
  - [片段]：
    shlex_try_join(...)
      .unwrap_or_else(|_| "<command included NUL byte>".to_string())

  - [定位]：fn normalize_tokens 1290-1294
  - [类别]：[兜底] / [状态扩散]
  - [不变量]：`bash -c/-lc` 脚本无法解析时，不应把原脚本当作“已分词”的安全等价物扩散到后续流程。
  - [首次坏点]：shlex_split(script) 失败后退化为 vec!["bash", flag, script]，让未解析的原始脚本以“正常 tokens”姿态继续传播。
  - [建议]：改为携带解析失败状态（Result/enum），上层决定降级显示或放弃语义解析，避免后续基于错误 tokens 的判定。
  - [片段]：
    shlex_split(script)
      .unwrap_or_else(|| vec!["bash".to_string(), flag.clone(), script.clone()])

- [文件]：codex-rs/core/src/plan_tool.rs
  - [定位]：fn parse_update_plan_arguments 99-111（重点 105-107）
  - [类别]：[异常吞噬] / [默认值掩盖]
  - [不变量]：函数调用参数解析失败应作为“明确失败”对外暴露，避免成功/失败三态不清。
  - [首次坏点]：Err 分支返回 FunctionCallOutput，`success: None` 而非 `Some(false)`，上游难以区分失败与未标注状态。
  - [建议]：将失败路径设置为 `success: Some(false)`，并提供结构化错误码；或在 handle_update_plan 层统一失败信号。
  - [片段]：
    output: FunctionCallOutputPayload {
      content: format!("failed to parse function arguments: {e}"),
      success: None,
    }

- [文件]：codex-rs/core/src/project_doc.rs
  - [定位]：fn get_user_instructions 29-43（重点 38-41）
  - [类别]：[异常吞噬] / [默认值掩盖]
  - [不变量]：当配置允许且存在候选文档时，读取失败应可感知；单纯回退到原始 instructions（或 None）会掩盖真实故障。
  - [首次坏点]：读取项目文档 Err 时仅记录日志并返回 `config.user_instructions.clone()`，调用者无法根据返回值区分“无文档”与“读取失败”。
  - [建议]：返回错误或在返回值中携带来源标记（例如 enum：None | FromConfig | FromDocs | Error），由调用端决定 UI 呈现和重试策略。
  - [片段]：
    Err(e) => {
      error!("error trying to find project doc: {e:#}");
      config.user_instructions.clone()
    }

- [文件]：codex-rs/core/src/rollout.rs
  - [定位]：impl JsonlWriter::write_line 361-367（重点 364）
  - [类别]：[异常吞噬]
  - [不变量]：日志落盘必须保证写入错误可见；否则会产生“以为已持久化”的假象。
  - [首次坏点]：`let _ = self.file.write_all(...).await;` 丢弃写入错误，仅依赖随后的 flush 报错，可能导致部分行丢失而无任何错误上抛。
  - [建议]：直接 `?` 传播 write_all 的错误，必要时对 flush 也进行单独错误处理并回传。
  - [片段]：
    let _ = self.file.write_all(json.as_bytes()).await;
    self.file.flush().await?;

  - [定位]：fn resume 175-206（重点 179-182）
  - [类别]：[异常吞噬]
  - [不变量]：回放文件格式错误应可观测；静默跳过会破坏可审计性。
  - [首次坏点]：每行 JSON 解析失败时 `Err(_) => continue`，无日志且无法统计丢弃条目。
  - [建议]：至少 warn 记录行号与内容摘要；视需求决定是否整体失败或部分跳过。
  - [片段]：
    let v: Value = match serde_json::from_str(line) {
      Ok(v) => v,
      Err(_) => continue,
    };

- [文件]：codex-rs/core/src/safety.rs —— 无明显问题

- [文件]：codex-rs/core/src/seatbelt.rs
  - [定位]：fn create_seatbelt_command_args 59-66（canonicalize 回退）
  - [类别]：[兜底] / [默认值掩盖]
  - [不变量]：安全策略应基于真实规范化路径；规范化失败应显式失败或降级有明确提示。
  - [首次坏点]：`canonicalize().unwrap_or_else(|_| wr.root.clone())` 在路径异常时静默回退为未规范化路径，可能导致策略与实际生效范围不一致。
  - [建议]：在生成策略前就校验各可写根路径是否可 canonicalize；失败时返回错误或记录告警并从策略中剔除该根，避免误放权。
  - [片段]：
    let canonical_root = wr.root.canonicalize()
      .unwrap_or_else(|_| wr.root.clone());

- [文件]：codex-rs/core/src/shell.rs —— 无明显问题

- [文件]：codex-rs/core/src/spawn.rs —— 无明显问题
    - 执行失败后重试审批（第 4030–4047 行，重点第 4041 行）
    - 会话选择回退：`Op::ConvSend`（第 2079–2083 行）
  - [类别]：[默认值掩盖] / [状态扩散]
  - [不变量]：
    - 审批流中“用户决定”必须真实来源于用户；系统通道错误不得伪装为“拒绝”。
    - 发送目标会话必须明确且有效；无效 id 不应静默回退至 root 会话。
  - [首次坏点]：`rx_approve.await.unwrap_or_default()` 在命令审批路径将通道取消视为 `Denied`，破坏“系统错误与用户决策分离”的不变量。
  - [建议]：
    - 两处审批等待均改为显式区分 `Ok`/`Err(Canceled)`，后者转化为明确的错误/中止事件，由入口层处理；禁止把它当作“用户拒绝”。
    - `ConvSend` 的会话解析失败时直接返回错误事件并拒绝继续，而非 `unwrap_or_else(|| root_conversation_id())` 回退。
  - [片段]：
    match rx_approve.await.unwrap_or_default() { /* ... */ }
    ...
    .or_else(pick_recent).unwrap_or_else(|| sess.root_conversation_id());

- [文件] codex-rs/core/src/config_profile.rs —— 无明显问题

- [文件] codex-rs/core/src/config.rs —— 无明显问题

- [文件] codex-rs/core/src/config_types.rs —— 无明显问题
## 批次 5（会话自动）

- [文件] codex-rs/core/src/model_family.rs —— 无明显问题

- [文件] codex-rs/core/src/model_provider_info.rs
  - [定位]：`request_max_retries` 与 `stream_max_retries`（241-246, 248-253）
  - [类别]：[默认值掩盖]
  - [不变量]：用户可配置的重试次数应满足区间约束（0..=MAX），越界应在配置入口被拒绝或显式报错，而非在消费处静默裁剪。
  - [首次坏点]：在 getter 中用 `.min(MAX_*)` 对越界值静默夹断，使“坏值”以“看似合理”的值进入系统，掩盖配置错误来源。
  - [建议]：在配置解析/加载阶段（例如从 `~/.codex/config.toml` 反序列化后）进行范围校验并返回带定位信息的错误；getter 仅返回已验证的值，不再做兜底裁剪。
  - [片段]：
    ```rust
    pub fn request_max_retries(&self) -> u64 {
        self.request_max_retries.unwrap_or(DEFAULT_REQUEST_MAX_RETRIES).min(MAX_REQUEST_MAX_RETRIES)
    }
    pub fn stream_max_retries(&self) -> u64 {
        self.stream_max_retries.unwrap_or(DEFAULT_STREAM_MAX_RETRIES).min(MAX_STREAM_MAX_RETRIES)
    }
    ```

- [文件] codex-rs/core/src/message_history.rs
  - [定位]：`history_metadata`（158-162, 169-172, 178-184）
  - [类别]：[异常吞噬][默认值掩盖]
  - [不变量]：历史文件的“标识+条目数”必须反映真实状态；I/O 异常需与“空历史/不存在”明确区分，不能把任意错误当作计数为 0。
  - [首次坏点]：`Err(_) => return (0, 0)` 把除 `NotFound` 之外的所有错误都映射为“空”，使上游无法分辨读取失败与确实无历史。
  - [建议]：将返回类型改为 `Result<(u64, usize)>` 并仅对 `NotFound` 走 `(0,0)` 分支；或至少记录错误并让调用方获知失败状态，从“入口”处纠正环境/权限问题，而非在消费点兜底。
  - [片段]：
    ```rust
    let meta = match fs::metadata(&path).await {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (0, 0),
        Err(_) => return (0, 0),
    };
    ```

- [文件] codex-rs/core/src/storage_policy.rs —— 无明显问题

- [文件] codex-rs/core/src/terminal.rs —— 无明显问题

- [文件] codex-rs/core/src/tool_apply_patch.rs —— 无明显问题

- [文件] codex-rs/core/src/turn_diff_tracker.rs
  - [定位]：`on_patch_begin`（66-74）；`get_file_diff`（268-270）；`blob_bytes`（430-447）
  - [类别]：[默认值掩盖][异常吞噬][状态扩散]
  - [不变量]：基线快照与当前快照应忠实反映磁盘文件的“模式+内容+可读性”。I/O/权限异常不应被当作“空内容/文件缺失”，模式判定失败不应默认为 `Regular`。
  - [首次坏点]：`blob_bytes` 将读取错误 `.ok()` 为 `None`，与“文件不存在”合流；`on_patch_begin` 用 `unwrap_or(FileMode::Regular)` 与 `unwrap_or_default()` 将失败视为“正常空值”，导致后续 diff 把“读取失败”表现为“新增/删除/二进制不同”等看似成功的状态。
  - [建议]：
    - 让 `blob_bytes` 返回 `Result<Vec<u8>>`，并在调用处区分“不存在”（新增/删除）与“读取失败”（携带错误上抛或打断流程）。
    - 对 `file_mode_for_path` 的 `None` 作为错误分支处理，而非默认 `Regular`。
    - 若基线采集失败，记录错误状态并让 `get_unified_diff` 返回 `Err`，避免“坏状态”扩散为“看似一致”的 diff。
  - [片段]：
    ```rust
    let mode_val = mode.unwrap_or(FileMode::Regular);
    let content = blob_bytes(path, &mode_val).unwrap_or_default();
    // ...
    let current_mode = file_mode_for_path(&current_external_path).unwrap_or(FileMode::Regular);
    ```

- [文件] codex-rs/core/src/user_agent.rs —— 无明显问题

- [文件] codex-rs/core/src/user_notification.rs —— 无明显问题

- [文件] codex-rs/core/src/util.rs —— 无明显问题

## 批次 7（会话自动）

- [文件]：codex-rs/mcp-server/src/patch_approval.rs
  - [定位]：函数 on_patch_approval_response（第 119–131 行、134–139 行）
  - [类别]：[兜底] / [默认值掩盖] / [状态扩散]
  - [不变量]：审批“决策”必须仅由用户响应产生；传输/反序列化错误需以“明确错误或带原因的拒绝”呈现，不能与“用户拒绝”混为一谈。
  - [首次坏点]：接收通道错误时将错误直接映射为 `Denied` 并提交，随后反序列化失败也以 `unwrap_or_else` 默认 `Denied`，使非用户决策被当作用户拒绝，来源信息丢失。
  - [建议]：在请求入口区分“错误/超时/通道关闭”与“用户决策”：
    - 扩展/改造提交流程为显式错误事件，或在拒绝决策中附带结构化错误 reason（而非静默 Denied）。
    - 将区分逻辑收敛在 `on_patch_approval_response` 或更上游（发送请求的一侧），避免在下游消费者再加 if 兜底。
  - [片段]：
    let response = serde_json::from_value::<PatchApprovalResponse>(value).unwrap_or_else(|err| {
        error!("failed to deserialize PatchApprovalResponse: {err}");
        PatchApprovalResponse { decision: ReviewDecision::Denied }
    });

- [文件]：codex-rs/mcp-server/src/tool_handlers/mod.rs —— 无明显问题

- [文件]：codex-rs/mcp-server/tests/all.rs —— 无明显问题

- [文件]：codex-rs/mcp-server/tests/common/lib.rs —— 无明显问题

- [文件]：codex-rs/mcp-server/tests/common/mcp_process.rs —— 无明显问题

- [文件]：codex-rs/mcp-server/tests/common/mock_model_server.rs —— 无明显问题

- [文件]：codex-rs/mcp-server/tests/common/responses.rs —— 无明显问题

- [文件]：codex-rs/mcp-server/tests/suite/auth.rs —— 无明显问题

- [文件]：codex-rs/mcp-server/tests/suite/codex_message_processor_flow.rs —— 无明显问题

- [文件]：codex-rs/mcp-server/tests/suite/codex_tool.rs
  - [定位]：函数 patch_approval_triggers_elicitation（第 239 行）
  - [类别]：[状态扩散]
  - [不变量]：JSON-RPC 请求 id 的唯一性与关联性应由发送方分配并在接收时使用真实值进行关联；测试不应“猜测/硬编码”请求 id。
  - [首次坏点]：硬编码 `let elicitation_request_id = RequestId::Integer(0);`，假定请求 id 为 0。此做法削弱了对协议不变量的校验，一旦服务端变更 id 分配策略，测试将产生伪阳性/伪阴性。
  - [建议]：改为使用实际捕获到的 `elicitation_request.id`（与同文件 `shell_command_approval_triggers_elicitation` 中做法保持一致），在源头保证关联正确，避免在下游构造“看似正确”的期望值。
  - [片段]：
    let elicitation_request_id = RequestId::Integer(0);
## 批次 8（会话自动）

- [文件]：codex-rs/linux-sandbox/src/landlock.rs
  - [定位]：
    - 函数 install_filesystem_landlock_rules_on_current_thread（第 64–81 行）
    - 函数 install_network_seccomp_filter_on_current_thread（第 124–126 行，注释与实现不一致）
  - [类别]：[兜底] / [状态扩散]
  - [不变量]：
    - 文件写入应仅限于显式配置的可写根；若内核/能力不足以完全施加策略，应当失败而非“部分放行”。
    - 网络策略的语义应清晰一致；注释与实现必须保持一致，避免误导后续改动。
  - [首次坏点]：
    - 使用 `CompatLevel::BestEffort` 并仅在 `status.ruleset == NotEnforced` 时报错，允许“部分生效”的状态继续运行，弱化了“要么完全施加、要么失败”的策略契约。
    - `socketpair` 规则处注释写着“always deny”，但实际使用与 `socket` 相同的 `unix_only_rule`（允许 AF_UNIX），语义不一致，易造成后续误改。
  - [建议]：
    - 在 install_filesystem_landlock_rules_on_current_thread 源头改为严格兼容级别（如严格/失败优先），或显式校验 `status` 为“完全生效”；若仅“部分生效”，返回明确的 `SandboxErr`，不要在消费端再加 if 兜底。
    - 澄清 `socketpair` 的期望：若确实要拒绝，改为对该 syscall 插入无条件拒绝规则；若允许 AF_UNIX，请更新注释以匹配实现，防止语义漂移。
  - [片段]：
    ruleset = Ruleset::default()
        .set_compatibility(CompatLevel::BestEffort)
        .handle_access(access_rw)?
        .create()?;
    ...
    rules.insert(libc::SYS_socketpair, vec![unix_only_rule]); // always deny (…?)

- [文件]：codex-rs/linux-sandbox/src/lib.rs —— 无明显问题

- [文件]：codex-rs/linux-sandbox/src/linux_run_main.rs —— 无明显问题

- [文件]：codex-rs/linux-sandbox/src/main.rs —— 无明显问题

- [文件]：codex-rs/linux-sandbox/tests/all.rs —— 无明显问题

- [文件]：codex-rs/linux-sandbox/tests/suite/landlock.rs
  - [定位]：函数 assert_network_blocked（第 129–180 行）
  - [类别]：[兜底] / [默认值掩盖]
  - [不变量]：测试应验证“网络被沙箱拒绝”的明确语义；将“命令缺失/其他失败”与“被沙箱拒绝”混为同一成功路径，会掩盖真实问题。
  - [首次坏点]：将 `process_exec_tool_call` 的 `Ok(output)` 与 `Err(SandboxErr::Denied(..))` 合并为三元组后，只断言 `exit_code != 0` 即视为通过；这把“二进制缺失(127) / 其他非沙箱原因失败”也当作“网络已封禁”。
  - [建议]：
    - 区分失败原因：明确匹配 `Err(SandboxErr::Denied(..))` 或检查子进程的错误码/错误类型（如 EPERM/Operation not permitted）；对 127 等“缺失”场景标记为“跳过”而非“通过”。
    - 或选用保证存在的系统工具作为被测目标，避免把“缺失”当成功。
  - [片段]：
    let (exit_code, stdout, stderr) = match result {
        Ok(output) => (output.exit_code, output.stdout.text, output.stderr.text),
        Err(CodexErr::Sandbox(SandboxErr::Denied(exit_code, stdout, stderr))) => (exit_code, stdout, stderr),
        _ => panic!("expected sandbox denied error, got: {result:?}"),
    };

- [文件]：codex-rs/linux-sandbox/tests/suite/mod.rs —— 无明显问题

- [文件]：codex-rs/linux-sandbox/Cargo.toml —— 无明显问题
# 杂乱扫描报告
## 批次 10（会话自动）
- codex-rs/cli/Cargo.toml:
  - 无明显问题
- codex-rs/cli/src/debug_sandbox.rs:
  - 入口不变量破坏：`SandboxType::Landlock` 分支通过 `config.codex_linux_sandbox_exe.expect("codex-linux-sandbox executable not found")` 直接 `panic`。不变量/契约：本函数返回 `anyhow::Result<()>`，应以受控错误传递失败，而非进程崩溃；并且当选择 Landlock 时必须具备有效的 `codex_linux_sandbox_exe`。首次破坏点：上述 `expect(...)`。建议将缺失态作为显式错误上抛，避免下游兜底或进程级异常。
  - 异常专一化不足：`config_overrides.parse_overrides().map_err(anyhow::Error::msg)?` 将原始错误类型抹平为消息字符串，弱化了错误判别与定位。入口位置：CLI 覆盖项解析阶段。建议保留具体错误类型/上下文而非泛化为 `anyhow::Error::msg`。
- codex-rs/cli/src/exit_status.rs:
  - 回退兜底逻辑：Unix 分支在既无 `code()` 又无 `signal()` 时使用 `std::process::exit(1)` 作为默认退出码；Windows 分支同样在无 `code()` 时以 `1` 兜底。被破坏的不变量：进程 `ExitStatus` 在 Unix 上应当具备 `code` 或 `signal` 二者之一；遇到第三种状态应视为异常/不可达。首次出现坏状态的位置：对 `status.code()`/`status.signal()` 分支的最终 `else`。建议用更明确的处理（例如 `unreachable!`/明确错误日志并携带状态细节）以免默认值掩盖潜在系统/平台异常。
- codex-rs/cli/src/lib.rs:
  - 无明显问题
- codex-rs/cli/src/login.rs:
  - 无明显问题（`load_config_or_exit` 在 CLI 辅助函数中以退出码结束流程符合预期，未见吞异常伪成功的情况）
- codex-rs/cli/src/main.rs:
  - 无明显问题
- codex-rs/cli/src/proto.rs:
  - 异常吞噬/伪正常：读取 stdin 时对 `lines.next_line()` 的结果使用通配 `_` 分支将 `Err(e)` 与 `Ok(None)`（EOF）合并处理为 `info!("Submission queue closed")` 后 `break`。入口优先视角下，坏状态首次出现于读取 stdin 返回 `Err(e)`；当前实现将其伪装为正常关闭，掩盖了真实 I/O 错误。建议区分 `Err(e)` 与 EOF，并对前者记录错误并按需上抛/退出，而非走正常关闭路径。
  - 另外：事件序列化失败分支仅 `error!` 后 `continue`，会丢弃该事件并继续流转。虽未伪装成功，但若协议要求事件序列完整性，应在入口侧加约束或将错误显性上抛以避免静默丢帧。
## 批次 11（会话自动）

- [文件]：codex-rs/apply-patch/src/lib.rs
  - [类别]：异常专一化/兜底；潜在兜底插入；无用代码
  - [不变量]：
    - 非 I/O 类错误（语义/一致性错误）不可在入口统一降级为 I/O 错误；错误类型需保真以便上游决策。
    - 基于上下文的变更定位应遵循作者意图，若缺少明确锚点不得盲目兜底到 EOF。
  - [首次坏点]：`apply_hunks` 中对 `apply_hunks_to_files` 的错误处理（约第 430 行附近）：
    - `Err(err)` 分支里仅尝试 `downcast_ref::<std::io::Error>()`，否则用 `std::io::Error::other(err)` 包装并返回 `ApplyPatchError::IoError`。这会把非 I/O 的语义错误（例如 `ComputeReplacements`）统统映射为 I/O 错误，破坏错误专一化与可观测性。
    - 建议：先尝试 `err.downcast::<ApplyPatchError>()` 并原样返回；仅当确为 I/O 时映射为 `IoError`；其它类型维持原始类别或显式区分（例如 Correctness/Parse）。
  - [次要坏点]：同函数顶部 `_existing_paths` 计算分支（含 `metadata(...).map(|m| m.is_file()).unwrap_or(false)`），但该变量未使用。即使使用，这种 `unwrap_or(false)` 也会把 I/O 错误当作“文件不存在/非文件”吞掉，属于“入口兜底”。建议删除无用代码或在真实需求处显式处理 I/O 错误。
  - [次要坏点]：`compute_replacements` 中当 `old_lines.is_empty()` 时无条件在 EOF 插入（忽略 `change_context`，约第 540 行附近）。若作者提供了 `change_context` 但遗漏 `*** End of File` 标记，会产生“看似成功”的错位插入。建议：
    - 仅在 `is_end_of_file == true` 时允许 EOF 插入；否则依据 `change_context` 先定位，找不到应报错而非兜底到 EOF。

- [文件]：codex-rs/apply-patch/src/main.rs —— 无明显问题

- [文件]：codex-rs/apply-patch/src/parser.rs
  - [类别]：回退兜底逻辑（全局放宽）
  - [不变量]：补丁边界与格式的宽松解析应“按需启用”，而非对所有调用方一刀切放宽；入口需与调用场景（模型/工具）一致化。
  - [首次坏点]：`PARSE_IN_STRICT_MODE: bool = false` 常量使所有解析默认进入宽松模式；`check_patch_boundaries_lenient` 会对形如 heredoc 包裹的输入进行剥壳并继续解析，可能在期望严格模式的上下文里接受无效补丁。
  - [建议]：将严格/宽松作为调用参数显式传入，或仅在通过 `bash -lc` heredoc 入口时启用宽松；严谨起见，`last.ends_with("EOF")` 可改为严格等值匹配（或去除尾随空白后等值）。

- [文件]：codex-rs/apply-patch/src/seek_sequence.rs
  - [类别]：复杂分支/多级回退匹配
  - [不变量]：上下文匹配应当保守且唯一；当仅在“归一化（punctuation/空白）”层面才能匹配或出现多候选时，应失败或要求额外锚点，而非静默选择任一位置。
  - [首次坏点]：最终一轮“最宽松”匹配通过 `normalise()` 把多类 Unicode 标点与空白映射为 ASCII/空格后比较（约第 40–90 行）。在存在多个相似上下文时可能误命中，导致变更位置偏移。
  - [建议]：
    - 在归一化层匹配时，限制搜索窗口（例如基于上一个命中位置的近邻）并在出现多解时报错；
    - 或仅在 `chunk.is_end_of_file == true` 等特殊情形启用归一化匹配，避免影响一般情形。

- [文件]：codex-rs/apply-patch/src/standalone_executable.rs
  - [类别]：默认值掩盖错误
  - [不变量]：进程以 0 退出应当保证标准输出已成功刷新；刷写失败不能被视为成功。
  - [首次坏点]：`run_main` 成功路径里 `let _ = stdout.flush();` 忽略错误后仍返回 0。若管道断开/写入失败，用户将看到“成功退出”而丢失输出。
  - [建议]：检查 `flush()` 结果；失败则向 stderr 报错并返回非零退出码。

- [文件]：codex-rs/apply-patch/tests/all.rs —— 无明显问题

- [文件]：codex-rs/apply-patch/tests/suite/cli.rs —— 无明显问题

- [文件]：codex-rs/apply-patch/tests/suite/mod.rs —— 无明显问题

- [文件]：codex-rs/apply-patch/Cargo.toml —— 无明显问题
## 批次 12（会话自动）

- [文件]：codex-rs/ansi-escape/src/lib.rs
  - [定位]：
    - 函数 ansi_escape_line（第 6–19 行，rg:"ansi_escape_line"）
    - 函数 ansi_escape（第 21–39 行，rg:"pub fn ansi_escape(")
  - [类别]：[回退兜底逻辑] / [不变量破坏]
  - [不变量]：当调用方“期望单行”时，输入应满足“正好一行”；多行或零行均应在入口处被显式拒绝或返回错误，而非在消费点以裁剪或空值继续。
  - [首次坏点]：`ansi_escape_line` 在检测到多行时仅 `warn!` 并返回首行，默默丢弃其余内容；当解析结果为零行时返回空行。这两处均属于在消费点兜底，掩盖了上游对单行约束的违反。
  - [建议]：
    - 入口优先：将 `ansi_escape_line` 的契约改为 `Result<Line, Error>` 或引入 `SingleLine` 新类型，调用前保证/验证为单行；或在更上游约束输入来源，避免在此处截断。
    - 不增分支优先：去除“返回首行/空行”的降级路径，改为在首个发现点返回明确错误，避免错误扩散到后续渲染。
    - 异常专一化：`ansi_escape` 目前对 `NomError` 与 `Utf8Error` 分支化处理是好的，但使用 `panic!()` 无消息体，降低可观测性；可考虑返回 `Result<Text, ParseError>` 由调用方按场景处理，或在 `panic!` 中包含上下文信息（保持专一化，不要 catch-all 后伪装成功）。
  - [片段]：
    match text.lines.as_slice() {
        [] => Line::from("")
      , [only] => only.clone()
      , [first, rest @ ..] => { warn!(...); first.clone() }
    }

- [文件]：codex-rs/ansi-escape/Cargo.toml —— 无明显问题
## 批次 14（会话自动）

- [文件]：codex-rs/arg0/Cargo.toml —— 无明显问题

- [文件]：codex-rs/arg0/src/lib.rs
  - [定位]：
    - `arg0_dispatch_or_else` 子命令分派与 `apply_patch` 执行（第 56–74 行，rg:"CODEX_APPLY_PATCH_ARG1"）
    - `load_dotenv` 与 `set_filtered`（第 113–137 行，rg:"load_dotenv|set_filtered"）
    - `prepend_path_entry_for_apply_patch` 警告并继续（第 83–91 行，rg:"could not update PATH"）
  - [类别]：[异常吞噬/默认值掩盖错误] / [回退兜底逻辑]
  - [不变量]：
    - 子命令执行失败应返回可观测、可诊断的错误上下文，而非仅整数码；
    - dotenv 解析失败属于配置错误，应在首次出现时被显式暴露，而非静默跳过；
    - PATH 注入用于保证 `apply_patch` 可用，若失败应在入口层面明确告知调用方影响范围。
  - [首次坏点]：
    - 第 63–66 行：`codex_apply_patch::apply_patch(..)` 的 `Err(_) => 1` 将具体错误丢弃，仅以 `exit_code=1` 退出，掩盖真实失败原因；
    - 第 130 行：`iter.into_iter().flatten()` 展平并忽略逐项 dotenv 解析错误，属于行级异常吞噬；
    - 第 114–122 行：`dotenvy::from_path_iter/.dotenv_iter` 失败被整体忽略，无告警；
    - 第 83–91 行：PATH 更新失败仅 `eprintln!` 警告后继续运行，形成兜底路径，若后续依赖 PATH 的功能失败，根因被后移。
  - [建议]：入口优先地在上述三个入口点返回/记录专一化错误（包含文件名/行号/无效键名等），避免在消费点补救；对 PATH 失败，明确标注“功能退化范围”。

- [文件]：codex-rs/execpolicy/src/arg_matcher.rs —— 无明显问题

- [文件]：codex-rs/execpolicy/src/arg_resolver.rs —— 无明显问题

- [文件]：codex-rs/execpolicy/src/arg_type.rs —— 无明显问题

- [文件]：codex-rs/execpolicy/src/execv_checker.rs
  - [定位]：
    - `ensure_absolute_path` 错误映射（第 110–115 行，rg:"CannotCanonicalizePath"）
    - `is_executable_file` Windows 分支（第 131–135 行，rg:"PATHEXT"）
  - [类别]：[异常吞噬/信息丢失] / [回退兜底逻辑]
  - [不变量]：路径规范化失败应保留尽可能完整的错误上下文；Windows 可执行性应与环境约定（如 `PATHEXT`）一致，而非以“只要是文件”兜底。
  - [首次坏点]：
    - 第 112–115 行：将 `absolutize` 的错误降维为 `error.kind()`，丢失具体路径/链路细节；
    - 第 131–135 行：Windows 分支以 `metadata.is_file()` 作为可执行判定，并以 TODO 占位，属于回退逻辑，可能允许不可执行的目标通过筛选。
  - [建议]：专一化错误包含源错误 Display/路径上下文；Windows 分支按 `PATHEXT` 或显式白名单校验，避免“文件即可执行”的降级。

- [文件]：codex-rs/execpolicy/src/opt.rs —— 无明显问题

- [文件]：codex-rs/execpolicy/src/policy_parser.rs
  - [定位]：
    - `policy_builtins` 内多处 `unwrap()`（第 172–178、187–193、203–209 行，rg:"unwrap\(\)"）
    - `parse()` 将构建错误统一映射为 `Starlark ErrorKind::Other`（第 69–71 行）
  - [类别]：[异常专一化] / [信息降级]
  - [不变量]：嵌入式执行期错误应以可预期的专一化通道返回，不应依赖 panic 路径；错误类型不应统一折叠为 `Other` 以免丢失可诊断性。
  - [首次坏点]：
    - 第 172–178 等：对 `eval.extra` 的 `unwrap()` 链在环境断言被破坏时直接 panic，无法由上层捕获并转化；
    - 第 69–71 行：将 `Policy::new` 的编译错误包进 `Other`，弱化上游对错误类型的分支处理能力。
  - [建议]：改为明确错误分支返回（保留具体枚举/信息），避免 `unwrap()`；错误转换时保留原始类型或映射到细粒度分类。

- [文件]：codex-rs/execpolicy/src/policy.rs
  - [定位]：
    - `check` 中多规格匹配失败时的 `last_err` 回退（第 72–85 行，rg:"last_err"）
  - [类别]：[复杂分支] / [信息降级]
  - [不变量]：当存在多个候选 `ProgramSpec` 时，失败信息应帮助定位“最早/最关键的破约束”，而非简单以“最后一次错误”覆盖。
  - [首次坏点]：第 79–81 行在循环中每次用最后一次错误覆盖 `last_err`，丢失此前更具诊断价值的上下文（如第一个破坏的不变量）。
  - [建议]：保留和返回“最佳错误”（例如首个失败或带最高优先级的失败），或聚合失败摘要，避免信息被尾部覆盖。

- [文件]：codex-rs/execpolicy/src/valid_exec.rs —— 无明显问题
## 批次 15（会话自动）

- [文件]：codex-rs/exec/src/cli.rs —— 无明显问题

- [文件]：codex-rs/exec/src/event_processor.rs
  - [定位]：
    - `handle_last_message` 将缺失消息写入空内容（第 20–29 行，rg:"handle_last_message|unwrap_or_default"）
    - `write_last_message_file` 仅打印写入失败，调用方无法感知（第 31–37 行，rg:"write_last_message_file|std::fs::write"）
  - [类别]：[回退兜底逻辑] / [异常吞噬/默认值掩盖错误]
  - [不变量]：
    - 当指定“输出最后消息文件”时，应当存在有效的 `last_agent_message`；缺失应作为入口错误暴露，而非写入空串；
    - 文件写入失败应能在调用栈向上传递或专一化上报，避免仅在 stderr 打印而让上游继续“看似成功”。
  - [首次坏点]：
    - 第 21 行：`unwrap_or_default()` 将 `None` 回退为空字符串，首次引入“坏值”；
    - 第 31–36 行：写入失败仅 `eprintln!`，未返回错误信号，上游拿不到失败状态。
  - [建议]：在入口专一化处理 `None`（显式错误或带原因的空结果），写文件错误返回给调用者或记录为结构化事件而非单纯日志。

- [文件]：codex-rs/exec/src/event_processor_with_human_output.rs
  - [定位]：
    - `McpToolCallEnd` 仅在 `Ok(result)` 时打印详细结果，`Err` 分支信息丢失（第 335–364 行，rg:"McpToolCallEnd|if let Ok\(res\)")
    - `AgentReasoningRawContent` / `AgentReasoningRawContentDelta` 状态切换不一致（第 227–252 行，rg:"raw_reasoning_started"）
    - `escape_command` 失败后以空格拼接回退（第 551–553 行，rg:"escape_command|unwrap_or_else"）
    - `ExecCommandEnd` 找不到 `call_id` 时使用占位标签（第 289–306 行，rg:"exec\('\{call_id\}'\)")
  - [类别]：[异常吞噬/信息丢失] / [复杂分支] / [回退兜底逻辑]
  - [不变量]：
    - 工具调用失败时应能观察到明确失败原因（错误值/诊断），而非只给出“failed”标题；
    - 原始推理输出的“开始/结束”应由同一触发条件维护单一真值不变量（正在流式输出 ⇔ `raw_reasoning_started == true`），避免跨事件分支产生不一致；
    - Shell 命令展示应准确反映参数边界，回退为空格拼接会丢失必要的转义与可读性。
  - [首次坏点]：
    - 第 355–363 行：`if let Ok(res) = result` 分支忽略 `Err` 细节，导致错误信息在入口处被丢弃；
    - 第 231–238、246–251 行：`RawContent` 分支打印但不置位，`RawContentDelta` 才置位，下一次 `RawContent` 又清位，状态机易错；
    - 第 551–553 行：`try_join(..).unwrap_or_else(..)` 将转义失败回退为简单拼接。
  - [建议]：在 `McpToolCallEnd` 的 `Err` 情况下输出结构化错误；统一由 delta 开始置位、完成事件重置，或在完整事件内显式设定状态；对命令展示失败给出带原因的占位而非静默降级。

- [文件]：codex-rs/exec/src/event_processor_with_json_output.rs
  - [定位]：
    - 其他事件序列化打印使用 `if let Ok(..)`，失败静默丢弃（第 55–60 行，rg:"serde_json::to_string\(&event\)")
    - `TaskComplete` 通过 `handle_last_message` 将 `None` 回退为空（联动 `event_processor.rs` 的入口问题）（第 48–52 行）
  - [类别]：[异常吞噬/默认值掩盖错误]
  - [不变量]：JSON 模式应保证“事件可完整回放/消费”，序列化失败不应静默。
  - [首次坏点]：第 56–58 行：序列化错误被忽略，丢失事件；`None`→空串的坏值注入源自被调用的 `handle_last_message`。
  - [建议]：记录序列化失败为专一化错误事件或返回上游；对 `last_agent_message=None` 走显式缺省语义，而非写空串。

- [文件]：codex-rs/exec/src/lib.rs
  - [定位]：
    - `cwd: cwd.map(|p| p.canonicalize().unwrap_or(p))` 将规范化失败静默回退（第 145 行，rg:"canonicalize|unwrap_or")
    - Ctrl-C 中断时 `conversation.submit(Op::Interrupt).await.ok()` 吞掉错误（第 210–217 行，rg:"Op::Interrupt|ok\(\)")
    - 发送首批图片时使用 `while let Ok(event) = next_event()`，错误直接终止循环无告警（第 251–262 行，rg:"while let Ok\(event\) ="）
  - [类别]：[回退兜底逻辑] / [异常吞噬]
  - [不变量]：
    - 工作目录应当是有效、可规范化的路径；失败应被明确处理，以免后续路径逻辑基于非规范路径运行；
    - 中断信号必须可靠传达或显式上报失败；
    - 在“等待首个任务完成”的握手过程中，流终止与错误应区分对待并可观测。
  - [首次坏点]：
    - 第 145 行：`unwrap_or(p)` 引入静默降级的非规范路径；
    - 第 213 行：`ok()` 直接丢弃 `submit` 失败；
    - 第 251 行：`while let Ok(..)` 抑制 `Err` 的可见性，导致“失败但无输出”。
  - [建议]：入口处对 `cwd` 失败给出专一化错误；中断提交失败应记录（或升级）到上层；等待图片完成的循环区分 `Err` 与非目标事件并记录错误上下文。

- [文件]：codex-rs/exec/src/main.rs —— 无明显问题

- [文件]：codex-rs/exec/Cargo.toml —— 无明显问题

- [文件]：codex-rs/exec/tests/all.rs —— 无明显问题

- [文件]：codex-rs/exec/tests/suite/apply_patch.rs —— 无明显问题

- [文件]：codex-rs/exec/tests/suite/sandbox.rs —— 无明显问题
## 批次 16（会话自动）

- codex-rs/file-search/src/cli.rs
  - 无明显问题。

- codex-rs/file-search/src/lib.rs
  - 回退/异常吞噬：`get_file_path` 对 `ignore::DirEntry` 的错误分支直接 `return None`（约 L200–L203），以及对 `strip_prefix` 的错误也返回 `None`（L208–L211）。这会在遍历阶段吞掉 I/O/权限/解析类错误，消费端无法分辨“无匹配”与“读取失败”。
    - 首次坏状态：匹配 `entry_result` 的 `Err(_)` 被无条件吞掉（L201–L203）。
    - 被破坏不变量/契约：遍历阶段的错误应被显式记录或上抛，而不是当作“无条目”。
    - 建议：限定性处理错误（例如仅忽略特定类型，如“不存在/权限不足”），其余通过 `Result` 上抛或计数/告警；至少在 `Reporter` 中输出告警。
  - 默认值掩盖错误：当 `cancel_flag` 被设置时，`run` 直接以空结果 `Ok(FileSearchResults { matches: vec![], total_match_count: 0 })` 返回（L214–L220）。这将把“被取消”伪装成“无匹配”。
    - 首次坏状态：`cancel_flag.load(..)` 为 `true` 时的早退分支（L215）。
    - 被破坏不变量/契约：结果的语义应能区分“成功完成且无匹配”与“中途取消”。
    - 建议：返回专门的取消错误（如自定义错误枚举）或在结果中带上 `cancelled: bool` 字段；上游据此决定提示文案。
  - 回退兜底逻辑：`run_main` 在未提供 `pattern` 时通过外部命令列目录（Unix: `ls -al`；Windows: `cmd /c <dir>`，约 L70–L93）。该行为依赖外部可执行的存在且 Windows 分支命令可疑（通常应为 `cmd /c dir`），并与“文件搜索”主职责耦合。
    - 首次坏状态：`pattern` 为 `None` 分支进入外部命令路径（L72–L93）。
    - 被破坏不变量/契约：入口参数为空时的行为应明确且自洽（要么报错提示输入，要么以受控方式列目录），不应依赖环境命令并静默改变程序模式。
    - 建议：用 Rust 标准库列目录并明确提示；或直接返回参数错误，让调用者提供 `pattern`。
  - 脆弱不变量依赖：`walker.run(|| { .. index_counter.fetch_add(..) .. best_matchers_per_worker[index] .. })` 依赖外部库“回调被调用次数 = 线程数 + 1”的假设来建立 `index` 与预分配 `BestMatchesList` 的一一对应（L167–L175、L338–L355）。若上游库实现变动，可能越界并触发未定义行为（通过 `UnsafeCell` 的 `unsafe` 访问）。
    - 首次坏状态：`index` 无边界检查即作为下标使用（L171–L173）。
    - 被破坏不变量/契约：`index < best_matchers_per_worker.len()` 必须成立，但当前未强制。
    - 建议：使用 `min(index, len-1)` 或基于线程本地存储/通道传递上下文，消解对外部实现细节的强依赖；或添加断言并在越界时上抛明确错误。

- codex-rs/file-search/src/main.rs
  - 潜在异常崩溃：多处 `unwrap()`/`expect()`（如 JSON 序列化 `serde_json::to_string(..).unwrap()`，以及 `indices.expect("--compute-indices was specified")`）。若序列化失败或上游未按约定填充 `indices`，将直接 `panic`，不利于用户态可恢复错误处理。
    - 首次坏状态：`report_match` JSON 分支中的 `unwrap()`；以及高亮分支对 `indices` 的 `expect()`。
    - 被破坏不变量/契约：输出阶段应以 I/O/序列化错误为“可预期失败”进行专一化处理，而非进程级崩溃。
    - 建议：改为 `to_string().map_err(..)` 或 `to_writer` 并传播/记录错误；对 `indices` 做显式校验并在缺失时退回到非高亮输出，同时记录告警，而非 `panic`。

- codex-rs/file-search/Cargo.toml
  - 无明显问题。

## 批次 17（会话自动）
- codex-rs/login/src/auth_manager.rs
  - 初始加载异常被吞噬：`new()` 调用 `CodexAuth::from_codex_home(..)` 后直接 `.ok().flatten()`，任何 I/O/解析错误（例如畸形 `auth.json`）都会被转化为 `None`，下游仅看到“未登录”。
    - 首次坏状态：`from_codex_home` 返回 `Err(..)` 时（构造阶段）。
    - 被破坏不变量/契约：当 `auth.json` 存在但无效时，应上抛或记录明确错误，而不是退化为“无认证”。
  - 读写锁错误被视作“无状态”：`auth()` 用 `read().ok()` 返回 `None`；`preferred_auth_method()` 在 `read()` 失败时 `unwrap_or(AuthMode::ApiKey)`；`reload()` 在 `write()` 失败时直接返回 `false`。
    - 首次坏状态：`RwLock` 读/写失败（Poison）被静默兜底。
    - 被破坏不变量/契约：缓存不可读/不可写应视为错误路径，并与“未登录/未变更”区分。

- codex-rs/login/src/lib.rs
  - 默认值掩盖错误：`get_token()` 在 `AuthMode::ApiKey` 下使用 `unwrap_or_default()` 返回空字符串，掩盖“API Key 缺失”的坏值，调用方难以分辨成功与配置错误。
    - 首次坏状态：`self.api_key` 为 `None` 或空且 `mode == ApiKey`。
    - 被破坏不变量/契约：`ApiKey` 模式下应保证密钥非空，缺失应返回特定错误而非空字符串。
  - 回退插入默认结构：`update_tokens()` 对 `auth_dot_json.tokens` 使用 `get_or_insert_with(TokenData::default)`，当磁盘上缺失 `tokens` 时直接构造默认值（含空 `access_token`/`refresh_token`）。
    - 首次坏状态：磁盘 `auth.json` 无 `tokens` 字段仍被当作“可刷新”结构处理。
    - 被破坏不变量/契约：令牌结构是否存在应反映真实登录状态；默认化会掩盖“未登录/文件不完整”。

- codex-rs/login/src/pkce.rs
  - 无明显问题。

- codex-rs/login/src/server.rs
  - 异常吞噬导致“看似成功”：回调处理中 `obtain_api_key(..).await.ok()` 直接丢弃错误，随后仍持久化并跳转成功页；若 API Key 交换失败，整体流程仍给出“成功”信号。
    - 首次坏状态：API Key 交换请求失败被 `.ok()` 吞噬。
    - 被破坏不变量/契约：成功页与持久化应建立在关键步骤（令牌交换、API Key 获取）全部成功之上。
  - 读失败回退为默认：`persist_tokens_async()` 使用 `read_or_default()`，任何 `try_read_auth_json` 的错误（含格式错误）都会返回空结构，随后写回覆盖，可能导致静默数据丢失/掩盖损坏。
    - 首次坏状态：读取 `auth.json` 失败被 `Err(_)` 全量兜底。
    - 被破坏不变量/契约：已有凭据/配置损坏应被显式暴露，避免以空结构覆盖原文件。
  - 静默降级的派生默认：`jwt_auth_claims()` 在 JWT 解析失败时仅打印 `eprintln!` 并返回空 `Map`，`compose_success_url()` 继续以空字符串/`false` 组装成功 URL。
    - 首次坏状态：无效 JWT 导致 claims 解析失败被吞噬。
    - 被破坏不变量/契约：成功页依赖的必需字段应校验并在缺失时失败，而非以占位默认继续。
  - 其它：`webbrowser::open(..)` 的返回值被忽略，若浏览器无法打开，用户侧无提示；可考虑记录但非关键。

- codex-rs/login/src/token_data.rs
  - 策略型回退：`PlanType::Unknown/None` 被视为“应使用 API Key”（`should_use_api_key`）。这是策略选择，但在计划类型不可判定时强制回退可能误导模式选择。
    - 首次坏状态：`chatgpt_plan_type` 缺失/未知即判定为需 API Key。
    - 被破坏不变量/契约：当来源信息不全时应偏向显式确认/错误，而非强行选择模式（视产品策略酌情）。

- codex-rs/login/tests/all.rs
  - 无明显问题（测试聚合入口）。

- codex-rs/login/tests/suite/login_server_e2e.rs
  - 无明显问题（测试中允许 `unwrap`/`expect`，并带有网络沙箱跳过逻辑）。

- codex-rs/login/tests/suite/mod.rs
  - 无明显问题。

- codex-rs/login/Cargo.toml
  - 无明显问题。
## 批次 19（会话自动）
- codex-rs/mcp-types/src/lib.rs
  - 入口兜底：`JSONRPCRequest`/`JSONRPCNotification`/`JSONRPCError` 的 `jsonrpc` 字段标注 `#[serde(default = "default_jsonrpc")]`，当上游缺失该必填字段时会被静默填充为 `"2.0"`。这违反 JSON‑RPC 的不变量（必须显式提供且为 "2.0"），易掩盖输入缺陷。首个破坏点在反序列化入口（该文件定义的结构体标签）。
  - 不变量弱化：多处结构体将应为常量判别的 `type` 字段建模为 `String`（伴随注释 // &'static str = "…"，如 `AudioContent`/`ImageContent`/`EmbeddedResource`/`PromptReference`/`ResourceLink`/`ResourceTemplateReference` 等）。这允许形成非法状态（错误的 `type` 字符串）并把校验推迟到下游消费点。首个破坏点是类型定义阶段（字段类型选择为 `String`）。
  - 可读性/错误分类：`TryFrom<JSONRPCRequest> for ClientRequest` 与 `TryFrom<JSONRPCNotification> for ServerNotification` 对未知 `method` 构造 `serde_json::Error::io(std::io::ErrorKind::InvalidData)`，将协议数据错误伪装为 IO 错误，不利于上层按类别处理。虽非吞噬，但会误导异常分类。
  - 轻微兜底：上述 `TryFrom` 在提取 `params` 时使用 `unwrap_or(serde_json::Value::Null)`。对必填参数会在随后的反序列化中报错，但以 Null 兜底会模糊“缺失 vs. 显式为 null”的差异，入口语义不够严格。
  - 潜在 panic：多处 `impl From<...> for serde_json::Value` 使用 `serde_json::to_value(...).unwrap()`。当前类型均 `Serialize`，理论上不会失败，但一旦未来类型演化导致序列化失败，将在转换处直接 panic，属于“失败即崩”的硬错误路径。
- codex-rs/mcp-types/tests/all.rs
  - 无明显问题。
- codex-rs/mcp-types/tests/suite/initialize.rs
  - 无明显问题（覆盖了 `initialize` 正常路径的解析与 `TryFrom` 分发，没有额外兜底或复杂分支）。
- codex-rs/mcp-types/tests/suite/progress_notification.rs
  - 无明显问题（验证了进度通知解析，同样未引入兜底/吞噬）。
- codex-rs/mcp-types/tests/suite/mod.rs
  - 无明显问题。
- codex-rs/mcp-types/schema/2025-03-26/schema.json
  - 无明显问题（多处通过 `const` 明确 `type` 判别值，契约清晰）。
- codex-rs/mcp-types/schema/2025-06-18/schema.json
  - 无明显问题（同样使用 `const` 约束判别值并细化属性）。
- codex-rs/mcp-types/generate_mcp_types.py
  - 不变量被弱化（生成阶段根因）：当属性在 schema 中带 `const`（如判别 `type`），`map_type` 会返回 `&'static str = "…"`，但 `define_struct(...).fields.append(..., supports_const=False)` 导致最终落地为 `pub <field>: String // &'static str = ...`。这在类型定义源头放弃了编译期约束，是前述 lib.rs 中“`type` 为 String”的根因，属于入口处破坏不变量。
  - 回退兜底：`infer_result_type` 对缺失专用 `…Result` 的请求回退为通用 `Result = serde_json::Value`。这弱化了类型契约，会把“schema 未定义/错误”沉默为“任意 JSON”，建议在生成期显式报错而非回退。
  - 继续执行而非失败：`add_trait_impl` 对“意外字段”仅 `print("Warning: ...")` 后继续生成，可能在 schema 漂移时产出不一致代码，属于吞噬异常信号（未中止生成）。
  - 语义兜底：生成的 `TryFrom` 分支对 `params` 使用 `unwrap_or(Null)`，如前所述会掩盖“缺失 vs null”的差异，入口不够严格；此外对未知 `method` 统一映射为 `serde_json::Error::io(InvalidData)`，错误分类不专一。
- codex-rs/mcp-types/Cargo.toml
  - 无明显问题。
## 批次 20（会话自动）

- codex-rs/ollama/src/client.rs
  - 入口处兜底：`reqwest::Client::builder().build().unwrap_or_else(|_| reqwest::Client::new())` 在构造失败时静默回退到默认客户端。被破坏的不变量/契约：HTTP 客户端应反映配置（超时/代理/TLS 等），构造失败应尽早失败而非回退到“看似可用”的默认。首个破坏点：`try_from_provider` 内客户端构建处。
  - 异常吞噬与同质化：`probe_server` 将任何发送错误统一映射为同一条 `OLLAMA_CONNECTION_ERROR`，并丢弃底层 `reqwest` 错误类型与细节，仅记录 warn 日志。被破坏的不变量：错误应可用于定位具体原因（DNS、连接被拒绝、证书等），而非被同质化。首个破坏点：`self.client.get(url).send().await.map_err(...)`。
  - 回退为“成功但空结果”：`fetch_models` 对非 2xx 响应直接 `Ok(Vec::new())` 返回，且 JSON 结构异常时使用 `unwrap_or_default()` 返回空列表。被破坏的不变量：空列表应仅代表“服务正常但确实没有模型”，而非“请求失败/数据异常”。首个破坏点：`if !resp.status().is_success() { return Ok(Vec::new()); }` 与 `names.unwrap_or_default()`。
  - 复杂分支/逻辑失效：`uses_openai_compat` 计算为 `is_openai_compatible_base_url(base_url) || matches!(provider.wire_api, WireApi::Chat) && is_openai_compatible_base_url(base_url)`，按运算优先级实际等价于仅判断 `is_openai_compatible_base_url(base_url)`，`wire_api` 判断被短路，导致提供者声明的协议不生效。被破坏的不变量：兼容模式应由配置与 URL 一致地决定。首个破坏点：`try_from_provider` 中 `uses_openai_compat` 的赋值。
  - 异常吞噬：`pull_model_stream` 内字节流错误分支 `Err(_) => { return; }` 直接结束流，不推送错误事件也不返回错误，导致上游只能得到“意外结束”。被破坏的不变量：网络/流错误应以明确错误传播。首个破坏点：`while let Some(chunk) = stream.next().await { match chunk { ... Err(_) => { return; } } }`。
  - 静默忽略坏数据：逐行解析时 UTF-8 解码失败或 JSON 解析失败会被跳过而不记录/上报，可能丢失服务端的错误内容。首个破坏点：`if let Ok(text) = std::str::from_utf8(&line) { ... if let Ok(value) = serde_json::from_str::<JsonValue>(text) { ... } }`。
  - 无效/死代码信号：`let _pending: VecDeque<PullEvent> = VecDeque::new();` 未使用，提示历史兜底或未完成的逻辑残留。

- codex-rs/ollama/src/lib.rs
  - 文档-实现契约不一致：注释称“仅当选择默认 OSS 模型或未提供 -m 时才下载”，实现却对当前配置的模型一律检查并缺失即拉取。被破坏的不变量：行为应与对外契约一致，避免误导使用者。首个破坏点：`ensure_oss_ready` 注释与其内部逻辑不一致。
  - 异常吞噬/默认值掩盖错误：`fetch_models` 出错分支仅 `warn`，仍返回 `Ok(())`，让上层继续执行，可能导致调用方误以为“环境已就绪”。被破坏的不变量：函数成功返回应意味着“本地环境就绪或可明确拉取”，而非“探测失败但继续”。首个破坏点：`match ollama_client.fetch_models().await { Err(err) => { tracing::warn!(...); } }`。

- codex-rs/ollama/src/parser.rs
  - 默认值掩盖坏数据：`digest` 缺失时 `unwrap_or("")`，仍会在仅有 `total/completed` 时生成 `ChunkProgress{ digest: "" }`。被破坏的不变量：进度事件应可唯一归属到分层/摘要；空摘要使聚合/展示逻辑退化。首个破坏点：`let digest = ... .unwrap_or("")`。
  - 容错策略可能掩盖结构错误：仅要 `total` 或 `completed` 存在就发出进度事件，若另一字段缺失将以 `None` 继续传播；若这是协议层不变量（两者应配对出现），应在入口校验而非下游兜底。（如当前协议允许单边字段则可忽略。）

- codex-rs/ollama/src/pull.rs
  - 速率计算兜底：`dt.max(0.001)` 防止除零，是合理防御，但属于回退兜底逻辑。若期望严格度量，应在时间基准异常时标注状态而非硬编码下限。（轻微，影响有限。）
  - 错误事件处理：`PullEvent::Error(_)` 分支选择不打印以避免重复，由调用方处理；该设计清晰，不属异常吞噬。

- codex-rs/ollama/src/url.rs
  - 无明显问题。

- codex-rs/ollama/Cargo.toml
  - 无明显问题。
## 批次 21（会话自动）
- codex-rs/protocol/src/config_types.rs
  - 无明显问题。

- codex-rs/protocol/src/lib.rs
  - 无明显问题。

- codex-rs/protocol/src/mcp_protocol.rs
  - 无明显问题。

- codex-rs/protocol/src/message_history.rs
  - 无明显问题。

- codex-rs/protocol/src/models.rs
  - 成功标志与负载不一致：`ResponseInputItem::McpToolCallOutput` 转换为 `ResponseItem::FunctionCallOutput` 时，`success` 取自原始 `Result::is_ok()`，而内容序列化失败被 `unwrap_or_else(|e| format!("JSON serialization error: {e}"))` 吞噬，仍返回 `success=true` 搭配错误字符串。
    - 首次坏状态：在 `impl From<ResponseInputItem> for ResponseItem` 内构造 `FunctionCallOutputPayload` 时，`serde_json::to_string(&result)` 失败被转为字符串且 `success` 未随之变为失败。
    - 被破坏不变量/契约：`success=true` 应保证 `content` 为 `CallToolResult` 的合法 JSON 序列化结果；否则会让下游把“失败”当“成功”。
  - 本地图片读取失败被静默丢弃：将 `InputItem::LocalImage` 转换时，`std::fs::read(path)` 失败仅 `tracing::warn!` 然后丢弃该项（返回 `None` 过滤）。
    - 首次坏状态：文件读取返回 `Err` 即被吞噬，仅留日志，无结构化错误返回。
    - 被破坏不变量/契约：输入中的每个 `LocalImage` 要么转为 `InputImage`，要么以明确的错误项/错误结果返回，避免“看似成功但缺少内容”。
  - 字段省略策略不一致可能造成协议歧义：`Reasoning` 项的 `content` 字段使用 `#[serde(default, skip_serializing_if = "should_serialize_reasoning_content")]`，当 `content=None` 时函数返回 `false` 导致序列化为 `null`，而 `Some` 且不含 `ReasoningText` 时则被省略。
    - 首次坏状态：`should_serialize_reasoning_content` 对 `None` 与“空/无关内容”的处理不一致，导致线上的字段形状不稳定。
    - 被破坏不变量/契约：字段缺省与显式 `null` 应有一致的语义约定，否则消费端难以进行稳定判定。
  - 扩展性 catch-all：`ResponseItem::Other` 捕获未知类型，若消费端未显式处理该分支，可能造成未知响应被静默忽略。
    - 首次坏状态：反序列化阶段即进入 `Other` 分支。
    - 被破坏不变量/契约：未知类型应至少被记录/上报，避免“看似成功”的继续处理。

- codex-rs/protocol/src/parse_command.rs
  - 无明显问题（取决于上游解析如何构造 `Unknown`/`Noop`；需确保消费端不会把它们当“成功命令”继续执行）。

- codex-rs/protocol/src/plan_tool.rs
  - 无明显问题。

- codex-rs/protocol/src/protocol.rs
  - FromStr 非直觉契约：`impl FromStr for SandboxPolicy` 通过 `serde_json::from_str(s)` 解析，实质要求 JSON 形状（如 `{ "mode": "read-only" }`），而非常见的“人类可读枚举字符串”。
    - 首次坏状态：调用方若传入 `"read-only"` 等简单字符串将直接失败，易诱发调用处增补兜底分支。
    - 被破坏不变量/契约：`FromStr` 通常与 `Display` 保持可逆的人类可读语义；这里的契约偏离预期。
  - 结构不变量与实现偏差：`WritableRoot` 注释声明 `root/read_only_subpaths` “绝对路径（by construction）”，但 `get_writable_roots_with_cwd(cwd)` 直接 `roots.push(cwd.to_path_buf())` 且不做 `canonicalize/absolutize`，当 `cwd` 为相对路径时将在创建处破坏不变量。
    - 首次坏状态：`get_writable_roots_with_cwd` 接收到相对 `cwd`。
    - 被破坏不变量/契约：后续 `is_path_writable` 以 `starts_with` 判定，基于未规范化路径可能产生歧义或被旁路。
  - 设计取舍提示：`has_full_disk_read_access()` 恒为 `true`，若上游误以为可收紧读权限，可能转而在消费点补 if 实现“伪限制”。应在类型/文档层强调“读不可收紧”，避免下游兜底。

- codex-rs/protocol/Cargo.toml
  - 无明显问题。
## 批次 24（会话自动）

### codex-rs/tui/src/bottom_pane/file_search_popup.rs
- 无明显问题

### codex-rs/tui/src/bottom_pane/list_selection_view.rs
- 无明显问题

### codex-rs/tui/src/bottom_pane/mod.rs
- 无明显问题

### codex-rs/tui/src/bottom_pane/popup_consts.rs
- 无明显问题

### codex-rs/tui/src/bottom_pane/scroll_state.rs
- 无明显问题

### codex-rs/tui/src/bottom_pane/selection_popup_common.rs
- 无明显问题

### codex-rs/tui/src/bottom_pane/status_indicator_view.rs
- 无明显问题

### codex-rs/tui/src/bottom_pane/textarea.rs
- 入口优先/不变量：`cursor_pos_with_state` 将可视行索引从 `usize` 转为 `u16` 时使用 `try_into().unwrap_or(0)` 回退为 0。首个坏点在该转换与回退；这会在超大行数（或异常情况下）把屏幕行静默重置到 0，破坏“不变量：光标屏幕坐标应与实际可视行一致”，并用默认值掩盖潜在溢出/越界问题。建议在更上游限制/收敛类型与范围（例如在换行与滚动计算中即使用 `u16`/显式 `saturating_*` 与 `min` 约束），而非在消费点以默认值兜底。

### codex-rs/tui/src/chatwidget/agent.rs
- 入口优先/不变量：`spawn_agent` 在后台任务内调用 `server.new_conversation(config).await`，一旦 `Err(e)` 仅记录日志后 `return`。但外围函数已提前返回了 `UnboundedSender<Op>`。首个坏点是“返回 sender 的时机”早于“确保会话与转发循环就绪”。这破坏了“不变量：返回给调用方的 sender 一定连接到存活的接收端并有转发循环消费”。随后 UI 继续 `send` 看似成功，但接收端已被丢弃（后台任务提前退出，`codex_op_rx` 被 drop），导致操作无人处理，且上游若未检查 `send` 结果，会被静默吞没。
- 异常专一化：事件转发循环使用 `while let Ok(event) = conversation.next_event().await`。一旦底层流出错（`Err`），循环静默退出，未向 UI 发出“会话终止/错误”类事件，造成错误被吞。应专门上报该类错误并在上游建立明确状态机，而非依赖静默结束。
- 不增分支优先：建议将“是否返回 sender”的决策上移为在会话成功创建后再返回，或将函数返回类型改为 `Result<UnboundedSender<Op>, E>`；或在失败路径上向 UI 发送明确的失败事件并标记 agent 未就绪。不要在下游增加额外 if 兜底。

### codex-rs/tui/src/chatwidget/interrupts.rs
- 无明显问题
## 批次 25（会话自动）

### codex-rs/tui/src/chatwidget.rs
- 入口优先/不变量：`handle_exec_end_now` 在 `running_commands.remove(&ev.call_id)` 为 `None` 时，使用 `vec![ev.call_id.clone()]` 与空 `parsed` 兜底，继续生成历史单元。首个坏状态是“收到 ExecEnd 之前未记录对应的 ExecBegin（或 call_id 不一致/事件乱序）”。被破坏不变量：每个 `ExecCommandEnd` 事件必须有唯一匹配的 `ExecCommandBegin`（同一 `call_id`）并已登记。当前做法用“看似合理的命令文本”掩盖了上游时序/一致性问题。
- 异常专一化/默认值掩盖：`handle_mcp_end_now` 计算成功标志时使用 `ev.result.as_ref().map(|r| !r.is_error.unwrap_or(false)).unwrap_or(false)`。当 `result` 存在但 `is_error` 缺失时被当作成功（`true`）。这用默认值掩盖了工具结果状态不明确的错误，破坏“结果必须显式声明是否错误”的契约。应在上游保证字段完整或在此处显式标记为“状态不明”的失败路径。
- 信号语义混淆：`on_interrupted_turn` 通过 `finalize_turn_with_error_message("Tell the model what to do differently")` 写入“错误事件”。用户主动中断在语义上并非错误，该做法会在历史中产生误导性“错误”记录，且吞没了真实中断原因/上下文。建议用专门的“已中断”状态，并保留队列恢复逻辑。
- 发送失败反馈不足：`submit_user_message` 在 `codex_op_tx.send(...)` 失败时仅 `tracing::error!`，UI 不呈现失败状态，调用方也无法获知需重试，形成“看似成功”的假象。首次坏点：`send` 失败的返回值被仅日志化。

### codex-rs/tui/src/chatwidget_stream_tests.rs
- 无明显问题

### codex-rs/tui/src/chatwidget/tests.rs
- 无明显问题（存在仅用于回放旧日志的兼容性回填 `upgrade_event_payload_for_tests`，属测试辅助，不影响业务路径）

### codex-rs/tui/src/citation_regex.rs
- 无明显问题

### codex-rs/tui/src/clipboard_paste.rs
- 无明显问题

### codex-rs/tui/src/cli.rs
- 无明显问题

### codex-rs/tui/src/common.rs
- 无明显问题

### codex-rs/tui/src/custom_terminal.rs
- 无明显问题（`Drop::drop` 中失败仅 `eprintln!`，属可接受的资源回收降级处理）

### codex-rs/tui/src/diff_render.rs
- 回退兜底逻辑与错误掩盖：
  - `create_diff_summary` 中 `FileChange::Delete` 分支通过 `std::fs::read_to_string(path).ok().map(|s| s.lines().count()).unwrap_or(0)` 估计删除行数。若读文件失败会静默记为 0 行，掩盖“无法读取原文件”的真实错误。首次坏点：读取失败被 `ok()/unwrap_or(0)` 吞没，破坏“不变量：删除统计应基于可验证的原始内容”。
  - `render_patch_details` 的删除分支使用 `read_to_string(path).unwrap_or_default()` 渲染原内容，读失败退化为空内容，导致 UI 展示看似“空改动”。同样属于默认值掩盖错误。
- 容错回退：`count_from_unified` 在 `diffy::Patch::from_str` 失败时手动扫描 `+/-` 统计。此为合理的统计降级，但若配合上面的读文件失败兜底，会进一步弱化错误可见性。建议优先确保输入统一差异（来源数据）可靠，而非在 UI 层多处回退。

### codex-rs/tui/src/exec_command.rs
- 回退兜底逻辑：`escape_command` 在 `shlex::try_join(...)` 失败时 `unwrap_or_else(|_| command.join(" "))`。当参数包含需要转义的空格或特殊字符时，失败回退会输出不安全/不可逆的拼接字符串，破坏“转义后的命令应可被安全复现”的不变量。首次坏点：对 join 失败使用“空格拼接”掩盖错误。应返回错误或以可证明安全的方式降级，而非输出看似可执行的伪命令行。

## 批次 26（会话自动）

### codex-rs/tui/src/file_search.rs
- 异常吞噬/默认值掩盖错误：`file_search::run(...).map(|res| res.matches).unwrap_or_default()` 将任何错误（包括非取消类错误）静默为“空结果”。不变量：搜索失败与“无匹配”需可区分；首次破坏点在 `spawn_file_search` 线程中对 `run()` 结果使用 `unwrap_or_default`。建议专一化处理：明确区分取消（可忽略）与真实失败（应上报/记录）。

### codex-rs/tui/src/get_git_diff.rs
- 异常吞噬：并发收集未跟踪文件 diff 时，`while let Some(res) = join_set.join_next().await` 分支中 `Err(_) => {}` 吞掉任务 Join 失败/崩溃，表现为“看似成功但缺少部分输出”。不变量：子任务失败需可见；首次破坏点在该 `match` 的 `Err(_)` 分支。
- 回退兜底逻辑：`null_device.to_str().unwrap_or("/dev/null")` 在路径非 Unicode 时退化到固定字符串，Windows 场景下会产生错误的空设备路径，掩盖入参问题。入口在构造 `null_path` 时。应保证平台正确性或在失败时显式报错。
- 合理专一化：对 `io::ErrorKind::NotFound` 的特判用于处理文件在列举后被删除的竞态，属于明确类型的异常专一化，保持不变。

### codex-rs/tui/src/history_cell.rs
- 默认值掩盖：`desired_height(...).try_into().unwrap_or(0)` 在转换失败时返回 0 高度，破坏“不变量：渲染高度应与内容一致”，导致 UI 消失。首次破坏点在 `HistoryCell::desired_height`。
- 默认值掩盖（展示层）：`serde_json::to_string(v).unwrap_or_else(|_| v.to_string())` 在序列化失败时退化为 `Value::to_string()`（可能非 JSON 语义），虽属展示层，但会使参数显示“不可信却看似正常”。入口在 `format_mcp_invocation`。
- 其他 `unwrap_or_default()`/`unwrap_or("")` 用于 UI 文本拼接，风险较低，但若作为状态依据应避免。

### codex-rs/tui/src/insert_history.rs
- 异常吞噬：大量 `queue!(...).ok()` 忽略 I/O 结果（如 `SetScrollRegion`、`MoveTo`、`Print` 等），一旦写入失败会静默丢失，终端状态与内部状态出现偏差。入口首个破坏点例如：设置滚动区域 `queue!(writer, SetScrollRegion(...)).ok()`。
- 回退兜底：`terminal.backend().size().unwrap_or(Size::new(0, 0))` 获取失败时使用 0×0，后续区域计算将进入不合法状态，属于“坏状态扩散”。入口在 `insert_history_lines_to_writer` 获取屏幕尺寸处。建议早失败并提示，而非传播 0 尺寸。
- 复杂分支/状态：自定义滚动区与游标移动逻辑较复杂，且在错误被吞时难以推断终端状态。应减少静默分支，保证不变量（滚动区/光标）的一致性。

### codex-rs/tui/src/lib.rs
- 回退兜底：`cli.cwd.clone().map(|p| p.canonicalize().unwrap_or(p))` 规范化失败时直接回退原路径，可能继续在无效/不存在路径上运行。入口在 cwd 规范化。应在入口处显式失败，避免坏路径扩散。
- 异常吞噬：`get_login_status` 读取认证失败时仅 `error!` 日志并返回 `NotAuthenticated`，把“读取损坏/权限错误”与“未配置”混为一谈。入口在 `CodexAuth::from_codex_home` 的 `Err(err)` 分支。建议专一化处理（如区分文件损坏/权限/缺失）。

### codex-rs/tui/src/live_wrap.rs
- 回退兜底：`UnicodeWidthChar::width(ch).unwrap_or(0)` 对未知宽度字符记 0，可能导致文本挤压而非截断。虽为 UI 近似，但属于默认值掩盖。入口在 `take_prefix_by_width`。可考虑对非常见类别记录告警或统一按 1 处理。

### codex-rs/tui/src/main.rs
- 无明显问题

### codex-rs/tui/src/markdown.rs
- 无明显问题

### codex-rs/tui/src/markdown_stream.rs
- 无明显问题

### codex-rs/tui/src/onboarding/auth.rs
- 异常吞噬：`start_chatgpt_login` 内异步任务 `match r { Ok(()) => ..., _ => { ... } }` 将所有非成功结果（含具体错误）合并为回退 `PickMode`，并未向用户呈现失败原因；注释中还有已注释掉的错误展示。入口在 `child.block_until_done().await` 结果分支。应专一化错误并给予用户可见反馈。
## 批次 27（会话自动）

- codex-rs/tui/src/onboarding/mod.rs
  - 无明显问题

- codex-rs/tui/src/onboarding/onboarding_screen.rs
  - 事件绘制错误被静默丢弃：`run_onboarding_app` 的 `TuiEvent::Draw` 分支使用 `let _ = tui.draw(...)`，未记录也未传播错误。入口位置：`run_onboarding_app` 的 `match event` 分支。被破坏不变量：渲染失败应被观察到并记录（或触发降级流程），而非“看似成功”。
  - 布局测量含经验性偏移：`used_rows` 返回 `last_non_empty.map(|v| v + 2)`，带有“+2”魔数。入口位置：`used_rows`。潜在影响：当内容高度与测量不一致时，可能造成截断或空白行溢出，属于下游消费点兜底的隐患。

- codex-rs/tui/src/onboarding/trust_directory.rs
  - 持久化失败后仍将步骤标记完成：`handle_trust` 中对 `set_project_trusted` 的错误仅记录日志并设置 `self.error`，但无论成功与否，都会执行 `self.selection = Some(Trust)`，从而使 `get_step_state` 返回 `Complete`。入口位置：`handle_trust`。被破坏不变量：`selection = Some(Trust)` 应意味着“信任状态已成功持久化”；当前实现使状态与事实脱节，可能导致后续流程基于错误前提继续。
  - 信任目标的回退兜底：`resolve_root_git_project_for_trust(&cwd).unwrap_or_else(|| cwd.clone())` 在解析失败时直接回退到 `cwd`。入口位置：`handle_trust`。被破坏不变量：信任目标应为“明确解析的工程根”；回退可能把非预期目录置为信任，建议在上游明确区分“未检测到工程根”与“解析错误”，并提示用户而非静默回退。

- codex-rs/tui/src/onboarding/welcome.rs
  - 无明显问题

- codex-rs/tui/src/pager_overlay.rs
  - 使用哨兵值打破范围不变量：`End` 键将 `scroll_offset = usize::MAX`，随后依赖渲染路径再行夹取。入口位置：`PagerView::handle_key_event`。被破坏不变量：`scroll_offset` 应始终处于 `[0, max_scroll]`，当前做法对未来读者/新调用点有隐患（若在渲染前被消费）。
  - 百分比显示的兜底可能误导：`render_bottom_bar` 在 `wrapped.is_empty()` 或 `max_scroll == 0` 时显示 `100%`。入口位置：`render_bottom_bar`。被破坏不变量：顶部应更接近 `0%`（或显式 `—`/`N/A`），当前兜底给出“看似完成”的状态。

- codex-rs/tui/src/render/line_utils.rs
  - 无明显问题

- codex-rs/tui/src/render/markdown_utils.rs
  - 无明显问题（逻辑采用简化的围栏匹配与空块剔除，未见异常吞噬或过度兜底）。

- codex-rs/tui/src/render/mod.rs
  - 无明显问题

- codex-rs/tui/src/session_log.rs
  - 目录解析失败后的静默降级：`maybe_init` 对 `codex_core::config::log_dir(config)` 的 `Err(_)` 直接回退到 `temp_dir()`，未记录具体失败原因。入口位置：`maybe_init` 构造日志路径。被破坏不变量：日志目录来源应可追溯与可观测；当前实现掩盖配置错误，导致“看似成功但写在临时目录”。
  - 写入前置条件的静默短路：`write_json_line` 在 `file` 尚未初始化时直接 `return`，属于静默吞噬。入口位置：`write_json_line`。建议在调用侧保证不变量（已启用且已打开），或在此处记录一次性告警以便排查。
  - 互斥锁中毒恢复采用 `into_inner()`：这是合理的降级，但应确保仅用于日志通道，避免迁移至业务路径时沿用而掩盖并发错误。

- codex-rs/tui/src/shimmer.rs
  - 能力检测的保守兜底：`supports_color::on_cached(...).map(|l| l.has_16m).unwrap_or(false)` 在未知时回退为不支持真彩。入口位置：`shimmer_spans`。属于安全的降级策略，无异常吞噬，但会在检测异常时呈现较差效果。
## 批次 34（会话自动）

- scripts/asciicheck.py
  - 首坏点：L108-L123 在执行 --fix 后未重新校验当前内容，返回值仍基于修复前的 `errors`；且输出 “Fixed X of Y” 使用旧错误计数，可能在存在未替换字符时产生“看似成功”的错觉，或在全部修复成功时仍退出非零。
  - 不变量/契约：启用 --fix 后，最终文件应满足“仅 ASCII + 允许码点”，退出码与提示信息应基于修复后的真实状态。
  - 建议（不增分支优先）：修复后重新扫描并以新错误集决定退出码与提示；计数应基于实际被替换的违规位置，而非预扫描的总数；不要在消费点额外兜底。
  - 首坏点：L96-L99 仅放行 0x20–0x7E 的可打印范围，导致 ASCII 控制字符如 CR（\r, 0x0D，常见于 CRLF）被判为非法；与脚本顶部“仅限制非 ASCII”的描述不一致。
  - 不变量/契约：文档与实现一致；若目标是“ASCII 可打印字符”，需在入口清晰声明并在读取阶段规范化换行（如统一 LF），避免在下游通过分支兜底。
  - 备注：UnicodeDecodeError 处理专一（L74-L87），无 catch-all 异常吞噬；列号计算基于字节可能与“字符列”不一致，但不影响主不变量判断。

- scripts/readme_toc.py
  - 首坏点：L60-L66 的 slug 规则仅保留 `[0-9a-z\s-]`，会丢弃非 ASCII 字符，导致与 GitHub 实际锚点算法不一致（中文、重音字符等标题将生成错误链接）。
  - 不变量/契约：ToC 链接应与 GitHub 渲染的锚点一致。
  - 建议（入口优先）：采用 GitHub 同源的 slug 生成规则或复用成熟实现，从源头统一锚点算法，而非在下游增加 if 修补。
  - 首坏点：L79-L86 仅取首个 BEGIN/END 标记，不校验唯一性与顺序（`begin < end`）。若标记重复或顺序颠倒，`lines[begin+1:end]` 将为空或切片异常语义，但流程仍可能继续进入修复路径。
  - 不变量/契约：存在且唯一的一对标记，且 `begin < end`。
  - 建议：在入口断言标记唯一且顺序正确；异常类型专一化并明确报错，不做 catch-all。
  - 备注：代码块检测仅依赖以 ``` 起始的行（L45-L49），对 ~~~ 栅栏或非常规围栏不鲁棒，但未见异常吞噬。

- codex-rs/scripts/create_github_release.sh
  - 首坏点：L56-L61 分支名与 tag 同名（`rust-v$VERSION`）。`git checkout -b "$TAG"`（L57）创建与 tag 同名的本地分支，随后 `git tag -a "$TAG"`（L61）再创建同名 tag，名称冲突易致引用歧义与后续操作混乱。
  - 不变量/契约：引用命名在不同命名空间应避免歧义；发布流程应最小化临时状态。
  - 建议（不增分支优先）：使用独立前缀（如 `release/rust-v$VERSION`）或直接在 `main` 上提交并打 tag，避免同名；减少无必要的临时分支。
  - 首坏点：L58 以正则直接改 `Cargo.toml` 的 `version`，仅匹配行首且不容忍前导空白，且只修改根 `Cargo.toml`。在 workspace 或缩进场景可能未改动任何内容，后续仍尝试提交/打 tag，留下多余的临时分支。
  - 不变量/契约：版本来源唯一且变更已生效；提交/打 tag 前应验证变更确实发生。
  - 建议：在入口使用 TOML 语义化工具（如 `cargo set-version`）或在提交前检查 diff/grep 结果；若未变更应明确失败并回滚临时状态，避免“看似成功”的默认路径继续。
  - 首坏点：未预检 tag 是否已存在；仅在 L61 创建 tag 时失败。
  - 不变量/契约：版本/tag 唯一。
  - 建议：创建前显式检测 `refs/tags/$TAG` 是否存在并给出明确错误。
  - 备注：全局 `set -euo pipefail` 已减少异常吞噬；远端获取失败已专一化处理。

- codex-cli/package.json
  - 无明显问题
## 批次 35（会话自动）

- package.json
  - 无明显问题

- pnpm-workspace.yaml
  - 风险：设置 `ignoredBuiltDependencies: ["esbuild"]` 会跳过安装脚本/本地构建，依赖预编译二进制。遇到非标架构或离线环境时，失败不一定在安装阶段暴露，可能由下游兜底分支承担或延后到运行期才失败。
  - 建议：仅在受控环境使用；在 CI 增加二进制可用性检查，或移除此项以在安装阶段专一化失败。

- pnpm-lock.yaml
  - 风险：`settings.autoInstallPeers: true` 会自动安装 peerDependencies，弱化入口的显式依赖申明与冲突暴露，属于“自动回填”兜底行为，可能导致不兼容在运行期才显现。
  - 建议：在 CI 中设为 `false` 并显式声明所需 peers，或用 `overrides` 固定版本以避免隐式分支。

- cliff.toml
  - 发现：`commit_parsers` 中含 `".*"` 的兜底分组，且 `filter_unconventional = false`，非规范提交不会被阻断，全部归入“Other”。
  - 影响：这是流程层面的回退兜底，降低变更入口的规范约束，可能让错误类型或破坏性变更在记录层被“掩盖”。
  - 建议：在提交检查或 CI 中强制 conventional commits（而非依赖 changelog 侧兜底），或将 `filter_unconventional = true` 并在 pre-commit/CI 阶段专一化失败。

- codex-rs/Cargo.toml
  - 无明显问题
  - 备注：`[workspace.lints.clippy]` 将 `expect_used/unwrap_used` 设为 `deny`，有助于减少兜底式异常处理；`[patch.crates-io]` 指向 git 分支会增加供应链变更路径复杂度，但与异常吞噬无直接关联。

- codex-rs/rust-toolchain.toml
  - 无明显问题

- codex-rs/rustfmt.toml
  - 无明显问题
  - 备注：注释建议忽略 rustfmt 警告属风格工具层面，不构成代码层异常吞噬/兜底。

- codex-rs/rustfmt.stable.toml
  - 无明显问题

- codex-rs/clippy.toml
  - 无明显问题
  - 备注：仅在测试中允许 `unwrap/expect`，符合“异常专一化”。
## 批次 36（会话自动）

- codex-rs/exec/tests/fixtures/sse_apply_patch_add.json
  - 空数组兜底：`response.output` 为空数组，可能被消费端当作“无输出亦成功”的默认路径，掩盖应有结果缺失。
  - 默认值掩盖：`usage.input_tokens_details` 与 `output_tokens_details` 为 `null`，若解析端期待对象，易触发“以空对象兜底”或出现 NPE 风险。
  - 协议枚举：事件 `type` 依赖字符串枚举；需与其他夹具的变体一致处理，避免在不同工具调用类型间产生状态混淆。

- codex-rs/exec/tests/fixtures/sse_apply_patch_freeform_add.json
  - 空数组兜底：完成事件携带 `output: []`，存在“无输出即成功”的隐性默认。
  - 默认值掩盖：`usage.*_details = null` 与上同，可能诱发下游以空对象/默认值兜底。

- codex-rs/exec/tests/fixtures/sse_apply_patch_freeform_update.json
  - 空数组兜底：`output: []` 同前，若业务期望至少一条输出，将掩盖错误。
  - 默认值掩盖：`usage.*_details = null` 同前，解析端需显式区分 `null` 与对象缺失，而非默认回填。

- codex-rs/exec/tests/fixtures/sse_apply_patch_update.json
  - 双层编码：`item.arguments` 为字符串包裹的 JSON，而非对象本体；容易促使消费端容忍两种形态并在失败时以宽松解析兜底，增加分支复杂度。
  - 协议状态差异：`item.type = "function_call"`（而非其他夹具中的 `custom_tool_call`），语义相近但类型不同，若未统一抽象易造成状态混淆或重复分支。
  - 空数组兜底/默认值掩盖：`output: []` 与 `usage.*_details = null` 同前。

- codex-rs/exec/tests/fixtures/sse_response_completed.json
  - 完成但无输出：仅有 `response.completed` 且 `output: []`，若调用方默认视为成功，可能掩盖生成阶段缺失输出的错误。
  - 默认值掩盖：`usage.*_details = null` 同前。

- codex-rs/core/tests/fixtures/completed_template.json
  - 模板同上：`response.completed` 携带空 `output` 与 `null` 细节字段；作为模板示例，会将“无输出亦成功”的模式固化到用例中。

- codex-rs/core/tests/fixtures/incomplete_sse.json
  - 必需字段缺失：仅有 `{ "type": "response.output_item.done" }`，缺少 `item`。若消费端以空对象/默认值兜底，错误会被延后或静默吞没；应在解析期即显式失败。
