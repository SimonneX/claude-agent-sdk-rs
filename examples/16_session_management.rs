//! Example demonstrating session management and memory clearing
//!
//! This example shows how to:
//! 1. Use different session IDs to maintain separate conversation contexts
//! 2. Use fork_session to create fresh sessions that don't inherit history
//! 3. Switch between sessions dynamically
//! 4. Pre-assign a session id at startup via `session_id` (Python SDK v0.1.52
//!    parity, distinct from `resume` which re-opens an existing session)
//!
//! Run with: cargo run --example 16_session_management

use claude_agent_sdk_rs::{ClaudeAgentOptions, ClaudeClient, ContentBlock, Message};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Session Management Example ===\n");

    // Example 1: Using different session IDs for separate conversations
    println!("--- Example 1: Separate Sessions ---");
    example_separate_sessions().await?;

    println!("\n--- Example 2: Fork Session for Fresh Start ---");
    example_fork_session().await?;

    println!("\n--- Example 3: Dynamic Session Switching ---");
    example_dynamic_switching().await?;

    println!("\n--- Example 4: Pre-Assigned session_id at Startup ---");
    example_preassigned_session_id().await?;

    Ok(())
}

/// Example 1: Use different session IDs to maintain separate contexts
async fn example_separate_sessions() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions {
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        ..Default::default()
    };
    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // Session 1: Math conversation
    println!("Session 1: Asking about math");
    client
        .query_with_session("What is 2 + 2?", "math-session")
        .await?;

    {
        let mut stream = client.receive_response();
        while let Some(msg) = stream.next().await {
            if let Message::Assistant(assistant) = msg? {
                for block in &assistant.message.content {
                    if let ContentBlock::Text(text_block) = block {
                        println!("Claude (math-session): {}", text_block.text);
                    }
                }
            }
        }
    }

    // Session 2: Completely different conversation about programming
    println!("\nSession 2: Asking about programming (different context)");
    client
        .query_with_session("What is Rust?", "programming-session")
        .await?;

    {
        let mut stream = client.receive_response();
        while let Some(msg) = stream.next().await {
            if let Message::Assistant(assistant) = msg? {
                for block in &assistant.message.content {
                    if let ContentBlock::Text(text_block) = block {
                        println!("Claude (programming-session): {}", text_block.text);
                    }
                }
            }
        }
    }

    // Back to Session 1: Claude remembers the math context
    println!("\nBack to Session 1: Follow-up on math");
    client
        .query_with_session("What about 3 + 3?", "math-session")
        .await?;

    {
        let mut stream = client.receive_response();
        while let Some(msg) = stream.next().await {
            if let Message::Assistant(assistant) = msg? {
                for block in &assistant.message.content {
                    if let ContentBlock::Text(text_block) = block {
                        println!("Claude (math-session): {}", text_block.text);
                    }
                }
            }
        }
    }

    client.disconnect().await?;
    Ok(())
}

/// Example 2: Use fork_session to start completely fresh
async fn example_fork_session() -> Result<(), Box<dyn std::error::Error>> {
    // Create options with fork_session enabled
    let options = ClaudeAgentOptions::builder()
        .model("sonnet".to_string()) // Use Sonnet for lower cost
        .fork_session(true)
        .max_turns(1)
        .build();

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    println!("With fork_session=true, resumed sessions start fresh");
    println!("This is useful when you want to completely clear memory/context");

    client.query("Remember this number: 42").await?;

    {
        let mut stream = client.receive_response();
        while let Some(msg) = stream.next().await {
            if let Message::Assistant(assistant) = msg? {
                for block in &assistant.message.content {
                    if let ContentBlock::Text(text_block) = block {
                        println!("Claude: {}", text_block.text);
                    }
                }
            }
        }
    }

    client.disconnect().await?;
    Ok(())
}

/// Example 3: Use new_session() convenience method
async fn example_dynamic_switching() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions {
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        ..Default::default()
    };
    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // Start first conversation
    println!("Starting first conversation");
    client.query("My favorite color is blue").await?;

    {
        let mut stream = client.receive_response();
        while let Some(msg) = stream.next().await {
            if let Message::Assistant(assistant) = msg? {
                for block in &assistant.message.content {
                    if let ContentBlock::Text(text_block) = block {
                        println!("Claude: {}", text_block.text);
                    }
                }
            }
        }
    }

    // Switch to new session using convenience method
    println!("\nSwitching to new session (different context)");
    client
        .new_session("session-2", "My favorite color is red")
        .await?;

    {
        let mut stream = client.receive_response();
        while let Some(msg) = stream.next().await {
            if let Message::Assistant(assistant) = msg? {
                for block in &assistant.message.content {
                    if let ContentBlock::Text(text_block) = block {
                        println!("Claude: {}", text_block.text);
                    }
                }
            }
        }
    }

    client.disconnect().await?;
    Ok(())
}

/// Example 4: Pre-assign a session id at startup with `session_id`
///
/// `session_id` (Python SDK v0.1.52) is distinct from `resume`:
/// - `session_id` pre-assigns the id of a NEW session, so the caller knows up-front
///   what id to log/persist/share for later lookups.
/// - `resume` re-opens an EXISTING session by id.
///
/// Use `session_id` when your application generates session ids (e.g. tied to a
/// tenant, request, or user) before starting Claude.
async fn example_preassigned_session_id() -> Result<(), Box<dyn std::error::Error>> {
    let preassigned = "demo-session-550e8400-e29b-41d4-a716-446655440000";

    let options = ClaudeAgentOptions::builder()
        .model("sonnet".to_string()) // Use Sonnet for lower cost
        .session_id(preassigned)
        .max_turns(1)
        .build();

    println!(
        "Starting a NEW session with caller-supplied id: {}",
        preassigned
    );
    println!("(This is different from `resume`, which re-opens an EXISTING session)");

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    client.query("Say hi in one short sentence.").await?;

    let mut stream = client.receive_response();
    while let Some(msg) = stream.next().await {
        if let Message::Assistant(assistant) = msg? {
            for block in &assistant.message.content {
                if let ContentBlock::Text(text_block) = block {
                    println!("Claude: {}", text_block.text);
                }
            }
        }
    }
    drop(stream);

    client.disconnect().await?;
    Ok(())
}
