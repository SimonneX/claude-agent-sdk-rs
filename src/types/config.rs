//! Configuration types for Claude Agent SDK

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use typed_builder::TypedBuilder;

use super::efficiency::EfficiencyConfig;
use super::hooks::{HookEvent, HookMatcher};
use super::mcp::McpServers;
use super::permissions::CanUseToolCallback;
use super::plugin::SdkPluginConfig;

/// Main configuration options for Claude Agent
#[derive(Clone, TypedBuilder)]
#[builder(doc)]
pub struct ClaudeAgentOptions {
    /// Tools configuration (list of tool names or preset)
    #[builder(default, setter(into, strip_option))]
    pub tools: Option<Tools>,
    /// List of allowed tool names
    #[builder(default, setter(into))]
    pub allowed_tools: Vec<String>,
    /// System prompt configuration
    #[builder(default, setter(into, strip_option))]
    pub system_prompt: Option<SystemPrompt>,
    /// MCP server configuration
    #[builder(default)]
    pub mcp_servers: McpServers,
    /// Permission mode
    #[builder(default, setter(strip_option))]
    pub permission_mode: Option<PermissionMode>,
    /// Whether to continue the conversation
    #[builder(default = false)]
    pub continue_conversation: bool,
    /// Session ID to resume
    #[builder(default, setter(into, strip_option))]
    pub resume: Option<String>,
    /// Pre-assigned session id for a NEW session (distinct from `resume`,
    /// which re-opens an existing one). Mirrors Python SDK v0.1.52 `session_id`.
    #[builder(default, setter(into, strip_option))]
    pub session_id: Option<String>,
    /// Maximum number of turns
    #[builder(default, setter(strip_option))]
    pub max_turns: Option<u32>,
    /// List of disallowed tool names
    #[builder(default, setter(into))]
    pub disallowed_tools: Vec<String>,
    /// Model to use
    #[builder(default, setter(into, strip_option))]
    pub model: Option<String>,
    /// Fallback model to use if primary model fails
    #[builder(default, setter(into, strip_option))]
    pub fallback_model: Option<String>,
    /// Beta features to enable
    /// See https://docs.anthropic.com/en/api/beta-headers
    #[builder(default, setter(into))]
    pub betas: Vec<SdkBeta>,
    /// Maximum budget in USD for the conversation
    #[builder(default, setter(strip_option))]
    pub max_budget_usd: Option<f64>,
    /// Maximum tokens for thinking blocks
    #[builder(default, setter(strip_option))]
    pub max_thinking_tokens: Option<u32>,
    /// Tool name for permission prompts
    #[builder(default, setter(into, strip_option))]
    pub permission_prompt_tool_name: Option<String>,
    /// Working directory
    #[builder(default, setter(into, strip_option))]
    pub cwd: Option<PathBuf>,
    /// Path to Claude CLI
    #[builder(default, setter(into, strip_option))]
    pub cli_path: Option<PathBuf>,
    /// Settings file path
    #[builder(default, setter(into, strip_option))]
    pub settings: Option<String>,
    /// Additional directories to include
    #[builder(default, setter(into))]
    pub add_dirs: Vec<PathBuf>,
    /// Environment variables
    #[builder(default)]
    pub env: HashMap<String, String>,
    /// Extra CLI arguments
    #[builder(default)]
    pub extra_args: HashMap<String, Option<String>>,
    /// Maximum buffer size for subprocess output
    #[builder(default, setter(strip_option))]
    pub max_buffer_size: Option<usize>,
    /// Callback for stderr output
    #[builder(default, setter(strip_option))]
    pub stderr_callback: Option<Arc<dyn Fn(String) + Send + Sync>>,
    /// Callback for tool usage permission
    #[builder(default, setter(strip_option))]
    pub can_use_tool: Option<CanUseToolCallback>,
    /// Hook callbacks
    #[builder(default, setter(strip_option))]
    pub hooks: Option<HashMap<HookEvent, Vec<HookMatcher>>>,
    /// User identifier
    #[builder(default, setter(into, strip_option))]
    pub user: Option<String>,
    /// Whether to include partial messages in stream
    #[builder(default = false)]
    pub include_partial_messages: bool,
    /// Whether to fork the session
    #[builder(default = false)]
    pub fork_session: bool,
    /// Custom agent definitions
    #[builder(default, setter(strip_option))]
    pub agents: Option<HashMap<String, AgentDefinition>>,
    /// Setting sources to use.
    ///
    /// When `None`, the SDK does **not** load any filesystem settings,
    /// providing isolation for SDK applications.
    ///
    /// Programmatic options (like `agents`, `allowed_tools`) always override filesystem settings.
    #[builder(default, setter(strip_option))]
    pub setting_sources: Option<Vec<SettingSource>>,
    /// Sandbox configuration for bash command isolation
    /// Filesystem and network restrictions are derived from permission rules (Read/Edit/WebFetch),
    /// not from these sandbox settings.
    #[builder(default, setter(strip_option))]
    pub sandbox: Option<SandboxSettings>,
    /// Plugin configurations for custom plugins
    #[builder(default, setter(into))]
    pub plugins: Vec<SdkPluginConfig>,
    /// Output format for structured outputs (matches Messages API structure)
    /// Example: `json!({"type": "json_schema", "schema": {"type": "object", "properties": {...}}})`
    #[builder(default, setter(strip_option))]
    pub output_format: Option<serde_json::Value>,
    /// Enable file checkpointing to track file changes during the session.
    /// When enabled, files can be rewound to their state at any user message
    /// using `ClaudeClient.rewind_files()`.
    #[builder(default = false)]
    pub enable_file_checkpointing: bool,

