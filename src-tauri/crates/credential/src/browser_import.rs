/// 浏览器密码导入模块 — 纯 Rust 实现
/// Firefox: 通过 FFI 调用 libnss3 解密 logins.json
/// Chrome/Edge: 通过 macOS Keychain 获取 AES 密钥，解密 Login Data SQLite

use aes::cipher::{BlockDecryptMut, KeyIvInit};
use cbc::Decryptor;
use serde::{Deserialize, Serialize};
use std::ffi::{c_char, c_int, c_uint, c_void, CString};
use std::os::raw::c_uchar;

type Aes128Cbc = Decryptor<aes::Aes128>;

// ── NSS 结构体定义 ──────────────────────────────────────────────────────────

/// NSS SECItem，对应 C:
/// typedef struct SECItemStr { SECItemType type; unsigned char *data; unsigned int len; }
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SECItem {
    type_: c_uint,   // SECItemType (enum, unsigned int)
    data: *mut c_uchar,
    len: c_uint,
}

/// NSS SECStatus 返回值
const SEC_SUCCESS: c_int = 0;

/// SECItemType 常量
const SI_BUFFER: c_uint = 0;

// ── NSS 函数指针类型 ───────────────────────────────────────────────────────

type NssInitFn = unsafe extern "C" fn(*const c_char) -> c_int;
type NssShutdownFn = unsafe extern "C" fn() -> c_int;
type Pk11GetInternalKeySlotFn = unsafe extern "C" fn() -> *mut c_void;
type Pk11CheckUserPasswordFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;
type Pk11FreeSlotFn = unsafe extern "C" fn(*mut c_void);
type Pk11SdrDecryptFn = unsafe extern "C" fn(*mut SECItem, *mut SECItem, *mut c_void) -> c_int;
type SecitemZfreeItemFn = unsafe extern "C" fn(*mut SECItem, c_int);

// ── NSS 上下文 ──────────────────────────────────────────────────────────────

struct NssContext {
    _libs: Vec<libloading::Library>,
    nss_init: NssInitFn,
    nss_shutdown: NssShutdownFn,
    pk11_get_internal_key_slot: Pk11GetInternalKeySlotFn,
    pk11_check_user_password: Pk11CheckUserPasswordFn,
    pk11_free_slot: Pk11FreeSlotFn,
    pk11_sdr_decrypt: Pk11SdrDecryptFn,
    secitem_zfree_item: SecitemZfreeItemFn,
}

impl NssContext {
    fn load(lib_dir: &str) -> Result<Self, String> {
        // macOS 上 RTLD_NOW=0x2, RTLD_GLOBAL=0x8。必须用 GLOBAL 否则
        // libnss3 加载后看不到 libfreebl3/libsoftokn3 的符号，调用时崩溃。
        const DLFLAGS: i32 = 0x2 | 0x8;

        macro_rules! load_lib {
            ($dir:expr, $name:expr) => {
                unsafe {
                    libloading::os::unix::Library::open(
                        Some(std::path::Path::new($dir).join($name)),
                        DLFLAGS,
                    )
                    .map_err(|e| format!("加载 {} 失败: {}", $name, e))?
                }
            };
        }

        macro_rules! get_fn {
            ($lib:expr, $name:expr) => {
                unsafe {
                    *$lib.get::<*mut c_void>($name.as_bytes())
                        .map_err(|e| format!("获取符号 {} 失败: {}", $name, e))?
                }
            };
        }

        // 只需加载 libnss3（及其依赖 libmozglue）。softokn3/freebl3
        // 由 NSS 内部通过 PR_LoadLibrary 加载，不需要我们手动 dlopen。
        // libfreebl3/libsoftokn3 有 @rpath/libnss3.dylib 依赖，独立
        // dlopen 会因 rpath 未设置而失败，但 NSS 自己的加载机制能处理。
        let _mozglue = load_lib!(lib_dir, "libmozglue.dylib");
        let nss_lib = load_lib!(lib_dir, "libnss3.dylib");

        // 提取函数指针
        let ctx = NssContext {
            nss_init: unsafe { std::mem::transmute(get_fn!(nss_lib, "NSS_Init")) },
            nss_shutdown: unsafe { std::mem::transmute(get_fn!(nss_lib, "NSS_Shutdown")) },
            pk11_get_internal_key_slot: unsafe {
                std::mem::transmute(get_fn!(nss_lib, "PK11_GetInternalKeySlot"))
            },
            pk11_check_user_password: unsafe {
                std::mem::transmute(get_fn!(nss_lib, "PK11_CheckUserPassword"))
            },
            pk11_free_slot: unsafe {
                std::mem::transmute(get_fn!(nss_lib, "PK11_FreeSlot"))
            },
            pk11_sdr_decrypt: unsafe {
                std::mem::transmute(get_fn!(nss_lib, "PK11SDR_Decrypt"))
            },
            secitem_zfree_item: unsafe {
                std::mem::transmute(get_fn!(nss_lib, "SECITEM_ZfreeItem"))
            },
            _libs: vec![_mozglue.into(), nss_lib.into()],
        };

        Ok(ctx)
    }

