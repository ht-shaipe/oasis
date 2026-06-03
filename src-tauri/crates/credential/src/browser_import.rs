/// 浏览器密码导入模块 — CSV 导入方式
/// 支持 Chrome / Edge / Firefox / Brave / Safari 导出的 CSV 文件
/// CSV 格式自动检测：
///   - Chrome/Edge/Brave: name,url,username,password
///   - Firefox: "url","username","password",...
///   - Safari: 导出的 CSV 也兼容 Chrome 格式

use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use std::fs;

// ── 浏览器类型 ──────────────────────────────────────────────────────────

/// 支持的浏览器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BrowserType {
    Firefox,
    Chrome,
    Edge,
    Brave,
    Safari,
}

impl BrowserType {
    pub fn display_name(&self) -> &str {
        match self {
            BrowserType::Firefox => "Firefox",
            BrowserType::Chrome => "Google Chrome",
            BrowserType::Edge => "Microsoft Edge",
            BrowserType::Brave => "Brave Browser",
            BrowserType::Safari => "Safari",
        }
    }

    pub fn is_installed(&self) -> bool {
        match self {
            BrowserType::Firefox => {
                dirs_next::home_dir()
                    .map(|h| h.join("Library/Application Support/Firefox/Profiles").exists())
                    .unwrap_or(false)
            }
            BrowserType::Chrome => {
                dirs_next::home_dir()
                    .map(|h| h.join("Library/Application Support/Google/Chrome").exists())
                    .unwrap_or(false)
            }
            BrowserType::Edge => {
                dirs_next::home_dir()
                    .map(|h| h.join("Library/Application Support/Microsoft Edge").exists())
                    .unwrap_or(false)
            }
            BrowserType::Brave => {
                dirs_next::home_dir()
                    .map(|h| h.join("Library/Application Support/BraveSoftware/Brave-Browser").exists())
                    .unwrap_or(false)
            }
            BrowserType::Safari => {
                dirs_next::home_dir()
                    .map(|h| h.join("Library/Keychains/login.keychain-db").exists())
                    .unwrap_or(false)
            }
        }
    }
}

/// 导入的凭证条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserCredential {
    pub id: usize,
    pub url: String,
    pub username: String,
    pub password: String,
    pub browser: String,
}

/// 扫描已安装的浏览器（仍保留，供前端展示使用提示）
pub fn scan_installed_browsers() -> Vec<BrowserType> {
    vec![
        BrowserType::Firefox,
        BrowserType::Chrome,
        BrowserType::Edge,
        BrowserType::Brave,
        BrowserType::Safari,
    ]
    .into_iter()
    .filter(|b| b.is_installed())
    .collect()
}

// ── CSV 解析 ────────────────────────────────────────────────────────────

/// 解析浏览器导出的 CSV 密码文件，返回 BrowserCredential 列表
/// csv_path: CSV 文件路径
/// 自动检测表头格式（Chrome 格式 / Firefox 格式）
pub fn parse_csv_passwords(csv_path: &str) -> Result<Vec<BrowserCredential>, String> {
    let content = fs::read_to_string(csv_path)
        .map_err(|e| format!("无法读取 CSV 文件: {}", e))?;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());

    let headers = reader
        .headers()
        .map_err(|e| format!("无法解析 CSV 表头: {}", e))?
        .clone();

    let header_fields: Vec<String> = headers.iter().map(|h| h.trim().to_lowercase()).collect();

    // 检测格式：Chrome 格式表头含 "name", Firefox 格式表头含 "url"（带引号）
    // Chrome/Edge/Brave: name,url,username,password
    // Firefox: "url","username","password",...
    let format = detect_csv_format(&header_fields);

    let mut results = Vec::new();
    let mut id_counter: usize = 0;

    for record in reader.records() {
        let record = record.map_err(|e| format!("CSV 记录解析失败: {}", e))?;

        let (url, username, password) = match format {
            CsvFormat::Chrome => (
                record.get(1).unwrap_or("").to_string(),
                record.get(2).unwrap_or("").to_string(),
                record.get(3).unwrap_or("").to_string(),
            ),
            CsvFormat::Firefox => (
                record.get(0).unwrap_or("").to_string(),
                record.get(1).unwrap_or("").to_string(),
                record.get(2).unwrap_or("").to_string(),
            ),
        };

        // 跳过空密码行
        if password.is_empty() {
            continue;
        }

        results.push(BrowserCredential {
            id: id_counter,
            url,
            username,
            password,
            browser: "csv".to_string(),
        });
        id_counter += 1;
    }

    // 非空密码排到最前面
    results.sort_by_key(|c| if c.password.is_empty() { 1 } else { 0 });

    Ok(results)
}

