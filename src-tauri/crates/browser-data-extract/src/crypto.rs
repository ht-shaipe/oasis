use aes::Aes128;
use cbc::Decryptor as CbcDecryptor;
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use hmac::Hmac;
use pbkdf2::pbkdf2_hmac;
use sha1::Sha1;

type Aes128CbcDec = CbcDecryptor<Aes128>;
type HmacSha1 = Hmac<Sha1>;

/// Chromium PBKDF2 盐值（所有平台通用）
const CHROMIUM_SALT: &[u8] = b"saltysalt";
/// macOS Keychain 密码的 PBKDF2 迭代次数
const CHROMIUM_MACOS_ITERATIONS: u32 = 1003;
/// Linux 默认密码的 PBKDF2 迭代次数
const CHROMIUM_LINUX_ITERATIONS: u32 = 1;
/// Linux 无 Keyring 时的硬编码密码
const CHROMIUM_LINUX_PASSWORD: &[u8] = b"peanuts";

/// Chromium 浏览器的主密钥
///
/// 用于解密 Chromium 系浏览器中 AES 加密的密码、Cookie 和信用卡数据。
/// - macOS/Linux: 16 字节 AES-128-CBC 密钥
/// - Windows: 32 字节 AES-256-GCM 密钥
#[derive(Debug, Clone)]
pub struct ChromiumKey {
    /// v10 版本加密密钥（Chrome 80+）
    pub v10: Option<Vec<u8>>,
}

impl ChromiumKey {
    /// 密钥是否为空（无法解密）
    pub fn is_empty(&self) -> bool {
        self.v10.is_none()
    }
}

// ── macOS 密钥获取 ──────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
pub fn retrieve_chromium_key(user_data_dir: &str, _keychain_label: &str) -> ChromiumKey {
    let local_state_path = std::path::Path::new(user_data_dir).join("Local State");
    if !local_state_path.exists() {
        eprintln!("[browser-data-extract] Local State not found, trying keychain directly");
        let key = derive_macos_key_via_security(_keychain_label);
        eprintln!("[browser-data-extract] keychain key: {} bytes", key.len());
        return ChromiumKey { v10: Some(key) };
    }

    let content = match std::fs::read_to_string(&local_state_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[browser-data-extract] read Local State failed: {}, trying keychain", e);
            let key = derive_macos_key_via_security(_keychain_label);
            return ChromiumKey { v10: Some(key) };
        }
    };

    let ls: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[browser-data-extract] parse Local State failed: {}, trying keychain", e);
            let key = derive_macos_key_via_security(_keychain_label);
            return ChromiumKey { v10: Some(key) };
        }
    };

    let encrypted_key_b64 = match ls
        .get("os_crypt")
        .and_then(|oc| oc.get("encrypted_key"))
        .and_then(|k| k.as_str())
    {
        Some(k) => k,
        None => {
            eprintln!("[browser-data-extract] no os_crypt.encrypted_key, trying keychain for '{}'", _keychain_label);
            let key = derive_macos_key_via_security(_keychain_label);
            eprintln!("[browser-data-extract] keychain key: {} bytes", key.len());
            return ChromiumKey { v10: Some(key) };
        }
    };

    let encrypted_key = match base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        encrypted_key_b64,
    ) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("[browser-data-extract] base64 decode failed: {}, trying keychain", e);
            let key = derive_macos_key_via_security(_keychain_label);
            return ChromiumKey { v10: Some(key) };
        }
    };

    eprintln!("[browser-data-extract] encrypted_key: {} bytes, prefix: {:?}", 
        encrypted_key.len(), &encrypted_key[..encrypted_key.len().min(5)]);

    if encrypted_key.len() < 5 || &encrypted_key[..5] != b"DPAPI" {
        eprintln!("[browser-data-extract] no DPAPI prefix, trying keychain");
        let key = derive_macos_key_via_security(_keychain_label);
        eprintln!("[browser-data-extract] keychain key: {} bytes", key.len());
        return ChromiumKey { v10: Some(key) };
    }

    // macOS doesn't use DPAPI - the key in Local State is for Windows.
    // On macOS, always derive from Keychain.
    eprintln!("[browser-data-extract] DPAPI prefix found (Windows format), using keychain fallback for '{}'", _keychain_label);
    let key = derive_macos_key_via_security(_keychain_label);
    eprintln!("[browser-data-extract] final key: {} bytes", key.len());
    ChromiumKey { v10: Some(key) }
}

