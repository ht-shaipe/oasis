//! copyright © ecdata.cn 2026 - present
//! 插件 UI Schema 类型定义
//!
//! 设计原则：
//! - 通用 component schema：插件通过 `component` 字段声明要渲染的 gpui-component 组件
//! - 宿主渲染器按 component 名分发，未实现的组件优雅降级
//! - `props` 为自由 JSON，由渲染器按 component 类型解析
//! - `bind` 从插件 state 取值，`on_action` 触发回调

// 模块
mod manifest;
mod nodes;
mod props;
mod schema;
mod state;
mod template;

// 公开导出
pub use manifest::{ButtonDef, HostContext, HostEnv, InfoField, WasmManifest};
pub use nodes::UiNode;
pub use props::{prop_array, prop_bool, prop_i64, prop_str, prop_str_or};
pub use schema::UiSchema;
pub use state::{state_get, state_get_i64, state_get_str, state_interpolate};
pub use template::*;
