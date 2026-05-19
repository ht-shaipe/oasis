//! Widget 动态加载器 — 通过 libloading 扫描 plugins/dylib/ 目录
//!
//! 加载 cdylib 格式的挂件插件，读取 `widget_manifest_json` 和 `widget_factory` 符号，
//! 注册到 PluginRegistry。创建视图时使用 dylib_factory 字段直接调用工厂函数指针。

use std::path::{Path, PathBuf};

use gpui::App;
use libloading::{Library, Symbol};
use plugin_sdk::{WIDGET_FACTORY_SYMBOL, WIDGET_MANIFEST_SYMBOL};

use crate::plugins::{DylibFactoryFn, PluginManifest, PluginRegistry, RegisteredPlugin};

// ---------------------------------------------------------------------------
// LoadedWidget — 持有动态库句柄
// ---------------------------------------------------------------------------

/// 已加载的动态库挂件
pub struct LoadedWidget {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub icon_emoji: String,
    pub window_width: f32,
    pub window_height: f32,
    pub _library: Library, // 持有 Library 防止卸载
    pub factory: DylibFactoryFn,
}

// ---------------------------------------------------------------------------
// 全局存储 — 防止 Library 被 drop
// ---------------------------------------------------------------------------

/// 全局存储已加载的 dylib 挂件（Library 不能 drop，否则符号失效）
pub struct DylibWidgetStore {
    pub widgets: Vec<LoadedWidget>,
}

impl gpui::Global for DylibWidgetStore {}

// ---------------------------------------------------------------------------
// 扫描与加载
// ---------------------------------------------------------------------------

/// 扫描目录下所有 dylib 文件，尝试加载为挂件插件
fn scan_dylib_widgets(dir: &Path) -> Vec<LoadedWidget> {
    let mut widgets = Vec::new();

    let Ok(entries) = std::fs::read_dir(dir) else {
        tracing::info!("📂 dylib 目录不存在，跳过: {}", dir.display());
        return widgets;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // macOS: .dylib, Linux: .so, Windows: .dll
        if ext != "dylib" && ext != "so" && ext != "dll" {
            continue;
        }

        match load_widget(&path) {
            Ok(widget) => {
                tracing::info!("✅ 加载 dylib 挂件: {} ({})", widget.id, path.display());
                widgets.push(widget);
            }
            Err(e) => {
                tracing::error!("❌ 加载 dylib 挂件失败 {}: {}", path.display(), e);
            }
        }
    }

    widgets
}

/// 从 manifest JSON 解析的字段
#[derive(Debug, Clone, serde::Deserialize)]
struct WidgetManifestData {
    id: String,
    display_name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    icon_emoji: String,
    #[serde(default)]
    icon_svg: String,
    #[serde(default = "default_width")]
    window_width: f32,
    #[serde(default = "default_height")]
    window_height: f32,
}

fn default_width() -> f32 { 400.0 }
fn default_height() -> f32 { 300.0 }

/// 加载单个 cdylib 挂件
fn load_widget(path: &PathBuf) -> Result<LoadedWidget, Box<dyn std::error::Error>> {
    unsafe {
        let library = Library::new(path)?;

        // 读取清单 JSON
        let manifest_fn: Symbol<unsafe extern "C" fn() -> *const std::ffi::c_char> =
            library.get(WIDGET_MANIFEST_SYMBOL)?;
        let manifest_ptr = manifest_fn();
        let manifest_cstr = std::ffi::CStr::from_ptr(manifest_ptr);
        let manifest_json = manifest_cstr.to_str()?;
        let manifest: WidgetManifestData = serde_json::from_str(manifest_json)?;

        // 读取工厂函数
        let factory: Symbol<DylibFactoryFn> = library.get(WIDGET_FACTORY_SYMBOL)?;
        let factory_fn = *factory.into_raw();

        Ok(LoadedWidget {
            id: manifest.id.clone(),
            display_name: manifest.display_name.clone(),
            description: manifest.description.clone(),
            icon_emoji: manifest.icon_emoji.clone(),
            window_width: manifest.window_width,
            window_height: manifest.window_height,
            _library: library,
            factory: factory_fn,
        })
    }
}

// ---------------------------------------------------------------------------
// 注册到 PluginRegistry
// ---------------------------------------------------------------------------

/// 扫描并注册所有 dylib 挂件
pub fn register_dylib_widgets(cx: &mut App) {
    let dylib_dir = PathBuf::from("plugins/dylib");
    let widgets = scan_dylib_widgets(&dylib_dir);

    for widget in &widgets {
        let plugin_manifest = PluginManifest {
            id: widget.id.clone(),
            display_name: widget.display_name.clone(),
            description: widget.description.clone(),
            icon: String::new(),
            window_width: widget.window_width,
            window_height: widget.window_height,
        };

        let registered = RegisteredPlugin {
            manifest: plugin_manifest,
            icon_svg: "",
            icon_emoji: Some(widget.icon_emoji.clone()),
            create_view: |_window, _cx| unreachable!(), // dylib 插件不走此路径
            dylib_factory: Some(widget.factory),
            is_wasm: false,
        };

        cx.global_mut::<PluginRegistry>().plugins.push(registered);
        tracing::info!("🔌 注册 dylib 挂件: {}", widget.id);
    }

    // 存储 Library 句柄防止卸载
    cx.set_global(DylibWidgetStore { widgets });
}
