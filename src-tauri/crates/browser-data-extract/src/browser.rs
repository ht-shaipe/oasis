use crate::models::{BrowserInfo, BrowserKind, ProfileInfo};
use std::path::PathBuf;

// ── macOS 平台 ────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod platform {
    use super::*;

    pub fn home_dir() -> PathBuf {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
    }

    pub fn app_support() -> PathBuf {
        home_dir().join("Library/Application Support")
    }

    /// macOS 已知的 Chromium 系浏览器列表
    ///
    /// 数据目录位于 `~/Library/Application Support/<Browser>/`
    pub fn chromium_browsers() -> Vec<BrowserTemplate> {
        let base = app_support();
        vec![
            BrowserTemplate {
                key: "chrome".into(),
                name: "Google Chrome".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: base.join("Google/Chrome"),
            },
            BrowserTemplate {
                key: "chrome_beta".into(),
                name: "Google Chrome Beta".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: base.join("Google/Chrome Beta"),
            },
            BrowserTemplate {
                key: "chromium".into(),
                name: "Chromium".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: base.join("Chromium"),
            },
            BrowserTemplate {
                key: "edge".into(),
                name: "Microsoft Edge".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: base.join("Microsoft Edge"),
            },
            BrowserTemplate {
                key: "brave".into(),
                name: "Brave Browser".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: base.join("BraveSoftware/Brave-Browser"),
            },
            BrowserTemplate {
                key: "vivaldi".into(),
                name: "Vivaldi".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: base.join("Vivaldi"),
            },
            BrowserTemplate {
                key: "arc".into(),
                name: "Arc".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: base.join("Arc"),
            },
            BrowserTemplate {
                key: "opera".into(),
                name: "Opera".into(),
                kind: BrowserKind::ChromiumOpera,
                user_data_dir: base.join("com.operasoftware.Opera"),
            },
            BrowserTemplate {
                key: "opera_gx".into(),
                name: "Opera GX".into(),
                kind: BrowserKind::ChromiumOpera,
                user_data_dir: base.join("com.operasoftware.OperaGX"),
            },
            BrowserTemplate {
                key: "yandex".into(),
                name: "Yandex Browser".into(),
                kind: BrowserKind::ChromiumYandex,
                user_data_dir: base.join("Yandex/YandexBrowser"),
            },
            BrowserTemplate {
                key: "coccoc".into(),
                name: "CocCoc Browser".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: base.join("CocCoc/Browser"),
            },
        ]
    }

    /// macOS Firefox 数据目录：`~/Library/Application Support/Firefox/`
    pub fn firefox_browsers() -> Vec<BrowserTemplate> {
        let base = app_support();
        vec![BrowserTemplate {
            key: "firefox".into(),
            name: "Firefox".into(),
            kind: BrowserKind::Firefox,
            user_data_dir: base.join("Firefox"),
        }]
    }

    /// macOS Safari 数据目录：`~/Library/Application Support/Safari/`
    pub fn safari_browsers() -> Vec<BrowserTemplate> {
        let base = app_support();
        vec![BrowserTemplate {
            key: "safari".into(),
            name: "Safari".into(),
            kind: BrowserKind::Safari,
            user_data_dir: base.join("Safari"),
        }]
    }
}

