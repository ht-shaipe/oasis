use chrono::DateTime;

use crate::browser;
use crate::crypto;
use crate::models::*;

impl BrowserKind {
    /// 获取 Chromium 系浏览器在 macOS Keychain 中的标签名
    ///
    /// 用于 `security find-generic-password -wa <label>` 获取安全存储密码。
    pub fn keychain_label(&self, _key: &str) -> String {
        match self {
            BrowserKind::Chromium => "Chrome Safe Storage".to_string(),
            BrowserKind::ChromiumOpera => "Opera Safe Storage".to_string(),
            BrowserKind::ChromiumYandex => "Yandex Browser Safe Storage".to_string(),
            _ => String::new(),
        }
    }
}

/// 从指定浏览器提取数据
///
/// 根据浏览器引擎类型自动选择解密方式，遍历所有 Profile 提取指定类型的数据。
///
/// # 参数
/// - `browser_key` — 浏览器标识键，如 "chrome"、"firefox"、"safari"
/// - `data_types` — 要提取的数据类型列表
///
/// # 返回
/// 每个浏览器 Profile × 数据类型对应一个 `BrowserExtractResult`
pub fn extract_from_browser(
    browser_key: &str,
    data_types: &[DataType],
) -> Result<Vec<BrowserExtractResult>, String> {
    let browser_info = browser::get_browser_by_key(browser_key)
        .ok_or_else(|| format!("browser not found: {}", browser_key))?;

    let mut all_results = Vec::new();

    match browser_info.kind {
        BrowserKind::Chromium | BrowserKind::ChromiumOpera | BrowserKind::ChromiumYandex => {
            let key = crypto::retrieve_chromium_key(
                &browser_info.user_data_dir,
                &browser_info.kind.keychain_label(browser_key),
            );

            for profile in &browser_info.profiles {
                for dt in data_types {
                    let result = extract_chromium_data(
                        &browser_info,
                        profile,
                        &key,
                        dt,
                    );
                    all_results.push(result);
                }
            }
        }
        BrowserKind::Firefox => {
            for profile in &browser_info.profiles {
                for dt in data_types {
                    let result = extract_firefox_data(&browser_info, profile, dt);
                    all_results.push(result);
                }
            }
        }
        BrowserKind::Safari => {
            for profile in &browser_info.profiles {
                for dt in data_types {
                    let result = extract_safari_data(&browser_info, profile, dt);
                    all_results.push(result);
                }
            }
        }
    }

    Ok(all_results)
}

// ── Chromium 数据提取 ──────────────────────────────────────────────────

fn extract_chromium_data(
    browser_info: &BrowserInfo,
    profile: &ProfileInfo,
    key: &crypto::ChromiumKey,
    data_type: &DataType,
) -> BrowserExtractResult {
    let mut result = BrowserExtractResult::empty(
        &browser_info.key,
        &browser_info.name,
        &browser_info.kind,
        &profile.name,
        data_type.clone(),
    );

    match data_type {
        DataType::Password => {
            result.logins = extract_chromium_passwords(&profile.path, key);
        }
        DataType::Cookie => {
            result.cookies = extract_chromium_cookies(&profile.path, key, &browser_info.user_data_dir);
        }
        DataType::Bookmark => {
            result.bookmarks = extract_chromium_bookmarks(&profile.path);
        }
        DataType::History => {
            result.history = extract_chromium_history(&profile.path);
        }
        DataType::Download => {
            result.downloads = extract_chromium_downloads(&profile.path);
        }
        DataType::CreditCard => {
            result.credit_cards = extract_chromium_credit_cards(&profile.path, key);
        }
        DataType::Extension => {
            result.extensions = extract_chromium_extensions(&profile.path, &browser_info.kind);
        }
    }

    result
}

