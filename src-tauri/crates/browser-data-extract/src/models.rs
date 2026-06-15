use chrono::DateTime;
use serde::{Deserialize, Serialize};

/// 浏览器引擎类型
///
/// 不同引擎使用不同的加密方式和数据存储路径：
/// - `Chromium` — 标准 Chromium 内核（Chrome、Edge、Brave 等）
/// - `ChromiumYandex` — Yandex 浏览器，使用两层 AES-GCM 加密
/// - `ChromiumOpera` — Opera 系列，扩展数据路径不同
/// - `Firefox` — 使用 NSS 加密体系
/// - `Safari` — 使用 macOS Keychain 和 plist 存储
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserKind {
    Chromium,
    ChromiumYandex,
    ChromiumOpera,
    Firefox,
    Safari,
}

/// 可提取的浏览器数据类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    Password,
    Cookie,
    Bookmark,
    History,
    Download,
    CreditCard,
    Extension,
}

impl DataType {
    /// 返回所有支持的数据类型
    pub fn all() -> Vec<DataType> {
        vec![
            DataType::Password,
            DataType::Cookie,
            DataType::Bookmark,
            DataType::History,
            DataType::Download,
            DataType::CreditCard,
            DataType::Extension,
        ]
    }

    /// 返回数据类型的标签名称（用于序列化/显示）
    pub fn label(&self) -> &str {
        match self {
            DataType::Password => "password",
            DataType::Cookie => "cookie",
            DataType::Bookmark => "bookmark",
            DataType::History => "history",
            DataType::Download => "download",
            DataType::CreditCard => "credit_card",
            DataType::Extension => "extension",
        }
    }
}

/// 已安装浏览器的信息
///
/// 包含浏览器的唯一标识、显示名称、引擎类型、用户数据目录和所有 Profile。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserInfo {
    /// 浏览器唯一标识键，如 "chrome"、"firefox"、"safari"
    pub key: String,
    /// 浏览器显示名称，如 "Google Chrome"、"Firefox"
    pub name: String,
    /// 浏览器引擎类型
    pub kind: BrowserKind,
    /// 用户数据目录的绝对路径
    pub user_data_dir: String,
    /// 已发现的 Profile 列表
    pub profiles: Vec<ProfileInfo>,
}

/// 浏览器 Profile 信息
///
/// Chromium 系浏览器的每个 Profile 有独立的数据目录（如 "Default"、"Profile 1"），
/// Firefox 的每个 Profile 是随机命名的目录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    /// Profile 显示名称
    pub name: String,
    /// Profile 数据目录的绝对路径
    pub path: String,
}

/// 登录凭证条目（浏览器保存的密码）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginEntry {
    /// 网站 URL
    pub url: String,
    /// 用户名
    pub username: String,
    /// 解密后的密码
    pub password: String,
    /// 创建时间，格式 "YYYY-MM-DD HH:MM:SS"
    pub created_at: Option<String>,
}

/// Cookie 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieEntry {
    /// Cookie 所属域名
    pub host: String,
    /// Cookie 路径
    pub path: String,
    /// Cookie 名称
    pub name: String,
    /// Cookie 值（Chromium 系已解密，Firefox 为明文）
    pub value: String,
    /// 是否仅 HTTPS
    pub is_secure: bool,
    /// 是否仅 HTTP（不可通过 JS 访问）
    pub is_http_only: bool,
    /// 过期时间，格式 "YYYY-MM-DD HH:MM:SS"
    pub expires_at: Option<String>,
}

/// 书签条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkEntry {
    /// 自增 ID
    pub id: i64,
    /// 书签名称
    pub name: String,
    /// 书签 URL
    pub url: String,
    /// 所属文件夹路径（如 "Bookmarks Bar/Dev/Rust"）
    pub folder: String,
    /// 创建时间
    pub created_at: Option<String>,
}

/// 浏览历史条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 页面 URL
    pub url: String,
    /// 页面标题
    pub title: String,
    /// 访问次数
    pub visit_count: i32,
    /// 最后访问时间
    pub last_visit: Option<String>,
}

/// 下载记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadEntry {
    /// 下载源 URL
    pub url: String,
    /// 本地保存路径
    pub target_path: String,
    /// 文件大小（字节）
    pub total_bytes: i64,
    /// 下载开始时间
    pub start_time: Option<String>,
}

