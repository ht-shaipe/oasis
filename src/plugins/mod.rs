
pub mod dyn_plugin_view;
pub mod plugin_window;
pub mod wasm_example;
pub mod wasm_host;
pub mod wasm_loader;
pub mod wasm_plugin_system;
pub mod wasm_plugin_view;
pub mod wasm_runtime;

// Aster (md-editor-plugin) inventory submit
use md_editor_plugin::AsterView;

use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyView, App, AppContext as _, Entity, Global, Window};
use serde::Deserialize;
use plugin_sdk::Plugin;
use libloading::{Library, Symbol};

// ---------------------------------------------------------------------------
// PluginEntry — inventory 提交类型（内置插件）
// ---------------------------------------------------------------------------

/// 插件注册入口，由每个内置插件通过 `inventory::submit!` 提交
pub struct PluginEntry {
    pub id: &'static str,
    pub manifest_toml: &'static str,
    pub icon_svg: &'static str,
    pub create_view: fn(&mut Window, &mut App) -> AnyView,
}

inventory::collect!(PluginEntry);

inventory::submit! {
    PluginEntry {
        id: "aster",
        manifest_toml: r#"
[plugin]
id = "aster"
display_name = "Aster Editor"
description = "Markdown WYSIWYG editor"
icon = "aster.svg"
window_width = 1000.0
window_height = 700.0
"#,
        icon_svg: "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'><path d='M17 3a2.85 2.83 0 1 1 4 4L7.5 18.5 2 20l1.5-5.5Z'/></svg>",
        create_view: |window: &mut Window, cx: &mut App| -> AnyView {
            cx.new(|cx| AsterView::new(window, cx)).into()
        },
    }
}

// ---------------------------------------------------------------------------
// PluginManifest — TOML 解析目标
// ---------------------------------------------------------------------------

fn default_icon() -> String {
    "icon.svg".to_string()
}

fn default_width() -> f32 {
    400.0
}

fn default_height() -> f32 {
    300.0
}

/// 插件清单，从 manifest.toml 解析
#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default = "default_width")]
    pub window_width: f32,
    #[serde(default = "default_height")]
    pub window_height: f32,
}

/// TOML 文件顶层包装（对应 `[plugin]` 段）
#[derive(Debug, Deserialize)]
struct PluginManifestFile {
    plugin: PluginManifest,
}

// ---------------------------------------------------------------------------
// Plugin trait
// ---------------------------------------------------------------------------

/// 所有插件必须实现的 trait
// ---------------------------------------------------------------------------
// RegisteredPlugin — 运行时存储
// ---------------------------------------------------------------------------

/// 运行时已注册的插件信息
pub struct RegisteredPlugin {
    pub manifest: PluginManifest,
    pub icon_svg: String,
    pub icon_emoji: Option<String>,
    /// 静态（inventory）插件的视图工厂
    pub create_view: Option<fn(&mut Window, &mut App) -> AnyView>,
    /// dylib 插件实例（cdylib 加载，保持 Library 句柄活跃）
    pub dyn_plugin: Option<(Library, Arc<dyn Plugin>)>,
    pub is_wasm: bool,
}

// ---------------------------------------------------------------------------
// PluginRegistry — 全局状态
// ---------------------------------------------------------------------------

/// 插件注册中心，作为 GPUI Global 存储
pub struct PluginRegistry {
    pub plugins: Vec<RegisteredPlugin>,
    pub open_windows: HashMap<String, Entity<plugin_window::PluginWindow>>,
}

impl Global for PluginRegistry {}

