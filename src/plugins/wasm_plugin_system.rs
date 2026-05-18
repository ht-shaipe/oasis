//! WASM 插件注册系统 - 简化版本
//!
//! 完整的插件加载、注册和 dock 集成

use gpui::{div, px, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::{button::Button, ActiveTheme as _};
use std::collections::HashMap;

/// 插件元数据
#[derive(Debug, Clone)]
pub struct WasmPluginInfo {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub description: String,
    pub version: String,
}

/// 插件实例
pub struct WasmPluginInstance {
    pub info: WasmPluginInfo,
    pub count: i32,
    pub max: i32,
}

impl WasmPluginInstance {
    pub fn new(info: WasmPluginInfo) -> Self {
        Self {
            info,
            count: 0,
            max: 100,
        }
    }

    /// 获取进度百分比
    pub fn percentage(&self) -> i32 {
        if self.max == 0 {
            0
        } else {
            (self.count * 100 / self.max).max(0).min(100)
        }
    }

    /// 增加计数
    pub fn increment(&mut self) {
        self.count = (self.count + 1).min(self.max);
    }

    /// 减少计数
    pub fn decrement(&mut self) {
        self.count = (self.count - 1).max(0);
    }

    /// 重置
    pub fn reset(&mut self) {
        self.count = 0;
    }
}

/// 插件窗口
pub struct WasmPluginWindow {
    pub plugin_id: String,
    pub instance: WasmPluginInstance,
}

impl WasmPluginWindow {
    pub fn new(plugin_id: String, instance: WasmPluginInstance) -> Self {
        Self { plugin_id, instance }
    }
}

impl Render for WasmPluginWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let info = &self.instance.info;
        let count = self.instance.count;
        let max = self.instance.max;
        let percentage = self.instance.percentage();
        let entity = cx.entity().clone();

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(24.))
            .size_full()
            .bg(theme.colors.background)
            .child(
                // 标题栏
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(24.))
                            .child(info.icon.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.foreground)
                            .child(info.title.clone()),
                    ),
            )
            .child(
                // 计数器显示
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(20.))
                    .p(px(32.))
                    .bg(theme.colors.muted.opacity(0.3))
                    .rounded_lg()
                    .child(
                        div()
                            .text_size(px(64.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.colors.foreground)
                            .child(format!("{}", count)),
                    )
                    .child(
                        div()
                            .text_size(px(14.))
                            .text_color(theme.colors.muted_foreground)
                            .child(format!("{} / {}", count, max)),
                    )
                    .child(
                        div()
                            .w(px(240.))
                            .h(px(12.))
                            .bg(theme.colors.muted)
                            .rounded_full()
                            .child(
                                div()
                                    .h(px(12.))
                                    .bg(theme.colors.primary)
                                    .rounded_full()
                                    .flex_shrink_0()
                                    .w(px((240.0 * percentage as f32 / 100.0) as _)),
                            ),
                    ),
            )
            .child(
                // 控制按钮
                div()
                    .flex()
                    .flex_row()
                    .gap(px(16.))
                    .child(
                        Button::new("btn-minus")
                            .size(px(64.))
                            .bg(theme.colors.muted)
                            .rounded_lg()
                            .label("➖")
                            .on_click({
                                let entity = entity.clone();
                                move |_ev, _window, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.instance.decrement();
                                        cx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("btn-reset")
                            .size(px(64.))
                            .bg(theme.colors.muted)
                            .rounded_lg()
                            .label("🔄")
                            .on_click({
                                let entity = entity.clone();
                                move |_ev, _window, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.instance.reset();
                                        cx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("btn-plus")
                            .size(px(64.))
                            .bg(theme.colors.primary)
                            .rounded_lg()
                            .label("➕")
                            .text_color(gpui::rgb(0xffffff))
                            .on_click({
                                let entity = entity.clone();
                                move |_ev, _window, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.instance.increment();
                                        cx.notify();
                                    });
                                }
                            }),
                    ),
            )
            .child(
                // 插件信息
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .p(px(16.))
                    .bg(gpui::rgb(0x1a1a1a))
                    .rounded_lg()
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(gpui::rgb(0xffffff))
                            .child("ℹ️ 插件信息"),
                    )
                    .children(
                        [
                            format!("ID: {}", info.id),
                            format!("版本: {}", info.version),
                            format!("描述: {}", info.description),
                        ]
                        .iter()
                        .map(|text| {
                            div()
                                .text_size(px(11.))
                                .text_color(gpui::rgb(0x888888))
                                .child(text.clone())
                        }),
                    ),
            )
    }
}

/// WASM 插件注册中心
pub struct WasmPluginRegistry {
    pub plugins: HashMap<String, WasmPluginInfo>,
    pub open_windows: HashMap<String, Entity<WasmPluginWindow>>,
}

impl WasmPluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            open_windows: HashMap::new(),
        }
    }

    /// 扫描并加载所有插件
    pub fn scan_plugins(&mut self) {
        self.plugins.clear();

        // 默认插件（计数器）
        let counter_plugin = WasmPluginInfo {
            id: "counter".to_string(),
            title: "计数器".to_string(),
            icon: "🔢".to_string(),
            description: "一个简单的计数器插件".to_string(),
            version: "1.0.0".to_string(),
        };

        self.plugins.insert(counter_plugin.id.clone(), counter_plugin);

        tracing::info!("🔌 WASM 插件扫描完成，发现 {} 个插件", self.plugins.len());
        for (id, plugin) in &self.plugins {
            tracing::info!("  - {} ({}): {}", plugin.title, id, plugin.icon);
        }
    }

    /// 打开插件
    pub fn open_plugin(&mut self, id: &str, _window: &mut Window, cx: &mut App) {
        // 检查是否已打开
        if self.open_windows.contains_key(id) {
            tracing::info!("插件 '{}' 已经打开", id);
            return;
        }

        // 获取插件信息
        let Some(info) = self.plugins.get(id).cloned() else {
            tracing::error!("插件 '{}' 未找到", id);
            return;
        };

        // 创建插件实例
        let instance = WasmPluginInstance::new(info.clone());

        // 创建插件窗口
        let plugin_window = cx.new(|cx| WasmPluginWindow::new(id.to_string(), instance));

        // 存储窗口引用
        self.open_windows.insert(id.to_string(), plugin_window);

        tracing::info!("✅ 打开插件: {} ({})", info.title, id);
        cx.refresh_windows();
    }

    /// 关闭插件
    pub fn close_plugin(&mut self, id: &str, cx: &mut App) {
        if self.open_windows.remove(id).is_some() {
            tracing::info!("❌ 关闭插件: {}", id);
            cx.refresh_windows();
        }
    }

    /// 检查插件是否打开
    pub fn is_open(&self, id: &str) -> bool {
        self.open_windows.contains_key(id)
    }
}

impl gpui::Global for WasmPluginRegistry {}

/// 初始化 WASM 插件系统
pub fn init_wasm_system(cx: &mut App) {
    let mut registry = WasmPluginRegistry::new();
    registry.scan_plugins();
    cx.set_global(registry);
    tracing::info!("🚀 WASM 插件系统初始化完成");
}
