# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust SDK for the Claude Code CLI, providing programmatic access with bidirectional streaming support. The SDK wraps the Claude Code CLI subprocess via stdio, translating JSON messages to Rust types.

## Commands

```bash
# Build
cargo build

# Build release
cargo build --release

# Build all examples
cargo build --examples

# Run tests (lib + integration). NOTE: 3 pre-existing doctest failures
# (redis dep, async-ctx) are unrelated to your changes — verify with --lib.
cargo test --features testing

# Lib tests only — fastest feedback loop, skips flaky doctests
cargo test --features testing --lib

# Run specific test
cargo test --features testing test_name

# Run tests with output
cargo test --features testing -- --nocapture

# Lint
cargo clippy --all-targets --all-features

# Format check
cargo fmt -- --check

# Build documentation
cargo doc --open

# Run examples
cargo run --example 01_hello_world
cargo run --example 06_bidirectional_client
```

## Architecture

### Layered Structure

```
src/lib.rs → Public API re-exports
src/client.rs → ClaudeClient (bidirectional streaming)
src/query.rs → Simple query functions (query, query_stream)
src/internal/ → Private implementation (NOT reachable from tests/;
                  use inline #[cfg(test)] mod tests for these)
  ├── transport/ → SubprocessTransport (stdio communication with Claude CLI)
  ├── message_parser.rs → JSON → Message parsing
  └── query_full.rs → Full query state management
src/types/ → All type definitions
  ├── config.rs → ClaudeAgentOptions (typed-builder pattern)
  ├── messages.rs → Message enum, content blocks
  ├── hooks.rs → Hook system (PreToolUse, PostToolUse, etc.)
  ├── mcp.rs → MCP server configuration, tool! macro
  ├── permissions.rs → Permission modes and callbacks
  ├── plugin.rs → Plugin loading configuration
  └── efficiency.rs → Built-in efficiency hooks
src/testing/ → Mock framework (feature-gated: "testing")
```

### Key Patterns

- **typed-builder**: All config types use `#[derive(TypedBuilder)]` for ergonomic builder pattern
- **Lock-free**: Uses `flume` channels and `DashMap` for concurrent access without deadlocks
- **Transport trait**: Pluggable transport layer allows mock testing

### Two Query Modes

1. **Simple Query** (`query()`, `query_stream()`): One-shot, no session management
2. **Bidirectional Client** (`ClaudeClient`): Multi-turn conversations, dynamic control (interrupt, permission mode changes)

### MCP Tools

Use `tools` parameter to restrict built-in tools, `allowed_tools` to grant permission for MCP tools (format: `mcp__{server}__{tool}`).

### Testing

The `testing` feature enables a mock framework:
- `MockTransport`: Replace subprocess with scripted responses
- `ScenarioBuilder`: Define message sequences
- Message builders: `AssistantMessageBuilder`, `ResultMessageBuilder`, etc.
- `QueryFull::new_with_transport` is gated on `feature = "testing"`

Enable with `#[cfg(feature = "testing")]` or run tests with `cargo test --features testing`.

**Bidirectional control protocol round-trip pattern** (used in
`src/internal/query_full.rs` and `tests/client_control_tests.rs`): spawn the
control method as a task → poll `MockTransport::written_messages_async()`
until the request appears → extract `request_id` from the JSON →
`mock.inject(...)` a `{"type":"control_response","response":{"subtype":
"success","request_id":"...","response":<payload>}}` envelope → await the
task. The payload MUST be nested inside the inner `response` field — that's
the actual CLI wire format (see `tools/capture_control_protocol.py`). The
SDK strips that wrapper once at the channel sender so callers receive the
unwrapped payload (matches Python `_send_control_request`).

## Important Files

- `src/lib.rs`: Public API entry point, re-export list
- `src/types/config.rs`: ClaudeAgentOptions definition
- `src/types/messages.rs`: Message types
- `src/types/mcp.rs`: MCP server and `tool!` macro
- `examples/`: 26 comprehensive examples covering all features
- `examples/MCP_INTEGRATION.md`: MCP tool integration guide