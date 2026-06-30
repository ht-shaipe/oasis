//! Normalized streaming events — unified across all CLI agents.
//!
//! Every agent adapter converts its native stdout JSONL into `NormalizedEvent`
//! variants. The frontend only consumes these normalized events, never raw agent
//! output.

use serde::{Deserialize, Serialize};

/// A single event emitted during an agent streaming session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NormalizedEvent {
    /// Incremental text delta (accumulated by frontend).
    TextDelta { delta: String },

    /// Complete message snapshot (rare — used for tool_use blocks only).
    Message { content: Vec<ContentBlock> },

    /// An agent started a tool invocation.
    ToolUseStart {
        call_id: String,
        tool: String,
        input: serde_json::Value,
    },

    /// A tool invocation finished.
    ToolUseResult {
        call_id: String,
        output: serde_json::Value,
        is_error: bool,
    },

    /// Incremental thinking / reasoning delta.
    Thinking { delta: String },

    /// Agent requests user approval for a sensitive action.
    ApprovalRequest {
        request_id: String,
        approval_kind: ApprovalKind,
        payload: serde_json::Value,
    },

    /// The real session id was resolved (originally pending-<pid>).
    SessionResolved { session_id: String },

    /// A conversation turn ended.
    TurnComplete {
        reason: TurnEndReason,
        usage: Option<UsageStats>,
    },

    /// Recoverable or fatal error.
    Error {
        message: String,
        recoverable: bool,
    },

    /// Passthrough raw event for unsupported event types.
    Raw {
        agent: String,
        raw: serde_json::Value,
    },
}

impl NormalizedEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            NormalizedEvent::TextDelta { .. } => "text_delta",
            NormalizedEvent::Message { .. } => "message",
            NormalizedEvent::ToolUseStart { .. } => "tool_use_start",
            NormalizedEvent::ToolUseResult { .. } => "tool_use_result",
            NormalizedEvent::Thinking { .. } => "thinking",
            NormalizedEvent::ApprovalRequest { .. } => "approval_request",
            NormalizedEvent::SessionResolved { .. } => "session_resolved",
            NormalizedEvent::TurnComplete { .. } => "turn_complete",
            NormalizedEvent::Error { .. } => "error",
            NormalizedEvent::Raw { .. } => "raw",
        }
    }
}

/// Content blocks that make up a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: serde_json::Value, is_error: bool },
    Thinking { thinking: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnEndReason {
    Complete,
    Aborted,
    Error,
    MaxTokens,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalKind {
    Command,
    FileWrite,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_cost: Option<f64>,
    pub context_remaining: Option<u64>,
}
