//! Context usage types for Claude Agent SDK
//!
//! Types returned by get_context_usage() method

use serde::{Deserialize, Serialize};

/// Context usage response from get_context_usage()
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUsageResponse {
    /// Context usage categories
    pub categories: Vec<ContextUsageCategory>,
    /// Total tokens used
    #[serde(rename = "totalTokens")]
    pub total_tokens: u64,
    /// Maximum tokens available
    #[serde(rename = "maxTokens")]
    pub max_tokens: u64,
    /// Raw max tokens (before adjustments)
    #[serde(rename = "rawMaxTokens", skip_serializing_if = "Option::is_none")]
    pub raw_max_tokens: Option<u64>,
    /// Usage percentage
    pub percentage: f64,
    /// Current model
    pub model: String,
    /// Whether auto compact is enabled
    #[serde(rename = "isAutoCompactEnabled")]
    pub is_auto_compact_enabled: bool,
    /// Memory files
    #[serde(rename = "memoryFiles", skip_serializing_if = "Option::is_none")]
    pub memory_files: Option<Vec<String>>,
    /// MCP tools
    #[serde(rename = "mcpTools", skip_serializing_if = "Option::is_none")]
    pub mcp_tools: Option<Vec<String>>,
    /// Agents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<Vec<String>>,
    /// Auto compact threshold
    #[serde(rename = "autoCompactThreshold", skip_serializing_if = "Option::is_none")]
    pub auto_compact_threshold: Option<f64>,
    /// Deferred builtin tools
    #[serde(rename = "deferredBuiltinTools", skip_serializing_if = "Option::is_none")]
    pub deferred_builtin_tools: Option<Vec<String>>,
    /// System tools
    #[serde(rename = "systemTools", skip_serializing_if = "Option::is_none")]
    pub system_tools: Option<Vec<String>>,
    /// System prompt sections
    #[serde(rename = "systemPromptSections", skip_serializing_if = "Option::is_none")]
    pub system_prompt_sections: Option<Vec<String>>,
    /// Slash commands
    #[serde(rename = "slashCommands", skip_serializing_if = "Option::is_none")]
    pub slash_commands: Option<Vec<String>>,
    /// Skills
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    /// Message breakdown
    #[serde(rename = "messageBreakdown", skip_serializing_if = "Option::is_none")]
    pub message_breakdown: Option<MessageBreakdown>,
    /// API usage
    #[serde(rename = "apiUsage", skip_serializing_if = "Option::is_none")]
    pub api_usage: Option<ApiUsage>,
}

/// Context usage category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUsageCategory {
    /// Category name
    pub name: String,
    /// Tokens in this category
    pub tokens: u64,
    /// Display color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Whether this is deferred
    #[serde(rename = "isDeferred", skip_serializing_if = "Option::is_none")]
    pub is_deferred: Option<bool>,
}

/// Message breakdown statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBreakdown {
    /// Number of user messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_messages: Option<u32>,
    /// Number of assistant messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_messages: Option<u32>,
    /// Number of tool results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_results: Option<u32>,
}

/// API usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsage {
    /// Input tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    /// Output tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    /// Total tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_usage_category_serialization() {
        let category = ContextUsageCategory {
            name: "system_prompt".to_string(),
            tokens: 1000,
            color: Some("blue".to_string()),
            is_deferred: None,
        };

        let json = serde_json::to_value(&category).unwrap();
        assert_eq!(json["name"], "system_prompt");
        assert_eq!(json["tokens"], 1000);
        assert_eq!(json["color"], "blue");
    }

    #[test]
    fn test_context_usage_response_serialization() {
        let response = ContextUsageResponse {
            categories: vec![ContextUsageCategory {
                name: "test".to_string(),
                tokens: 500,
                color: None,
                is_deferred: None,
            }],
            total_tokens: 500,
            max_tokens: 100000,
            raw_max_tokens: Some(100000),
            percentage: 0.5,
            model: "claude-sonnet-4".to_string(),
            is_auto_compact_enabled: true,
            memory_files: None,
            mcp_tools: None,
            agents: None,
            auto_compact_threshold: None,
            deferred_builtin_tools: None,
            system_tools: None,
            system_prompt_sections: None,
            slash_commands: None,
            skills: None,
            message_breakdown: None,
            api_usage: None,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["totalTokens"], 500);
        assert_eq!(json["maxTokens"], 100000);
        assert_eq!(json["percentage"], 0.5);
        assert_eq!(json["isAutoCompactEnabled"], true);
    }

    #[test]
    fn test_message_breakdown() {
        let breakdown = MessageBreakdown {
            user_messages: Some(3),
            assistant_messages: Some(3),
            tool_results: Some(5),
        };

        let json = serde_json::to_value(&breakdown).unwrap();
        assert_eq!(json["user_messages"], 3);
        assert_eq!(json["assistant_messages"], 3);
        assert_eq!(json["tool_results"], 5);
    }

    #[test]
    fn test_api_usage() {
        let usage = ApiUsage {
            input_tokens: Some(1000),
            output_tokens: Some(500),
            total_tokens: Some(1500),
        };

        let json = serde_json::to_value(&usage).unwrap();
        assert_eq!(json["input_tokens"], 1000);
        assert_eq!(json["output_tokens"], 500);
        assert_eq!(json["total_tokens"], 1500);
    }
}