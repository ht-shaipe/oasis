//! copyright © ecdata.cn 2026 - present
//! Plugin trait 核心定义，无 UI 类型

use crate::PluginMeta;

// 从 ui-schema 导入 UI 类型 + 辅助函数
pub use ui_schema::{
    UiSchema, UiNode, ButtonDef, InfoField,
    prop_str, prop_str_or, prop_i64, prop_bool, prop_array,
    state_get, state_get_str, state_get_i64, state_interpolate,
};

// ---------------------------------------------------------------------------
// Plugin Trait（插件必须实现的核心接口）
// ---------------------------------------------------------------------------

/// 插件接口 — 所有内置/dylib 插件必须实现
pub trait Plugin: Send + Sync + 'static {
    /// 插件唯一标识
    fn id(&self) -> &str;

    /// 插件清单
    fn meta(&self) -> PluginMeta;

    /// 当前状态（JSON，供宿主通用渲染器渲染）
    fn state(&self) -> serde_json::Value;

    /// 处理用户动作，返回更新后的状态
    fn handle_action(&self, action: &str, params: serde_json::Value) -> serde_json::Value;

    /// UI schema（声明式布局）
    fn ui_schema(&self) -> UiSchema;

    /// 插件被加载时调用（可选）
    #[allow(unused_variables)]
    fn on_load(&self) {}

    /// 插件被卸载时调用（可选）
    #[allow(unused_variables)]
    fn on_unload(&self) {}
}