/// 提取 Chromium 系浏览器的保存密码
///
/// 数据源：`<profile>/Login Data` SQLite 数据库
///
/// 读取 `logins` 表的 `origin_url`、`username_value`、`password_value`、`date_created`，
/// 对 `password_value` 使用 Chromium 密钥解密。
fn extract_chromium_passwords(
    profile_path: &str,
    key: &crypto::ChromiumKey,
) -> Vec<LoginEntry> {
    let db_path = std::path::Path::new(profile_path).join("Login Data");
    if !db_path.exists() {
        return vec![];
    }

    let temp_path = match crypto::copy_to_temp(&db_path.to_string_lossy()) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let conn = match rusqlite::Connection::open(&temp_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut stmt = match conn.prepare(
        "SELECT origin_url, username_value, password_value, date_created FROM logins",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut logins = Vec::new();
    let rows = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let username: String = row.get(1)?;
        let password_enc: Vec<u8> = row.get(2)?;
        let date_created: i64 = row.get(3).unwrap_or(0);
        Ok((url, username, password_enc, date_created))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            let (_, _, ref enc, _) = row;
            let password = if enc.is_empty() {
                String::new()
            } else {
                crypto::decrypt_chromium_value(enc, key)
                    .map(|v| String::from_utf8_lossy(&v).to_string())
                    .unwrap_or_default()
            };

            logins.push(LoginEntry {
                url: row.0,
                username: row.1,
                password,
                created_at: chromium_epoch_to_datetime(row.3),
            });
        }
    }

    let _ = std::fs::remove_file(&temp_path);
    logins
}

