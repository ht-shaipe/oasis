use crate::browser;
use crate::extract;
use crate::models::*;

/// 扫描系统中已安装的浏览器
///
/// 返回所有检测到的浏览器及其 Profile 信息。
/// 仅包含数据目录实际存在的浏览器。
///
/// # 返回
///
/// ```json
/// [
///   {
///     "key": "chrome",
///     "name": "Google Chrome",
///     "kind": "chromium",
///     "user_data_dir": "/Users/x/Library/Application Support/Google/Chrome",
///     "profiles": [
///       { "name": "Person 1", "path": "/Users/x/.../Default" }
///     ]
///   }
/// ]
/// ```
#[tauri::command]
pub fn discover_browsers() -> Result<Vec<BrowserInfo>, String> {
    Ok(browser::discover_browsers())
}

#[tauri::command]
pub fn check_fda_status() -> Result<browser::FdaStatus, String> {
    Ok(browser::check_fda_status())
}

/// 从指定浏览器提取多种类型的数据
///
/// # 参数
/// - `browser_key` — 浏览器标识键（如 "chrome"、"firefox"、"safari"）
/// - `data_types` — 要提取的数据类型列表（如 `["password", "cookie"]`）
///
/// # 返回
/// 每个浏览器 Profile × 数据类型对应一个 `BrowserExtractResult`，
/// 仅 `data_type` 匹配的字段会填充数据。
#[tauri::command]
pub fn extract_browser_data(
    browser_key: String,
    data_types: Vec<DataType>,
) -> Result<Vec<BrowserExtractResult>, String> {
    extract::extract_from_browser(&browser_key, &data_types)
}

/// 从所有已安装浏览器提取指定类型的数据
///
/// 等价于对每个浏览器调用 `extract_browser_data` 后合并结果。
///
/// # 参数
/// - `data_types` — 要提取的数据类型列表
#[tauri::command]
pub fn extract_all_browser_data(
    data_types: Vec<DataType>,
) -> Result<Vec<BrowserExtractResult>, String> {
    let browsers = browser::discover_browsers();
    let mut all_results = Vec::new();

    for b in &browsers {
        if let Ok(results) = extract::extract_from_browser(&b.key, &data_types) {
            all_results.extend(results);
        }
    }

    Ok(all_results)
}

/// 从指定浏览器提取保存的密码
///
/// 便捷方法，等价于 `extract_browser_data(browser_key, ["password"])`
/// 但直接返回扁平的 `Vec<LoginEntry>`。
///
/// # 参数
/// - `browser_key` — 浏览器标识键
#[tauri::command]
pub fn extract_browser_passwords(
    browser_key: String,
) -> Result<Vec<LoginEntry>, String> {
    let results = extract::extract_from_browser(&browser_key, &[DataType::Password])?;
    let mut logins = Vec::new();
    for r in results {
        logins.extend(r.logins);
    }
    Ok(logins)
}

/// 从指定浏览器提取 Cookie
///
/// 便捷方法，等价于 `extract_browser_data(browser_key, ["cookie"])`
/// 但直接返回扁平的 `Vec<CookieEntry>`。
///
/// # 参数
/// - `browser_key` — 浏览器标识键
#[tauri::command]
pub fn extract_browser_cookies(
    browser_key: String,
) -> Result<Vec<CookieEntry>, String> {
    let results = extract::extract_from_browser(&browser_key, &[DataType::Cookie])?;
    let mut cookies = Vec::new();
    for r in results {
        cookies.extend(r.cookies);
    }
    Ok(cookies)
}

/// 从指定浏览器提取书签
///
/// 便捷方法，等价于 `extract_browser_data(browser_key, ["bookmark"])`
/// 但直接返回扁平的 `Vec<BookmarkEntry>`。
///
/// # 参数
/// - `browser_key` — 浏览器标识键
#[tauri::command]
pub fn extract_browser_bookmarks(
    browser_key: String,
) -> Result<Vec<BookmarkEntry>, String> {
    let results = extract::extract_from_browser(&browser_key, &[DataType::Bookmark])?;
    let mut bookmarks = Vec::new();
    for r in results {
        bookmarks.extend(r.bookmarks);
    }
    Ok(bookmarks)
}

/// 从指定浏览器提取浏览历史
///
/// 便捷方法，等价于 `extract_browser_data(browser_key, ["history"])`
/// 但直接返回扁平的 `Vec<HistoryEntry>`。
///
/// # 参数
/// - `browser_key` — 浏览器标识键
#[tauri::command]
pub fn extract_browser_history(
    browser_key: String,
) -> Result<Vec<HistoryEntry>, String> {
    let results = extract::extract_from_browser(&browser_key, &[DataType::History])?;
    let mut history = Vec::new();
    for r in results {
        history.extend(r.history);
    }
    Ok(history)
}

/// 从指定浏览器提取下载记录
///
/// 便捷方法，等价于 `extract_browser_data(browser_key, ["download"])`
/// 但直接返回扁平的 `Vec<DownloadEntry>`。
///
/// # 参数
/// - `browser_key` — 浏览器标识键
#[tauri::command]
pub fn extract_browser_downloads(
    browser_key: String,
) -> Result<Vec<DownloadEntry>, String> {
    let results = extract::extract_from_browser(&browser_key, &[DataType::Download])?;
    let mut downloads = Vec::new();
    for r in results {
        downloads.extend(r.downloads);
    }
    Ok(downloads)
}