impl PluginRegistry {
    /// 初始化注册中心：遍历 inventory 提交项，解析 manifest 并收集
    pub fn init(cx: &mut App) {
        let mut plugins = Vec::new();
        for entry in inventory::iter::<PluginEntry> {
            match toml::from_str::<PluginManifestFile>(entry.manifest_toml) {
                Ok(file) => {
                    let manifest = file.plugin;
                    if manifest.id == entry.id {
                        plugins.push(RegisteredPlugin {
                            manifest,
                            icon_svg: entry.icon_svg.to_string(),
                            icon_emoji: None,
                            create_view: Some(entry.create_view),
                            dyn_plugin: None,
                            is_wasm: false,
                        });
                    } else {
                        tracing::error!(
                            "Plugin id mismatch: manifest.id={} != entry.id={}",
                            manifest.id,
                            entry.id
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse manifest for plugin '{}': {}", entry.id, e);
                }
            }
        }

        // 扫描并加载 plugins/dylib/ 目录下的 cdylib 插件
        Self::scan_dylib_plugins(&mut plugins);

        cx.set_global(Self {
            plugins,
            open_windows: HashMap::new(),
        });
        tracing::info!(
            "🔌 PluginRegistry initialized with {} plugins",
            cx.global::<Self>().plugins.len()
        );
    }

    /// 打开插件窗口
    pub fn open_plugin(id: &str, window: &mut Window, cx: &mut App) {
        // 读取插件信息（结束后 cx 借用释放）
        let (title, window_width, window_height, content) = {
            let registry = cx.global::<Self>();
            if registry.open_windows.contains_key(id) {
                tracing::info!("Plugin '{}' already open, focusing...", id);
                return;
            }
            let Some(registered) = registry.plugins.iter().find(|p| p.manifest.id == id) else {
                tracing::error!("Plugin '{}' not found in registry", id);
                return;
            };

            // 收集所有需要的数据，cx 借用仅持续到这里
            let title = registered.manifest.display_name.clone();
            let window_width = registered.manifest.window_width;
            let window_height = registered.manifest.window_height;
            let plugin = registered.dyn_plugin.as_ref().map(|(lib, p)| (lib, Arc::clone(p)));
            let create_view = registered.create_view;

            (title, window_width, window_height, (plugin, create_view))
        }; // cx 借用在这里完全释放

        // 现在 cx 可自由借用
        let content: AnyView = if let Some((_lib, plugin)) = content.0 {
            let view = dyn_plugin_view::DynPluginView::create_from_plugin(plugin);
            cx.new(|_| view).into()
        } else if let Some(create_view) = content.1 {
            create_view(window, cx)
        } else {
            tracing::error!("Plugin '{}' has no create_view and no dyn_plugin", id);
            return;
        };

        // 创建 PluginWindow 实体
        let plugin_window = cx.new(|_| plugin_window::PluginWindow::new(
            id,
            title,
            (window_width, window_height),
            content,
        ));

        // 存入 open_windows
        cx.global_mut::<Self>()
            .open_windows
            .insert(id.to_string(), plugin_window);
        cx.refresh_windows();
    }

    /// 关闭插件窗口
    pub fn close_plugin(id: &str, cx: &mut App) {
        if cx.global_mut::<Self>().open_windows.remove(id).is_some() {
            tracing::info!("Plugin '{}' window closed", id);
        }
        cx.refresh_windows();
    }

    /// 检查插件窗口是否已打开
    pub fn is_open(id: &str, cx: &App) -> bool {
        cx.global::<Self>().open_windows.contains_key(id)
    }

    /// 获取已注册插件的引用
    pub fn get_plugin<'a>(id: &str, cx: &'a App) -> Option<&'a RegisteredPlugin> {
        cx.global::<Self>().plugins.iter().find(|p| p.manifest.id == id)
    }

    /// 注册 WASM 插件
    pub fn register_wasm_plugin(
        cx: &mut App,
        id: String,
        display_name: String,
        icon_emoji: String,
        description: String,
        create_view: fn(&mut Window, &mut App) -> AnyView,
    ) {
        let manifest = PluginManifest {
            id: id.clone(),
            display_name,
            description,
            icon: format!("{}.svg", id),
            window_width: 400.0,
            window_height: 500.0,
        };

        let plugin = RegisteredPlugin {
            manifest,
            icon_svg: String::new(),
            icon_emoji: Some(icon_emoji),
            create_view: Some(create_view),
            dyn_plugin: None,
            is_wasm: true,
        };

        cx.global_mut::<Self>().plugins.push(plugin);
        tracing::info!("✅ 注册 WASM 插件: {}", id);
    }

    /// 注册 dylib 插件（由 scan_dylib_plugins 调用，或直接调用）
    #[allow(dead_code)]
    fn register_dylib_plugin(
        &mut self,
        id: String,
        display_name: String,
        icon: String,
        lib: Library,
        plugin: Arc<dyn Plugin>,
    ) {
        let manifest = PluginManifest {
            id: id.clone(),
            display_name,
            description: plugin.meta().description.clone(),
            icon,
            window_width: 400.0,
            window_height: 500.0,
        };
        self.plugins.push(RegisteredPlugin {
            manifest,
            icon_svg: String::new(),
            icon_emoji: None,
            create_view: None,
            dyn_plugin: Some((lib, plugin)),
            is_wasm: false,
        });
        tracing::info!("✅ 注册 dylib 插件: {}", id);
    }

