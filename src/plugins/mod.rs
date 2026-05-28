pub mod dyn_plugin_view;
pub(crate) mod plugin_render;
pub mod plugin_window;
pub mod wasm_example;
pub mod wasm_host;
pub mod wasm_loader;
pub mod wasm_plugin_system;
pub mod wasm_plugin_view;
pub mod wasm_runtime;

use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyView, App, AppContext as _, Entity, Global, Window};
use libloading::{Library, Symbol};
use plugin_sdk::Plugin;
use serde::Deserialize;

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
    pub icon_svg_path: Option<String>,
    pub icon_emoji: Option<String>,
    /// 静态（inventory/手动注册）插件的视图工厂
    pub create_view: Option<fn(&mut Window, &mut App) -> AnyView>,
    /// dylib 插件实例（cdylib 加载，保持 Library 句柄活跃）
    pub dyn_plugin: Option<(Library, Arc<dyn Plugin>)>,
    /// rlib 插件实例（静态链接，无需 Library 句柄）
    pub rlib_plugin: Option<Arc<dyn Plugin>>,
    pub is_wasm: bool,
    /// 子进程插件：可执行文件路径（相对于 plugins/ 目录或绝对路径）
    pub subprocess_exec: Option<String>,
    /// 子进程句柄（启动后持有）
    pub subprocess: Option<std::process::Child>,
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
                            icon_svg_path: None,
                            icon_emoji: None,
                            create_view: Some(entry.create_view),
                            dyn_plugin: None,
                            rlib_plugin: None,
                            is_wasm: false,
                            subprocess_exec: None,
                            subprocess: None,
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

        // 扫描并加载 plugins/ 目录下的插件（子进程/dylib）
        if std::env::var("OASIS_DISABLE_DYLIB_PLUGINS").ok().as_deref() == Some("1") {
            tracing::warn!("Skipping dylib plugin scan (OASIS_DISABLE_DYLIB_PLUGINS=1)");
        } else {
            Self::scan_dylib_plugins(&mut plugins);
        }

        // 手动注册 rlib 插件（静态链接，共享 gpui 全局状态）
        // 仅当 plugins/ 目录中未发现同名子进程版本时才注册 rlib 版本
        let has_subprocess_md = plugins.iter().any(|p| p.manifest.id == "md-editor-plugin");
        if !has_subprocess_md {
            plugins.push(RegisteredPlugin {
                manifest: PluginManifest {
                    id: "md-editor-plugin".to_string(),
                    display_name: "Markdown 编辑器".to_string(),
                    description: "Aster 风格 Markdown 编辑器".to_string(),
                    icon: "md-editor-plugin.svg".to_string(),
                    window_width: 900.0,
                    window_height: 700.0,
                },
                icon_svg: String::new(),
                icon_svg_path: None,
                icon_emoji: Some("📝".to_string()),
                create_view: Some(md_editor_plugin::create_aster_view),
                dyn_plugin: None,
                            rlib_plugin: None,
                is_wasm: false,
                subprocess_exec: None,
                subprocess: None,
            });
            tracing::info!("✅ 注册 rlib 插件: md-editor-plugin");
        } else {
            tracing::info!("⏭️  rlib md-editor-plugin 跳过（子进程版本已注册）");
        }

        // Toolbox 插件（rlib + Arc<dyn Plugin>，UiSchema 渲染）
        let has_subprocess_toolbox = plugins.iter().any(|p| p.manifest.id == "toolbox");
        if !has_subprocess_toolbox {
            let toolbox_plugin = toolbox_plugin::create_toolbox_plugin();
            plugins.push(RegisteredPlugin {
                manifest: PluginManifest {
                    id: "toolbox".to_string(),
                    display_name: "工具箱".to_string(),
                    description: "实用工具集：CSV 统计/分割、批量重命名、网络扫描等".to_string(),
                    icon: "toolbox-plugin.svg".to_string(),
                    window_width: 900.0,
                    window_height: 700.0,
                },
                icon_svg: String::new(),
                icon_svg_path: None,
                icon_emoji: Some("🧰".to_string()),
                create_view: None,
                dyn_plugin: None,
                is_wasm: false,
                subprocess_exec: None,
                subprocess: None,
                rlib_plugin: Some(toolbox_plugin),
            });
            tracing::info!("✅ 注册 rlib 插件: toolbox");
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
        tracing::info!("🧪 open_plugin called: {}", id);
        let (
            title,
            window_width,
            window_height,
            create_view_fn,
            dyn_plugin_ref,
            rlib_plugin_ref,
            subprocess_exec,
        ) = {
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
                registered.dyn_plugin.as_ref().map(|(lib, p)| (lib, Arc::clone(p))),
                registered.rlib_plugin.clone(),
                registered.subprocess_exec.clone(),
            )
        };

        // 优先级：subprocess > create_view > dyn_plugin
        // 子进程模式：独立进程，自有 gpui 窗口，不嵌入宿主
        if let Some(exec_path) = subprocess_exec {
            tracing::info!("🚀 启动子进程插件 '{}' from: {}", id, exec_path);
            match std::process::Command::new(&exec_path)
                .spawn()
            {
                Ok(child) => {
                    // 存储子进程句柄，以便后续关闭
                    cx.global_mut::<Self>()
                        .plugins
                        .iter_mut()
                        .find(|p| p.manifest.id == id)
                        .map(|p| p.subprocess = Some(child));
                    tracing::info!("✅ 子进程插件 '{}' 已启动", id);
                    // 子进程自己管理窗口，宿主无需创建 PluginWindow
                    return;
                }
                Err(e) => {
                    tracing::error!("❌ 启动子进程插件 '{}' 失败: {}", id, e);
                    return;
                }
            }
        }

        // 内嵌模式：create_view > rlib_plugin > dyn_plugin
        let content: AnyView = if let Some(create_view) = create_view_fn {
            tracing::info!("Creating plugin '{}' from create_view", id);
            create_view(window, cx)
        } else if let Some(plugin) = rlib_plugin_ref {
            // rlib 插件：静态链接，无 Library 句柄
            tracing::info!("Creating plugin '{}' from rlib_plugin (DynPluginView)", id);
            cx.new(|_cx| dyn_plugin_view::DynPluginView::create_from_plugin(plugin))
                .into()
        } else if let Some((_, plugin)) = dyn_plugin_ref {
            // dylib 插件：保持 Library 句柄
            tracing::info!("Creating plugin '{}' from DynPluginView", id);
            cx.new(|_cx| dyn_plugin_view::DynPluginView::create_from_plugin(plugin))
                .into()
        } else {
            // 测试模式：没有实际插件时，用静态 UiSchema 显示占位视图
            tracing::info!("🧪 测试模式：为 '{}' 创建静态测试视图", id);
            let test_schema = ui_schema::UiSchema::flex_col()
                .gap(16)
                .child(
                    ui_schema::UiNode::new("display")
                        .prop("style", serde_json::json!("icon-large"))
                        .prop("text", serde_json::json!("🧰")),
                )
                .child(
                    ui_schema::UiNode::label(id)
                        .prop("size", serde_json::json!(18))
                        .prop("weight", serde_json::json!("semibold")),
                )
                .child(
                    ui_schema::UiNode::label("插件视图加载中...".to_string())
                        .prop("style", serde_json::json!("muted")),
                );
            let test_state = serde_json::json!({});
            cx.new(|_cx| {
                dyn_plugin_view::TestPluginView {
                    schema: test_schema,
                    state: test_state,
                }
            }).into()
        };

        tracing::info!("🧪 Creating PluginWindow entity for '{}'", id);
        let plugin_window = cx.new(|_| {
            tracing::info!("🧪 PluginWindow::new for '{}'", id);
            plugin_window::PluginWindow::new(id, title, (window_width, window_height), content)
        });
        tracing::info!("🧪 PluginWindow created, inserting into open_windows");

        cx.global_mut::<Self>()
            .open_windows
            .insert(id.to_string(), plugin_window);
        tracing::info!("🧪 Calling refresh_windows");
        cx.refresh_windows();
        tracing::info!("🧪 Plugin '{}' window opened successfully", id);
    }

    /// 关闭插件窗口
    pub fn close_plugin(id: &str, cx: &mut App) {
        // 关闭内嵌窗口
        if cx.global_mut::<Self>().open_windows.remove(id).is_some() {
            tracing::info!("Plugin '{}' window closed", id);
        }

        // 终止子进程
        let registry = cx.global_mut::<Self>();
        if let Some(registered) = registry.plugins.iter_mut().find(|p| p.manifest.id == id) {
            if let Some(ref mut child) = registered.subprocess {
                let _ = child.kill();
                let _ = child.wait(); // 回收僵尸进程
                tracing::info!("Plugin '{}' subprocess killed", id);
            }
            registered.subprocess = None;
        }

        cx.refresh_windows();
    }

    /// 检查插件窗口是否已打开
    pub fn is_open(id: &str, cx: &App) -> bool {
        cx.global::<Self>().open_windows.contains_key(id)
    }

    /// 获取已注册插件的引用
    pub fn get_plugin<'a>(id: &str, cx: &'a App) -> Option<&'a RegisteredPlugin> {
        cx.global::<Self>()
            .plugins
            .iter()
            .find(|p| p.manifest.id == id)
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
            icon_svg_path: None,
            icon_emoji: Some(icon_emoji),
            create_view: Some(create_view),
            dyn_plugin: None,
                            rlib_plugin: None,
            is_wasm: true,
            subprocess_exec: None,
            subprocess: None,
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
            icon_svg_path: None,
            icon_emoji: None,
            create_view: None,
            dyn_plugin: Some((lib, plugin)),
            is_wasm: false,
            subprocess_exec: None,
            subprocess: None,
            rlib_plugin: None,
        });
        tracing::info!("✅ 注册 dylib 插件: {}", id);
    }

    /// 扫描 plugins/ 目录下的子目录，每个子目录是一个插件
    /// 目录结构：plugins/{plugin_id}/
    ///   - dylib 模式：lib{plugin_id}.dylib + manifest.toml + icon.svg
    ///   - 子进程模式：{plugin_id}(可执行文件) + manifest.toml + icon.svg
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

            // 读取 manifest.toml（可选）
            let manifest = Self::read_manifest(&dir_path, &plugin_id);

            // 读取 icon.svg（可选）
            let icon_svg_path = dir_path.join("icon.svg");
            let icon_svg = if icon_svg_path.exists() {
                    std::fs::read_to_string(&icon_svg_path).unwrap_or_default()
                } else {
                    String::new()
                };
                let icon_svg_path_str = if icon_svg_path.exists() {
                    Some(icon_svg_path.to_string_lossy().to_string())
                } else {
                    None
                };

            // 优先检测子进程可执行文件
            let exec_path = Self::find_executable(&dir_path, &plugin_id);
            if exec_path.is_some() {
                let manifest = manifest.unwrap_or_else(|| PluginManifest {
                    id: plugin_id.clone(),
                    display_name: plugin_id.clone(),
                    description: String::new(),
                    icon: "icon.svg".to_string(),
                    window_width: 800.0,
                    window_height: 600.0,
                });
                plugins.push(RegisteredPlugin {
                    manifest,
                    icon_svg,
                    icon_svg_path: icon_svg_path_str.clone(),
                    icon_emoji: None,
                    create_view: None,
                    dyn_plugin: None,
                            rlib_plugin: None,
                    is_wasm: false,
                    subprocess_exec: exec_path.map(|p| p.to_string_lossy().to_string()),
                    subprocess: None,
                });
                tracing::info!("✅ 发现子进程插件: {}", plugin_id);
                continue;
            }

            // 其次检测 dylib 文件
            let dylib_path = Self::find_dylib(&dir_path, &plugin_id);
            let Some(dylib_path) = dylib_path else {
                tracing::debug!("🔌 目录 {:?} 中未找到 dylib 或可执行文件，跳过", dir_path);
                continue;
            };

            unsafe {
                let lib = match Library::new(&dylib_path) {
                    Ok(lib) => lib,
                    Err(e) => {
                        tracing::error!("❌ 加载 dylib {:?} 失败: {}", dylib_path, e);
                        continue;
                    }
                };

                // Plugin trait + UiSchema 模式
                let create: Symbol<unsafe fn() -> Arc<dyn Plugin>> = match lib.get(b"plugin_entry")
                {
                    Ok(sym) => sym,
                    Err(e) => {
                        tracing::error!("❌ {:?} 无 plugin_entry 符号: {}", dylib_path, e);
                        continue;
                    }
                };

                let plugin = create();
                let meta = plugin.meta();

                let manifest = manifest.unwrap_or_else(|| PluginManifest {
                    id: plugin_id.clone(),
                    display_name: meta.name.clone(),
                    description: meta.description.clone(),
                    icon: format!("{}.svg", plugin_id),
                    window_width: 400.0,
                    window_height: 500.0,
                });

                plugins.push(RegisteredPlugin {
                    manifest,
                    icon_svg,
                    icon_svg_path: icon_svg_path_str,
                    icon_emoji: Some(meta.icon.clone()),
                    create_view: None,
                    dyn_plugin: Some((lib, plugin)),
                    is_wasm: false,
                    subprocess_exec: None,
                    subprocess: None,
                    rlib_plugin: None,
                });
                tracing::info!("✅ 加载 dylib 插件: {}", plugin_id);
            }
        }
    }

    /// 在插件目录中查找 dylib 文件
    fn find_dylib(dir: &std::path::Path, plugin_id: &str) -> Option<std::path::PathBuf> {
        // macOS: lib{plugin_id}.dylib
        let mac_path = dir.join(format!("{}.dylib", plugin_id));
        if mac_path.exists() {
            return Some(mac_path);
        }
        // Linux: lib{plugin_id}.so
        let linux_path = dir.join(format!("{}.so", plugin_id));
        if linux_path.exists() {
            return Some(linux_path);
        }
        None
    }

    /// 在插件目录中查找可执行文件（子进程模式）
    fn find_executable(dir: &std::path::Path, plugin_id: &str) -> Option<std::path::PathBuf> {
        let exec_path = dir.join(plugin_id);
        if exec_path.exists() {
            // 验证是否可执行
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = std::fs::metadata(&exec_path) {
                    if meta.permissions().mode() & 0o111 != 0 {
                        return Some(exec_path);
                    }
                }
            }
            #[cfg(not(unix))]
            {
                // Windows: 检查 .exe
                let win_path = dir.join(format!("{}.exe", plugin_id));
                if win_path.exists() {
                    return Some(win_path);
                }
            }
        }
        None
    }

    /// 从插件目录读取 manifest.toml
    fn read_manifest(dir: &std::path::Path, plugin_id: &str) -> Option<PluginManifest> {
        let manifest_path = dir.join("manifest.toml");
        if !manifest_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&manifest_path).ok()?;
        let file: PluginManifestFile = toml::from_str(&content).ok()?;
        if file.plugin.id != plugin_id {
            tracing::warn!(
                "⚠️ manifest id '{}' 与目录名 '{}' 不匹配",
                file.plugin.id, plugin_id
            );
        }
        Some(file.plugin)
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
