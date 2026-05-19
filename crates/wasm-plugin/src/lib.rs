//! WASM 插件 — DSL 计数器（Host Imports 版）
//!
//! 编译: `cargo build --target wasm32-unknown-unknown --release`
//! 产物: `target/wasm32-unknown-unknown/release/wasm_plugin.wasm`
//!
//! 插件直接调用宿主注入的函数获取上下文，不再靠 JSON action 传参。
//!
//! 导出函数（WASM → 宿主调用）:
//!   - `plugin_get_manifest() -> i32`
//!   - `plugin_get_state()    -> i32`
//!   - `plugin_handle_action(ptr, len) -> i32`
//!   - `buffer_ptr()  -> i32`
//!   - `buffer_len()  -> i32`
//!
//! 导入函数（宿主 → WASM 调用）:
//!   - `env::host_get_context(ptr, len) -> i32`   获取宿主上下文
//!   - `env::host_log(ptr, len)`                    打印日志
//!   - `env::host_read_file(ptr, len) -> i32`      读文件
//!   - `env::host_write_file(ptr, len, cptr, clen)` 写文件
//!   - `env::host_show_notification(ptr, len)`       显示通知

use wasm_plugin_types::*;

// ---------------------------------------------------------------------------
// 宿主函数声明（Host Imports）
// ---------------------------------------------------------------------------

extern "C" {
    /// 获取宿主上下文 JSON，写入共享 buffer，返回长度
    /// ptr/len: 请求参数（可传空字符串，未来可扩展过滤字段）
    fn host_get_context(ptr: i32, len: i32) -> i32;

    /// 打印日志到宿主控制台
    fn host_log(ptr: i32, len: i32);

    /// 读取文件内容，写入共享 buffer，返回长度
    /// ptr/len: 文件路径
    fn host_read_file(ptr: i32, len: i32) -> i32;

    /// 写入文件
    /// ptr/len: 文件路径, cptr/clen: 文件内容
    fn host_write_file(ptr: i32, len: i32, cptr: i32, clen: i32);

    /// 在宿主显示通知
    /// ptr/len: 通知文本
    fn host_show_notification(ptr: i32, len: i32);
}

// ---------------------------------------------------------------------------
// 共享线性内存 buffer
// ---------------------------------------------------------------------------

const BUF_CAPACITY: usize = 8192;
static mut OUTPUT_BUF: [u8; BUF_CAPACITY] = [0u8; BUF_CAPACITY];

fn write_to_buf(s: &str) -> i32 {
    let bytes = s.as_bytes();
    let len = bytes.len().min(BUF_CAPACITY);
    unsafe {
        OUTPUT_BUF[..len].copy_from_slice(&bytes[..len]);
        OUTPUT_BUF[len] = 0;
    }
    len as i32
}

unsafe fn read_from_mem(ptr: i32, len: i32) -> String {
    let ptr = ptr as usize;
    let len = len as usize;
    if ptr == 0 || len == 0 || len > BUF_CAPACITY {
        return String::new();
    }
    let slice = core::slice::from_raw_parts(ptr as *const u8, len);
    String::from_utf8_lossy(slice).into_owned()
}

/// 将字符串写入 buffer，返回 (ptr, len)
unsafe fn write_str_to_buf(s: &str) -> (i32, i32) {
    let len = write_to_buf(s);
    let ptr = OUTPUT_BUF.as_ptr() as i32;
    (ptr, len)
}

/// 调用宿主函数获取上下文
fn get_host_context() -> HostContext {
    unsafe {
        let (ptr, len) = write_str_to_buf(""); // 空请求 = 全量上下文
        let result_len = host_get_context(ptr, len);
        if result_len <= 0 {
            return HostContext {
                current_file: None,
                selected_text: None,
                work_dir: None,
                locale: "en".into(),
                extra: serde_json::Value::Null,
            };
        }
        let json = read_from_mem(OUTPUT_BUF.as_ptr() as i32, result_len);
        serde_json::from_str(&json).unwrap_or(HostContext {
            current_file: None,
            selected_text: None,
            work_dir: None,
            locale: "en".into(),
            extra: serde_json::Value::Null,
        })
    }
}

