pub mod calculator;
pub mod dsl_counter;
pub mod notepad;
pub mod plugin_window;
pub mod ui_dsl;
pub mod wasm_content;
pub mod wasm_example;
pub mod wasm_host;
pub mod wasm_loader;
pub mod wasm_plugin_system;

use std::collections::HashMap;

use gpui::{AnyView, App, AppContext as _, Entity, Global, Window};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// PluginEntry — inventory 提交类型
// ---------------------------------------------------------------------------

/// 插件注册入口，由每个插件通过 `inventory::submit!` 提交
pub struct PluginEntry {
    pub id: &'static str,
    pub manifest_toml: &'static str,
    pub icon_svg: &'static str,
    pub create_view: fn(&mut Window, &mut App) -> AnyView,
}

inventory::collect!(PluginEntry);

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
pub trait Plugin: gpui::Render + 'static {
    /// 插件唯一标识，须与 manifest.toml 中的 id 一致
    fn plugin_id() -> &'static str;
    /// 构造插件视图
    fn new(window: &mut Window, cx: &mut gpui::Context<Self>) -> Self;
}

// ---------------------------------------------------------------------------
// RegisteredPlugin — 运行时存储
// ---------------------------------------------------------------------------

/// 运行时已注册的插件信息
pub struct RegisteredPlugin {
    pub manifest: PluginManifest,
    pub icon_svg: &'static str,
    pub icon_emoji: Option<String>,
    pub create_view: fn(&mut Window, &mut App) -> AnyView,
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
                            icon_svg: entry.icon_svg,
                            icon_emoji: None,
                            create_view: entry.create_view,
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
        // 先读取信息，释放对 cx 的借用
        let (title, window_width, window_height, create_view) = {
            let registry = cx.global::<Self>();
            if registry.open_windows.contains_key(id) {
                tracing::info!("Plugin '{}' already open, focusing...", id);
                return;
            }
            let Some(registered) = registry.plugins.iter().find(|p| p.manifest.id == id) else {
                tracing::error!("Plugin '{}' not found in registry", id);
                return;
            };
            (
                registered.manifest.display_name.clone(),
                registered.manifest.window_width,
                registered.manifest.window_height,
                registered.create_view,
            )
        };

        // 创建插件内容视图
        let content = create_view(window, cx);

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
            icon_svg: "",
            icon_emoji: Some(icon_emoji),
            create_view,
            is_wasm: true,
        };

        cx.global_mut::<Self>().plugins.push(plugin);
        tracing::info!("✅ 注册 WASM 插件: {}", id);
    }
}

/// 注册内置 WASM 插件
pub fn register_builtin_wasm_plugins(cx: &mut App) {
    // 注册计数器插件（原有）
    PluginRegistry::register_wasm_plugin(
        cx,
        "counter".to_string(),
        "计数器".to_string(),
        "🔢".to_string(),
        "一个简单的计数器插件".to_string(),
        crate::plugins::wasm_content::create_counter_view,
    );

    // 注册 DSL 计数器插件
    PluginRegistry::register_wasm_plugin(
        cx,
        "dsl_counter".to_string(),
        "DSL 计数器".to_string(),
        "🎨".to_string(),
        "声明式 UI DSL 示例插件".to_string(),
        crate::plugins::dsl_counter::create_dsl_counter_view,
    );
}
