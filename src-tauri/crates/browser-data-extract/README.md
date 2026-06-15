# oasis-browser-data-extract

浏览器数据提取库，支持从 Chromium、Firefox、Safari 等主流浏览器中提取密码、Cookie、书签、历史记录、下载记录、信用卡信息和扩展信息。

## 支持的浏览器

| 引擎类型 | 浏览器 |
|----------|--------|
| Chromium | Chrome、Chrome Beta、Chromium、Edge、Brave、Vivaldi、Arc、CocCoc |
| ChromiumOpera | Opera、Opera GX |
| ChromiumYandex | Yandex Browser |
| Firefox | Firefox |
| Safari | Safari（仅 macOS） |

## 支持的数据类型

| 类型 | Chromium | Firefox | Safari |
|------|----------|---------|--------|
| 密码 (Password) | ✅ AES 加密解密 | ✅ NSS PBE 解密 | ❌ |
| Cookie | ✅ AES 加密解密 | ✅ 明文读取 | ⚠️ 待实现 |
| 书签 (Bookmark) | ✅ JSON 解析 | ✅ SQLite | ✅ plist |
| 历史记录 (History) | ✅ SQLite | ✅ SQLite | ✅ SQLite |
| 下载记录 (Download) | ✅ SQLite | ✅ SQLite | ✅ plist |
| 信用卡 (CreditCard) | ✅ AES 加密解密 | ❌ | ❌ |
| 扩展 (Extension) | ✅ JSON | ✅ JSON | ❌ |

## 解密算法

### Chromium (macOS)

1. 通过 `security find-generic-password -wa <label>` 获取 Keychain 中存储的安全密码
2. PBKDF2(password, "saltysalt", 1003, SHA1) → 16 字节密钥
3. AES-128-CBC 解密，IV = `[0x20; 16]`

> macOS 上首次调用时可能弹出系统授权对话框，要求用户输入登录密码以允许访问 Keychain。

### Chromium (Windows)

1. 从 `Local State` 读取 `os_crypt.encrypted_key`（Base64 编码）
2. Base64 解码后去除 "DPAPI" 前缀
3. 调用 Windows DPAPI `CryptUnprotectData` 解密得到 32 字节密钥
4. AES-256-GCM 解密（v10/v20 前缀，布局：12 字节 nonce + 密文 + 16 字节 GCM tag）

### Chromium (Linux)

1. PBKDF2("peanuts", "saltysalt", 1, SHA1) → 16 字节密钥（无 Keyring 时）
2. 或通过 D-Bus Secret Service 获取密钥后同样使用 PBKDF2 派生
3. AES-128-CBC 解密

### Firefox

1. 从 `key4.db` 读取 `metaData.globalSalt` 和 `nssPrivate.a11` 条目
2. NSS PBE-SHA1-3DES 密钥派生：
   - `hp = SHA1(globalSalt)`
   - `ck = SHA1(hp || entrySalt)`
   - `hmac1 = HMAC-SHA1(ck, paddedSalt)`
   - `k1 = HMAC-SHA1(ck, paddedSalt || entrySalt)`
   - `k2 = HMAC-SHA1(ck, hmac1 || entrySalt)`
   - `dk = k1 || k2`（40 字节）
   - 主密钥 = `dk[..24]`（3DES 24 字节密钥）
3. 使用 3DES-CBC 或 AES-256-CBC 解密 `logins.json` 中的 base64 编码 ASN.1 PBE 加密字段

## 模块结构

```
src/
├── lib.rs        # Crate 入口，模块声明与 crate 级文档
├── models.rs     # 数据模型（BrowserKind、DataType、LoginEntry 等）与时间戳转换
├── browser.rs    # 浏览器发现：扫描已安装浏览器及其 Profile
├── crypto.rs     # 解密算法：AES-128-CBC、AES-256-GCM、3DES-CBC、PBKDF2、NSS PBE
├── extract.rs    # 数据提取核心逻辑：各浏览器各数据类型的提取实现
└── commands.rs   # Tauri 命令接口：前端 IPC 桥接层
```

### 模块说明

| 模块 | 职责 |
|------|------|
| `browser` | 平台相关的浏览器数据目录扫描、Profile 发现（Chromium `Local State`、Firefox `profiles.ini`、Safari 默认 Profile） |
| `crypto` | Chromium 密钥获取（macOS Keychain / Windows DPAPI / Linux PBKDF2）、加密值解密、Firefox NSS 解密、SQLite 文件临时复制 |
| `extract` | 按浏览器引擎类型分派提取逻辑，每个数据类型对应独立的提取函数 |
| `models` | 所有数据结构定义、Chromium/WebKit 时间戳转换工具函数 |
| `commands` | `#[tauri::command]` 标注的前端可调用接口，由 `build.rs` 自动发现注册 |