/// 通过 macOS `security` CLI 获取 Keychain 中存储的浏览器安全密码
///
/// 执行 `security find-generic-password -wa <label>` 获取明文密码，
/// 可能会弹出系统授权对话框要求用户输入登录密码。
#[cfg(target_os = "macos")]
fn retrieve_keychain_password(label: &str) -> Option<Vec<u8>> {
    use std::process::Command;

    let account_name = match label {
        "Chrome Safe Storage" => "Chrome",
        "Chromium Safe Storage" => "Chromium",
        "Opera Safe Storage" => "Opera",
        "Yandex Browser Safe Storage" => "Yandex Browser",
        "Microsoft Edge Safe Storage" => "Microsoft Edge",
        "Brave Safe Storage" => "Brave",
        "Vivaldi Safe Storage" => "Vivaldi",
        _ => "",
    };

    let output = if account_name.is_empty() {
        Command::new("security")
            .args(["find-generic-password", "-wa", label])
            .output()
            .ok()?
    } else {
        Command::new("security")
            .args(["find-generic-password", "-a", account_name, "-s", label, "-w"])
            .output()
            .ok()?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[browser-data-extract] keychain access failed for '{}': {}", label, stderr.trim());
        return None;
    }

    let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if password.is_empty() {
        return None;
    }
    Some(password.into_bytes())
}

/// 通过 macOS Keychain 密码派生 Chromium 解密密钥
///
/// 流程：Keychain 密码 → PBKDF2(password, "saltysalt", 1003, SHA1) → 16 字节密钥
#[cfg(target_os = "macos")]
fn derive_macos_key_via_security(label: &str) -> Vec<u8> {
    match retrieve_keychain_password(label) {
        Some(pwd) => pbkdf2_key(&pwd, CHROMIUM_MACOS_ITERATIONS, 16),
        None => pbkdf2_key(CHROMIUM_LINUX_PASSWORD, CHROMIUM_LINUX_ITERATIONS, 16),
    }
}

// ── Linux 密钥获取 ────

/// Linux 上获取 Chromium 密钥
///
/// 1. 尝试从 `Local State` 读取加密密钥
/// 2. 若无加密密钥或 DPAPI 前缀，使用硬编码密码 "peanuts"
/// 3. PBKDF2("peanuts", "saltysalt", 1, SHA1) → 16 字节 AES-128 密钥
#[cfg(target_os = "linux")]
pub fn retrieve_chromium_key(user_data_dir: &str, _keychain_label: &str) -> ChromiumKey {
    let local_state_path = std::path::Path::new(user_data_dir).join("Local State");
    if local_state_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&local_state_path) {
            if let Ok(ls) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(encrypted_key_b64) = ls
                    .get("os_crypt")
                    .and_then(|oc| oc.get("encrypted_key"))
                    .and_then(|k| k.as_str())
                {
                    if let Ok(encrypted_key) = base64::Engine::decode(
                        &base64::engine::general_purpose::STANDARD,
                        encrypted_key_b64,
                    ) {
                        if encrypted_key.starts_with(b"DPAPI") {
                            let key = pbkdf2_key(CHROMIUM_LINUX_PASSWORD, CHROMIUM_LINUX_ITERATIONS, 16);
                            return ChromiumKey { v10: Some(key) };
                        }
                    }
                }
            }
        }
    }

    let key = pbkdf2_key(CHROMIUM_LINUX_PASSWORD, CHROMIUM_LINUX_ITERATIONS, 16);
    ChromiumKey { v10: Some(key) }
}

// ── Windows 密钥获取 ───────────────────────────────────────────────────