    /// Skip CLI version check on connect (default: false).
    /// When true, saves ~100-500ms by skipping the version compatibility check.
    /// Use when you know the CLI version is compatible.
    #[builder(default = false)]
    pub skip_version_check: bool,

    /// Enable verbose CLI output (default: true).
    ///
    /// **Note:** This option is currently ignored because the SDK uses `--output-format=stream-json`
    /// which requires `--verbose` to be enabled. The CLI enforces this constraint.
    #[builder(default = true)]
    pub verbose: bool,

    /// Efficiency configuration for built-in efficiency hooks.
    ///
    /// When configured, the SDK automatically injects hooks to:
    /// - Remind about working directory at prompt submission
    /// - Track execution metrics (edits per file, build attempts, etc.)
    /// - Provide efficiency feedback and warnings when execution stops
    ///
    /// # Example
    ///
    /// ```no_run
    /// use claude_agent_sdk_rs::{ClaudeAgentOptions, EfficiencyConfig};
    ///
    /// let options = ClaudeAgentOptions::builder()
    ///     .efficiency(EfficiencyConfig::enabled())
    ///     .build();
    /// ```
    #[builder(default, setter(strip_option))]
    pub efficiency: Option<EfficiencyConfig>,

    /// Skills to enable. Use `Skills::All` to enable every skill, or
    /// `Skills::List(vec![...])` for a specific subset. `None` (default)
    /// leaves skills untouched. Mirrors Python SDK v0.1.62
    /// `skills: list[str] | Literal["all"] | None`.
    #[builder(default, setter(into, strip_option))]
    pub skills: Option<Skills>,

    /// Thinking configuration (structured, not just max_tokens)
    #[builder(default, setter(strip_option))]
    pub thinking: Option<ThinkingConfig>,

    /// Effort level
    #[builder(default, setter(into, strip_option))]
    pub effort: Option<String>,

    /// Task budget configuration
    #[builder(default, setter(strip_option))]
    pub task_budget: Option<TaskBudget>,

    /// Load timeout in milliseconds
    #[builder(default, setter(strip_option))]
    pub load_timeout_ms: Option<u64>,
}

impl Default for ClaudeAgentOptions {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// System prompt configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    /// Direct text prompt
    Text(String),
    /// Preset-based prompt
    Preset(SystemPromptPreset),
    /// Load system prompt from a file on disk
    File(SystemPromptFile),
}

impl From<String> for SystemPrompt {
    fn from(text: String) -> Self {
        SystemPrompt::Text(text)
    }
}

impl From<&str> for SystemPrompt {
    fn from(text: &str) -> Self {
        SystemPrompt::Text(text.to_string())
    }
}

