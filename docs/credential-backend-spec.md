# Credential Manager - Rust Backend Spec

## Project
Oasis Tauri 2 应用，项目根目录: /Users/shaipe/workspace/rust/tools/oasis
Rust 代码目录: src-tauri/src/

## Task
在 `src-tauri/src/credential/` 目录下实现凭证管理的完整 Rust 后端模块，包括加密、数据库、Tauri commands。

## 目录结构
```
src-tauri/src/
├── lib.rs                    # 修改: 注册新模块和 commands
└── credential/
    ├── mod.rs                # 模块入口，导出子模块和注册函数
    ├── crypto.rs             # 加密/解密/密钥派生
    ├── db.rs                 # SQLite 数据库操作
    ├── models.rs             # 数据结构定义
    └── commands.rs           # Tauri #[command] 接口
```

## 1. Cargo.toml 新增依赖

在 `[dependencies]` 中添加:
```toml
rusqlite = { version = "0.34", features = ["bundled"] }
ring = "0.17"
rand = "0.9"
base64 = "0.22"
```

## 2. crypto.rs - 加密模块

### 密钥派生链路
```
用户主密钥 (Master Password string)
  → PBKDF2-SHA256(600,000 rounds, 32-byte random salt)
  → key_hash (用于验证，存 DB)
  → HKDF-SHA256(salt=dek_salt 32-byte, info=b"oasis-credential-key")
  → DEK (32 bytes, base64 编码传给前端)
```

### 必须实现的函数
```rust
// 生成主密钥验证材料
pub fn derive_master_key(password: &str, salt: &[u8]) -> [u8; 32]

// 从主密钥派生 DEK
pub fn derive_dek(password: &str, dek_salt: &[u8]) -> [u8; 32]

// AES-256-GCM 加密，返回 (ciphertext, nonce)
pub fn encrypt(dek: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12])>

// AES-256-GCM 解密
pub fn decrypt(dek: &[u8; 32], ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>>

// 生成随机 salt (32 bytes)
pub fn generate_salt() -> [u8; 32]

// 生成随机 nonce (12 bytes)
pub fn generate_nonce() -> [u8; 12]
```

使用 `ring` crate:
- `ring::pbkdf2` for PBKDF2
- `ring::hkdf` for HKDF
- `ring::aead` for AES-256-GCM (`ring::aead::AES_256_GCM`)

## 3. db.rs - 数据库模块

### 初始化
```rust
pub fn init_db(app_data_dir: &Path) -> Result<rusqlite::Connection>
```
- 创建 `credentials.db` 文件于 Tauri app data 目录
- 执行建表 SQL
- 插入预置分类数据（如果分类表为空）

### Schema
```sql
CREATE TABLE IF NOT EXISTS master_key (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    key_hash      BLOB NOT NULL,
    salt          BLOB NOT NULL,
    dek_salt      BLOB NOT NULL,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS categories (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL UNIQUE,
    icon          TEXT,
    sort_order    INTEGER DEFAULT 0,
    created_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS credentials (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id   INTEGER NOT NULL REFERENCES categories(id),
    title         TEXT NOT NULL,
    username      TEXT,
    url           TEXT,
    encrypted_data BLOB NOT NULL,
    nonce         BLOB NOT NULL,
    tags          TEXT,
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);
```

### 预置分类
社交媒体, 邮箱, 开发工具, API密钥, 云服务, 数据库, 自定义

### CRUD 函数
```rust
// 主密钥
pub fn is_master_key_set(conn: &Connection) -> Result<bool>
pub fn set_master_key(conn: &Connection, key_hash: &[u8], salt: &[u8], dek_salt: &[u8]) -> Result<()>
pub fn verify_master_key(conn: &Connection, key_hash: &[u8]) -> Result<bool>
pub fn get_master_key_salts(conn: &Connection) -> Result<(Vec<u8>, Vec<u8>)>

// 分类
pub fn list_categories(conn: &Connection) -> Result<Vec<Category>>
pub fn create_category(conn: &Connection, name: &str, icon: Option<&str>) -> Result<Category>

// 凭证
pub fn list_credentials(conn: &Connection, category_id: Option<i64>) -> Result<Vec<Credential>>
pub fn get_credential(conn: &Connection, id: i64) -> Result<Credential>
pub fn create_credential(conn: &Connection, cred: &NewCredential) -> Result<Credential>
pub fn update_credential(conn: &Connection, id: i64, cred: &UpdateCredential) -> Result<Credential>
pub fn delete_credential(conn: &Connection, id: i64) -> Result<()>
```