/// Windows 上获取 Chromium 密钥
///
/// 1. 从 `Local State` 读取 `os_crypt.encrypted_key`
/// 2. Base64 解码后去除 "DPAPI" 前缀
/// 3. 调用 Windows DPAPI `CryptUnprotectData` 解密得到 32 字节 AES-256 密钥
#[cfg(target_os = "windows")]
pub fn retrieve_chromium_key(user_data_dir: &str, _keychain_label: &str) -> ChromiumKey {
    let local_state_path = std::path::Path::new(user_data_dir).join("Local State");
    if !local_state_path.exists() {
        return ChromiumKey { v10: None };
    }

    let content = match std::fs::read_to_string(&local_state_path) {
        Ok(c) => c,
        Err(_) => return ChromiumKey { v10: None },
    };

    let ls: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return ChromiumKey { v10: None },
    };

    let encrypted_key_b64 = match ls
        .get("os_crypt")
        .and_then(|oc| oc.get("encrypted_key"))
        .and_then(|k| k.as_str())
    {
        Some(k) => k,
        None => return ChromiumKey { v10: None },
    };

    let encrypted_key = match base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        encrypted_key_b64,
    ) {
        Ok(k) => k,
        Err(_) => return ChromiumKey { v10: None },
    };

    if encrypted_key.len() < 5 || &encrypted_key[..5] != b"DPAPI" {
        return ChromiumKey { v10: None };
    }

    let dpapi_blob = &encrypted_key[5..];
    match decrypt_windows_dpapi(dpapi_blob) {
        Some(key) => ChromiumKey { v10: Some(key) },
        None => ChromiumKey { v10: None },
    }
}

/// Windows DPAPI 解密
///
/// 调用 `CryptUnprotectData` Win32 API 解密 DPAPI 加密的密钥 blob。
/// 需要 `windows-sys` crate 的 Win32 Security 功能。
#[cfg(target_os = "windows")]
fn decrypt_windows_dpapi(blob: &[u8]) -> Option<Vec<u8>> {
    use std::ptr;
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPT_INTEGER_BLOB,
    };

    let mut data_in = CRYPT_INTEGER_BLOB {
        cbData: blob.len() as u32,
        pbData: blob.as_ptr() as *mut u8,
    };
    let mut data_out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    unsafe {
        if CryptUnprotectData(&mut data_in, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), 0, &mut data_out) != 0 {
            let result = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec();
            windows_sys::Win32::System::Memory::LocalFree(data_out.pbData as _);
            Some(result)
        } else {
            None
        }
    }
}

// ── PBKDF2 密钥派生 ─────────────────────────────────────────────────────

/// PBKDF2 密钥派生
///
/// 使用 HMAC-SHA1 和 Chromium 通用盐值 "saltysalt" 派生指定长度的密钥。
///
/// # 参数
/// - `password` — 密码字节
/// - `iterations` — 迭代次数（macOS=1003, Linux=1）
/// - `key_len` — 输出密钥长度（通常为 16 或 32 字节）
fn pbkdf2_key(password: &[u8], iterations: u32, key_len: usize) -> Vec<u8> {
    let mut key = vec![0u8; key_len];
    pbkdf2_hmac::<Sha1>(password, CHROMIUM_SALT, iterations, &mut key);
    key
}

// ── Chromium 加密值解密 ─────────────────────────────────────────────────

