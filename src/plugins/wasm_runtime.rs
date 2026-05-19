//! WASM 运行时 — 通用 wasmi 加载器（Host Imports 版）
//!
//! 宿主注入函数（Host Imports）:
//!   - `env::host_get_context`     — 获取宿主上下文（当前文件、选区等）
//!   - `env::host_log`             — 打印日志
//!   - `env::host_read_file`       — 读文件
//!   - `env::host_write_file`      — 写文件
//!   - `env::host_show_notification` — 显示通知
//!
//! WASM 插件通过 `extern "C"` 声明这些函数，直接调用获取宿主上下文。
//! 数据通过线性内存共享 buffer 传递（ptr + len），复杂数据 JSON 序列化。

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use wasmi::{Engine, Extern, Instance, Linker, Memory, Module, Store, Val};
use wasm_plugin_types::{HostContext, WasmManifest};

/// 共享 buffer 大小上限（与 WASM 侧 BUF_CAPACITY 一致）
const BUF_CAPACITY: usize = 8192;

// ---------------------------------------------------------------------------
// 宿主状态
// ---------------------------------------------------------------------------

/// 宿主上下文状态（注入给 WASM 插件的运行时数据）
#[derive(Debug, Clone)]
pub struct HostState {
    pub context: HostContext,
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            context: HostContext {
                current_file: None,
                selected_text: None,
                work_dir: std::env::current_dir().ok().map(|p| p.to_string_lossy().into()),
                locale: "en".into(),
                extra: serde_json::Value::Null,
            },
        }
    }
}

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
// WasmLoadedPlugin
// ---------------------------------------------------------------------------

/// 一个已加载的 WASM 插件实例
pub struct WasmLoadedPlugin {
    /// 插件清单（含 UI schema）
    pub manifest: WasmManifest,
    /// wasmi store（持有宿主状态）
    store: Store<HostState>,
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

        let host_state = HostState::default();
        let mut store = Store::new(&engine, host_state);

        // 创建 Linker 并注入宿主函数
        let mut linker = Linker::new(&engine);
        inject_host_functions(&mut store, &mut linker);

        let instance = linker
            .instantiate(&mut store, &module)
            .with_context(|| "实例化 WASM 模块失败（可能缺少宿主函数声明）")?
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

    /// 更新宿主上下文（WASM 下次调用 host_get_context 会拿到最新值）
    pub fn update_context(&mut self, ctx: HostContext) {
        self.store.data_mut().context = ctx;
    }

    /// 获取当前状态
    pub fn get_state(&mut self) -> Result<serde_json::Value> {
        let json = Self::read_json(
            &self.instance, &mut self.store, &self.memory,
            "plugin_get_state", &[],
        )?;
        Ok(serde_json::from_str(&json)
            .with_context(|| format!("解析插件状态 JSON 失败: {}", json))?)
    }

    /// 执行动作，返回新状态
    pub fn handle_action(&mut self, action: &str) -> Result<serde_json::Value> {
        let action_bytes = action.as_bytes();
        let buf_ptr = Self::call_i32_out(&self.instance, &mut self.store, "buffer_ptr", &[])?
            .context("buffer_ptr 返回值异常")? as usize;

        if action_bytes.len() > BUF_CAPACITY {
            anyhow::bail!("Action 数据过大: {} bytes (max {})", action_bytes.len(), BUF_CAPACITY);
        }

        self.memory.data_mut(&mut self.store)
            .get_mut(buf_ptr..buf_ptr + action_bytes.len())
            .map(|slice| slice.copy_from_slice(action_bytes))
            .context("写入 WASM 线性内存越界")?;

        let json = Self::read_json(
            &self.instance, &mut self.store, &self.memory,
            "plugin_handle_action",
            &[Val::I32(buf_ptr as i32), Val::I32(action_bytes.len() as i32)],
        )?;
        Ok(serde_json::from_str(&json)
            .with_context(|| format!("解析插件状态 JSON 失败: {}", json))?)
    }