impl From<SystemPromptPreset> for SystemPrompt {
    fn from(preset: SystemPromptPreset) -> Self {
        SystemPrompt::Preset(preset)
    }
}

impl From<SystemPromptFile> for SystemPrompt {
    fn from(file: SystemPromptFile) -> Self {
        SystemPrompt::File(file)
    }
}

/// System prompt preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPromptPreset {
    /// Type field (always "preset")
    #[serde(rename = "type")]
    pub type_: String,
    /// Preset name (e.g., "claude_code")
    pub preset: String,
    /// Text to append to the preset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub append: Option<String>,
    /// Strip per-user dynamic sections (env, cwd, etc.) injected by the preset.
    /// Mirrors Python SDK v0.1.57 `exclude_dynamic_sections`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_dynamic_sections: Option<bool>,
}

impl SystemPromptPreset {
    /// Create a new preset with the given name
    pub fn new(preset: impl Into<String>) -> Self {
        Self {
            type_: "preset".to_string(),
            preset: preset.into(),
            append: None,
            exclude_dynamic_sections: None,
        }
    }

    /// Create a preset with appended text
    pub fn with_append(preset: impl Into<String>, append: impl Into<String>) -> Self {
        Self {
            type_: "preset".to_string(),
            preset: preset.into(),
            append: Some(append.into()),
            exclude_dynamic_sections: None,
        }
    }

    /// Set the `exclude_dynamic_sections` flag (chainable).
    pub fn with_exclude_dynamic_sections(mut self, exclude: bool) -> Self {
        self.exclude_dynamic_sections = Some(exclude);
        self
    }
}

/// System prompt loaded from a file on disk.
///
/// Mirrors Python SDK v0.1.51 `SystemPromptFile`. Serializes to
/// `{ "type": "file", "path": "..." }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPromptFile {
    /// Type field (always "file")
    #[serde(rename = "type")]
    pub type_: String,
    /// Path to the system prompt file
    pub path: String,
}

impl SystemPromptFile {
    /// Create a new file-based system prompt config.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            type_: "file".to_string(),
            path: path.into(),
        }
    }
}

/// Permission mode for tool execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    /// Default permission mode
    #[serde(rename = "default")]
    Default,
    /// Accept edits automatically
    AcceptEdits,
    /// Plan mode
    #[serde(rename = "plan")]
    Plan,
    /// Bypass all permissions
    BypassPermissions,
    /// Suppress permission prompts entirely (no asking, no auto-accept)
    DontAsk,
    /// Automatic permission decisions (added in CLI/SDK v0.1.57)
    #[serde(rename = "auto")]
    Auto,
}

/// Controls which filesystem-based configuration sources the SDK loads settings from.
///
/// When multiple sources are loaded, settings are merged with this precedence (highest to lowest):
/// 1. `Local`
/// 2. `Project`
/// 3. `User`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SettingSource {
    /// User settings from `~/.claude/settings.json`
    User,
    /// Project settings from `.claude/settings.json` (team-shared settings)
    Project,
    /// Local settings from `.claude/settings.local.json` (highest priority, git-ignored)
    Local,
}

/// Custom agent definition
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct AgentDefinition {
    /// Agent description
    #[builder(setter(into))]
    pub description: String,
    /// Agent prompt
    #[builder(setter(into))]
    pub prompt: String,
    /// Tools available to the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub tools: Option<Vec<String>>,
    /// Model to use for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub model: Option<AgentModel>,
    /// Tools disallowed for the agent
    #[serde(skip_serializing_if = "Option::is_none", rename = "disallowedTools")]
    #[builder(default, setter(into, strip_option))]
    pub disallowed_tools: Option<Vec<String>>,
    /// Skills available to the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub skills: Option<Vec<String>>,
    /// Memory configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub memory: Option<serde_json::Value>,
    /// MCP servers specific to this agent
    #[serde(skip_serializing_if = "Option::is_none", rename = "mcpServers")]
    #[builder(default, setter(strip_option))]
    pub mcp_servers: Option<serde_json::Value>,
    /// Initial prompt for the agent
    #[serde(skip_serializing_if = "Option::is_none", rename = "initialPrompt")]
    #[builder(default, setter(into, strip_option))]
    pub initial_prompt: Option<String>,
    /// Maximum turns for the agent
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxTurns")]
    #[builder(default, setter(strip_option))]
    pub max_turns: Option<u32>,
    /// Whether to run in background
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub background: Option<bool>,
    /// Effort level for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub effort: Option<String>,
    /// Permission mode for the agent
    #[serde(skip_serializing_if = "Option::is_none", rename = "permissionMode")]
    #[builder(default, setter(strip_option))]
    pub permission_mode: Option<String>,
}