## 使用方式

### Rust API

```rust
use oasis_browser_data_extract::{browser, extract, models::DataType};

// 发现已安装的浏览器
let browsers = browser::discover_browsers();
for b in &browsers {
    println!("{}: {} ({:?}), {} profiles", b.key, b.name, b.kind, b.profiles.len());
}

// 提取指定浏览器的密码
let results = extract::extract_from_browser("chrome", &[DataType::Password]).unwrap();
for r in results {
    for login in &r.logins {
        println!("{}: {} / {}", login.url, login.username, login.password);
    }
}

// 提取多种数据类型
let results = extract::extract_from_browser("firefox", &[DataType::Bookmark, DataType::History]).unwrap();
for r in results {
    println!("[{}] {} entries", r.data_type.label(), r.bookmarks.len() + r.history.len());
}
```

### Tauri 前端调用

```typescript
import { invoke } from '@tauri-apps/api/core'

// 发现浏览器
const browsers = await invoke('discover_browsers')

// 提取 Chrome 密码
const passwords = await invoke('extract_browser_passwords', { browserKey: 'chrome' })

// 提取所有浏览器的 Cookie 和书签
const results = await invoke('extract_all_browser_data', {
  dataTypes: ['cookie', 'bookmark']
})

// 提取指定浏览器的多种数据
const data = await invoke('extract_browser_data', {
  browserKey: 'edge',
  dataTypes: ['password', 'history', 'download']
})
```

### Tauri 命令列表

| 命令 | 参数 | 返回类型 | 说明 |
|------|------|----------|------|
| `discover_browsers` | 无 | `Vec<BrowserInfo>` | 扫描已安装浏览器 |
| `extract_browser_data` | `browser_key`, `data_types` | `Vec<BrowserExtractResult>` | 提取指定浏览器的指定数据类型 |
| `extract_all_browser_data` | `data_types` | `Vec<BrowserExtractResult>` | 从所有浏览器提取指定数据类型 |
| `extract_browser_passwords` | `browser_key` | `Vec<LoginEntry>` | 提取密码（便捷方法） |
| `extract_browser_cookies` | `browser_key` | `Vec<CookieEntry>` | 提取 Cookie（便捷方法） |
| `extract_browser_bookmarks` | `browser_key` | `Vec<BookmarkEntry>` | 提取书签（便捷方法） |
| `extract_browser_history` | `browser_key` | `Vec<HistoryEntry>` | 提取历史记录（便捷方法） |
| `extract_browser_downloads` | `browser_key` | `Vec<DownloadEntry>` | 提取下载记录（便捷方法） |

## 数据模型

### BrowserInfo

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `String` | 浏览器标识键，如 `"chrome"`、`"firefox"` |
| `name` | `String` | 显示名称，如 `"Google Chrome"` |
| `kind` | `BrowserKind` | 引擎类型枚举 |
| `user_data_dir` | `String` | 用户数据目录绝对路径 |
| `profiles` | `Vec<ProfileInfo>` | 已发现的 Profile 列表 |

### LoginEntry

| 字段 | 类型 | 说明 |
|------|------|------|
| `url` | `String` | 网站 URL |
| `username` | `String` | 用户名 |
| `password` | `String` | 解密后的密码 |
| `created_at` | `Option<String>` | 创建时间（`"YYYY-MM-DD HH:MM:SS"`） |

### CookieEntry

| 字段 | 类型 | 说明 |
|------|------|------|
| `host` | `String` | Cookie 所属域名 |
| `path` | `String` | Cookie 路径 |
| `name` | `String` | Cookie 名称 |
| `value` | `String` | 解密后的 Cookie 值 |
| `is_secure` | `bool` | 是否仅 HTTPS |
| `is_http_only` | `bool` | 是否仅 HTTP |
| `expires_at` | `Option<String>` | 过期时间 |

### BookmarkEntry

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | 自增 ID |
| `name` | `String` | 书签名称 |
| `url` | `String` | 书签 URL |
| `folder` | `String` | 文件夹路径（如 `"Bookmarks Bar/Dev/Rust"`） |
| `created_at` | `Option<String>` | 创建时间 |

### HistoryEntry

| 字段 | 类型 | 说明 |
|------|------|------|
| `url` | `String` | 页面 URL |
| `title` | `String` | 页面标题 |
| `visit_count` | `i32` | 访问次数 |
| `last_visit` | `Option<String>` | 最后访问时间 |