    // ---- 内部辅助 ----

    fn get_func(instance: &Instance, store: &Store<HostState>, name: &str) -> Result<wasmi::Func> {
        instance
            .get_export(store, name)
            .and_then(Extern::into_func)
            .with_context(|| format!("WASM 导出函数 '{}' 不存在", name))
    }

    fn call_i32_out(
        instance: &Instance, store: &mut Store<HostState>, name: &str, args: &[Val],
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

    fn read_json(
        instance: &Instance, store: &mut Store<HostState>, memory: &Memory,
        func_name: &str, args: &[Val],
    ) -> Result<String> {
        let len = Self::call_i32_out(instance, store, func_name, args)?
            .context("WASM 函数返回值异常")? as usize;
        if len == 0 { return Ok(String::new()); }

        let buf_ptr = Self::call_i32_out(instance, store, "buffer_ptr", &[])?
            .context("buffer_ptr 返回值异常")? as usize;

        let mut buf = vec![0u8; len];
        memory.data(store).get(buf_ptr..buf_ptr + len)
            .map(|slice| buf.copy_from_slice(slice))
            .context("读取 WASM 线性内存越界")?;

        String::from_utf8(buf).context("WASM 返回数据不是有效 UTF-8")
    }
}

// ---------------------------------------------------------------------------
// 宿主函数注入
// ---------------------------------------------------------------------------

/// 向 Linker 注入宿主环境函数
///
/// 使用 wasmi `Func::wrap` 注册，闭包签名自动映射为 WASM 类型：
///   - `(Caller<'_, HostState>, i32, i32) -> i32` → `(i32, i32) -> i32`
///   - `(Caller<'_, HostState>, i32, i32)`         → `(i32, i32) -> ()`
fn inject_host_functions(store: &mut Store<HostState>, linker: &mut Linker<HostState>) {
    let module = wasm_plugin_types::HostEnv::MODULE_NAME;

    // host_get_context(ptr: i32, len: i32) -> i32
    // 将宿主上下文 JSON 写入 WASM 线性内存，返回长度
    linker
        .define(
            module,
            wasm_plugin_types::HostEnv::FN_GET_CONTEXT,
            wasmi::Func::wrap(&mut *store, host_get_context),
        )
        .expect("注入 host_get_context 失败");

    // host_log(ptr: i32, len: i32)
    linker
        .define(
            module,
            wasm_plugin_types::HostEnv::FN_LOG,
            wasmi::Func::wrap(&mut *store, host_log),
        )
        .expect("注入 host_log 失败");

    // host_read_file(ptr: i32, len: i32) -> i32
    linker
        .define(
            module,
            wasm_plugin_types::HostEnv::FN_READ_FILE,
            wasmi::Func::wrap(&mut *store, host_read_file),
        )
        .expect("注入 host_read_file 失败");

    // host_write_file(path_ptr: i32, path_len: i32, content_ptr: i32, content_len: i32)
    linker
        .define(
            module,
            wasm_plugin_types::HostEnv::FN_WRITE_FILE,
            wasmi::Func::wrap(&mut *store, host_write_file),
        )
        .expect("注入 host_write_file 失败");

    // host_show_notification(ptr: i32, len: i32)
    linker
        .define(
            module,
            wasm_plugin_types::HostEnv::FN_SHOW_NOTIFICATION,
            wasmi::Func::wrap(&mut *store, host_show_notification),
        )
        .expect("注入 host_show_notification 失败");

    tracing::info!("✅ 宿主环境函数注入完成（5 个）");
}

// ---------------------------------------------------------------------------
// 宿主函数实现
// ---------------------------------------------------------------------------

/// 获取宿主上下文，写入 WASM 线性内存
fn host_get_context(mut caller: wasmi::Caller<'_, HostState>, ptr: i32, _len: i32) -> i32 {
    let ctx = &caller.data().context;
    let json = match serde_json::to_string(ctx) {
        Ok(j) => j,
        Err(e) => {
            tracing::error!("序列化 HostContext 失败: {}", e);
            return 0;
        }
    };

    let json_bytes = json.as_bytes();
    let write_len = json_bytes.len().min(BUF_CAPACITY);

    let memory = match caller.get_export("memory").and_then(Extern::into_memory) {
        Some(m) => m,
        None => {
            tracing::error!("WASM 实例没有 memory 导出");
            return 0;
        }
    };

    let ptr_usize = ptr as usize;
    let data = memory.data_mut(&mut caller);
    if let Some(slice) = data.get_mut(ptr_usize..ptr_usize + write_len) {
        slice.copy_from_slice(&json_bytes[..write_len]);
        write_len as i32
    } else {
        tracing::error!("host_get_context: 写入越界 ptr={} len={}", ptr_usize, write_len);
        0
    }
}

/// 日志
fn host_log(caller: wasmi::Caller<'_, HostState>, ptr: i32, len: i32) {
    let msg = read_wasm_string(&caller, ptr, len);
    tracing::info!("🔌 [WASM] {}", msg);
}

/// 读文件
fn host_read_file(mut caller: wasmi::Caller<'_, HostState>, ptr: i32, len: i32) -> i32 {
    let path = read_wasm_string(&caller, ptr, len);
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("WASM 读取文件失败 {}: {}", path, e);
            return -1;
        }
    };

    let content_bytes = content.as_bytes();
    let write_len = content_bytes.len().min(BUF_CAPACITY);

    let memory = match caller.get_export("memory").and_then(Extern::into_memory) {
        Some(m) => m,
        None => return -1,
    };

    // 写到 WASM 的 buffer_ptr 位置
    // 先找到 buffer_ptr：从 WASM 导出获取
    let buf_ptr = match caller.get_export("buffer_ptr").and_then(Extern::into_func) {
        Some(f) => {
            let mut out = [Val::I32(0)];
            match f.call(&mut caller, &[], &mut out) {
                Ok(()) => match out[0] {
                    Val::I32(v) => v as usize,
                    _ => return -1,
                },
                Err(_) => return -1,
            }
        }
        None => return -1,
    };

    let data = memory.data_mut(&mut caller);
    if let Some(slice) = data.get_mut(buf_ptr..buf_ptr + write_len) {
        slice.copy_from_slice(&content_bytes[..write_len]);
        write_len as i32
    } else {
        -1
    }
}