/// Model selection for agents
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentModel {
    /// Claude Sonnet
    Sonnet,
    /// Claude Opus
    Opus,
    /// Claude Haiku
    Haiku,
    /// Inherit from parent
    Inherit,
}

/// SDK Beta features
/// See https://docs.anthropic.com/en/api/beta-headers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SdkBeta {
    /// Extended context window (1M tokens)
    #[serde(rename = "context-1m-2025-08-07")]
    Context1M,
}

/// Thinking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ThinkingConfig {
    /// Adaptive thinking (default)
    Adaptive,
    /// Enabled with budget
    Enabled {
        /// Budget tokens for thinking
        #[serde(rename = "budget_tokens")]
        budget_tokens: u32,
    },
    /// Disabled
    Disabled,
}

impl Default for ThinkingConfig {
    fn default() -> Self {
        ThinkingConfig::Adaptive
    }
}

/// Task budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBudget {
    /// Total budget
    pub total: f64,
}

/// Skills configuration. Either every skill (`All`) or an explicit subset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Skills {
    /// Enable all skills (serializes as the string `"all"`).
    #[serde(with = "skills_all_serde")]
    All,
    /// Enable a specific list of skills by name.
    List(Vec<String>),
}

mod skills_all_serde {
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S: Serializer>(s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str("all")
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<(), D::Error> {
        let value = String::deserialize(d)?;
        if value == "all" {
            Ok(())
        } else {
            Err(D::Error::custom(format!("expected \"all\", got {:?}", value)))
        }
    }
}

impl From<Vec<String>> for Skills {
    fn from(list: Vec<String>) -> Self {
        Skills::List(list)
    }
}

impl From<Vec<&str>> for Skills {
    fn from(list: Vec<&str>) -> Self {
        Skills::List(list.into_iter().map(String::from).collect())
    }
}

impl<const N: usize> From<[&str; N]> for Skills {
    fn from(arr: [&str; N]) -> Self {
        Skills::List(arr.into_iter().map(String::from).collect())
    }
}

/// Tools configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tools {
    /// List of tool names
    List(Vec<String>),
    /// Preset configuration
    Preset(ToolsPreset),
}

/// Tools preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsPreset {
    /// Type field (always "preset")
    #[serde(rename = "type")]
    pub type_: String,
    /// Preset name (e.g., "claude_code")
    pub preset: String,
}

// Conversion implementations for Tools to allow ergonomic API usage
// e.g., .tools(["Write"]) instead of .tools(Tools::from(vec!["Write"]))

impl From<Vec<String>> for Tools {
    fn from(list: Vec<String>) -> Self {
        Tools::List(list)
    }
}

impl From<Vec<&str>> for Tools {
    fn from(list: Vec<&str>) -> Self {
        Tools::List(list.into_iter().map(String::from).collect())
    }
}

impl<const N: usize> From<[&str; N]> for Tools {
    fn from(arr: [&str; N]) -> Self {
        Tools::List(arr.into_iter().map(String::from).collect())
    }
}

impl<const N: usize> From<[String; N]> for Tools {
    fn from(arr: [String; N]) -> Self {
        Tools::List(arr.into_iter().collect())
    }
}

impl From<&[&str]> for Tools {
    fn from(slice: &[&str]) -> Self {
        Tools::List(slice.iter().map(|s| s.to_string()).collect())
    }
}

impl From<ToolsPreset> for Tools {
    fn from(preset: ToolsPreset) -> Self {
        Tools::Preset(preset)
    }
}

impl ToolsPreset {
    /// Create a new tools preset
    pub fn new(preset: impl Into<String>) -> Self {
        Self {
            type_: "preset".to_string(),
            preset: preset.into(),
        }
    }