    unsafe fn init(&self, profile: &str) -> Result<(), String> { unsafe {
        let path = CString::new(format!("sql:{}", profile))
            .map_err(|e| format!("路径编码错误: {}", e))?;
        let ret = (self.nss_init)(path.as_ptr());
        if ret != SEC_SUCCESS {
            Err(format!("NSS_Init 失败，返回码 {}", ret))
        } else {
            Ok(())
        }
    }}

    unsafe fn authenticate_slot(&self) -> Result<(), String> { unsafe {
        let slot = (self.pk11_get_internal_key_slot)();
        if slot.is_null() {
            return Err("无法获取 internal key slot".into());
        }
        let empty_pass = CString::new("").unwrap();
        let ret = (self.pk11_check_user_password)(slot, empty_pass.as_ptr());
        (self.pk11_free_slot)(slot);
        if ret != SEC_SUCCESS {
            Err(format!("需要 Master Password 或认证失败，返回码 {}", ret))
        } else {
            Ok(())
        }
    }}

    unsafe fn shutdown(&self) { unsafe {
        (self.nss_shutdown)();
    }}
}

// ── SDR 解密 ────────────────────────────────────────────────────────────────

unsafe fn decrypt_sdr_impl(ctx: &NssContext, enc_b64: &str) -> Result<String, String> { unsafe {
    use base64::Engine as _;

    let raw = base64::engine::general_purpose::STANDARD
        .decode(enc_b64)
        .map_err(|e| format!("base64 解码失败: {}", e))?;

    if raw.is_empty() {
        return Ok(String::new());
    }

    // 构建输入 SECItem（栈分配，NSS 不能修改 data 指针）
    let mut input_item = SECItem {
        type_: SI_BUFFER,
        data: raw.as_ptr() as *mut c_uchar,
        len: raw.len() as c_uint,
    };

    let mut output_item = SECItem {
        type_: 0,
        data: std::ptr::null_mut(),
        len: 0,
    };

    let ret = (ctx.pk11_sdr_decrypt)(&mut input_item, &mut output_item, std::ptr::null_mut());
    if ret != SEC_SUCCESS {
        if !output_item.data.is_null() {
            (ctx.secitem_zfree_item)(&mut output_item, 0);
        }
        return Err(format!("PK11SDR_Decrypt 失败，返回码 {}", ret));
    }

    if output_item.data.is_null() || output_item.len == 0 {
        return Ok(String::new());
    }

    let result = std::slice::from_raw_parts(output_item.data, output_item.len as usize);
    let decrypted = String::from_utf8_lossy(result).to_string();
    (ctx.secitem_zfree_item)(&mut output_item, 0);

    Ok(decrypted)
}}

// ── 公开接口 ────────────────────────────────────────────────────────────────

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
            BrowserType::Safari => false,
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

/// 扫描已安装的浏览器
pub fn scan_installed_browsers() -> Vec<BrowserType> {
    vec![
        BrowserType::Firefox,
        BrowserType::Chrome,
        BrowserType::Edge,
        BrowserType::Brave,
    ]
    .into_iter()
    .filter(|b| b.is_installed())
    .collect()
}

/// 查找 Firefox Profile 目录
fn find_firefox_profile() -> Result<String, String> {
    let home = dirs_next::home_dir()
        .ok_or_else(|| "无法获取用户主目录".to_string())?;
    let profiles_root = home.join("Library/Application Support/Firefox/Profiles");

    let entries = std::fs::read_dir(&profiles_root)
        .map_err(|e| format!("无法读取 {}: {}", profiles_root.display(), e))?;

    let mut profiles: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter(|p| {
            p.join("logins.json").exists()
                && p.join("key4.db").exists()
                && p.join("cert9.db").exists()
        })
        .collect();

    // 优先选择 *.default-release
    profiles.sort_by(|a, b| {
        let a_is_default = a
            .file_name()
            .map(|n| n.to_string_lossy().ends_with(".default-release"))
            .unwrap_or(false);
        let b_is_default = b
            .file_name()
            .map(|n| n.to_string_lossy().ends_with(".default-release"))
            .unwrap_or(false);
        b_is_default.cmp(&a_is_default) // .default-release 优先
    });

    profiles
        .first()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| format!("未找到 Firefox Profile: {}", profiles_root.display()))
}

