//! Toolbox Plugin — 工具箱插件（rlib + 手动注册）
//!
//! 所有工具通过 UiSchema 声明式描述 UI，由宿主通用渲染器统一渲染。
//! 状态以 JSON 存储在插件内，`handle_action` 处理所有用户操作。

mod state;
mod plugin;
mod tools;

use std::sync::Arc;
use plugin_sdk::Plugin;
use crate::plugin::ToolboxPlugin;

pub use state::*;

// ---------------------------------------------------------------------------
// 视图工厂导出（供宿主 init() 手动注册调用）
// ---------------------------------------------------------------------------

/// 创建 ToolboxPlugin 实例（返回 Arc<dyn Plugin>）
pub fn create_toolbox_plugin() -> Arc<dyn Plugin> {
    Arc::new(ToolboxPlugin::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbox_creation() {
        let plugin = ToolboxPlugin::new();
        assert_eq!(plugin.id(), "toolbox");
        let meta = plugin.meta();
        assert_eq!(meta.name, "工具箱");
    }

    #[test]
    fn test_home_schema() {
        let plugin = ToolboxPlugin::new();
        use plugin_sdk::Plugin;
        let schema = plugin.ui_schema();
        assert!(!schema.children.is_empty());
        for (i, child) in schema.children.iter().enumerate() {
            assert!(!child.component.is_empty(), "child {} has empty component", i);
        }
    }

    #[test]
    fn test_all_tool_schemas() {
        let plugin = ToolboxPlugin::new();
        use plugin_sdk::Plugin;
        let schema = plugin.ui_schema();
        assert!(!schema.children.is_empty());
    }

    #[test]
    fn test_demo_schema() {
        let schema = crate::tools::demo::schema_ui_schema_demo();
        assert!(!schema.children.is_empty());
    }
}
