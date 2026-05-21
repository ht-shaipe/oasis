//! copyright © ecdata.cn 2026 - present
//! 插件 SDK — Trait 接口定义，无 gpui 依赖
//! created shaipe by 2026-05-19 11:42

mod meta;
pub use meta::PluginMeta;

mod plugin;
pub use plugin::{
    Plugin, UiSchema, UiNode, ButtonDef, InfoField,
    prop_str, prop_str_or, prop_i64, prop_bool, prop_array,
    state_get, state_get_str, state_get_i64, state_interpolate,
};

mod error;
pub use error::PluginError;

/// 插件入口函数类型（dylib 导出，供宿主 libloading 调用）
pub type PluginFactoryFn = fn() -> Box<dyn Plugin>;
