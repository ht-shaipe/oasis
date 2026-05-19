//! WASM 插件共享类型定义
//!
//! 宿主和 WASM 插件共用，改一处两边同步。
//! 只依赖 serde + serde_json，兼容 wasm32-unknown-unknown。

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 插件清单
// ---------------------------------------------------------------------------

/// 插件清单（含 UI schema，ui 字段为 JSON 值，宿主反序列化为 plugin_sdk::UiSchema）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmManifest {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub description: String,
    pub version: String,
    /// JSON 值，宿主反序列化为 plugin_sdk::UiSchema
    pub ui: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 宿主环境 API（Host Imports）
// ---------------------------------------------------------------------------

/// 宿主注入给 WASM 插件的环境函数
///
/// WASM 插件通过 `extern "C"` 声明这些函数，直接调用获取宿主上下文。
/// 数据传递通过共享线性内存（ptr + len），JSON 序列化。
///
/// 用法示例（WASM 侧）：
/// ```rust
/// // 声明宿主函数
/// extern "C" {
///     fn host_get_context(ptr: i32, len: i32) -> i32;
///     fn host_log(ptr: i32, len: i32);
///     fn host_read_file(ptr: i32, len: i32) -> i32;
/// }
///
/// // 调用
/// let ctx_json = host_get_context(ptr, len);  // 获取宿主上下文
/// host_log(msg_ptr, msg_len);                 // 打印日志到宿主
/// let file_json = host_read_file(ptr, len);   // 读文件内容
/// ```
pub struct HostEnv;

impl HostEnv {
    /// 宿主环境模块名（WASM import module name）
    pub const MODULE_NAME: &'static str = "env";

    /// 宿主注入的函数名列表（文档用，代码里按名注册）
    pub const FN_GET_CONTEXT: &'static str = "host_get_context";
    pub const FN_LOG: &'static str = "host_log";
    pub const FN_READ_FILE: &'static str = "host_read_file";
    pub const FN_WRITE_FILE: &'static str = "host_write_file";
    pub const FN_SHOW_NOTIFICATION: &'static str = "host_show_notification";
}

// ---------------------------------------------------------------------------
// 宿主上下文数据结构
// ---------------------------------------------------------------------------

/// 宿主传给 WASM 插件的上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostContext {
    /// 当前打开的文件路径（如果有）
    pub current_file: Option<String>,
    /// 当前选中的文本
    pub selected_text: Option<String>,
    /// 当前工作目录
    pub work_dir: Option<String>,
    /// 应用语言
    pub locale: String,
    /// 自定义扩展数据（插件自由使用）
    #[serde(default)]
    pub extra: serde_json::Value,
}
