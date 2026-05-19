//! copyright © ecdata.cn 2026 - present
//! 挂件 trait — 动态库插件的核心接口

use serde::{Deserialize, Serialize};

/// 挂件清单 — 动态库导出的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetManifest {
    /// 唯一标识
    pub id: String,
    /// 显示名称
    pub display_name: String,
    /// 描述
    #[serde(default)]
    pub description: String,
    /// 图标 emoji
    #[serde(default)]
    pub icon_emoji: String,
    /// SVG 图标内容
    #[serde(default)]
    pub icon_svg: String,
    /// 窗口宽度
    #[serde(default = "default_width")]
    pub window_width: f32,
    /// 窗口高度
    #[serde(default = "default_height")]
    pub window_height: f32,
}

fn default_width() -> f32 { 400.0 }
fn default_height() -> f32 { 300.0 }

/// 挂件注册入口 — 每个动态库插件通过 `inventory::submit!` 提交
pub struct WidgetEntry {
    /// 挂件唯一标识
    pub id: &'static str,
    /// 清单 JSON
    pub manifest_json: &'static str,
    /// SVG 图标
    pub icon_svg: &'static str,
    /// 视图工厂函数
    pub create_view: fn(&mut gpui::Window, &mut gpui::App) -> gpui::AnyView,
}

inventory::collect!(WidgetEntry);

/// 挂件 trait — 所有动态库插件必须实现
///
/// 比起 `Plugin` trait，Widget trait 侧重于 UI 组件的动态加载：
/// - 实现者编译为 `cdylib`，运行时由宿主通过 `libloading` 加载
/// - 通过 FFI 导出 `widget_factory` 和 `widget_manifest` 两个 C 符号
/// - 宿主调用 `widget_factory` 获取 `AnyView`，在 PluginWindow 中渲染
pub trait Widget: gpui::Render + 'static {
    /// 挂件唯一标识
    fn widget_id() -> &'static str;

    /// 挂件清单
    fn manifest() -> WidgetManifest;

    /// 构造挂件视图（简化版本，不需要 window 参数）
    ///
    /// 注意：由于 FFI 限制，此方法不再接受 window 参数。
    /// 如果需要访问窗口，可以通过 cx.window() 或 cx.window_mut()。
    fn new(cx: &mut gpui::Context<Self>) -> Self;
}