/// 信用卡条目（仅 Chromium 系浏览器支持）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCardEntry {
    /// 唯一标识
    pub guid: String,
    /// 持卡人姓名
    pub name: String,
    /// 解密后的卡号
    pub number: String,
    /// 过期月份
    pub exp_month: String,
    /// 过期年份
    pub exp_year: String,
}

/// 浏览器扩展条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionEntry {
    /// 扩展名称
    pub name: String,
    /// 扩展 ID
    pub id: String,
    /// 扩展描述
    pub description: String,
    /// 扩展版本号
    pub version: String,
    /// 是否已启用
    pub enabled: bool,
}

/// 单次浏览器数据提取的结果
///
/// 每个结果对应一个浏览器的一个 Profile 的一种数据类型。
/// 仅 `data_type` 对应的字段会有数据，其余为空 Vec。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserExtractResult {
    /// 浏览器标识键
    pub browser_key: String,
    /// 浏览器显示名称
    pub browser_name: String,
    /// 浏览器引擎类型
    pub browser_kind: BrowserKind,
    /// Profile 名称
    pub profile_name: String,
    /// 本次提取的数据类型
    pub data_type: DataType,
    /// 登录凭证（DataType::Password 时填充）
    pub logins: Vec<LoginEntry>,
    /// Cookie（DataType::Cookie 时填充）
    pub cookies: Vec<CookieEntry>,
    /// 书签（DataType::Bookmark 时填充）
    pub bookmarks: Vec<BookmarkEntry>,
    /// 历史记录（DataType::History 时填充）
    pub history: Vec<HistoryEntry>,
    /// 下载记录（DataType::Download 时填充）
    pub downloads: Vec<DownloadEntry>,
    /// 信用卡（DataType::CreditCard 时填充）
    pub credit_cards: Vec<CreditCardEntry>,
    /// 扩展（DataType::Extension 时填充）
    pub extensions: Vec<ExtensionEntry>,
}

impl BrowserExtractResult {
    /// 创建一个空的结果容器
    ///
    /// 仅 `data_type` 对应的字段会在后续填充，其余保持空 Vec。
    pub fn empty(browser_key: &str, browser_name: &str, kind: &BrowserKind, profile: &str, data_type: DataType) -> Self {
        Self {
            browser_key: browser_key.to_string(),
            browser_name: browser_name.to_string(),
            browser_kind: kind.clone(),
            profile_name: profile.to_string(),
            data_type,
            logins: vec![],
            cookies: vec![],
            bookmarks: vec![],
            history: vec![],
            downloads: vec![],
            credit_cards: vec![],
            extensions: vec![],
        }
    }
}

/// Chromium 时间戳 → 日期时间字符串
///
/// Chromium 使用 Windows 文件时间戳：自 1601-01-01 UTC 的微秒数。
/// 偏移量 = 11644473600_000_000 微秒。
///
/// # 参数
/// - `epoch_micros` — Chromium 时间戳（微秒）
///
/// # 返回
/// 格式 "YYYY-MM-DD HH:MM:SS" 的字符串，无效输入返回 None
pub fn chromium_epoch_to_datetime(epoch_micros: i64) -> Option<String> {
    let unix_micros = epoch_micros - 11644473600_000_000;
    if unix_micros <= 0 {
        return None;
    }
    let secs = unix_micros / 1_000_000;
    let nsecs = ((unix_micros % 1_000_000) * 1000) as u32;
    DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
}

/// WebKit/Core Data 时间戳 → 日期时间字符串
///
/// Safari 和 Apple 框架使用 Core Data 时间戳：自 2001-01-01 UTC 的秒数。
/// 偏移量 = 978307200 秒。
///
/// # 参数
/// - `epoch_secs` — WebKit 时间戳（秒，可为浮点数）
///
/// # 返回
/// 格式 "YYYY-MM-DD HH:MM:SS" 的字符串，无效输入返回 None
pub fn webkit_epoch_to_datetime(epoch_secs: f64) -> Option<String> {
    let unix_secs = epoch_secs as i64 + 978307200;
    DateTime::from_timestamp(unix_secs, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
}
