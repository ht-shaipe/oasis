//! UI DSL - 声明式 UI 描述系统
//!
//! 插件返回 UI 描述 JSON，宿主通用渲染器负责渲染

pub mod widgets;
pub mod renderer;

pub use widgets::*;
pub use renderer::*;
