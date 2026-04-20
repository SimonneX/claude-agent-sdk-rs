# Python SDK vs Rust SDK - 最终功能对齐审查

**审查日期**: 2026-04-21  
**Rust SDK 版本**: v0.1.63  
**Python SDK 参考**: main 分支 (https://github.com/anthropics/claude-agent-sdk-python)

## 审查结论

v0.1.63 已实现核心功能对齐（151 测试通过）。以下是相对于 Python SDK 仍然缺失的功能清单。

---

## 1. 错误类型别名 (5项) ⚠️ 命名差异

Python SDK 使用的错误类型名称与 Rust SDK 不同：

| Python SDK | Rust SDK 当前 | 建议 |
|------------|---------------|------|
| `ClaudeSDKError` | `ClaudeError` | 添加类型别名 |
| `CLIConnectionError` | `ConnectionError` | 添加类型别名 |
| `CLINotFoundError` | `CliNotFoundError` | 添加类型别名 |
| `CLIJSONDecodeError` | `JsonDecodeError` | 添加类型别名 |
| `ProcessError` | `ProcessError` | ✅ 已对齐 |

**文件**: `src/errors.rs`

```rust
// 建议添加的类型别名
pub type ClaudeSDKError = ClaudeError;
pub type CLIConnectionError = ConnectionError;
pub type CLINotFoundError = CliNotFoundError;
pub type CLIJSONDecodeError = JsonDecodeError;
```

---

## 2. Session Store 异步变体 (9项) ❌ 缺失

Python SDK 提供了使用自定义 `SessionStore` 的异步函数变体：

| 函数 | 状态 | 说明 |
|------|------|------|
| `list_sessions_from_store` | ❌ 缺失 | 从自定义 store 列出 sessions |
| `get_session_info_from_store` | ❌ 缺失 | 从自定义 store 获取 session 信息 |
| `get_session_messages_from_store` | ❌ 缺失 | 从自定义 store 获取消息 |
| `list_subagents_from_store` | ❌ 缺失 | 从自定义 store 列出子代理 |
| `get_subagent_messages_from_store` | ❌ 缺失 | 从自定义 store 获取子代理消息 |
| `rename_session_via_store` | ❌ 缺失 | 通过 store 重命名 session |
| `tag_session_via_store` | ❌ 缺失 | 通过 store 标记 session |
| `delete_session_via_store` | ❌ 缺失 | 通过 store 删除 session |
| `fork_session_via_store` | ❌ 缺失 | 通过 store 分叉 session |

**文件**: `src/sessions.rs`

**优先级**: 中 - 仅在需要自定义 session 持久化时使用

---

## 3. SessionStore Trait 扩展 (2项) ❌ 缺失

Python `SessionStore` 协议的可选方法在 Rust 中未实现：

| 方法 | 状态 | 说明 |
|------|------|------|
| `delete` | ❌ 缺失 | 删除指定 session 的数据 |
| `list_subkeys` | ❌ 缺失 | 列出指定 session 的子键 |

**文件**: `src/types/session_store.rs`

**优先级**: 低 - 可选方法，非核心功能

---

## 4. CanUseTool 类型别名 (1项) ❌ 缺失

Python SDK 导出 `CanUseTool` 类型别名：

| Python SDK | Rust SDK 当前 | 建议 |
|------------|---------------|------|
| `CanUseTool` | 只有 `CanUseToolCallback` | 添加 `pub type CanUseTool = CanUseToolCallback;` |

**文件**: `src/types/permissions.rs` 或 `src/lib.rs`

---

## 5. BaseHookInput (1项) ❌ 缺失

Python SDK 有 `BaseHookInput` 基类，包含所有 hook input 的公共字段：

```python
class BaseHookInput(TypedDict):
    session_id: str
    transcript_path: str
    cwd: str
    permission_mode: Optional[str]
```

Rust SDK 中没有对应的基类/trait，每个 hook input struct 独立定义这些字段。

**评估**: 低优先级 - Rust 的结构体字段重复是惯用写法，无需基类

---

## 6. Hook 特定输出类型导出方式 ⚠️ 设计差异

Python SDK 单独导出每个 hook specific output 类型：
- `PostToolUseFailureSpecificOutput`
- `NotificationHookSpecificOutput`
- `SubagentStartHookSpecificOutput`
- `PermissionRequestHookSpecificOutput`

Rust SDK 使用 `HookSpecificOutput` 枚举统一包装：
```rust
pub enum HookSpecificOutput {
    PostToolUseFailure(PostToolUseFailureHookSpecificOutput),
    Notification(NotificationHookSpecificOutput),
    SubagentStart(SubagentStartHookSpecificOutput),
    PermissionRequest(PermissionRequestHookSpecificOutput),
    // ...
}
```

**评估**: ✅ 功能等价，Rust 的设计更符合类型安全

---

## 缺失功能统计

| 类别 | 缺失数量 | 优先级 |
|------|----------|--------|
| 错误类型别名 | 4 | 低 |
| CanUseTool 类型别名 | 1 | 低 |
| Session Store 异步变体 | 9 | 中 |
| SessionStore trait 方法 | 2 | 低 |
| BaseHookInput | 1 | 低 |
| **总计** | **17** | - |

---

## 建议实施顺序

### 阶段 1: 类型别名 (简单, 向后兼容)
- 添加错误类型别名
- 添加 `CanUseTool` 类型别名

### 阶段 2: Session Store 扩展 (需要设计)
- 实现 `SessionStore::delete` 和 `list_subkeys`
- 实现 9 个 `*_from_store` / `*_via_store` 函数

### 阶段 3: 可选 (评估是否需要)
- 考虑是否需要 `BaseHookInput` trait

---

## 当前状态

**已对齐 (v0.1.63)**:
- ✅ 10 个 Hook 事件及其 Input/Output 类型
- ✅ 所有消息类型 (Task*, RateLimit, MirrorError)
- ✅ ClaudeClient 5 个新方法
- ✅ Session 管理函数 (8个，基础实现)
- ✅ SessionStore trait 和 InMemorySessionStore
- ✅ AgentDefinition 全部字段
- ✅ ClaudeAgentOptions 全部字段
- ✅ ThinkingConfig 枚举
- ✅ MCP 状态类型
- ✅ ContextUsage 类型
- ✅ Permission 系统完整实现

**测试状态**: 151 测试通过

---

## 附注

Rust SDK 采用的一些设计差异是合理的：

1. **HookSpecificOutput 枚举** - 比单独导出更符合 Rust 类型系统
2. **无 BaseHookInput** - Rust 结构体字段重复是惯用写法
3. **错误类型命名** - Rust 惯例 (ClaudeError vs ClaudeSDKError) 是可以接受的差异

这些差异不会改变功能等价性，主要是命名和代码组织风格的不同。