### DownloadEntry

| 字段 | 类型 | 说明 |
|------|------|------|
| `url` | `String` | 下载源 URL |
| `target_path` | `String` | 本地保存路径 |
| `total_bytes` | `i64` | 文件大小（字节） |
| `start_time` | `Option<String>` | 下载开始时间 |

### CreditCardEntry

| 字段 | 类型 | 说明 |
|------|------|------|
| `guid` | `String` | 唯一标识 |
| `name` | `String` | 持卡人姓名 |
| `number` | `String` | 解密后的卡号 |
| `exp_month` | `String` | 过期月份 |
| `exp_year` | `String` | 过期年份 |

### ExtensionEntry

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | 扩展名称 |
| `id` | `String` | 扩展 ID |
| `description` | `String` | 扩展描述 |
| `version` | `String` | 版本号 |
| `enabled` | `bool` | 是否已启用 |

## 时间戳转换

| 来源 | 纪元 | 偏移量 | 函数 |
|------|------|--------|------|
| Chromium | 自 1601-01-01 UTC 的微秒数 | -11644473600000000 | `chromium_epoch_to_datetime` |
| Firefox (PRTime) | 自 1970-01-01 UTC 的微秒数 | 0 | 直接除以 1000000 |
| Safari (Core Data) | 自 2001-01-01 UTC 的秒数 | +978307200 | `webkit_epoch_to_datetime` |

## 浏览器数据目录路径

| 平台 | 浏览器 | 路径 |
|------|--------|------|
| macOS | Chrome | `~/Library/Application Support/Google/Chrome` |
| macOS | Edge | `~/Library/Application Support/Microsoft Edge` |
| macOS | Firefox | `~/Library/Application Support/Firefox` |
| macOS | Safari | `~/Library/Safari` |
| Linux | Chrome | `~/.config/google-chrome` |
| Linux | Firefox | `~/.mozilla/firefox` |
| Windows | Chrome | `%LOCALAPPDATA%\Google\Chrome\User Data` |
| Windows | Edge | `%LOCALAPPDATA%\Microsoft\Edge\User Data` |
| Windows | Firefox | `%APPDATA%\Mozilla\Firefox` |
| Windows | Opera | `%APPDATA%\Opera Software\Opera Stable` |

## Chromium 加密值版本前缀

| 前缀 | 平台 | 解密方式 |
|------|------|----------|
| `v10` | macOS/Linux | AES-128-CBC（16 字节密钥） |
| `v10` | Windows | AES-256-GCM（32 字节 DPAPI 密钥） |
| `v11` | Linux | AES-128-CBC（D-Bus Secret Service 密钥） |
| `v20` | Windows | AES-256-GCM（App-Bound Encryption） |
| 无前缀 | Windows（Chrome <80） | 原始 DPAPI blob |

## 注意事项

- 浏览器运行时会锁定 SQLite 数据库文件，本库通过将文件复制到临时目录 (`/tmp/oasis-browser-extract/`) 来绕过此限制，同时复制 `-wal` 和 `-shm` 侧载文件以保证数据完整性
- macOS 上访问 Keychain 可能触发系统授权弹窗
- Windows DPAPI 解密需要 `windows-sys` crate
- Safari Cookie（`Cookies.binarycookies`）解析尚未实现
- Firefox 不支持信用卡数据提取
- Chrome 130+ 的 Cookie 值包含 SHA256 域名哈希前缀，本库会自动检测并去除

## 依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| `aes` | 0.8 | AES 加解密 |
| `aes-gcm` | 0.10 | AES-256-GCM（Windows Chromium） |
| `cbc` | 0.1 | CBC 模式（AES-128-CBC / 3DES-CBC） |
| `des` | 0.8 | 3DES-EDE3（Firefox NSS） |
| `pbkdf2` | 0.12 | PBKDF2 密钥派生 |
| `hmac` | 0.12 | HMAC-SHA1（Firefox NSS） |
| `sha1` | 0.10 | SHA-1 哈希 |
| `sha2` | 0.10 | SHA-256（Cookie 哈希前缀） |
| `rusqlite` | 0.34 | SQLite 数据库读取 |
| `plist` | 1 | Safari plist 文件解析 |
| `base64` | 0.22 | Base64 编解码 |
| `serde` / `serde_json` | 1 | 数据序列化 |
| `chrono` | 0.4 | 时间戳处理 |
| `dirs` | 6 | 平台目录路径 |
| `tauri` | 2 | Tauri 命令接口 |
