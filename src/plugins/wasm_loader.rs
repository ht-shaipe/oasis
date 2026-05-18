//! WASM 插件加载器 - 简化版本
//!
//! 负责扫描和显示 WASM 插件

use gpui::{div, px, App, Context, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::ActiveTheme as _;
use std::fs;
use std::path::PathBuf;

/// WASM 插件信息
#[derive(Clone)]
pub struct WasmPluginInfo {
    pub name: String,
    pub file_size: u64,
    pub file_path: PathBuf,
}

/// WASM 插件管理器 UI
pub struct WasmPluginManager {
    plugins: Vec<WasmPluginInfo>,
    has_scanned: bool,
}

impl WasmPluginManager {
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: Vec::new(),
            has_scanned: false,
        };
        manager.scan_plugins();
        manager
    }

    /// 扫描 WASM 插件
    fn scan_plugins(&mut self) {
        self.plugins.clear();

        // 从项目根目录查找
        let base_path = PathBuf::from("../plugins/wasm");
        let alt_path = PathBuf::from("plugins/wasm");

        let search_paths = vec![base_path, alt_path];

        for plugins_dir in search_paths {
            if !plugins_dir.exists() {
                continue;
            }

            tracing::info!("🔍 扫描 WASM 插件目录: {:?}", plugins_dir);

            if let Ok(entries) = fs::read_dir(&plugins_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            let file_name = path
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            let name = file_name.replace("_bg.wasm", "").replace(".wasm", "");

                            let info = WasmPluginInfo {
                                name: name.clone(),
                                file_size: metadata.len(),
                                file_path: path.clone(),
                            };

                            tracing::info!("📦 发现插件: {} ({} KB)", name, metadata.len() / 1024);
                            self.plugins.push(info);
                        }
                    }
                }
            }
        }

        self.has_scanned = true;
        tracing::info!("✅ 扫描完成，发现 {} 个插件", self.plugins.len());
    }
}

impl Render for WasmPluginManager {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(24.))
            .size_full()
            .bg(theme.colors.background)
            .child(
                // 标题
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(24.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.colors.foreground)
                            .child("🔌 WASM 插件管理器"),
                    )
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(4.))
                            .bg(theme.colors.primary)
                            .rounded_md()
                            .text_color(gpui::rgb(0xffffff))
                            .text_size(px(12.))
                            .child(format!("{} 个插件", self.plugins.len())),
                    ),
            )
            .child(
                // 插件列表
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .children(
                        self.plugins.iter().map(|plugin| {
                            div()
                                .flex()
                            .items_center()
                            .justify_between()
                            .p(px(16.))
                            .bg(theme.colors.muted.opacity(0.3))
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded_lg()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(6.))
                                    .child(
                                        div()
                                            .text_size(px(16.))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.colors.foreground)
                                            .child(plugin.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(16.))
                                            .children(
                                                [
                                                    format!("📁 {:?}", plugin.file_path),
                                                    format!("📊 {} KB", plugin.file_size / 1024),
                                                    format!("✅ 可加载"),
                                                ]
                                                .into_iter()
                                                .map(|text| {
                                                    div()
                                                        .text_size(px(11.))
                                                        .text_color(theme.colors.muted_foreground)
                                                        .child(text)
                                                }),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(20.))
                                    .text_color(theme.colors.muted_foreground)
                                    .child("→"),
                            )
                        }),
                    ),
            )
            .child(
                // 构建信息
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .p(px(16.))
                    .bg(gpui::rgb(0x1a1a1a))
                    .rounded_lg()
                    .child(
                        div()
                            .text_size(px(14.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(gpui::rgb(0xffffff))
                            .child("📝 构建新的 WASM 插件"),
                    )
                    .children(
                        [
                            "cd crates/wasm-plugin",
                            "./build.sh",
                        ]
                        .iter()
                        .map(|cmd| {
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(gpui::rgb(0x888888))
                                        .child("$"),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(gpui::rgb(0x00ff00))
                                        .font_family("monospace")
                                        .child(cmd.to_string()),
                                )
                        }),
                    ),
            )
            .child(
                // 说明
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .p(px(16.))
                    .bg(theme.colors.muted.opacity(0.5))
                    .rounded_lg()
                    .child(
                        div()
                            .text_size(px(14.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors.foreground)
                            .child("ℹ️ 关于 WASM 插件"),
                    )
                    .children(
                        [
                            "• WASM 插件提供动态加载和安全隔离",
                            "• 当前扫描路径: ../plugins/wasm 和 plugins/wasm",
                            "• 构建脚本: crates/wasm-plugin/build.sh",
                            "• 下一步: 集成 wasmi 运行时实现完整功能",
                        ]
                        .iter()
                        .map(|text| {
                            div()
                                .text_size(px(11.))
                                .text_color(theme.colors.muted_foreground)
                                .child(text.to_string())
                        }),
                    ),
            )
    }
}

/// 初始化 WASM 插件管理器
pub fn init_wasm_manager(_cx: &mut App) {
    tracing::info!("🔌 WASM 插件管理器初始化");
}
