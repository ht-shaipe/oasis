//! WASM 插件共享类型定义
//!
//! 宿主和 WASM 插件共用，改一处两边同步。
//! 只依赖 serde + serde_json，兼容 wasm32-unknown-unknown。

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
    pub ui: UiSchema,
}

// ---------------------------------------------------------------------------
// UI Schema
// ---------------------------------------------------------------------------

/// UI 布局定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSchema {
    pub layout: String,
    pub children: Vec<UiNode>,
}

/// UI 节点 — 宿主通用渲染器根据 type 分发渲染
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UiNode {
    #[serde(rename = "display")]
    Display { field: String, style: String },

    #[serde(rename = "progress")]
    Progress { field: String },

    #[serde(rename = "label")]
    Label { text: String },

    #[serde(rename = "button_row")]
    ButtonRow { buttons: Vec<ButtonDef> },

    #[serde(rename = "info")]
    Info { fields: Vec<InfoField> },
}

/// 按钮定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonDef {
    pub label: String,
    pub action: String,
    pub variant: String,
}

/// 信息字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoField {
    pub label: String,
    pub field: String,
}
