# Python SDK vs Rust SDK - 缺失功能清单

本文档记录 Rust SDK 相比官方 Python SDK 缺失的功能，供后续实现参考。

## 已完成 ✅

- [x] Hook 事件: PostToolUseFailure, Notification, SubagentStart, PermissionRequest (src/types/hooks.rs)
- [x] Session 管理函数: list_sessions, get_session_info 等 8 个函数 (src/sessions.rs, placeholder)
- [x] 消息类型: TaskStartedMessage, TaskProgressMessage, TaskNotificationMessage, RateLimitEvent (src/types/messages.rs)
- [x] ClaudeClient 方法: get_mcp_status, reconnect_mcp_server, toggle_mcp_server, stop_task, get_context_usage (src/client.rs)
- [x] AgentDefinition 字段扩展: disallowed_tools, skills, memory, mcp_servers, initial_prompt, max_turns, background, effort, permission_mode (src/types/config.rs)
- [x] ClaudeAgentOptions 字段扩展: skills, thinking, effort, task_budget, load_timeout_ms (src/types/config.rs)
- [x] ThinkingConfig 类型: ThinkingConfigAdaptive, ThinkingConfigEnabled, ThinkingConfigDisabled (src/types/config.rs)
- [x] MCP 状态类型: McpServerStatus, McpStatusResponse, McpServerInfo, McpToolInfo, McpToolAnnotations (src/types/mcp.rs)
- [x] ResultMessage 字段扩展: stop_reason, model_usage, permission_denials, errors, uuid (src/types/messages.rs)
- [x] ContextUsage 类型: ContextUsageResponse, ContextUsageCategory (src/types/context.rs)
- [x] 控制协议消息: mcp_status, mcp_reconnect, mcp_toggle, stop_task, context_usage (src/internal/query_full.rs)
- [x] SessionStore trait: SessionStore trait, InMemorySessionStore, 相关类型 (src/types/session_store.rs)

## 待完善 ⏳

### 1. Session 管理函数实现 (src/sessions.rs)

当前是 placeholder 实现，需要添加实际的文件 I/O 来读写 Claude Code session 文件：
- 从 `~/.claude/projects/{project_hash}/sessions/*.jsonl` 读取 session
- 解析 JSONL 格式的 transcript
- 实现完整的 CRUD 操作

### 2. 控制协议响应解析 (src/internal/query_full.rs)

当前方法发送请求并返回解析后的类型，但 CLI 可能还未实现所有控制协议消息的响应。

## 实现状态总结

| 功能 | 状态 | 文件位置 |
|-----|------|---------|
| Hook 事件 (10个) | ✅ 完成 | src/types/hooks.rs |
| Session 管理函数 | ⏳ placeholder | src/sessions.rs |
| 消息类型 (4个新增) | ✅ 完成 | src/types/messages.rs |
| ClaudeClient 方法 (5个) | ✅ 完成 | src/client.rs |
| AgentDefinition 字段 | ✅ 完成 | src/types/config.rs |
| ClaudeAgentOptions 字段 | ✅ 完成 | src/types/config.rs |
| ThinkingConfig 类型 | ✅ 完成 | src/types/config.rs |
| MCP 状态类型 | ✅ 完成 | src/types/mcp.rs |
| ResultMessage 字段 | ✅ 完成 | src/types/messages.rs |
| ContextUsage 类型 | ✅ 完成 | src/types/context.rs |
| 控制协议消息 | ✅ 完成 | src/internal/query_full.rs |
| SessionStore trait | ✅ 完成 | src/types/session_store.rs |

## 测试验证

所有 141 个单元测试通过，clippy 检查仅有少量无害警告。

## 参考资源

- Python SDK: https://github.com/anthropics/claude-agent-sdk-python
- Python SDK types.py: https://raw.githubusercontent.com/anthropics/claude-agent-sdk-python/main/src/claude_agent_sdk/types.py