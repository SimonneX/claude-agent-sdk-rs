//! Message parser for converting JSON to typed messages

use crate::errors::{MessageParseError, Result};
use crate::types::messages::Message;

/// Message parser for CLI output
pub struct MessageParser;

impl MessageParser {
    /// Parse a JSON value into a Message, consuming the value
    ///
    /// This method consumes the JSON value to avoid unnecessary cloning.
    /// On parse error, the original data is not available in the error
    /// since it was consumed during the parse attempt.
    pub fn parse(data: serde_json::Value) -> Result<Message> {
        serde_json::from_value(data).map_err(|e| {
            MessageParseError::new(
                format!("Failed to parse message: {}", e),
                None, // Don't include original data to avoid cloning overhead
            )
            .into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ClaudeError;
    use serde_json::json;

    fn assert_parse_err(value: serde_json::Value) -> String {
        match MessageParser::parse(value) {
            Err(ClaudeError::MessageParse(e)) => e.message,
            Err(other) => panic!("expected MessageParse error, got {:?}", other),
            Ok(msg) => panic!("expected error, parsed: {:?}", msg),
        }
    }

    #[test]
    fn parse_assistant_minimal() {
        let value = json!({
            "type": "assistant",
            "message": {"content": []}
        });
        let msg = MessageParser::parse(value).unwrap();
        assert!(matches!(msg, Message::Assistant(_)));
    }

    #[test]
    fn parse_system() {
        let value = json!({
            "type": "system",
            "subtype": "init",
            "session_id": "s1"
        });
        let msg = MessageParser::parse(value).unwrap();
        match msg {
            Message::System(s) => {
                assert_eq!(s.subtype, "init");
                assert_eq!(s.session_id.as_deref(), Some("s1"));
            }
            other => panic!("expected System, got {:?}", other),
        }
    }

    #[test]
    fn parse_result() {
        let value = json!({
            "type": "result",
            "subtype": "query_complete",
            "duration_ms": 100,
            "duration_api_ms": 80,
            "is_error": false,
            "num_turns": 1,
            "session_id": "s1"
        });
        let msg = MessageParser::parse(value).unwrap();
        match msg {
            Message::Result(r) => {
                assert!(!r.is_error);
                assert_eq!(r.num_turns, 1);
            }
            other => panic!("expected Result, got {:?}", other),
        }
    }

    #[test]
    fn parse_user_with_tool_result() {
        let value = json!({
            "type": "user",
            "text": "ok",
            "uuid": "u1",
            "parent_tool_use_id": "t1"
        });
        let msg = MessageParser::parse(value).unwrap();
        match msg {
            Message::User(u) => {
                assert_eq!(u.uuid.as_deref(), Some("u1"));
                assert_eq!(u.parent_tool_use_id.as_deref(), Some("t1"));
            }
            other => panic!("expected User, got {:?}", other),
        }
    }

    #[test]
    fn parse_stream_event() {
        let value = json!({
            "type": "stream_event",
            "uuid": "evt1",
            "session_id": "s1",
            "event": {"foo": "bar"}
        });
        let msg = MessageParser::parse(value).unwrap();
        match msg {
            Message::StreamEvent(e) => {
                assert_eq!(e.uuid, "evt1");
                assert_eq!(e.event["foo"], "bar");
            }
            other => panic!("expected StreamEvent, got {:?}", other),
        }
    }

    #[test]
    fn parse_control_cancel_request_passthrough() {
        let value = json!({
            "type": "control_cancel_request",
            "request_id": "r1"
        });
        let msg = MessageParser::parse(value).unwrap();
        match msg {
            Message::ControlCancelRequest(v) => {
                assert_eq!(v["request_id"], "r1");
            }
            other => panic!("expected ControlCancelRequest, got {:?}", other),
        }
    }

    #[test]
    fn parse_task_started() {
        let value = json!({
            "type": "task_started",
            "task_id": "t1",
            "description": "doing X",
            "uuid": "u1",
            "session_id": "s1"
        });
        let msg = MessageParser::parse(value).unwrap();
        assert!(matches!(msg, Message::TaskStarted(_)));
    }

    #[test]
    fn parse_task_progress() {
        let value = json!({
            "type": "task_progress",
            "task_id": "t1",
            "description": "tick",
            "uuid": "u1",
            "session_id": "s1"
        });
        let msg = MessageParser::parse(value).unwrap();
        assert!(matches!(msg, Message::TaskProgress(_)));
    }

    #[test]
    fn parse_task_notification() {
        let value = json!({
            "type": "task_notification",
            "task_id": "t1",
            "status": "completed",
            "uuid": "u1",
            "session_id": "s1"
        });
        let msg = MessageParser::parse(value).unwrap();
        assert!(matches!(msg, Message::TaskNotification(_)));
    }

    #[test]
    fn parse_rate_limit() {
        let value = json!({
            "type": "rate_limit",
            "rate_limit_info": {"status": "allowed"},
            "uuid": "u1",
            "session_id": "s1"
        });
        let msg = MessageParser::parse(value).unwrap();
        assert!(matches!(msg, Message::RateLimit(_)));
    }

    #[test]
    fn parse_mirror_error() {
        let value = json!({
            "type": "mirror_error",
            "error": "boom"
        });
        let msg = MessageParser::parse(value).unwrap();
        match msg {
            Message::MirrorError(m) => assert_eq!(m.error, "boom"),
            other => panic!("expected MirrorError, got {:?}", other),
        }
    }

    #[test]
    fn unknown_type_yields_parse_error() {
        let msg = assert_parse_err(json!({"type": "definitely_not_a_type"}));
        assert!(
            msg.contains("Failed to parse message"),
            "unexpected message: {msg}"
        );
    }

    #[test]
    fn missing_type_field_yields_parse_error() {
        // serde untagged on `tag = "type"` will fail without the discriminant
        assert_parse_err(json!({"foo": "bar"}));
    }

    #[test]
    fn malformed_result_missing_required_fields() {
        // Result requires duration_ms, num_turns, session_id, etc.
        assert_parse_err(json!({
            "type": "result",
            "subtype": "x"
        }));
    }

    #[test]
    fn malformed_content_block_unknown_type() {
        // ContentBlock uses `tag = "type"`, unknown type fails
        assert_parse_err(json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "wat", "text": "x"}]
            }
        }));
    }

    #[test]
    fn malformed_text_block_missing_text_field() {
        assert_parse_err(json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text"}]
            }
        }));
    }

    #[test]
    fn parse_does_not_carry_data_in_error() {
        // Confirms the documented behavior: parse error message has no `data` payload
        match MessageParser::parse(json!({"type": "?"})) {
            Err(ClaudeError::MessageParse(e)) => assert!(e.data.is_none()),
            other => panic!("expected MessageParse, got {:?}", other),
        }
    }
}