// ── Linux 平台 ────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod platform {
    use super::*;

    pub fn home_dir() -> PathBuf {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
    }

    pub fn config_dir() -> PathBuf {
        dirs::config_dir().unwrap_or_else(|| home_dir().join(".config"))
    }

    /// Linux Chromium 系浏览器列表
    ///
    /// 数据目录位于 `~/.config/<browser-name>/`
    pub fn chromium_browsers() -> Vec<BrowserTemplate> {
        let cfg = config_dir();
        vec![
            BrowserTemplate {
                key: "chrome".into(),
                name: "Google Chrome".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: cfg.join("google-chrome"),
            },
            BrowserTemplate {
                key: "chrome_beta".into(),
                name: "Google Chrome Beta".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: cfg.join("google-chrome-beta"),
            },
            BrowserTemplate {
                key: "chromium".into(),
                name: "Chromium".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: cfg.join("chromium"),
            },
            BrowserTemplate {
                key: "edge".into(),
                name: "Microsoft Edge".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: cfg.join("microsoft-edge"),
            },
            BrowserTemplate {
                key: "brave".into(),
                name: "Brave Browser".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: cfg.join("BraveSoftware/Brave-Browser"),
            },
            BrowserTemplate {
                key: "vivaldi".into(),
                name: "Vivaldi".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: cfg.join("vivaldi"),
            },
            BrowserTemplate {
                key: "opera".into(),
                name: "Opera".into(),
                kind: BrowserKind::ChromiumOpera,
                user_data_dir: cfg.join("opera"),
            },
        ]
    }

    /// Linux Firefox 数据目录：`~/.mozilla/firefox/`
    pub fn firefox_browsers() -> Vec<BrowserTemplate> {
        let cfg = config_dir();
        vec![BrowserTemplate {
            key: "firefox".into(),
            name: "Firefox".into(),
            kind: BrowserKind::Firefox,
            user_data_dir: cfg.join("mozilla").join("firefox"),
        }]
    }

    /// Linux 不支持 Safari
    pub fn safari_browsers() -> Vec<BrowserTemplate> {
        vec![]
    }
}

// ── Windows 平台 ──────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    pub fn local_app_data() -> PathBuf {
        dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Local"))
    }

    pub fn roaming_app_data() -> PathBuf {
        dirs::data_dir().unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"))
    }

    /// Windows Chromium 系浏览器列表
    ///
    /// 大多数位于 `%LOCALAPPDATA%\<Browser>\User Data`，
    /// Opera 系列位于 `%APPDATA%\Opera Software\...`
    pub fn chromium_browsers() -> Vec<BrowserTemplate> {
        let local = local_app_data();
        let roaming = roaming_app_data();
        vec![
            BrowserTemplate {
                key: "chrome".into(),
                name: "Google Chrome".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: local.join("Google/Chrome/User Data"),
            },
            BrowserTemplate {
                key: "chrome_beta".into(),
                name: "Google Chrome Beta".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: local.join("Google/Chrome Beta/User Data"),
            },
            BrowserTemplate {
                key: "chromium".into(),
                name: "Chromium".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: local.join("Chromium/User Data"),
            },
            BrowserTemplate {
                key: "edge".into(),
                name: "Microsoft Edge".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: local.join("Microsoft/Edge/User Data"),
            },
            BrowserTemplate {
                key: "brave".into(),
                name: "Brave Browser".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: local.join("BraveSoftware/Brave-Browser/User Data"),
            },
            BrowserTemplate {
                key: "vivaldi".into(),
                name: "Vivaldi".into(),
                kind: BrowserKind::Chromium,
                user_data_dir: local.join("Vivaldi/User Data"),
            },
            BrowserTemplate {
                key: "opera".into(),
                name: "Opera".into(),
                kind: BrowserKind::ChromiumOpera,
                user_data_dir: roaming.join("Opera Software/Opera Stable"),
            },
            BrowserTemplate {
                key: "opera_gx".into(),
                name: "Opera GX".into(),
                kind: BrowserKind::ChromiumOpera,
                user_data_dir: roaming.join("Opera Software/Opera GX Stable"),
            },
            BrowserTemplate {
                key: "yandex".into(),
                name: "Yandex Browser".into(),
                kind: BrowserKind::ChromiumYandex,
                user_data_dir: local.join("Yandex/YandexBrowser/User Data"),
            },
        ]
    }

    /// Windows Firefox 数据目录：`%APPDATA%\Mozilla\Firefox\`
    pub fn firefox_browsers() -> Vec<BrowserTemplate> {
        let roaming = roaming_app_data();
        vec![BrowserTemplate {
            key: "firefox".into(),
            name: "Firefox".into(),
            kind: BrowserKind::Firefox,
            user_data_dir: roaming.join("Mozilla/Firefox"),
        }]
    }

    /// Windows 不支持 Safari
    pub fn safari_browsers() -> Vec<BrowserTemplate> {
        vec![]
    }
}

/// 浏览器模板，用于描述一个已知浏览器的安装信息
struct BrowserTemplate {
    /// 唯一标识键
    key: String,
    /// 显示名称
    name: String,
    /// 引擎类型
    kind: BrowserKind,
    /// 用户数据目录路径
    user_data_dir: PathBuf,
}