/// 纯 Rust 实现：从 Firefox 读取密码
pub fn read_firefox_passwords() -> Result<Vec<BrowserCredential>, String> {
    let lib_dir = "/Applications/Firefox.app/Contents/MacOS";
    if !std::path::Path::new(lib_dir).exists() {
        return Err(format!("未找到 Firefox NSS 库目录: {}", lib_dir));
    }

    let profile = find_firefox_profile()
        .map_err(|e| format!("[Step1-Profile] {}", e))?;

    let ctx = NssContext::load(lib_dir)
        .map_err(|e| format!("[Step2-LoadNSS] {}", e))?;

    unsafe {
        ctx.init(&profile)
            .map_err(|e| format!("[Step3-NSS_Init] {}", e))?;
        ctx.authenticate_slot()
            .map_err(|e| format!("[Step4-Auth] {}", e))?;
    }

    // 读取 logins.json
    let logins_path = std::path::Path::new(&profile).join("logins.json");
    let json_str =
        std::fs::read_to_string(&logins_path)
            .map_err(|e| format!("[Step5-ReadLogins] {}", e))?;
    let data: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("[Step6-ParseJSON] {}", e))?;

    let logins = data["logins"]
        .as_array()
        .ok_or_else(|| "[Step7-LoginsArray] logins.json 格式错误".to_string())?;

    let mut results = Vec::with_capacity(logins.len());
    for (idx, login) in logins.iter().enumerate() {
        let url = login["hostname"].as_str().unwrap_or("").to_string();
        let enc_user = login["encryptedUsername"].as_str().unwrap_or("");
        let enc_pass = login["encryptedPassword"].as_str().unwrap_or("");

        let username = if enc_user.is_empty() {
            String::new()
        } else {
            unsafe { decrypt_sdr_impl(&ctx, enc_user).unwrap_or_default() }
        };

        let password = if enc_pass.is_empty() {
            String::new()
        } else {
            unsafe { decrypt_sdr_impl(&ctx, enc_pass).unwrap_or_default() }
        };

        results.push(BrowserCredential {
            id: idx,
            url,
            username,
            password,
            browser: "firefox".to_string(),
        });
    }

    unsafe {
        ctx.shutdown();
    }

    Ok(results)
}

// ── Chrome / Edge 密码读取 (macOS Keychain + SQLite) ─────────────────────

/// 从 macOS Keychain 获取 Chrome/Edge 的 Safe Storage 密钥
fn get_chromium_safe_storage_key(keychain_name: &str) -> Result<Vec<u8>, String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-wa", keychain_name])
        .output()
        .map_err(|e| format!("调用 security 命令失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Keychain 未找到 '{}'。请确保浏览器曾保存过密码。\nstderr: {}",
            keychain_name,
            stderr.trim()
        ));
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Chrome/Edge 将密钥以十六进制字符串存储在 Keychain 中
    if raw.chars().all(|c| c.is_ascii_hexdigit()) && raw.len() >= 32 {
        hex::decode(&raw).map_err(|e| format!("十六进制解码失败: {}", e))
    } else {
        // 少数情况下可能是原始字节，截取/填充到 16 字节
        let mut key = raw.into_bytes();
        key.resize(16, 0);
        Ok(key)
    }
}

/// 解密 Chrome/Edge 的密码 blob（AES-128-CBC，PKCS7 padding）
fn decrypt_chromium_password(encrypted: &[u8], key: &[u8]) -> Result<String, String> {
    if encrypted.len() < 3 + 16 {
        return Err(format!("加密数据太短 ({} bytes)", encrypted.len()));
    }

    let version = &encrypted[..3];
    if version != b"v10" && version != b"v11" {
        return Err(format!(
            "不支持的加密版本: {:?}",
            String::from_utf8_lossy(version)
        ));
    }

    let iv = &encrypted[3..19];
    let ciphertext = &encrypted[19..];

    let cipher = Aes128Cbc::new_from_slices(key, iv)
        .map_err(|e| format!("AES cipher 初始化失败: {}", e))?;

    let mut buf = ciphertext.to_vec();
    let plaintext = cipher
        .decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buf)
        .map_err(|e| format!("AES 解密失败: {:?}", e))?;

    String::from_utf8(plaintext.to_vec()).map_err(|e| format!("UTF-8 解码错误: {}", e))
}

