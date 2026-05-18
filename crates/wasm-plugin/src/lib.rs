//! WASM 插件 — DSL 计数器
//!
//! 编译: `cargo build --target wasm32-unknown-unknown --release`
//! 产物: `target/wasm32-unknown-unknown/release/wasm_plugin.wasm`
//!
//! 宿主通过 wasmi 加载后调用导出函数:
//!   - `plugin_get_manifest() -> i32`  返回 JSON 长度（写入共享 buffer）
//!   - `plugin_get_state()    -> i32`  返回 JSON 长度
//!   - `plugin_handle_action(ptr, len) -> i32`  执行动作，返回新 state JSON 长度
//!   - `buffer_ptr()  -> i32`  共享 buffer 起始地址
//!   - `buffer_len()  -> i32`  共享 buffer 长度

use serde::Serialize;
use wasm_plugin_types::*;

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
// JSON 序列化结构（仅状态，manifest 用共享类型）
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct StateJson {
    count: i32,
    max: i32,
    percentage: i32,
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
        let state = StateJson {
            count: COUNTER.count,
            max: COUNTER.max,
            percentage: COUNTER.percentage(),
        };
        let json = serde_json::to_string(&state).unwrap_or_default();
        write_to_buf(&json)
    }
}

#[no_mangle]
pub extern "C" fn plugin_handle_action(ptr: i32, len: i32) -> i32 {
    let action = unsafe { read_from_mem(ptr, len) };
    unsafe {
        match action.as_str() {
            "increment" => COUNTER.count = (COUNTER.count + 1).min(COUNTER.max),
            "decrement" => COUNTER.count = (COUNTER.count - 1).max(0),
            "reset" => COUNTER.count = 0,
            _ => {}
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