    /// 扫描 plugins/ 目录下的子目录，每个子目录是一个插件
    /// 目录结构：plugins/{plugin_id}/{plugin_id}.dylib + icon.svg
    fn scan_dylib_plugins(plugins: &mut Vec<RegisteredPlugin>) {
        let base_dir = std::env::current_dir().unwrap_or_default();
        let plugins_dir = base_dir.join("plugins");
        if !plugins_dir.exists() {
            tracing::info!("🔌 插件目录不存在: {:?}", plugins_dir);
            return;
        }

        let entries = match std::fs::read_dir(&plugins_dir) {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("❌ 无法读取插件目录 {:?}: {}", plugins_dir, e);
                return;
            }
        };

        for entry in entries.flatten() {
            let dir_path = entry.path();
            if !dir_path.is_dir() {
                continue;
            }
            // 跳过 wasm 目录（WASM 插件走另一套加载流程）
            if dir_path.file_name().is_some_and(|n| n == "wasm") {
                continue;
            }

            let plugin_id = match dir_path.file_name().and_then(|n| n.to_str()) {
                Some(id) => id.to_string(),
                None => continue,
            };

            // 查找 dylib 文件：{plugin_id}.dylib (macOS) 或 {plugin_id}.so (Linux)
            let dylib_path = Self::find_dylib(&dir_path, &plugin_id);
            let Some(dylib_path) = dylib_path else {
                tracing::debug!("🔌 目录 {:?} 中未找到 dylib，跳过", dir_path);
                continue;
            };

            // 读取 icon.svg（可选）
            let icon_svg_path = dir_path.join("icon.svg");
            let icon_svg = if icon_svg_path.exists() {
                match std::fs::read_to_string(&icon_svg_path) {
                    Ok(svg) => svg,
                    Err(e) => {
                        tracing::warn!("⚠️ 读取 {:?} 失败: {}", icon_svg_path, e);
                        String::new()
                    }
                }
            } else {
                String::new()
            };

            unsafe {
                let lib = match Library::new(&dylib_path) {
                    Ok(lib) => lib,
                    Err(e) => {
                        tracing::error!("❌ 加载 dylib {:?} 失败: {}", dylib_path, e);
                        continue;
                    }
                };

                let create: Symbol<unsafe fn() -> Arc<dyn Plugin>> = match lib.get(b"plugin_entry") {
                    Ok(sym) => sym,
                    Err(e) => {
                        tracing::error!("❌ {:?} 无 plugin_entry 符号: {}", dylib_path, e);
                        continue;
                    }
                };

                let plugin = create();
                let meta = plugin.meta();

                plugins.push(RegisteredPlugin {
                    manifest: PluginManifest {
                        id: plugin_id.clone(),
                        display_name: meta.name.clone(),
                        description: meta.description.clone(),
                        icon: format!("{}.svg", plugin_id),
                        window_width: 400.0,
                        window_height: 500.0,
                    },
                    icon_svg,
                    icon_emoji: Some(meta.icon.clone()),
                    create_view: None,
                    dyn_plugin: Some((lib, plugin)),
                    is_wasm: false,
                });
                tracing::info!("✅ 加载 dylib 插件: {}", plugin_id);
            }
        }
    }

    /// 在插件目录中查找 dylib 文件
    fn find_dylib(dir: &std::path::Path, plugin_id: &str) -> Option<std::path::PathBuf> {
        // macOS: lib{plugin_id}.dylib
        let mac_path = dir.join(format!("lib{}.dylib", plugin_id));
        if mac_path.exists() {
            return Some(mac_path);
        }
        // Linux: lib{plugin_id}.so
        let linux_path = dir.join(format!("lib{}.so", plugin_id));
        if linux_path.exists() {
            return Some(linux_path);
        }
        None
    }
}

/// 注册内置 WASM 插件
pub fn register_builtin_wasm_plugins(cx: &mut App) {
    // DSL 计数器 — 通用 WASM 加载，宿主无插件特定代码
    PluginRegistry::register_wasm_plugin(
        cx,
        "dsl_counter".to_string(),
        "DSL 计数器".to_string(),
        "🔢".to_string(),
        "WASM 插件：声明式 DSL 计数器".to_string(),
        crate::plugins::wasm_plugin_view::WasmPluginView::create_view,
    );
}
