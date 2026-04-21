//! Example 26: Filesystem-based Agents
//!
//! Loads agents from `.claude/agents/*.md` files via the `setting_sources(["project"])`
//! option, instead of declaring them inline through `AgentDefinition` (see
//! `09_agents.rs` for the inline pattern).
//!
//! This is the Rust port of the Python SDK example
//! `examples/filesystem_agents.py` (issue #406 regression coverage):
//! it verifies that filesystem agents reliably load via `setting_sources`.
//!
//! Expects `<repo>/.claude/agents/test-agent.md` to exist (shipped with this SDK).
//!
//! Run with: cargo run --example 26_filesystem_agents

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ClaudeClient, ContentBlock, Message, SettingSource,
};
use futures::StreamExt;

/// Extract agent names from a SystemMessage `init` payload.
/// Agents may appear either as bare strings or as objects with a `name` field.
fn extract_agents(data: &serde_json::Value) -> Vec<String> {
    let Some(arr) = data.get("agents").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|a| match a {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Object(_) => a
                .get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string()),
            _ => None,
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Filesystem Agents Example ===");
    println!("Testing: setting_sources=[Project] with .claude/agents/test-agent.md\n");

    // Use the SDK repo directory (which ships .claude/agents/test-agent.md).
    let sdk_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();

    let options = ClaudeAgentOptions::builder()
        .setting_sources(vec![SettingSource::Project])
        .cwd(sdk_dir)
        .build();

    let mut message_types: Vec<&'static str> = Vec::new();
    let mut agents_found: Vec<String> = Vec::new();

    let mut client = ClaudeClient::new(options);
    client.connect().await?;
    client.query("Say hello in exactly 3 words").await?;

    let mut stream = client.receive_response();
    while let Some(msg) = stream.next().await {
        match msg? {
            Message::System(sys) => {
                message_types.push("SystemMessage");
                if sys.subtype == "init" {
                    agents_found = extract_agents(&sys.data);
                    println!("Init message received. Agents loaded: {:?}", agents_found);
                }
            }
            Message::Assistant(asst) => {
                message_types.push("AssistantMessage");
                for block in &asst.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Assistant: {}", text.text);
                    }
                }
            }
            Message::Result(result) => {
                message_types.push("ResultMessage");
                println!(
                    "Result: subtype={}, cost=${:.4}",
                    result.subtype,
                    result.total_cost_usd.unwrap_or(0.0)
                );
            }
            _ => {}
        }
    }
    drop(stream);
    client.disconnect().await?;

    println!("\n=== Summary ===");
    println!("Message types received: {:?}", message_types);
    println!("Total messages: {}", message_types.len());

    let has_init = message_types.contains(&"SystemMessage");
    let has_assistant = message_types.contains(&"AssistantMessage");
    let has_result = message_types.contains(&"ResultMessage");
    let has_test_agent = agents_found.iter().any(|a| a == "test-agent");

    println!();
    if has_init && has_assistant && has_result {
        println!("SUCCESS: Received full response (init, assistant, result)");
    } else {
        println!("FAILURE: Did not receive full response");
        println!("  - Init: {}", has_init);
        println!("  - Assistant: {}", has_assistant);
        println!("  - Result: {}", has_result);
    }

    if has_test_agent {
        println!("SUCCESS: test-agent was loaded from filesystem");
    } else {
        println!("WARNING: test-agent was NOT loaded (may not exist in .claude/agents/)");
    }

    Ok(())
}