/// 解密 Chromium 加密的值（密码、Cookie、信用卡号等）
///
/// Chromium 加密值的格式为：3 字节版本前缀 + 加密数据。
///
/// | 前缀 | 平台 | 解密方式 |
/// |------|------|----------|
/// | `v10` | macOS/Linux | AES-128-CBC (16 字节密钥) 或 AES-256-GCM (32 字节密钥) |
/// | `v10` | Windows | AES-256-GCM (32 字节 DPAPI 解密密钥) |
/// | `v20` | Windows | AES-256-GCM (App-Bound Encryption) |
/// | `v11` | Linux | AES-128-CBC (D-Bus Secret Service 密钥) |
/// | 无前缀 | Windows | 原始 DPAPI blob |
///
/// # 参数
/// - `encrypted` — 加密值（含版本前缀）
/// - `key` — Chromium 主密钥
///
/// # 返回
/// 解密后的明文字节，或错误信息
pub fn decrypt_chromium_value(encrypted: &[u8], key: &ChromiumKey) -> Result<Vec<u8>, String> {
    if encrypted.len() < 3 {
        if encrypted.is_empty() {
            return Ok(vec![]);
        }
        return Err("encrypted value too short".into());
    }

    let version = &encrypted[..3];

    match version {
        b"v10" | b"v20" => {
            let v10_key = key.v10.as_ref().ok_or("no v10 key available")?;
            if v10_key.len() == 32 {
                decrypt_aes_256_gcm(&encrypted[3..], v10_key)
            } else if v10_key.len() == 16 {
                decrypt_aes_128_cbc(&encrypted[3..], v10_key)
            } else {
                Err(format!("unexpected key length: {}", v10_key.len()))
            }
        }
        b"v11" => {
            let v10_key = key.v10.as_ref().ok_or("no v11 key available")?;
            if v10_key.len() == 16 {
                decrypt_aes_128_cbc(&encrypted[3..], v10_key)
            } else if v10_key.len() == 32 {
                decrypt_aes_256_gcm(&encrypted[3..], v10_key)
            } else {
                Err(format!("unexpected key length for v11: {}", v10_key.len()))
            }
        }
        _ => {
            #[cfg(target_os = "windows")]
            {
                decrypt_windows_dpapi(encrypted).ok_or_else(|| "DPAPI decryption failed".into())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err(format!("unknown version prefix: {:?}", version))
            }
        }
    }
}

/// AES-128-CBC 解密（macOS/Linux Chromium v10/v11）
///
/// IV 固定为 `[0x20; 16]`（Chromium 的默认 IV），
/// 使用 PKCS7 填充。
fn decrypt_aes_128_cbc(data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 16 {
        return Err(format!("key must be 16 bytes, got {}", key.len()));
    }
    let iv = [0x20u8; 16];
    let key_arr: [u8; 16] = key.try_into().map_err(|_| "key conversion failed")?;

    let decryptor = Aes128CbcDec::new(&key_arr.into(), &iv.into());
    let mut buf = data.to_vec();
    let padding_len = (16 - (data.len() % 16)) % 16;
    buf.extend(vec![0; padding_len]);

    let decrypted = decryptor
        .decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buf)
        .map_err(|e| format!("AES-128-CBC decryption failed: {:?}", e))?;

    Ok(decrypted.to_vec())
}

/// AES-256-GCM 解密（Windows Chromium v10/v20）
///
/// 数据布局：12 字节 nonce + 密文 + 16 字节 GCM tag
fn decrypt_aes_256_gcm(data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err(format!("key must be 32 bytes, got {}", key.len()));
    }
    if data.len() < 12 + 16 {
        return Err("data too short for AES-256-GCM (need nonce + tag)".into());
    }

    let nonce = &data[..12];
    let ciphertext_with_tag = &data[12..];

    use aes_gcm::{Aes256Gcm, Nonce};
    use aes_gcm::aead::{Aead, KeyInit};

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("AES-256-GCM key init failed: {:?}", e))?;
    let nonce = Nonce::from_slice(nonce);

    cipher
        .decrypt(nonce, ciphertext_with_tag)
        .map_err(|e| format!("AES-256-GCM decryption failed: {:?}", e))
}

/// 去除 Chrome 130+ 的 Cookie SHA256 域名哈希前缀
///
/// Chrome 130 版本开始在解密后的 Cookie 值前添加 `SHA256(host_key)` 哈希（32 字节），
/// 用于域绑定验证。此函数检测并去除该前缀。
///
/// # 参数
/// - `value` — 解密后的 Cookie 值字节
/// - `host` — Cookie 的域名
///
/// # 返回
/// 去除哈希前缀后的 Cookie 值
pub fn strip_cookie_hash(value: &[u8], host: &str) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(host.as_bytes());
    let hash = hasher.finalize();

    if value.len() > 32 && value[..32] == hash[..] {
        value[32..].to_vec()
    } else {
        value.to_vec()
    }
}

// ── Firefox NSS 解密 ─────────────────────────────────────────────────────

