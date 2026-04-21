//! Example 25: Skills configuration
//!
//! Demonstrates the `skills` option (Python SDK v0.1.62 parity). Skills are
//! folded into `--allowedTools` so the CLI grants Claude access to the
//! corresponding `Skill(...)` entries:
//!
//! - `Skills::All`           → adds the bare `Skill` tool (every skill enabled)
//! - `Skills::List(vec![…])` → adds `Skill(<name>)` per entry (selective)
//!
//! Note: prior to this option being wired, the `skills` field existed but was
//! silently ignored — see CHANGELOG. Use `Skills::All` when you want Claude
//! to discover every available skill, and `Skills::List` to scope to a known set.
//!
//! Run with: cargo run --example 25_skills

use claude_agent_sdk_rs::{ClaudeAgentOptions, ContentBlock, Message, Skills, query};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Skills Examples ===\n");

    skills_all().await?;
    skills_list().await?;

    Ok(())
}

async fn skills_all() -> anyhow::Result<()> {
    println!("=== Skills::All (enable every skill) ===");

    let options = ClaudeAgentOptions::builder()
        .skills(Skills::All)
        .model("sonnet".to_string()) // Use Sonnet for lower cost
        .build();

    let messages = query("Briefly: which skills do you have available?", Some(options)).await?;

    display_messages(&messages);
    println!();

    Ok(())
}

async fn skills_list() -> anyhow::Result<()> {
    println!("=== Skills::List (selective) ===");
    println!("(Each name becomes Skill(<name>) in --allowedTools)");

    let options = ClaudeAgentOptions::builder()
        .skills(vec!["code-review", "data-analysis"])
        .model("sonnet".to_string()) // Use Sonnet for lower cost
        .build();

    let messages = query(
        "What skills can you invoke right now? List them.",
        Some(options),
    )
    .await?;

    display_messages(&messages);
    println!();

    Ok(())
}

fn display_messages(messages: &[Message]) {
    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }
}
