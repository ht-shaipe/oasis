//! # Oasis Browser Data Extract
//!
//! 浏览器数据提取库，支持从 Chromium、Firefox、Safari 等主流浏览器中提取
//! 密码、Cookie、书签、历史记录、下载记录、信用卡信息和扩展信息。
//!
//! ## 模块结构
//!
//! - [`browser`] — 浏览器发现与 Profile 扫描
//! - [`crypto`]  — 解密算法（AES-128-CBC、AES-256-GCM、3DES-CBC、PBKDF2、NSS）
//! - [`extract`] — 数据提取核心逻辑
//! - [`models`]  — 数据模型与时间戳转换
//! - [`commands`] — Tauri 命令接口
//!
//! ## 支持的浏览器
//!
//! | 引擎类型 | 浏览器 |
//! |----------|--------|
//! | Chromium | Chrome、Chrome Beta、Chromium、Edge、Brave、Vivaldi、Arc、CocCoc |
//! | ChromiumOpera | Opera、Opera GX |
//! | ChromiumYandex | Yandex Browser |
//! | Firefox | Firefox |
//! | Safari | Safari（仅 macOS） |
//!
//! ## 支持的数据类型
//!
//! | 类型 | Chromium | Firefox | Safari |
//! |------|----------|---------|--------|
//! | 密码 (Password) | ✅ AES 加密解密 | ✅ NSS PBE 解密 | ❌ |
//! | Cookie | ✅ AES 加密解密 | ✅ 明文读取 | ⚠️ 待实现 |
//! | 书签 (Bookmark) | ✅ JSON 解析 | ✅ SQLite | ✅ plist |
//! | 历史记录 (History) | ✅ SQLite | ✅ SQLite | ✅ SQLite |
//! | 下载记录 (Download) | ✅ SQLite | ✅ SQLite | ✅ plist |
//! | 信用卡 (CreditCard) | ✅ AES 加密解密 | ❌ | ❌ |
//! | 扩展 (Extension) | ✅ JSON | ✅ JSON | ❌ |
//!
//! ## 解密算法
//!
//! ### Chromium (macOS)
//! 1. 从 `security find-generic-password` 获取 Keychain 密码
//! 2. PBKDF2(password, "saltysalt", 1003, SHA1) → 16 字节密钥
//! 3. AES-128-CBC 解密，IV = `[0x20; 16]`
//!
//! ### Chromium (Windows)
//! 1. 从 `Local State` 读取 base64 编码的加密密钥
//! 2. 去除 "DPAPI" 前缀，调用 CryptUnprotectData 解密
//! 3. AES-256-GCM 解密（v10/v20 前缀）
//!
//! ### Chromium (Linux)
//! 1. PBKDF2("peanuts", "saltysalt", 1, SHA1) → 16 字节密钥
//! 2. AES-128-CBC 解密
//!
//! ### Firefox
//! 1. 从 key4.db 读取 globalSalt 和 nssPrivate 条目
//! 2. NSS PBE-SHA1 密钥派生：SHA1 + HMAC-SHA1 → 3DES-CBC 24 字节密钥
//! 3. 解密 logins.json 中的 base64 编码 ASN.1 PBE 加密字段
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use oasis_browser_data_extract::{browser, extract, models::DataType};
//!
//! // 发现已安装的浏览器
//! let browsers = browser::discover_browsers();
//! for b in &browsers {
//!     println!("{}: {} ({:?})", b.key, b.name, b.kind);
//! }
//!
//! // 提取指定浏览器的密码
//! let results = extract::extract_from_browser("chrome", &[DataType::Password]).unwrap();
//! for r in results {
//!     for login in &r.logins {
//!         println!("{}: {} / {}", login.url, login.username, login.password);
//!     }
//! }
//! ```

pub mod browser;
pub mod commands;
pub mod crypto;
pub mod extract;
pub mod models;