/// 提取 Chromium 系浏览器的 Cookie
///
/// 数据源：`<profile>/Network/Cookies` SQLite 数据库
///
/// 读取 `cookies` 表，对 `encrypted_value` 解密后去除 Chrome 130+ 的 SHA256 域名哈希前缀。
fn extract_chromium_cookies(
    profile_path: &str,
    key: &crypto::ChromiumKey,
    _user_data_dir: &str,
) -> Vec<CookieEntry> {
    let db_path = std::path::Path::new(profile_path).join("Network/Cookies");
    if !db_path.exists() {
        return vec![];
    }

    let temp_path = match crypto::copy_to_temp(&db_path.to_string_lossy()) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let conn = match rusqlite::Connection::open(&temp_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut stmt = match conn.prepare(
        "SELECT host_key, path, name, encrypted_value, is_secure, is_httponly, expires_utc FROM cookies",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut cookies = Vec::new();
    let rows = stmt.query_map([], |row| {
        let host: String = row.get(0)?;
        let path: String = row.get(1)?;
        let name: String = row.get(2)?;
        let enc_value: Vec<u8> = row.get(3)?;
        let is_secure: bool = row.get(4)?;
        let is_httponly: bool = row.get(5)?;
        let expires_utc: i64 = row.get(6).unwrap_or(0);
        Ok((host, path, name, enc_value, is_secure, is_httponly, expires_utc))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            let value = if row.3.is_empty() {
                String::new()
            } else {
                let decrypted = crypto::decrypt_chromium_value(&row.3, key)
                    .unwrap_or_default();
                let stripped = crypto::strip_cookie_hash(&decrypted, &row.0);
                String::from_utf8_lossy(&stripped).to_string()
            };

            cookies.push(CookieEntry {
                host: row.0,
                path: row.1,
                name: row.2,
                value,
                is_secure: row.4,
                is_http_only: row.5,
                expires_at: chromium_epoch_to_datetime(row.6),
            });
        }
    }

    let _ = std::fs::remove_file(&temp_path);
    cookies
}

/// 提取 Chromium 系浏览器的书签
///
/// 数据源：`<profile>/Bookmarks` JSON 文件
///
/// 递归遍历 `roots.bookmark_bar`、`roots.other`、`roots.synced` 下的书签树，
/// 保留文件夹层级路径。
fn extract_chromium_bookmarks(profile_path: &str) -> Vec<BookmarkEntry> {
    let bm_path = std::path::Path::new(profile_path).join("Bookmarks");
    if !bm_path.exists() {
        return vec![];
    }

    let content = match std::fs::read_to_string(&bm_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let bm: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut bookmarks = Vec::new();
    let mut id_counter: i64 = 0;

    fn walk_bookmark_tree(
        node: &serde_json::Value,
        folder: &str,
        bookmarks: &mut Vec<BookmarkEntry>,
        id_counter: &mut i64,
    ) {
        let children = node.get("children").and_then(|c| c.as_array());
        if let Some(children) = children {
            for child in children {
                let child_type = child
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                let name = child
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("Unnamed");

                match child_type {
                    "url" => {
                        let url = child
                            .get("url")
                            .and_then(|u| u.as_str())
                            .unwrap_or("")
                            .to_string();
                        let date_added = child
                            .get("date_added")
                            .and_then(|d| d.as_str())
                            .and_then(|d| d.parse::<i64>().ok())
                            .and_then(chromium_epoch_to_datetime);
                        *id_counter += 1;
                        bookmarks.push(BookmarkEntry {
                            id: *id_counter,
                            name: name.to_string(),
                            url,
                            folder: folder.to_string(),
                            created_at: date_added,
                        });
                    }
                    "folder" => {
                        let new_folder = if folder.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}/{}", folder, name)
                        };
                        walk_bookmark_tree(child, &new_folder, bookmarks, id_counter);
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(roots) = bm.get("roots") {
        for root_key in &["bookmark_bar", "other", "synced"] {
            if let Some(root_node) = roots.get(*root_key) {
                let folder_name = match *root_key {
                    "bookmark_bar" => "Bookmarks Bar",
                    "other" => "Other Bookmarks",
                    "synced" => "Mobile Bookmarks",
                    _ => root_key,
                };
                walk_bookmark_tree(root_node, folder_name, &mut bookmarks, &mut id_counter);
            }
        }
    }

    bookmarks
}

/// 提取 Chromium 系浏览器的浏览历史
///
/// 数据源：`<profile>/History` SQLite 数据库
///
/// 读取 `urls` 表，按 `last_visit_time` 降序排列。
fn extract_chromium_history(profile_path: &str) -> Vec<HistoryEntry> {
    let db_path = std::path::Path::new(profile_path).join("History");
    if !db_path.exists() {
        return vec![];
    }

    let temp_path = match crypto::copy_to_temp(&db_path.to_string_lossy()) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let conn = match rusqlite::Connection::open(&temp_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut stmt = match conn.prepare(
        "SELECT url, title, visit_count, last_visit_time FROM urls ORDER BY last_visit_time DESC",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut history = Vec::new();
    let rows = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let title: String = row.get(1)?;
        let visit_count: i32 = row.get(2)?;
        let last_visit: i64 = row.get(3).unwrap_or(0);
        Ok((url, title, visit_count, last_visit))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            history.push(HistoryEntry {
                url: row.0,
                title: row.1,
                visit_count: row.2,
                last_visit: chromium_epoch_to_datetime(row.3),
            });
        }
    }

    let _ = std::fs::remove_file(&temp_path);
    history
}

/// 提取 Chromium 系浏览器的下载记录
///
/// 数据源：`<profile>/History` SQLite 数据库的 `downloads` 表
fn extract_chromium_downloads(profile_path: &str) -> Vec<DownloadEntry> {
    let db_path = std::path::Path::new(profile_path).join("History");
    if !db_path.exists() {
        return vec![];
    }

    let temp_path = match crypto::copy_to_temp(&db_path.to_string_lossy()) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let conn = match rusqlite::Connection::open(&temp_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut stmt = match conn.prepare(
        "SELECT tab_url, target_path, total_bytes, start_time FROM downloads ORDER BY start_time DESC",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut downloads = Vec::new();
    let rows = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let target: String = row.get(1)?;
        let total_bytes: i64 = row.get(2)?;
        let start_time: i64 = row.get(3).unwrap_or(0);
        Ok((url, target, total_bytes, start_time))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            downloads.push(DownloadEntry {
                url: row.0,
                target_path: row.1,
                total_bytes: row.2,
                start_time: chromium_epoch_to_datetime(row.3),
            });
        }
    }

    let _ = std::fs::remove_file(&temp_path);
    downloads
}

/// 提取 Chromium 系浏览器的信用卡信息
///
/// 数据源：`<profile>/Web Data` SQLite 数据库的 `credit_cards` 表
///
/// `card_number_encrypted` 使用 Chromium 密钥 AES 加密，解密后为明文卡号。
fn extract_chromium_credit_cards(
    profile_path: &str,
    key: &crypto::ChromiumKey,
) -> Vec<CreditCardEntry> {
    let db_path = std::path::Path::new(profile_path).join("Web Data");
    if !db_path.exists() {
        return vec![];
    }

    let temp_path = match crypto::copy_to_temp(&db_path.to_string_lossy()) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let conn = match rusqlite::Connection::open(&temp_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut stmt = match conn.prepare(
        "SELECT guid, name_on_card, card_number_encrypted, expiration_month, expiration_year FROM credit_cards",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut cards = Vec::new();
    let rows = stmt.query_map([], |row| {
        let guid: String = row.get(0)?;
        let name: String = row.get(1)?;
        let enc_number: Vec<u8> = row.get(2)?;
        let exp_month: String = row.get(3)?;
        let exp_year: String = row.get(4)?;
        Ok((guid, name, enc_number, exp_month, exp_year))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            let number = crypto::decrypt_chromium_value(&row.2, key)
                .map(|v| String::from_utf8_lossy(&v).to_string())
                .unwrap_or_default();

            cards.push(CreditCardEntry {
                guid: row.0,
                name: row.1,
                number,
                exp_month: row.3,
                exp_year: row.4,
            });
        }
    }

    let _ = std::fs::remove_file(&temp_path);
    cards
}

/// 提取 Chromium 系浏览器的扩展信息
///
/// 数据源：`<profile>/Preferences` JSON 文件（Opera 为 `Secure Preferences`）
///
/// 读取 `extensions.settings`（Opera 为 `extensions.opsettings`）获取扩展列表。
fn extract_chromium_extensions(
    profile_path: &str,
    kind: &BrowserKind,
) -> Vec<ExtensionEntry> {
    let prefs_name = match kind {
        BrowserKind::ChromiumOpera => "Secure Preferences",
        _ => "Preferences",
    };
    let prefs_path = std::path::Path::new(profile_path).join(prefs_name);
    if !prefs_path.exists() {
        return vec![];
    }

    let content = match std::fs::read_to_string(&prefs_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let prefs: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let settings_key = match kind {
        BrowserKind::ChromiumOpera => "extensions.opsettings",
        _ => "extensions.settings",
    };

    let settings = prefs
        .pointer(settings_key)
        .and_then(|s| s.as_object());

    let mut extensions = Vec::new();
    if let Some(settings) = settings {
        for (id, val) in settings {
            let manifest = val.get("manifest");
            let name = manifest
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or(id)
                .to_string();
            let description = manifest
                .and_then(|m| m.get("description"))
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            let version = manifest
                .and_then(|m| m.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            let enabled = val
                .get("state")
                .and_then(|s| s.as_i64())
                .map(|s| s == 1)
                .unwrap_or(false);

            extensions.push(ExtensionEntry {
                name,
                id: id.clone(),
                description,
                version,
                enabled,
            });
        }
    }

    extensions
}

// ── Firefox 数据提取 ───────────────────────────────────────────────────

fn extract_firefox_data(
    browser_info: &BrowserInfo,
    profile: &ProfileInfo,
    data_type: &DataType,
) -> BrowserExtractResult {
    let mut result = BrowserExtractResult::empty(
        &browser_info.key,
        &browser_info.name,
        &browser_info.kind,
        &profile.name,
        data_type.clone(),
    );

    match data_type {
        DataType::Password => {
            result.logins = extract_firefox_passwords(&profile.path);
        }
        DataType::Cookie => {
            result.cookies = extract_firefox_cookies(&profile.path);
        }
        DataType::Bookmark => {
            result.bookmarks = extract_firefox_bookmarks(&profile.path);
        }
        DataType::History => {
            result.history = extract_firefox_history(&profile.path);
        }
        DataType::Download => {
            result.downloads = extract_firefox_downloads(&profile.path);
        }
        DataType::CreditCard => {}
        DataType::Extension => {
            result.extensions = extract_firefox_extensions(&profile.path);
        }
    }

    result
}

/// 提取 Firefox 保存的密码
///
/// 数据源：`<profile>/logins.json` + `<profile>/key4.db`
///
/// 使用 NSS PBE-SHA1-3DES 从 key4.db 派生主密钥，解密 logins.json 中的
/// base64 编码 ASN.1 PBE 加密字段。
fn extract_firefox_passwords(profile_path: &str) -> Vec<LoginEntry> {
    let logins_path = std::path::Path::new(profile_path).join("logins.json");
    let key4_path = std::path::Path::new(profile_path).join("key4.db");

    if !logins_path.exists() || !key4_path.exists() {
        return vec![];
    }

    let content = match std::fs::read(&logins_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    match crypto::decrypt_firefox_nss(&content, &key4_path.to_string_lossy()) {
        Ok(entries) => entries
            .into_iter()
            .map(|(url, username, password)| LoginEntry {
                url,
                username,
                password,
                created_at: None,
            })
            .collect(),
        Err(_) => vec![],
    }
}

/// 提取 Firefox Cookie
///
/// 数据源：`<profile>/cookies.sqlite`
///
/// Firefox 的 Cookie 值为明文存储，无需解密。
fn extract_firefox_cookies(profile_path: &str) -> Vec<CookieEntry> {
    let db_path = std::path::Path::new(profile_path).join("cookies.sqlite");
    if !db_path.exists() {
        return vec![];
    }

    let temp_path = match crypto::copy_to_temp(&db_path.to_string_lossy()) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let conn = match rusqlite::Connection::open(&temp_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut stmt = match conn.prepare(
        "SELECT host, path, name, value, isSecure, isHttpOnly, expiry FROM moz_cookies",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut cookies = Vec::new();
    let rows = stmt.query_map([], |row| {
        let host: String = row.get(0)?;
        let path: String = row.get(1)?;
        let name: String = row.get(2)?;
        let value: String = row.get(3)?;
        let is_secure: bool = row.get(4)?;
        let is_http_only: bool = row.get(5)?;
        let expiry: i64 = row.get(6).unwrap_or(0);
        Ok((host, path, name, value, is_secure, is_http_only, expiry))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            let expires_at = if row.6 > 0 {
                DateTime::from_timestamp(row.6, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            } else {
                None
            };
            cookies.push(CookieEntry {
                host: row.0,
                path: row.1,
                name: row.2,
                value: row.3,
                is_secure: row.4,
                is_http_only: row.5,
                expires_at,
            });
        }
    }

    let _ = std::fs::remove_file(&temp_path);
    cookies
}

/// 提取 Firefox 书签
///
/// 数据源：`<profile>/places.sqlite`
///
/// 从 `moz_bookmarks` JOIN `moz_places` 获取书签的 URL 和标题。
/// Firefox 书签时间戳为 PRTime（自 Unix 纪元的微秒数）。
fn extract_firefox_bookmarks(profile_path: &str) -> Vec<BookmarkEntry> {
    let db_path = std::path::Path::new(profile_path).join("places.sqlite");
    if !db_path.exists() {
        return vec![];
    }

    let temp_path = match crypto::copy_to_temp(&db_path.to_string_lossy()) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let conn = match rusqlite::Connection::open(&temp_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut stmt = match conn.prepare(
        "SELECT b.id, b.title, p.url, b.dateAdded FROM moz_bookmarks b JOIN moz_places p ON b.fk = p.id WHERE b.type = 1",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut bookmarks = Vec::new();
    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let title: String = row.get(1).unwrap_or_default();
        let url: String = row.get(2)?;
        let date_added: i64 = row.get(3).unwrap_or(0);
        Ok((id, title, url, date_added))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            let created_at = if row.3 > 0 {
                let secs = row.3 / 1_000_000;
                DateTime::from_timestamp(secs, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            } else {
                None
            };
            bookmarks.push(BookmarkEntry {
                id: row.0,
                name: row.1,
                url: row.2,
                folder: String::new(),
                created_at,
            });
        }
    }

    let _ = std::fs::remove_file(&temp_path);
    bookmarks
}

/// 提取 Firefox 浏览历史
///
/// 数据源：`<profile>/places.sqlite`
///
/// 从 `moz_places` 表获取 URL、标题、访问次数和最后访问时间。
fn extract_firefox_history(profile_path: &str) -> Vec<HistoryEntry> {
    let db_path = std::path::Path::new(profile_path).join("places.sqlite");
    if !db_path.exists() {
        return vec![];
    }

    let temp_path = match crypto::copy_to_temp(&db_path.to_string_lossy()) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let conn = match rusqlite::Connection::open(&temp_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut stmt = match conn.prepare(
        "SELECT p.url, p.title, p.visit_count, p.last_visit_date FROM moz_places p WHERE p.visit_count > 0 ORDER BY p.last_visit_date DESC",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut history = Vec::new();
    let rows = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let title: String = row.get(1).unwrap_or_default();
        let visit_count: i32 = row.get(2)?;
        let last_visit: i64 = row.get(3).unwrap_or(0);
        Ok((url, title, visit_count, last_visit))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            let last_visit = if row.3 > 0 {
                let secs = row.3 / 1_000_000;
                DateTime::from_timestamp(secs, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            } else {
                None
            };
            history.push(HistoryEntry {
                url: row.0,
                title: row.1,
                visit_count: row.2,
                last_visit,
            });
        }
    }

    let _ = std::fs::remove_file(&temp_path);
    history
}

/// 提取 Firefox 下载记录
///
/// 数据源：`<profile>/places.sqlite`
///
/// 通过 `moz_annos` JOIN `moz_bookmarks` JOIN `moz_places` 查询
/// `downloads/destinationURI` 标注的下载记录。
fn extract_firefox_downloads(profile_path: &str) -> Vec<DownloadEntry> {
    let db_path = std::path::Path::new(profile_path).join("places.sqlite");
    if !db_path.exists() {
        return vec![];
    }

    let temp_path = match crypto::copy_to_temp(&db_path.to_string_lossy()) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let conn = match rusqlite::Connection::open(&temp_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut stmt = match conn.prepare(
        "SELECT p.url, a.content, b.dateAdded FROM moz_annos a JOIN moz_bookmarks b ON a.id = b.fk JOIN moz_places p ON b.fk = p.id WHERE a.anno_attribute_id = (SELECT id FROM moz_anno_attributes WHERE name = 'downloads/destinationURI')",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut downloads = Vec::new();
    let rows = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let target: String = row.get(1).unwrap_or_default();
        let date_added: i64 = row.get(2).unwrap_or(0);
        Ok((url, target, date_added))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            let start_time = if row.2 > 0 {
                let secs = row.2 / 1_000_000;
                DateTime::from_timestamp(secs, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            } else {
                None
            };
            downloads.push(DownloadEntry {
                url: row.0,
                target_path: row.1,
                total_bytes: 0,
                start_time,
            });
        }
    }

    let _ = std::fs::remove_file(&temp_path);
    downloads
}

/// 提取 Firefox 扩展信息
///
/// 数据源：`<profile>/extensions.json`
///
/// 读取 `addons` 数组获取扩展的 id、名称、版本和启用状态。
fn extract_firefox_extensions(profile_path: &str) -> Vec<ExtensionEntry> {
    let ext_path = std::path::Path::new(profile_path).join("extensions.json");
    if !ext_path.exists() {
        return vec![];
    }

    let content = match std::fs::read_to_string(ext_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let ext: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let addons = ext.get("addons").and_then(|a| a.as_array());
    let mut extensions = Vec::new();

    if let Some(addons) = addons {
        for addon in addons {
            let id = addon
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("")
                .to_string();
            let name = addon
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            let description = addon
                .get("description")
                .and_then(|d| d.as_str())
                .or_else(|| addon.get("descriptionURL").and_then(|d| d.as_str()))
                .unwrap_or("")
                .to_string();
            let version = addon
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            let enabled = addon
                .get("active")
                .and_then(|a| a.as_bool())
                .unwrap_or(false);

            extensions.push(ExtensionEntry {
                name,
                id,
                description,
                version,
                enabled,
            });
        }
    }

    extensions
}

// ── Safari 数据提取 ──────────────────────────────────────────────────────

fn extract_safari_data(
    browser_info: &BrowserInfo,
    profile: &ProfileInfo,
    data_type: &DataType,
) -> BrowserExtractResult {
    let mut result = BrowserExtractResult::empty(
        &browser_info.key,
        &browser_info.name,
        &browser_info.kind,
        &profile.name,
        data_type.clone(),
    );

    match data_type {
        DataType::Bookmark => {
            result.bookmarks = extract_safari_bookmarks();
        }
        DataType::History => {
            result.history = extract_safari_history();
        }
        DataType::Download => {
            result.downloads = extract_safari_downloads();
        }
        DataType::Cookie => {
            result.cookies = extract_safari_cookies();
        }
        _ => {}
    }

    result
}

/// 提取 Safari 书签
///
/// 数据源：`~/Library/Safari/Bookmarks.plist`
///
/// 递归遍历 plist 的 `WebBookmarkTypeList`（文件夹）和
/// `WebBookmarkTypeLeaf`（书签）节点，保留文件夹层级。
fn extract_safari_bookmarks() -> Vec<BookmarkEntry> {
    let home = dirs::home_dir().unwrap_or_default();
    let plist_path = home.join("Library/Safari/Bookmarks.plist");
    if !plist_path.exists() {
        return vec![];
    }

    let plist: plist::Value = match plist::Value::from_file(&plist_path) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut bookmarks = Vec::new();
    let mut id_counter: i64 = 0;

    fn walk_plist_bookmarks(
        children: &plist::Dictionary,
        folder: &str,
        bookmarks: &mut Vec<BookmarkEntry>,
        id_counter: &mut i64,
    ) {
        if let Some(arr) = children.get("Children").and_then(|c| c.as_array()) {
            for child in arr {
                if let Some(dict) = child.as_dictionary() {
                    let child_type = dict
                        .get("WebBookmarkType")
                        .and_then(|t| t.as_string())
                        .unwrap_or("");
                    let title = dict
                        .get("Title")
                        .and_then(|t| t.as_string())
                        .unwrap_or("Untitled");

                    match child_type {
                        "WebBookmarkTypeList" => {
                            let new_folder = if folder.is_empty() {
                                title.to_string()
                            } else {
                                format!("{}/{}", folder, title)
                            };
                            walk_plist_bookmarks(dict, &new_folder, bookmarks, id_counter);
                        }
                        "WebBookmarkTypeLeaf" => {
                            let url = dict
                                .get("URLString")
                                .and_then(|u| u.as_string())
                                .unwrap_or("")
                                .to_string();
                            *id_counter += 1;
                            bookmarks.push(BookmarkEntry {
                                id: *id_counter,
                                name: title.to_string(),
                                url,
                                folder: folder.to_string(),
                                created_at: None,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    if let Some(root) = plist.as_dictionary() {
        walk_plist_bookmarks(root, "", &mut bookmarks, &mut id_counter);
    }

    bookmarks
}

/// 提取 Safari 浏览历史
///
/// 数据源：`~/Library/Safari/History.db` SQLite 数据库
///
/// Safari 使用 Core Data 时间戳（自 2001-01-01 UTC 的秒数），
/// 通过 `webkit_epoch_to_datetime` 转换。
fn extract_safari_history() -> Vec<HistoryEntry> {
    let home = dirs::home_dir().unwrap_or_default();
    let db_path = home.join("Library/Safari/History.db");
    if !db_path.exists() {
        return vec![];
    }

    let temp_path = match crypto::copy_to_temp(&db_path.to_string_lossy()) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let conn = match rusqlite::Connection::open(&temp_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut stmt = match conn.prepare(
        "SELECT hv.url, hv.title, hi.visit_time FROM history_visits hi JOIN history_items hv ON hi.history_item = hv.id ORDER BY hi.visit_time DESC",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut history = Vec::new();
    let rows = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let title: String = row.get(1).unwrap_or_default();
        let visit_time: f64 = row.get(2).unwrap_or(0.0);
        Ok((url, title, visit_time))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            let last_visit = webkit_epoch_to_datetime(row.2);
            history.push(HistoryEntry {
                url: row.0,
                title: row.1,
                visit_count: 1,
                last_visit,
            });
        }
    }

    let _ = std::fs::remove_file(&temp_path);
    history
}

/// 提取 Safari 下载记录
///
/// 数据源：`~/Library/Safari/Downloads.plist`
///
/// 读取 `DownloadHistory` 数组中每条记录的 `DownloadEntryURL` 和 `DownloadEntryPath`。
fn extract_safari_downloads() -> Vec<DownloadEntry> {
    let home = dirs::home_dir().unwrap_or_default();
    let plist_path = home.join("Library/Safari/Downloads.plist");
    if !plist_path.exists() {
        return vec![];
    }

    let plist: plist::Value = match plist::Value::from_file(&plist_path) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut downloads = Vec::new();
    if let Some(dict) = plist.as_dictionary() {
        if let Some(arr) = dict.get("DownloadHistory").and_then(|h| h.as_array()) {
            for item in arr {
                if let Some(entry) = item.as_dictionary() {
                    let url = entry
                        .get("DownloadEntryURL")
                        .and_then(|u| u.as_string())
                        .unwrap_or("")
                        .to_string();
                    let target = entry
                        .get("DownloadEntryPath")
                        .and_then(|p| {
                            if let Some(s) = p.as_string() {
                                Some(s.to_string())
                            } else if let Some(d) = p.as_data() {
                                Some(String::from_utf8_lossy(d).to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    downloads.push(DownloadEntry {
                        url,
                        target_path: target,
                        total_bytes: 0,
                        start_time: None,
                    });
                }
            }
        }
    }

    downloads
}

/// 提取 Safari Cookie
///
/// Safari Cookie 存储在 `Cookies.binarycookies` 二进制文件中，
/// 当前尚未实现解析，返回空列表。
fn extract_safari_cookies() -> Vec<CookieEntry> {
    vec![]
}