    /// Create the default claude_code preset
    pub fn claude_code() -> Self {
        Self::new("claude_code")
    }
}

/// Network configuration for sandbox
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct SandboxNetworkConfig {
    /// Unix socket paths accessible in sandbox (e.g., SSH agents)
    #[serde(skip_serializing_if = "Option::is_none", rename = "allowUnixSockets")]
    #[builder(default, setter(into, strip_option))]
    pub allow_unix_sockets: Option<Vec<String>>,

    /// Allow all Unix sockets (less secure)
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "allowAllUnixSockets"
    )]
    #[builder(default, setter(strip_option))]
    pub allow_all_unix_sockets: Option<bool>,

    /// Allow binding to localhost ports (macOS only)
    #[serde(skip_serializing_if = "Option::is_none", rename = "allowLocalBinding")]
    #[builder(default, setter(strip_option))]
    pub allow_local_binding: Option<bool>,

    /// HTTP proxy port if bringing your own proxy
    #[serde(skip_serializing_if = "Option::is_none", rename = "httpProxyPort")]
    #[builder(default, setter(strip_option))]
    pub http_proxy_port: Option<u16>,

    /// SOCKS5 proxy port if bringing your own proxy
    #[serde(skip_serializing_if = "Option::is_none", rename = "socksProxyPort")]
    #[builder(default, setter(strip_option))]
    pub socks_proxy_port: Option<u16>,
}

/// Violations to ignore in sandbox
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct SandboxIgnoreViolations {
    /// File paths for which violations should be ignored
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub file: Option<Vec<String>>,

    /// Network hosts for which violations should be ignored
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub network: Option<Vec<String>>,
}

