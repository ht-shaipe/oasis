//! copyright © ecdata.cn 2026 - present
//! UI Schema — 插件清单与状态类型

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 插件清单
// ---------------------------------------------------------------------------

/// 插件清单（含 UI schema）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmManifest {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub description: String,
    pub version: String,
    /// UI schema JSON
    pub ui: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 宿主环境 API（Host Imports，WASM 专用）
// ---------------------------------------------------------------------------

/// 宿主注入给 WASM 插件的环境函数
pub struct HostEnv;

impl HostEnv {
    pub const MODULE_NAME: &'static str = "env";
    pub const FN_GET_CONTEXT: &'static str = "host_get_context";
    pub const FN_LOG: &'static str = "host_log";
    pub const FN_READ_FILE: &'static str = "host_read_file";
    pub const FN_WRITE_FILE: &'static str = "host_write_file";
    pub const FN_SHOW_NOTIFICATION: &'static str = "host_show_notification";
}

/// 宿主传给 WASM 插件的上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostContext {
    pub current_file: Option<String>,
    pub selected_text: Option<String>,
    pub work_dir: Option<String>,
    pub locale: String,
    #[serde(default)]
    pub extra: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 向后兼容类型别名
// ---------------------------------------------------------------------------

/// 按钮定义（向后兼容，新代码直接用 UiNode::button）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonDef {
    pub label: String,
    pub action: String,
    #[serde(default)]
    pub variant: String,
}

/// 信息字段（向后兼容，新代码直接用 UiNode::info）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoField {
    pub label: String,
    pub field: String,
}
