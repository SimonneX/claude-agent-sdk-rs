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

# Run tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

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
src/internal/ → Private implementation
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

Enable with `#[cfg(feature = "testing")]` or run tests with `cargo test --features testing`.

## Important Files

- `src/lib.rs`: Public API entry point, re-export list
- `src/types/config.rs`: ClaudeAgentOptions definition
- `src/types/messages.rs`: Message types
- `src/types/mcp.rs`: MCP server and `tool!` macro
- `examples/`: 26 comprehensive examples covering all features
- `examples/MCP_INTEGRATION.md`: MCP tool integration guide