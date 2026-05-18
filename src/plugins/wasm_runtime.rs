//! WASM 运行时 — 通用 wasmi 加载器
//!
//! 负责：
//!   1. 扫描 `plugins/wasm/` 目录下的 `.wasm` 文件
//!   2. 用 wasmi 编译并实例化
//!   3. 调用通用导出函数：`plugin_get_manifest()`, `plugin_get_state()`,
//!      `plugin_handle_action(ptr, len)`
//!   4. 从线性内存共享 buffer 读取 JSON 返回值
//!
//! 宿主不包含任何插件特定逻辑，完全由 WASM 插件的 manifest/ui schema 驱动。
//! 类型定义来自共享 crate `wasm_plugin_types`，与 WASM 插件侧一致。

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use wasmi::{Engine, Extern, Instance, Linker, Memory, Module, Store, Val};
use wasm_plugin_types::WasmManifest;

// ---------------------------------------------------------------------------
// 扫描
// ---------------------------------------------------------------------------

/// 扫描 plugins/wasm/ 目录，返回所有 .wasm 文件路径
pub fn scan_wasm_plugins(base_dir: &Path) -> Vec<PathBuf> {
    let wasm_dir = base_dir.join("plugins").join("wasm");
    if !wasm_dir.exists() {
        tracing::info!("🔌 WASM 插件目录不存在: {:?}", wasm_dir);
        return Vec::new();
    }

    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&wasm_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                tracing::info!("📦 发现 WASM 插件: {}", path.display());
                result.push(path);
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// WasmLoadedPlugin — 通用加载器
// ---------------------------------------------------------------------------

/// 一个已加载的 WASM 插件实例
pub struct WasmLoadedPlugin {
    /// 插件清单（含 UI schema）
    pub manifest: WasmManifest,
    /// wasmi store
    store: Store<()>,
    /// wasmi 实例
    instance: Instance,
    /// 线性内存
    memory: Memory,
    /// .wasm 文件路径
    pub wasm_path: PathBuf,
}

impl WasmLoadedPlugin {
    /// 从 .wasm 文件加载插件
    pub fn load(wasm_path: &Path) -> Result<Self> {
        let wasm_bytes = std::fs::read(wasm_path)
            .with_context(|| format!("读取 WASM 文件失败: {}", wasm_path.display()))?;

        let engine = Engine::default();
        let module = Module::new(&engine, &wasm_bytes)
            .with_context(|| format!("编译 WASM 模块失败: {}", wasm_path.display()))?;

        // Linker 必须用同一个 engine 实例
        let linker = Linker::new(&engine);
        let mut store = Store::new(&engine, ());

        let instance = linker
            .instantiate(&mut store, &module)
            .with_context(|| "实例化 WASM 模块失败")?
            .start(&mut store)
            .with_context(|| "启动 WASM 实例失败")?;

        let memory = instance
            .get_memory(&store, "memory")
            .context("WASM 模块未导出 'memory'")?;

        // 读取清单
        let manifest_json = Self::read_json(&instance, &mut store, &memory, "plugin_get_manifest", &[])?;
        let manifest: WasmManifest = serde_json::from_str(&manifest_json)
            .with_context(|| format!("解析插件清单 JSON 失败: {}", manifest_json))?;

        tracing::info!("✅ WASM 插件加载成功: {} ({})", manifest.title, manifest.id);

        Ok(Self {
            manifest,
            store,
            instance,
            memory,
            wasm_path: wasm_path.to_path_buf(),
        })
    }

    /// 获取当前状态（动态 JSON，不同插件结构不同）
    pub fn get_state(&mut self) -> Result<serde_json::Value> {
        let json = Self::read_json(
            &self.instance,
            &mut self.store,
            &self.memory,
            "plugin_get_state",
            &[],
        )?;
        Ok(serde_json::from_str(&json)
            .with_context(|| format!("解析插件状态 JSON 失败: {}", json))?)
    }

    /// 执行动作，返回新状态
    pub fn handle_action(&mut self, action: &str) -> Result<serde_json::Value> {
        // 将 action name 写入 WASM 线性内存的共享 buffer
        let action_bytes = action.as_bytes();
        let buf_ptr = Self::call_i32_out(&self.instance, &mut self.store, "buffer_ptr", &[])?
            .context("buffer_ptr 返回值异常")? as usize;

        self.memory.data_mut(&mut self.store)
            .get_mut(buf_ptr..buf_ptr + action_bytes.len())
            .map(|slice| slice.copy_from_slice(action_bytes))
            .context("写入 WASM 线性内存越界")?;

        let json = Self::read_json(
            &self.instance,
            &mut self.store,
            &self.memory,
            "plugin_handle_action",
            &[Val::I32(buf_ptr as i32), Val::I32(action_bytes.len() as i32)],
        )?;
        Ok(serde_json::from_str(&json)
            .with_context(|| format!("解析插件状态 JSON 失败: {}", json))?)
    }

    // ---- 内部辅助 ----

    fn get_func(
        instance: &Instance,
        store: &Store<()>,
        name: &str,
    ) -> Result<wasmi::Func> {
        instance
            .get_export(store, name)
            .and_then(Extern::into_func)
            .with_context(|| format!("WASM 导出函数 '{}' 不存在", name))
    }

    fn call_i32_out(
        instance: &Instance,
        store: &mut Store<()>,
        name: &str,
        args: &[Val],
    ) -> Result<Option<i32>> {
        let func = Self::get_func(instance, store, name)?;
        let mut output = [Val::I32(0)];
        func.call(store, args, &mut output)
            .with_context(|| format!("调用 WASM 函数 '{}' 失败", name))?;
        Ok(match output[0] {
            Val::I32(v) => Some(v),
            _ => None,
        })
    }

    /// 调用返回 JSON 的 WASM 函数，从共享 buffer 读取结果
    fn read_json(
        instance: &Instance,
        store: &mut Store<()>,
        memory: &Memory,
        func_name: &str,
        args: &[Val],
    ) -> Result<String> {
        let len = Self::call_i32_out(instance, store, func_name, args)?
            .context("WASM 函数返回值异常")? as usize;

        if len == 0 {
            return Ok(String::new());
        }

        let buf_ptr = Self::call_i32_out(instance, store, "buffer_ptr", &[])?
            .context("buffer_ptr 返回值异常")? as usize;

        let mut buf = vec![0u8; len];
        memory.data(store).get(buf_ptr..buf_ptr + len)
            .map(|slice| buf.copy_from_slice(slice))
            .context("读取 WASM 线性内存越界")?;

        String::from_utf8(buf).context("WASM 返回数据不是有效 UTF-8")
    }
}
