//! Plugin IPC Protocol — 宿主与插件子进程之间的通信协议
//!
//! 传输层：Unix Domain Socket（macOS/Linux）或 Named Pipe（Windows）
//! 序列化：JSON（每行一条消息，newline-delimited JSON）

use serde::{Deserialize, Serialize};

// ─── 通用信封 ───────────────────────────────────────────────────────────────

/// IPC 消息信封
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// 消息 ID（请求-响应配对）
    pub id: u64,
    /// 消息类型
    pub kind: MessageKind,
    /// JSON 编码的 payload
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageKind {
    // ─── 宿主 → 插件 ─────────────────────────────
    /// 启动插件（传递 socket 路径、工作目录等）
    Init,
    /// 打开/激活插件窗口
    Open,
    /// 关闭插件窗口
    Close,
    /// 宿主退出，通知插件终止
    Shutdown,
    /// 通用 action 调用
    Action,
    /// 请求数据
    Query,

    // ─── 插件 → 宿主 ─────────────────────────────
    /// 插件初始化完成
    Ready,
    /// 插件状态变更通知
    Notify,
    /// 查询响应
    Response,
    /// 插件请求宿主服务（如文件访问、通知等）
    HostCall,
    /// 插件退出
    Exit,
}

// ─── 宿主 → 插件消息 ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitPayload {
    /// IPC socket 路径（插件连接用）
    pub socket_path: String,
    /// 插件工作目录
    pub work_dir: String,
    /// 宿主提供的配置
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPayload {
    /// 窗口标题
    pub title: String,
    /// 窗口尺寸
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPayload {
    /// action 名称
    pub action: String,
    /// action 参数
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPayload {
    /// 查询名称
    pub query: String,
    /// 查询参数
    pub params: serde_json::Value,
}

// ─── 插件 → 宿主消息 ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyPayload {
    /// 插件 ID
    pub plugin_id: String,
    /// 插件版本
    pub version: String,
    /// 插件能力描述
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyPayload {
    /// 通知类型
    pub event: String,
    /// 通知数据
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePayload {
    /// 是否成功
    pub ok: bool,
    /// 响应数据
    pub data: serde_json::Value,
    /// 错误信息（ok=false 时）
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostCallPayload {
    /// 宿主服务名称（如 "fs.read", "fs.write", "notify"）
    pub service: String,
    /// 调用参数
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitPayload {
    /// 退出原因
    pub reason: String,
}

// ─── 插件清单（manifest.toml） ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPackageManifest {
    pub plugin: PluginPackageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPackageInfo {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub version: String,
    /// 可执行文件名（相对插件目录）
    pub exec: String,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default = "default_width")]
    pub window_width: f32,
    #[serde(default = "default_height")]
    pub window_height: f32,
}

fn default_icon() -> String { "icon.svg".to_string() }
fn default_width() -> f32 { 800.0 }
fn default_height() -> f32 { 600.0 }

// ─── 辅助函数 ───────────────────────────────────────────────────────────────

impl Envelope {
    pub fn new(id: u64, kind: MessageKind, payload: impl Serialize) -> Self {
        Self {
            id,
            kind,
            payload: serde_json::to_value(payload).unwrap_or(serde_json::Value::Null),
        }
    }

    /// 解码 payload 为具体类型
    pub fn decode_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }
}
