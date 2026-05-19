//! copyright © ecdata.cn 2026 - present
//! 插件 UI Schema 类型定义
//! WASM/dylib 插件共用，宿主通用渲染器也用

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
// UI Schema（声明式 UI 描述）
// ---------------------------------------------------------------------------

/// UI 布局定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSchema {
    pub layout: String,
    #[serde(default)]
    pub children: Vec<UiNode>,
}

/// UI 节点 — 宿主通用渲染器根据 type 分发渲染
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UiNode {
    /// 单行文本展示
    #[serde(rename = "display")]
    Display { field: String, #[serde(default)] style: String },

    /// 进度条
    #[serde(rename = "progress")]
    Progress { field: String },

    /// 静态文本标签
    #[serde(rename = "label")]
    Label { text: String },

    /// 按钮行
    #[serde(rename = "button_row")]
    ButtonRow { buttons: Vec<ButtonDef> },

    /// 信息字段列表
    #[serde(rename = "info")]
    Info { fields: Vec<InfoField> },
}

/// 按钮定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonDef {
    pub label: String,
    pub action: String,
    #[serde(default)]
    pub variant: String,
}

/// 信息字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoField {
    pub label: String,
    pub field: String,
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