/// 写文件
fn host_write_file(caller: wasmi::Caller<'_, HostState>, pptr: i32, plen: i32, cptr: i32, clen: i32) {
    let path = read_wasm_string(&caller, pptr, plen);
    let content = read_wasm_string(&caller, cptr, clen);
    match std::fs::write(&path, &content) {
        Ok(()) => tracing::info!("WASM 写入文件成功: {}", path),
        Err(e) => tracing::error!("WASM 写入文件失败 {}: {}", path, e),
    }
}

/// 通知
fn host_show_notification(caller: wasmi::Caller<'_, HostState>, ptr: i32, len: i32) {
    let msg = read_wasm_string(&caller, ptr, len);
    tracing::info!("🔔 [WASM 通知] {}", msg);
    // TODO: 接入 GPUI 通知系统
}

// ---------------------------------------------------------------------------
// 辅助
// ---------------------------------------------------------------------------

/// 从 WASM 线性内存读取字符串
fn read_wasm_string(caller: &wasmi::Caller<'_, HostState>, ptr: i32, len: i32) -> String {
    let memory = match caller.get_export("memory").and_then(Extern::into_memory) {
        Some(m) => m,
        None => return String::new(),
    };
    let data = memory.data(caller);
    let ptr_usize = ptr as usize;
    let len_usize = len as usize;

    if len_usize == 0 || ptr_usize + len_usize > data.len() {
        return String::new();
    }

    match std::str::from_utf8(&data[ptr_usize..ptr_usize + len_usize]) {
        Ok(s) => s.to_string(),
        Err(_) => String::new(),
    }
}