/// 解密 Firefox 登录数据
///
/// Firefox 使用 NSS (Network Security Services) 加密体系：
/// 1. 从 `key4.db` 派生主密钥
/// 2. 使用 3DES-CBC 或 AES-256-CBC 解密 `logins.json` 中的字段
///
/// # 参数
/// - `logins_json` — `logins.json` 文件的原始字节
/// - `key4_db_path` — `key4.db` 文件的绝对路径
///
/// # 返回
/// `Vec<(hostname, username, password)>` 三元组列表
pub fn decrypt_firefox_nss(
    logins_json: &[u8],
    key4_db_path: &str,
) -> Result<Vec<(String, String, String)>, String> {
    let master_key = derive_firefox_master_key(key4_db_path)?;

    let logins: serde_json::Value =
        serde_json::from_slice(logins_json).map_err(|e| format!("parse logins.json: {}", e))?;

    let entries = logins
        .get("logins")
        .and_then(|l| l.as_array())
        .ok_or("no logins array")?;

    let mut results = Vec::new();
    for entry in entries {
        let hostname = entry
            .get("hostname")
            .and_then(|h| h.as_str())
            .unwrap_or("")
            .to_string();
        let enc_user_b64 = entry
            .get("encryptedUsername")
            .and_then(|e| e.as_str())
            .unwrap_or("");
        let enc_pass_b64 = entry
            .get("encryptedPassword")
            .and_then(|e| e.as_str())
            .unwrap_or("");

        let enc_user = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            enc_user_b64,
        )
        .unwrap_or_default();
        let enc_pass = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            enc_pass_b64,
        )
        .unwrap_or_default();

        let username = decrypt_nss_pbe(&enc_user, &master_key).unwrap_or_default();
        let password = decrypt_nss_pbe(&enc_pass, &master_key).unwrap_or_default();

        results.push((hostname, username, password));
    }

    Ok(results)
}

/// 从 Firefox key4.db 派生 NSS 主密钥
///
/// 密钥派生流程（PBE-SHA1-3DES）：
/// 1. 读取 `metaData` 表的 `globalSalt`
/// 2. 读取 `nssPrivate` 表的加密条目 `a11`
/// 3. `hp = SHA1(globalSalt)`
/// 4. `ck = SHA1(hp || entrySalt)`
/// 5. `hmac1 = HMAC-SHA1(ck, paddedSalt)`
/// 6. `k1 = HMAC-SHA1(ck, paddedSalt || entrySalt)`
/// 7. `k2 = HMAC-SHA1(ck, hmac1 || entrySalt)`
/// 8. `dk = k1 || k2`（40 字节）
/// 9. 主密钥 = `dk[..24]`（3DES 24 字节密钥）
fn derive_firefox_master_key(key4_db_path: &str) -> Result<Vec<u8>, String> {
    let conn = rusqlite::Connection::open(key4_db_path)
        .map_err(|e| format!("open key4.db: {}", e))?;

    let global_salt: Vec<u8> = conn
        .query_row(
            "SELECT item1 FROM metaData WHERE id = 'password-check'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("read globalSalt: {}", e))?;

    let nss_private: Vec<u8> = conn
        .query_row(
            "SELECT a11 FROM nssPrivate WHERE a102 = x'f8000000000000000000000000000001'",
            [],
            |row| row.get(0),
        )
        .or_else(|_| {
            conn.query_row(
                "SELECT a11 FROM nssPrivate",
                [],
                |row| row.get(0),
            )
        })
        .map_err(|e| format!("read nssPrivate: {}", e))?;

    let hp = sha1_hash(&global_salt);
    let ck = sha1_hash_concat(&hp, &nss_private);

    let mut padded_salt = vec![0u8; 20];
    padded_salt.copy_from_slice(&nss_private[..20.min(nss_private.len())]);
    while padded_salt.len() % 8 != 0 {
        padded_salt.push(0x04);
    }

    let hmac1 = hmac_sha1(&ck, &padded_salt);
    let k1 = hmac_sha1(&ck, &concat(&padded_salt, &nss_private));
    let k2 = hmac_sha1(&ck, &concat(&hmac1, &nss_private));
    let dk = concat(&k1, &k2);

    if dk.len() >= 24 {
        Ok(dk[..24].to_vec())
    } else {
        Err("derived key too short".into())
    }
}

/// 解密 NSS PBE 加密的字段
///
/// 根据加密数据的首字节判断加密类型：
/// - `0xf8` 或 `0x04` 开头 → ASN.1 PBE 结构
/// - 其他 → 直接使用主密钥 + IV 解密
fn decrypt_nss_pbe(encrypted: &[u8], master_key: &[u8]) -> Result<String, String> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }

    if encrypted[0] == 0xf8 || (encrypted.len() > 2 && encrypted[0] == 0x04) {
        decrypt_nss_asn1_pbe(encrypted, master_key)
    } else {
        let iv_len = if master_key.len() == 24 { 8 } else { 16 };
        if encrypted.len() < iv_len {
            return Ok(String::new());
        }
        let iv = &encrypted[..iv_len];
        let ciphertext = &encrypted[iv_len..];

        if master_key.len() == 24 {
            decrypt_3des_cbc(ciphertext, master_key, iv)
        } else {
            decrypt_aes_256_cbc_raw(ciphertext, master_key, iv)
        }
    }
}