/// 通过宿主打印日志
fn call_host_log(msg: &str) {
    unsafe {
        let (ptr, len) = write_str_to_buf(msg);
        host_log(ptr, len);
    }
}

/// 通过宿主显示通知
fn call_host_notification(msg: &str) {
    unsafe {
        let (ptr, len) = write_str_to_buf(msg);
        host_show_notification(ptr, len);
    }
}

// ---------------------------------------------------------------------------
// 计数器状态
// ---------------------------------------------------------------------------

static mut COUNTER: Counter = Counter { count: 0, max: 100 };

struct Counter {
    count: i32,
    max: i32,
}

impl Counter {
    unsafe fn percentage(&self) -> i32 {
        if self.max > 0 {
            (self.count * 100 / self.max).max(0).min(100)
        } else {
            0
        }
    }
}

// ---------------------------------------------------------------------------
// 导出函数
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn plugin_get_manifest() -> i32 {
    let manifest = WasmManifest {
        id: "dsl_counter".into(),
        title: "DSL 计数器".into(),
        icon: "🔢".into(),
        description: "WASM 插件：声明式 DSL 计数器".into(),
        version: "1.0.0".into(),
        ui: UiSchema {
            layout: "column".into(),
            children: vec![
                UiNode::Display {
                    field: "count".into(),
                    style: "large_number".into(),
                },
                UiNode::Label {
                    text: "{count} / {max}".into(),
                },
                UiNode::Progress {
                    field: "percentage".into(),
                },
                UiNode::ButtonRow {
                    buttons: vec![
                        ButtonDef {
                            label: "➖".into(),
                            action: "decrement".into(),
                            variant: "secondary".into(),
                        },
                        ButtonDef {
                            label: "🔄".into(),
                            action: "reset".into(),
                            variant: "secondary".into(),
                        },
                        ButtonDef {
                            label: "➕".into(),
                            action: "increment".into(),
                            variant: "primary".into(),
                        },
                    ],
                },
                UiNode::Info {
                    fields: vec![
                        InfoField { label: "ID".into(), field: "id".into() },
                        InfoField { label: "版本".into(), field: "version".into() },
                        InfoField { label: "描述".into(), field: "description".into() },
                    ],
                },
            ],
        },
    };
    let json = serde_json::to_string(&manifest).unwrap_or_default();
    write_to_buf(&json)
}

#[no_mangle]
pub extern "C" fn plugin_get_state() -> i32 {
    unsafe {
        let state = serde_json::json!({
            "count": COUNTER.count,
            "max": COUNTER.max,
            "percentage": COUNTER.percentage(),
        });
        let json = serde_json::to_string(&state).unwrap_or_default();
        write_to_buf(&json)
    }
}

#[no_mangle]
pub extern "C" fn plugin_handle_action(ptr: i32, len: i32) -> i32 {
    let action = unsafe { read_from_mem(ptr, len) };

    // 演示：action 可以直接调用宿主函数获取上下文
    // 比如 "show_context" 动作：读取宿主上下文并通知
    if action == "show_context" {
        let ctx = get_host_context();
        let msg = format!(
            "📋 上下文: 文件={:?}, 选区={:?}, 工作目录={:?}, 语言={}",
            ctx.current_file, ctx.selected_text, ctx.work_dir, ctx.locale
        );
        call_host_notification(&msg);
        call_host_log(&msg);
        return plugin_get_state();
    }

    unsafe {
        match action.as_str() {
            "increment" => {
                COUNTER.count = (COUNTER.count + 1).min(COUNTER.max);
                call_host_log(&format!("计数+1 → {}", COUNTER.count));
            }
            "decrement" => {
                COUNTER.count = (COUNTER.count - 1).max(0);
                call_host_log(&format!("计数-1 → {}", COUNTER.count));
            }
            "reset" => {
                COUNTER.count = 0;
                call_host_log("计数已重置");
                call_host_notification("计数器已归零");
            }
            _ => {
                call_host_log(&format!("未知动作: {}", action));
            }
        }
    }
    plugin_get_state()
}

#[no_mangle]
pub extern "C" fn buffer_ptr() -> i32 {
    unsafe { OUTPUT_BUF.as_ptr() as i32 }
}

#[no_mangle]
pub extern "C" fn buffer_len() -> i32 {
    BUF_CAPACITY as i32
}