/// Sandbox settings configuration
///
/// Controls how Claude Code sandboxes bash commands for filesystem
/// and network isolation.
///
/// **Important:** Filesystem and network restrictions are configured via permission
/// rules, not via these sandbox settings:
/// - Filesystem read restrictions: Use Read deny rules
/// - Filesystem write restrictions: Use Edit allow/deny rules
/// - Network restrictions: Use WebFetch allow/deny rules
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct SandboxSettings {
    /// Enable bash sandboxing (macOS/Linux only). Default: False
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub enabled: Option<bool>,

    /// Auto-approve bash commands when sandboxed. Default: True
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "autoAllowBashIfSandboxed"
    )]
    #[builder(default, setter(strip_option))]
    pub auto_allow_bash_if_sandboxed: Option<bool>,

    /// Commands that should run outside the sandbox (e.g., ["git", "docker"])
    #[serde(skip_serializing_if = "Option::is_none", rename = "excludedCommands")]
    #[builder(default, setter(into, strip_option))]
    pub excluded_commands: Option<Vec<String>>,

    /// Allow commands to bypass sandbox via dangerouslyDisableSandbox.
    /// When False, all commands must run sandboxed (or be in excludedCommands). Default: True
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "allowUnsandboxedCommands"
    )]
    #[builder(default, setter(strip_option))]
    pub allow_unsandboxed_commands: Option<bool>,

    /// Network configuration for sandbox
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub network: Option<SandboxNetworkConfig>,

    /// Violations to ignore
    #[serde(skip_serializing_if = "Option::is_none", rename = "ignoreViolations")]
    #[builder(default, setter(strip_option))]
    pub ignore_violations: Option<SandboxIgnoreViolations>,

    /// Enable weaker sandbox for unprivileged Docker environments
    /// (Linux only). Reduces security. Default: False
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "enableWeakerNestedSandbox"
    )]
    #[builder(default, setter(strip_option))]
    pub enable_weaker_nested_sandbox: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_from_str_array() {
        let tools: Tools = ["Write", "Read", "Bash"].into();
        match tools {
            Tools::List(list) => {
                assert_eq!(list, vec!["Write", "Read", "Bash"]);
            }
            _ => panic!("Expected Tools::List"),
        }
    }

    #[test]
    fn test_tools_from_single_str_array() {
        let tools: Tools = ["Write"].into();
        match tools {
            Tools::List(list) => {
                assert_eq!(list, vec!["Write"]);
            }
            _ => panic!("Expected Tools::List"),
        }
    }

    #[test]
    fn test_tools_from_empty_str_array() {
        let empty: [&str; 0] = [];
        let tools: Tools = empty.into();
        match tools {
            Tools::List(list) => {
                assert!(list.is_empty());
            }
            _ => panic!("Expected Tools::List"),
        }
    }

    #[test]
    fn test_tools_from_str_slice() {
        let arr: &[&str] = &["Edit", "Read"];
        let tools: Tools = arr.into();
        match tools {
            Tools::List(list) => {
                assert_eq!(list, vec!["Edit", "Read"]);
            }
            _ => panic!("Expected Tools::List"),
        }
    }

    #[test]
    fn test_tools_from_vec_str() {
        let tools: Tools = vec!["Write", "Read"].into();
        match tools {
            Tools::List(list) => {
                assert_eq!(list, vec!["Write", "Read"]);
            }
            _ => panic!("Expected Tools::List"),
        }
    }

    #[test]
    fn test_tools_from_vec_string() {
        let tools: Tools = vec!["Write".to_string(), "Read".to_string()].into();
        match tools {
            Tools::List(list) => {
                assert_eq!(list, vec!["Write", "Read"]);
            }
            _ => panic!("Expected Tools::List"),
        }
    }

    #[test]
    fn test_tools_from_string_array() {
        let tools: Tools = ["Write".to_string(), "Read".to_string()].into();
        match tools {
            Tools::List(list) => {
                assert_eq!(list, vec!["Write", "Read"]);
            }
            _ => panic!("Expected Tools::List"),
        }
    }

    #[test]
    fn test_tools_in_options_builder() {
        let options = ClaudeAgentOptions::builder()
            .tools(["Write", "Read", "Bash"])
            .build();
        match options.tools {
            Some(Tools::List(list)) => {
                assert_eq!(list, vec!["Write", "Read", "Bash"]);
            }
            _ => panic!("Expected Some(Tools::List)"),
        }
    }

    #[test]
    fn test_tools_in_options_struct() {
        let options = ClaudeAgentOptions {
            tools: Some(["Edit", "Read"].into()),
            ..Default::default()
        };
        match options.tools {
            Some(Tools::List(list)) => {
                assert_eq!(list, vec!["Edit", "Read"]);
            }
            _ => panic!("Expected Some(Tools::List)"),
        }
    }

    #[test]
    fn test_tools_preset() {
        let preset = ToolsPreset::claude_code();
        let tools: Tools = preset.into();
        match tools {
            Tools::Preset(p) => {
                assert_eq!(p.preset, "claude_code");
                assert_eq!(p.type_, "preset");
            }
            _ => panic!("Expected Tools::Preset"),
        }
    }

    #[test]
    fn test_thinking_config_adaptive() {
        let config = ThinkingConfig::Adaptive;
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["type"], "adaptive");
    }

    #[test]
    fn test_thinking_config_enabled() {
        let config = ThinkingConfig::Enabled { budget_tokens: 2000 };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["type"], "enabled");
        assert_eq!(json["budget_tokens"], 2000);
    }

    #[test]
    fn test_thinking_config_disabled() {
        let config = ThinkingConfig::Disabled;
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["type"], "disabled");
    }

    #[test]
    fn test_task_budget() {
        let budget = TaskBudget { total: 10.0 };
        let json = serde_json::to_value(&budget).unwrap();
        assert_eq!(json["total"], 10.0);
    }

    #[test]
    fn test_agent_definition_extended_fields() {
        let agent = AgentDefinition::builder()
            .description("Code reviewer")
            .prompt("Review code for issues")
            .skills(vec!["code-review".to_string()])
            .max_turns(5)
            .effort("high")
            .build();

        assert_eq!(agent.skills, Some(vec!["code-review".to_string()]));
        assert_eq!(agent.max_turns, Some(5));
        assert_eq!(agent.effort, Some("high".to_string()));
    }

    #[test]
    fn test_agent_definition_serialization() {
        let agent = AgentDefinition {
            description: "Test agent".to_string(),
            prompt: "Do something".to_string(),
            tools: Some(vec!["Read".to_string()]),
            model: Some(AgentModel::Sonnet),
            disallowed_tools: Some(vec!["Bash".to_string()]),
            skills: Some(vec!["skill1".to_string()]),
            memory: None,
            mcp_servers: None,
            initial_prompt: Some("Hello".to_string()),
            max_turns: Some(10),
            background: Some(true),
            effort: Some("medium".to_string()),
            permission_mode: Some("acceptEdits".to_string()),
        };

        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["description"], "Test agent");
        assert_eq!(json["disallowedTools"][0], "Bash");
        assert_eq!(json["maxTurns"], 10);
        assert_eq!(json["background"], true);
    }

    #[test]
    fn test_claude_agent_options_new_fields() {
        let options = ClaudeAgentOptions::builder()
            .skills(vec!["skill1".to_string()])
            .thinking(ThinkingConfig::Enabled { budget_tokens: 5000 })
            .effort("high")
            .task_budget(TaskBudget { total: 5.0 })
            .load_timeout_ms(30000)
            .build();

        assert_eq!(
            options.skills,
            Some(Skills::List(vec!["skill1".to_string()]))
        );
        assert!(options.thinking.is_some());
        assert_eq!(options.effort, Some("high".to_string()));
        assert!(options.task_budget.is_some());
        assert_eq!(options.load_timeout_ms, Some(30000));
    }

    #[test]
    fn test_permission_mode_new_variants_serde() {
        assert_eq!(
            serde_json::to_string(&PermissionMode::DontAsk).unwrap(),
            "\"dontAsk\""
        );
        assert_eq!(
            serde_json::to_string(&PermissionMode::Auto).unwrap(),
            "\"auto\""
        );
        let dont_ask: PermissionMode = serde_json::from_str("\"dontAsk\"").unwrap();
        assert_eq!(dont_ask, PermissionMode::DontAsk);
        let auto: PermissionMode = serde_json::from_str("\"auto\"").unwrap();
        assert_eq!(auto, PermissionMode::Auto);
    }

    #[test]
    fn test_system_prompt_file_serde() {
        let sp = SystemPrompt::File(SystemPromptFile::new("/etc/prompt.md"));
        let json = serde_json::to_value(&sp).unwrap();
        assert_eq!(json["type"], "file");
        assert_eq!(json["path"], "/etc/prompt.md");
    }

    #[test]
    fn test_system_prompt_preset_exclude_dynamic_sections_omitted_by_default() {
        let preset = SystemPromptPreset::new("claude_code");
        let json = serde_json::to_value(&preset).unwrap();
        assert!(json.get("exclude_dynamic_sections").is_none());
    }

    #[test]
    fn test_system_prompt_preset_exclude_dynamic_sections_set() {
        let preset = SystemPromptPreset::new("claude_code").with_exclude_dynamic_sections(true);
        let json = serde_json::to_value(&preset).unwrap();
        assert_eq!(json["exclude_dynamic_sections"], true);
    }

    #[test]
    fn test_session_id_field() {
        let options = ClaudeAgentOptions::builder()
            .session_id("abc-123")
            .build();
        assert_eq!(options.session_id, Some("abc-123".to_string()));
        assert!(options.resume.is_none());
    }

    #[test]
    fn test_skills_all_serializes_as_string() {
        let skills = Skills::All;
        let json = serde_json::to_value(&skills).unwrap();
        assert_eq!(json, serde_json::json!("all"));

        let parsed: Skills = serde_json::from_value(serde_json::json!("all")).unwrap();
        assert_eq!(parsed, Skills::All);
    }

    #[test]
    fn test_skills_list_serializes_as_array() {
        let skills = Skills::List(vec!["a".to_string(), "b".to_string()]);
        let json = serde_json::to_value(&skills).unwrap();
        assert_eq!(json, serde_json::json!(["a", "b"]));
    }

    #[test]
    fn test_skills_via_builder_from_vec() {
        let options = ClaudeAgentOptions::builder()
            .skills(vec!["x", "y"])
            .build();
        assert_eq!(
            options.skills,
            Some(Skills::List(vec!["x".to_string(), "y".to_string()]))
        );
    }

    #[test]
    fn test_skills_via_builder_all() {
        let options = ClaudeAgentOptions::builder().skills(Skills::All).build();
        assert_eq!(options.skills, Some(Skills::All));
    }
}