/// 解密 NSS ASN.1 PBE 加密结构
///
/// 解析 ASN.1 编码的 entrySalt，使用与 [`derive_firefox_master_key`] 相同的
/// PBE-SHA1 密钥派生流程生成 3DES 密钥进行解密。
fn decrypt_nss_asn1_pbe(encrypted: &[u8], master_key: &[u8]) -> Result<String, String> {
    let entry_salt = if encrypted.len() > 2 && encrypted[0] == 0x04 {
        let salt_len = encrypted[1] as usize;
        if encrypted.len() < 2 + salt_len {
            return Err("ASN1 PBE: salt too long".into());
        }
        encrypted[2..2 + salt_len].to_vec()
    } else {
        vec![]
    };

    let hp = sha1_hash(master_key);
    let ck = sha1_hash_concat(&hp, &entry_salt);

    let mut padded_salt = entry_salt.clone();
    while padded_salt.len() % 8 != 0 {
        padded_salt.push(0x04);
    }

    let hmac1 = hmac_sha1(&ck, &padded_salt);
    let k1 = hmac_sha1(&ck, &concat(&padded_salt, &entry_salt));
    let k2 = hmac_sha1(&ck, &concat(&hmac1, &entry_salt));
    let dk = concat(&k1, &k2);

    let data_start = if encrypted[0] == 0x04 {
        2 + encrypted[1] as usize
    } else {
        0
    };

    if encrypted.len() <= data_start {
        return Ok(String::new());
    }

    let ciphertext = &encrypted[data_start..];
    if dk.len() >= 24 && !ciphertext.is_empty() {
        decrypt_3des_cbc(ciphertext, &dk[..24], &dk[24..32.min(dk.len())])
    } else {
        Ok(String::new())
    }
}

/// 3DES-CBC 解密
///
/// 用于 Firefox NSS 旧版本（< v144）的登录数据解密。
/// 密钥 24 字节，IV 8 字节，PKCS7 填充。
fn decrypt_3des_cbc(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> Result<String, String> {
    if key.len() != 24 || iv.len() != 8 {
        return Err(format!("3DES: key=24, iv=8 required, got key={}, iv={}", key.len(), iv.len()));
    }

    use des::TdesEde3;
    use cbc::Decryptor as CbcDec;
    use cbc::cipher::{BlockDecryptMut, KeyIvInit};

    type TdesCbcDec = CbcDec<TdesEde3>;

    let key_arr: [u8; 24] = key.try_into().map_err(|_| "key conv")?;
    let iv_arr: [u8; 8] = iv.try_into().map_err(|_| "iv conv")?;

    let decryptor = TdesCbcDec::new(&key_arr.into(), &iv_arr.into());
    let mut buf = ciphertext.to_vec();
    let padding_len = (8 - (ciphertext.len() % 8)) % 8;
    buf.extend(vec![0; padding_len]);

    let decrypted = decryptor
        .decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buf)
        .map_err(|e| format!("3DES-CBC decrypt failed: {:?}", e))?;

    Ok(String::from_utf8_lossy(decrypted).to_string())
}