## 4. models.rs - 数据结构

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credential {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub encrypted_data: Vec<u8>,  // 原始 bytes，不序列化到前端
    pub nonce: Vec<u8>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// 返回给前端的视图（不含加密原始数据）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialView {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub category_name: Option<String>,
}

// 返回给前端的详情（含解密后的敏感数据）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialDetail {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub sensitive_data: SensitiveData,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub category_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SensitiveData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<std::collections::HashMap<String, String>>,
}

// 创建凭证请求
#[derive(Debug, Deserialize)]
pub struct NewCredential {
    pub category_id: i64,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub sensitive_data_json: String,  // 前端传来的 SensitiveData JSON 字符串
    pub dek_base64: String,           // 前端传来的 DEK (base64)
    pub tags: Option<String>,
    pub notes: Option<String>,
}

// 更新凭证请求
#[derive(Debug, Deserialize)]
pub struct UpdateCredential {
    pub id: i64,
    pub category_id: Option<i64>,
    pub title: Option<String>,
    pub username: Option<String>,
    pub url: Option<String>,
    pub sensitive_data_json: Option<String>,
    pub dek_base64: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
}
```

## 5. commands.rs - Tauri Commands

```rust
#[tauri::command]
pub fn is_master_key_set(app: tauri::AppHandle) -> Result<bool, String>

#[tauri::command]
pub fn setup_master_key(app: tauri::AppHandle, password: String) -> Result<String, String>
// 返回 DEK base64

#[tauri::command]
pub fn verify_master_key(app: tauri::AppHandle, password: String) -> Result<String, String>
// 验证主密钥，成功返回 DEK base64

#[tauri::command]
pub fn list_categories(app: tauri::AppHandle) -> Result<Vec<Category>, String>

#[tauri::command]
pub fn create_category(app: tauri::AppHandle, name: String, icon: Option<String>) -> Result<Category, String>

#[tauri::command]
pub fn list_credentials(app: tauri::AppHandle, category_id: Option<i64>) -> Result<Vec<CredentialView>, String>

#[tauri::command]
pub fn get_credential(app: tauri::AppHandle, id: i64, dek_base64: String) -> Result<CredentialDetail, String>

#[tauri::command]
pub fn create_credential(app: tauri::AppHandle, credential: NewCredential) -> Result<CredentialView, String>

#[tauri::command]
pub fn update_credential(app: tauri::command, app: tauri::AppHandle, credential: UpdateCredential) -> Result<CredentialView, String>

#[tauri::command]
pub fn delete_credential(app: tauri::AppHandle, id: i64) -> Result<(), String>

#[tauri::command]
pub fn change_master_key(app: tauri::AppHandle, old_password: String, new_password: String) -> Result<String, String>
// 返回新 DEK base64
```

### DB 连接管理
每个 command 中通过 `app.path().app_data_dir()` 获取数据目录，打开 SQLite 连接。不要用全局 state 管理连接，保持简单。

## 6. lib.rs 修改

```rust
mod credential;

// 在 tauri::Builder 的 invoke_handler 中注册所有 commands:
tauri::generate_handler![
    greet,
    update_tray_locale,
    credential::commands::is_master_key_set,
    credential::commands::setup_master_key,
    credential::commands::verify_master_key,
    credential::commands::list_categories,
    credential::commands::create_category,
    credential::commands::list_credentials,
    credential::commands::get_credential,
    credential::commands::create_credential,
    credential::commands::update_credential,
    credential::commands::delete_credential,
    credential::commands::change_master_key,
]
```

## 7. 关键注意事项

- **ring crate PBKDF2**: `ring::pbkdf2::derive(PBKDF2_HMAC_SHA256, 600_000.nonzero(), salt, password.as_bytes(), &mut hash)` — 注意 NonZeroU32 的用法
- **ring HKDF**: `ring::hkdf::Prk::new_salt(HKDF_SHA256, dek_salt).extract(password_hash).expand(&[b"oasis-credential-key"], DEK_LEN)`
- **ring AES-256-GCM**: 使用 `ring::aead::seal_in_place_separate_tag` / `open_in_place`
- **错误处理**: 所有 command 返回 `Result<T, String>`，用 `.map_err(|e| e.to_string())`
- **时间格式**: SQLite 中用 ISO 8601 字符串 `chrono` 或 `std::time` 格式化
- **编译**: 最后确保 `cargo build` 通过，0 errors
- **不要修改已有代码的逻辑**，只在 lib.rs 中添加模块注册和 command 注册

## 8. 验证

完成后运行:
```bash
cd /Users/shaipe/workspace/rust/tools/oasis/src-tauri && cargo build
```
确保编译通过。
