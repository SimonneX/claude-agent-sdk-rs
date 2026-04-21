# Claude Agent SDK Examples

This directory contains 26 comprehensive examples demonstrating all features of the Claude Agent SDK for Rust. Each example is fully documented and runnable.

## Quick Start

```bash
# Run any example
cargo run --example <name>

# For example
cargo run --example 01_hello_world
```

## Example Categories

### 📚 Basics (Examples 01-03)
Basic SDK usage and fundamentals.

### 🚀 Advanced (Examples 04-07)
Permissions, hooks, and dynamic control.

### 🛠️ MCP Integration (Example 08)
Custom in-process tools.

### ⚙️ Configuration (Examples 09-13)
Configuration options and customization.

### 🎯 Patterns (Examples 14-16)
Comprehensive patterns for common use cases.

### 💼 Production Features (Examples 17-20)
Production-ready features.

### 🔌 Plugin System (Examples 21-22)
Custom plugin loading and integration.

### 🖼️ Multimodal & Efficiency (Examples 23-24)
Image input and built-in efficiency hooks.

### 🧠 Skills (Example 25)
The `skills` option (`Skills::All` / `Skills::List`), folded into `--allowedTools`.

### 📁 Filesystem Agents (Example 26)
Loading agents from `.claude/agents/*.md` files via `setting_sources(["project"])` (Python SDK `filesystem_agents.py` parity).

### Recent v0.1.63 alignment additions
Other v0.1.63 features are folded into the relevant existing examples:
- `SystemPrompt::File` and `SystemPromptPreset::with_exclude_dynamic_sections` → `13_system_prompt.rs`
- `session_id` startup option (vs `resume`) → `16_session_management.rs`
- New `PermissionMode::DontAsk` / `PermissionMode::Auto` variants → noted in `04_permission_callbacks.rs`

## Learning Path

**Beginner:** 01, 02, 06
**Intermediate:** 04, 05, 08
**Advanced:** 14, 15, 16
**Production:** 17, 18, 21

For full details, see individual example files.