/// AES-256-CBC 解密（Firefox v144+ NSS 新版）
///
/// Firefox 144 开始支持 AES-256-CBC 替代 3DES-CBC。
/// 密钥 32 字节，IV 16 字节，PKCS7 填充。
fn decrypt_aes_256_cbc_raw(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> Result<String, String> {
    if key.len() != 32 || iv.len() != 16 {
        return Err(format!("AES-256-CBC: key=32, iv=16 required"));
    }

    use aes::Aes256;
    use cbc::Decryptor as CbcDec;
    use cbc::cipher::{BlockDecryptMut, KeyIvInit};

    type Aes256CbcDec = CbcDec<Aes256>;

    let key_arr: [u8; 32] = key.try_into().map_err(|_| "key conv")?;
    let iv_arr: [u8; 16] = iv.try_into().map_err(|_| "iv conv")?;

    let decryptor = Aes256CbcDec::new(&key_arr.into(), &iv_arr.into());
    let mut buf = ciphertext.to_vec();
    let padding_len = (16 - (ciphertext.len() % 16)) % 16;
    buf.extend(vec![0; padding_len]);

    let decrypted = decryptor
        .decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buf)
        .map_err(|e| format!("AES-256-CBC decrypt failed: {:?}", e))?;

    Ok(String::from_utf8_lossy(decrypted).to_string())
}

// ── 辅助函数 ───────────────────────────────────────────────────────────

/// SHA-1 哈希
fn sha1_hash(data: &[u8]) -> Vec<u8> {
    use sha1::Digest;
    let mut hasher = sha1::Sha1::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// SHA-1 哈希（拼接两个输入）
fn sha1_hash_concat(a: &[u8], b: &[u8]) -> Vec<u8> {
    use sha1::Digest;
    let mut hasher = sha1::Sha1::new();
    hasher.update(a);
    hasher.update(b);
    hasher.finalize().to_vec()
}

/// HMAC-SHA1
fn hmac_sha1(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::Mac;
    let mut mac = HmacSha1::new_from_slice(key).expect("HMAC key");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// 拼接两个字节切片
fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    result.extend_from_slice(a);
    result.extend_from_slice(b);
    result
}

// ── 文件操作 ─────────────────────────────────────────────────────────────

/// 将浏览器数据库文件复制到临时目录
///
/// 浏览器运行时会锁定 SQLite 数据库文件（尤其是 Windows），
/// 因此需要先将文件复制到临时目录再读取。
/// 同时会尝试复制 `-wal` 和 `-shm` 侧载文件以保证数据完整性。
///
/// # 参数
/// - `file_path` — 源文件路径
///
/// # 返回
/// 临时目录中的文件路径
pub fn copy_to_temp(file_path: &str) -> Result<std::path::PathBuf, String> {
    let src = std::path::Path::new(file_path);
    if !src.exists() {
        return Err(format!("file not found: {}", file_path));
    }

    let temp_dir = std::env::temp_dir().join("oasis-browser-extract");
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("create temp dir: {}", e))?;

    let file_name = src.file_name().unwrap_or_default().to_string_lossy();
    let unique_name = format!("{}-{}-{}", file_name, std::process::id(), {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    });
    let dest = temp_dir.join(&unique_name);

    std::fs::copy(src, &dest).map_err(|e| format!("copy file: {}", e))?;

    for ext in &["-wal", "-shm"] {
        let sidecar = src.with_extension({
            let base = src.extension().map(|e| format!("{}{}", e.to_string_lossy(), ext))
                .unwrap_or_else(|| ext[1..].to_string());
            base
        });
        if sidecar.exists() {
            let sidecar_dest = temp_dir.join(format!("{}{}", unique_name, ext));
            let _ = std::fs::copy(&sidecar, &sidecar_dest);
        }
    }

    Ok(dest)
}
