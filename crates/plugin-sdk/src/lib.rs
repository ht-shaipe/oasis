//! copyright © ecdata.cn 2026 - present
//! 插件
//! created shaipe by 2026-05-18 17:07:05

/// 插件元数据（进程 / 清单级信息）。
mod meta;
use std::fmt;

pub use meta::PluginMeta;

/// 插件接口定义
mod plugin;
pub use plugin::Plugin;

/// 宿主在加载阶段注入的只读上下文（可逐步扩展）。
mod context;
pub use context::PluginContext;


/// SDK 层错误类型。
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    // 插件错误
    #[error("plugin `{0}`: {1}")]
    Plugin(String, String),
    // 未知工具
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    // 无效参数
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
}

impl PluginError {
    pub fn plugin(id: impl fmt::Display, msg: impl fmt::Display) -> Self {
        Self::Plugin(id.to_string(), msg.to_string())
    }
}