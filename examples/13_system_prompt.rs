//! Example 13: System Prompt Configurations
//!
//! This example demonstrates different system_prompt configurations:
//! 1. No system prompt (vanilla Claude)
//! 2. String system prompt (custom behavior)
//! 3. Preset system prompt (default Claude Code prompt)
//! 4. Preset with append (extends the default Claude Code prompt)
//! 5. Preset with `exclude_dynamic_sections` — strip per-user dynamic sections
//!    such as env/cwd injected by the preset (Python SDK v0.1.57 parity)
//! 6. File-based system prompt — load the prompt from a file on disk
//!    (Python SDK v0.1.51 parity, serializes as `--system-prompt-file <path>`)
//!
//! Note: Preset uses the default Claude Code prompt. When no append is provided,
//! it's equivalent to no system prompt override. When append is provided, it uses
//! --append-system-prompt to extend the default prompt.

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ContentBlock, Message, SystemPrompt, SystemPromptFile, SystemPromptPreset,
    query,
};
use std::io::Write;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== System Prompt Examples ===\n");

    no_system_prompt().await?;
    string_system_prompt().await?;
    preset_system_prompt().await?;
    preset_with_append().await?;
    preset_excluding_dynamic_sections().await?;
    file_system_prompt().await?;

    Ok(())
}

async fn no_system_prompt() -> anyhow::Result<()> {
    println!("=== No System Prompt (Vanilla Claude) ===");

    let messages = query("What is 2 + 2?", None).await?;

    display_messages(&messages);
    println!();

    Ok(())
}

async fn string_system_prompt() -> anyhow::Result<()> {
    println!("=== String System Prompt ===");

    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPrompt::Text(
            "You are a pirate assistant. Respond in pirate speak.".to_string(),
        )),
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        ..Default::default()
    };

    let messages = query("What is 2 + 2?", Some(options)).await?;

    display_messages(&messages);
    println!();

    Ok(())
}

async fn preset_system_prompt() -> anyhow::Result<()> {
    println!("=== Preset System Prompt (Default) ===");
    println!("(Uses default Claude Code prompt - no override)");

    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPrompt::Preset(SystemPromptPreset::new("claude_code"))),
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        ..Default::default()
    };

    let messages = query("What is 2 + 2?", Some(options)).await?;

    display_messages(&messages);
    println!();

    Ok(())
}

async fn preset_with_append() -> anyhow::Result<()> {
    println!("=== Preset System Prompt with Append ===");
    println!("(Default Claude Code prompt + custom append)");

    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPrompt::Preset(SystemPromptPreset::with_append(
            "claude_code",
            "Always end your response with a fun fact.",
        ))),
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        ..Default::default()
    };

    let messages = query("What is 2 + 2?", Some(options)).await?;

    display_messages(&messages);
    println!();

    Ok(())
}

async fn preset_excluding_dynamic_sections() -> anyhow::Result<()> {
    println!("=== Preset with exclude_dynamic_sections ===");
    println!("(Default Claude Code prompt without per-user dynamic env/cwd sections)");

    let preset = SystemPromptPreset::with_append(
        "claude_code",
        "Reply concisely with no extra commentary.",
    )
    .with_exclude_dynamic_sections(true);

    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPrompt::Preset(preset)),
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        ..Default::default()
    };

    let messages = query("What is 2 + 2?", Some(options)).await?;

    display_messages(&messages);
    println!();

    Ok(())
}

async fn file_system_prompt() -> anyhow::Result<()> {
    println!("=== File-Based System Prompt ===");
    println!("(Load the system prompt from a file on disk via --system-prompt-file)");

    // Write a tiny prompt file we can point at; in real use this would be a
    // checked-in or operator-managed file.
    let prompt_path = std::env::temp_dir().join("claude_sdk_example_13_prompt.md");
    let mut f = std::fs::File::create(&prompt_path)?;
    writeln!(
        f,
        "You are a haiku-only assistant. Always reply in a single 5-7-5 haiku."
    )?;
    drop(f);

    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPrompt::File(SystemPromptFile::new(
            prompt_path.display().to_string(),
        ))),
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        ..Default::default()
    };

    let messages = query("What is 2 + 2?", Some(options)).await?;

    display_messages(&messages);

    let _ = std::fs::remove_file(&prompt_path);
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