/// 发现 Chromium 系浏览器的 Profile
///
/// 1. 读取 `Local State` 中的 `profile.info_cache` 获取 Profile 列表
/// 2. 若 `Local State` 不存在，则扫描子目录中包含 `Preferences` 的目录
/// 3. 最终 fallback 为 `Default` 目录
fn discover_chromium_profiles(user_data_dir: &std::path::Path) -> Vec<ProfileInfo> {
    let mut profiles = Vec::new();
    let local_state = user_data_dir.join("Local State");
    if !local_state.exists() {
        if let Ok(entries) = std::fs::read_dir(user_data_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p.join("Preferences").exists() {
                    let name = p
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    profiles.push(ProfileInfo {
                        name: name.clone(),
                        path: p.to_string_lossy().to_string(),
                    });
                }
            }
        }
        return profiles;
    }

    if let Ok(content) = std::fs::read_to_string(&local_state) {
        if let Ok(ls) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(info_cache) = ls.get("profile").and_then(|p| p.get("info_cache")) {
                if let Some(obj) = info_cache.as_object() {
                    for (key, val) in obj {
                        let display_name = val
                            .get("gaia_name")
                            .or_else(|| val.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(key)
                            .to_string();
                        let profile_path = user_data_dir.join(key);
                        if profile_path.join("Preferences").exists()
                            || profile_path.join("Secure Preferences").exists()
                        {
                            profiles.push(ProfileInfo {
                                name: display_name,
                                path: profile_path.to_string_lossy().to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    if profiles.is_empty() {
        let default_path = user_data_dir.join("Default");
        if default_path.exists() {
            profiles.push(ProfileInfo {
                name: "Default".into(),
                path: default_path.to_string_lossy().to_string(),
            });
        }
    }

    profiles
}

/// 发现 Firefox 的 Profile
///
/// 解析 `profiles.ini` 文件获取 Profile 名称和路径，
/// 支持 `IsRelative=1` 的相对路径。若 `profiles.ini` 不存在，
/// 则扫描目录中包含 `logins.json` 或 `places.sqlite` 的子目录。
fn discover_firefox_profiles(user_data_dir: &std::path::Path) -> Vec<ProfileInfo> {
    let mut profiles = Vec::new();
    let profiles_ini = user_data_dir.join("profiles.ini");
    if !profiles_ini.exists() {
        if let Ok(entries) = std::fs::read_dir(user_data_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir()
                    && (p.join("logins.json").exists()
                        || p.join("places.sqlite").exists())
                {
                    let name = p
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    profiles.push(ProfileInfo {
                        name: name.clone(),
                        path: p.to_string_lossy().to_string(),
                    });
                }
            }
        }
        return profiles;
    }

    if let Ok(content) = std::fs::read_to_string(&profiles_ini) {
        let mut current_name = String::new();
        let mut current_path = String::new();
        let mut current_is_relative: Option<i32> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                if !current_path.is_empty() {
                    let abs_path = if current_is_relative == Some(1) {
                        user_data_dir.join(&current_path)
                    } else {
                        PathBuf::from(&current_path)
                    };
                    if abs_path.join("logins.json").exists()
                        || abs_path.join("places.sqlite").exists()
                    {
                        profiles.push(ProfileInfo {
                            name: current_name.clone(),
                            path: abs_path.to_string_lossy().to_string(),
                        });
                    }
                }
                current_name.clear();
                current_path.clear();
                current_is_relative = None;
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "Name" => current_name = v.trim().to_string(),
                    "Path" => current_path = v.trim().to_string(),
                    "IsRelative" => current_is_relative = v.trim().parse().ok(),
                    _ => {}
                }
            }
        }
        if !current_path.is_empty() {
            let abs_path = if current_is_relative == Some(1) {
                user_data_dir.join(&current_path)
            } else {
                PathBuf::from(&current_path)
            };
            if abs_path.join("logins.json").exists()
                || abs_path.join("places.sqlite").exists()
            {
                profiles.push(ProfileInfo {
                    name: current_name.clone(),
                    path: abs_path.to_string_lossy().to_string(),
                });
            }
        }
    }

    profiles
}

/// 发现 Safari 的 Profile
///
/// Safari 仅有一个默认 Profile，直接返回数据目录本身。
fn discover_safari_profiles(user_data_dir: &std::path::Path) -> Vec<ProfileInfo> {
    if user_data_dir.exists() {
        vec![ProfileInfo {
            name: "Default".into(),
            path: user_data_dir.to_string_lossy().to_string(),
        }]
    } else {
        vec![]
    }
}

/// 扫描系统中已安装的浏览器
///
/// 依次检查 Chromium 系、Firefox、Safari 的数据目录是否存在，
/// 若存在则扫描其 Profile，返回所有可提取数据的浏览器列表。
///
/// # 返回
///
/// 包含所有已安装浏览器信息的 Vec，每个浏览器至少有一个 Profile。
/// 未安装的浏览器不会出现在列表中。
pub fn discover_browsers() -> Vec<BrowserInfo> {
    let mut browsers = Vec::new();

    for tmpl in platform::chromium_browsers() {
        if can_read_dir(&tmpl.user_data_dir) {
            let profiles = discover_chromium_profiles(&tmpl.user_data_dir);
            if !profiles.is_empty() {
                browsers.push(BrowserInfo {
                    key: tmpl.key,
                    name: tmpl.name,
                    kind: tmpl.kind,
                    user_data_dir: tmpl.user_data_dir.to_string_lossy().to_string(),
                    profiles,
                });
            }
        }
    }

    for tmpl in platform::firefox_browsers() {
        if can_read_dir(&tmpl.user_data_dir) {
            let profiles = discover_firefox_profiles(&tmpl.user_data_dir);
            if !profiles.is_empty() {
                browsers.push(BrowserInfo {
                    key: tmpl.key,
                    name: tmpl.name,
                    kind: tmpl.kind,
                    user_data_dir: tmpl.user_data_dir.to_string_lossy().to_string(),
                    profiles,
                });
            }
        }
    }

    for tmpl in platform::safari_browsers() {
        if can_read_dir(&tmpl.user_data_dir) {
            let profiles = discover_safari_profiles(&tmpl.user_data_dir);
            if !profiles.is_empty() {
                browsers.push(BrowserInfo {
                    key: tmpl.key,
                    name: tmpl.name,
                    kind: tmpl.kind,
                    user_data_dir: tmpl.user_data_dir.to_string_lossy().to_string(),
                    profiles,
                });
            }
        }
    }

    browsers
}

fn can_read_dir(path: &std::path::Path) -> bool {
    match std::fs::read_dir(path) {
        Ok(_) => true,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => false,
        Err(_) => path.exists(),
    }
}

/// macOS Full Disk Access 检查结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct FdaStatus {
    pub has_access: bool,
    pub message: String,
}

/// 检查当前进程是否拥有 macOS Full Disk Access 权限
///
/// macOS 对浏览器数据目录有单独的 FDA 保护，即使 `Application Support` 本身
/// 可读，浏览器子目录（如 Google/Chrome）仍可能返回 PermissionDenied。
/// 因此直接检测一个常见的浏览器目录是否可读。
pub fn check_fda_status() -> FdaStatus {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let app_support = home.join("Library/Application Support");
    let chrome_dir = app_support.join("Google/Chrome");

    let has_access = if chrome_dir.exists() {
        can_read_dir(&chrome_dir)
    } else {
        let edge_dir = app_support.join("Microsoft Edge");
        if edge_dir.exists() {
            can_read_dir(&edge_dir)
        } else {
            let firefox_dir = app_support.join("Firefox");
            if firefox_dir.exists() {
                can_read_dir(&firefox_dir)
            } else {
                true
            }
        }
    };

    if has_access {
        FdaStatus {
            has_access: true,
            message: String::new(),
        }
    } else {
        FdaStatus {
            has_access: false,
            message: "Oasis needs Full Disk Access to read browser data. Please grant it in System Settings > Privacy & Security > Full Disk Access.".into(),
        }
    }
}

/// 按浏览器标识键查找已安装浏览器
///
/// # 参数
/// - `key` — 浏览器标识，如 "chrome"、"firefox"、"safari"
///
/// # 返回
/// 找到则返回 `Some(BrowserInfo)`，否则 `None`
pub fn get_browser_by_key(key: &str) -> Option<BrowserInfo> {
    discover_browsers().into_iter().find(|b| b.key == key)
}