#[derive(Debug, PartialEq)]
enum CsvFormat {
    Chrome,  // name,url,username,password
    Firefox, // "url","username","password",...
}

fn detect_csv_format(header_fields: &[String]) -> CsvFormat {
    // Chrome 格式：第 0 列是 "name"
    if header_fields.first().map(|h| h == "name").unwrap_or(false) {
        return CsvFormat::Chrome;
    }
    // Firefox 格式：第 0 列是 "url"
    if header_fields.first().map(|h| h == "url").unwrap_or(false) {
        return CsvFormat::Firefox;
    }
    // 默认 Chrome 格式
    CsvFormat::Chrome
}

// ── 测试 ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_chrome_csv() {
        let csv_content = "name,url,username,password\nExample,https://example.com,user1,pass1\nTest,https://test.com,user2,pass2\nEmpty,https://empty.com,user3,";
        let tmp_path = "/tmp/test_chrome_passwords.csv";
        let mut f = std::fs::File::create(tmp_path).unwrap();
        f.write_all(csv_content.as_bytes()).unwrap();
        f.flush().unwrap();
        drop(f);

        let result = parse_csv_passwords(tmp_path);
        assert!(result.is_ok());
        let creds = result.unwrap();
        // 空密码行被跳过
        assert_eq!(creds.len(), 2);
        assert_eq!(creds[0].url, "https://example.com");
        assert_eq!(creds[0].username, "user1");
        assert_eq!(creds[0].password, "pass1");
        assert_eq!(creds[1].url, "https://test.com");

        let _ = std::fs::remove_file(tmp_path);
    }

    #[test]
    fn test_parse_firefox_csv() {
        let csv_content = "\"url\",\"username\",\"password\",\"httpRealm\",\"formActionOrigin\",\"guid\"\n\"https://example.com\",\"user1\",\"pass1\",\"\",\"\",\"\"\n\"https://test.com\",\"user2\",\"pass2\",\"\",\"\",\"\"\n\"https://empty.com\",\"user3\",\"\",\"\",\"\",\"\"";
        let tmp_path = "/tmp/test_firefox_passwords.csv";
        let mut f = std::fs::File::create(tmp_path).unwrap();
        f.write_all(csv_content.as_bytes()).unwrap();
        f.flush().unwrap();
        drop(f);

        let result = parse_csv_passwords(tmp_path);
        assert!(result.is_ok());
        let creds = result.unwrap();
        // 空密码行被跳过
        assert_eq!(creds.len(), 2);
        assert_eq!(creds[0].url, "https://example.com");
        assert_eq!(creds[0].username, "user1");
        assert_eq!(creds[0].password, "pass1");

        let _ = std::fs::remove_file(tmp_path);
    }

    #[test]
    fn test_detect_csv_format() {
        assert_eq!(
            detect_csv_format(&["name".to_string(), "url".to_string()]),
            CsvFormat::Chrome
        );
        assert_eq!(
            detect_csv_format(&["url".to_string(), "username".to_string()]),
            CsvFormat::Firefox
        );
    }
}
