//! copyright © ecdata.cn 2026 - present

/// SDK 层错误类型
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("plugin `{id}`: {msg}")]
    Plugin { id: String, msg: String },

    #[error("unknown tool: {0}")]
    UnknownTool(String),

    #[error("invalid arguments: {0}")]
    InvalidArguments(String),

    #[error("action failed: {0}")]
    ActionFailed(String),
}

impl PluginError {
    pub fn plugin(id: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::Plugin { id: id.into(), msg: msg.into() }
    }
}