/// 读取基于 Chromium 的浏览器密码（Chrome / Edge / Brave）
fn read_chromium_passwords(
    browser_label: &str,
    db_path: &str,
    keychain_name: &str,
) -> Result<Vec<BrowserCredential>, String> {
    if !std::path::Path::new(db_path).exists() {
        return Err(format!("未找到 {} 数据文件: {}", browser_label, db_path));
    }

    let key = get_chromium_safe_storage_key(keychain_name)
        .map_err(|e| format!("[Keychain] {}", e))?;

    if key.len() != 16 {
        return Err(format!("密钥长度 {} 不是 16 字节", key.len()));
    }

    // 复制 SQLite 文件避免锁定问题
    let tmp_path = format!("{}.tmp_import", db_path);
    std::fs::copy(db_path, &tmp_path)
        .map_err(|e| format!("复制 Login Data 失败: {}", e))?;

    let conn = rusqlite::Connection::open(&tmp_path)
        .map_err(|e| format!("打开 SQLite 失败: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT origin_url, username_value, password_value FROM logins")
        .map_err(|e| format!("SQL 准备失败: {}", e))?;

    let mut results = Vec::new();
    let rows = stmt
        .query_map([], |row| {
            let url: String = row.get(0)?;
            let username: String = row.get(1)?;
            let password_blob: Vec<u8> = row.get(2)?;
            Ok((url, username, password_blob))
        })
        .map_err(|e| format!("SQL 查询失败: {}", e))?;

    for (idx, row) in rows.enumerate() {
        let (url, username, password_blob) =
            row.map_err(|e| format!("读取行 {} 失败: {}", idx, e))?;

        if password_blob.is_empty() {
            continue;
        }

        let password = match decrypt_chromium_password(&password_blob, &key) {
            Ok(p) => p,
            Err(e) => {
                eprintln!(
                    "  [{}] 解密 {} @ {} 失败: {}",
                    browser_label, username, url, e
                );
                continue;
            }
        };

        results.push(BrowserCredential {
            id: results.len(),
            url,
            username,
            password,
            browser: browser_label.to_string(),
        });
    }

    // 清理临时文件
    let _ = std::fs::remove_file(&tmp_path);

    Ok(results)
}

/// 从 Google Chrome 读取密码
pub fn read_chrome_passwords() -> Result<Vec<BrowserCredential>, String> {
    let home = dirs_next::home_dir().ok_or("无法获取用户主目录")?;
    let db_path = home
        .join("Library/Application Support/Google/Chrome/Default/Login Data")
        .to_string_lossy()
        .to_string();
    read_chromium_passwords("chrome", &db_path, "Chrome Safe Storage")
}

/// 从 Microsoft Edge 读取密码
pub fn read_edge_passwords() -> Result<Vec<BrowserCredential>, String> {
    let home = dirs_next::home_dir().ok_or("无法获取用户主目录")?;
    let db_path = home
        .join("Library/Application Support/Microsoft Edge/Default/Login Data")
        .to_string_lossy()
        .to_string();
    read_chromium_passwords("edge", &db_path, "Microsoft Edge Safe Storage")
}

/// 从 Brave 读取密码
pub fn read_brave_passwords() -> Result<Vec<BrowserCredential>, String> {
    let home = dirs_next::home_dir().ok_or("无法获取用户主目录")?;
    let db_path = home
        .join("Library/Application Support/BraveSoftware/Brave-Browser/Default/Login Data")
        .to_string_lossy()
        .to_string();
    read_chromium_passwords("brave", &db_path, "Brave Safe Storage")
}

// ── 测试 ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_firefox_passwords() {
        match read_firefox_passwords() {
            Ok(passwords) => {
                eprintln!("成功读取 {} 条密码", passwords.len());
                for p in passwords.iter().take(3) {
                    eprintln!("  {} @ {} : {}", p.username, p.url, p.password);
                }
            }
            Err(e) => {
                panic!("读取失败: {}", e);
            }
        }
    }
}
