//! copyright © ecdata.cn 2026 - present
//! Plugin trait + UI Schema 定义

use serde::{Deserialize, Serialize};

use crate::PluginMeta;

// ---------------------------------------------------------------------------
// UI Schema（声明式 UI 描述，供宿主通用渲染器使用）
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
// Plugin Trait（插件必须实现的核心接口）
// ---------------------------------------------------------------------------

/// 插件接口 — 所有内置/dylib 插件必须实现
///
/// 宿主通过此 trait 与插件交互：
/// - 获取插件元数据（清单）
/// - 获取当前状态（JSON，供通用渲染器渲染）
/// - 处理用户动作（按钮点击等）
pub trait Plugin: Send + Sync + 'static {
    /// 插件唯一标识
    fn id(&self) -> &str;

    /// 插件清单
    fn meta(&self) -> PluginMeta;

    /// 当前状态（JSON，供宿主通用渲染器渲染）
    fn state(&self) -> serde_json::Value;

    /// 处理用户动作，返回更新后的状态
    ///
    /// - `action`: 动作标识（如 "digit:7", "op:+", "clear"）
    /// - `params`: 动作参数（JSON 对象）
    /// 返回：更新后的完整状态 JSON
    ///
    /// 注意：实现应使用内部可变性（RefCell）来修改状态，
    /// 因为 trait 要求 `&self`（dyn-safe）
    fn handle_action(&self, action: &str, params: serde_json::Value) -> serde_json::Value;

    /// UI schema（声明式布局，供宿主通用渲染器渲染）
    fn ui_schema(&self) -> UiSchema;

    /// 插件被加载时调用（可选）
    #[allow(unused_variables)]
    fn on_load(&self) {}

    /// 插件被卸载时调用（可选）
    #[allow(unused_variables)]
    fn on_unload(&self) {}
}
