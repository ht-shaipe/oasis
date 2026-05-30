# 凭据存储方案 (Credential Storage)

本文档详细说明 Oasis 凭据管理插件（Credential Manager）的后端存储逻辑和加密流程。

## 🔐 加密架构

为了确保用户密码和敏感数据的安全，我们采用了行业标准的对称加密和密钥派生算法。

### 1. 密钥派生链路 (Key Derivation)

当用户设置或输入主密钥（Master Password）时，程序会执行以下流程：

- **主密钥验证**:
    - 用户输入 `Password` + 随机 `Salt`。
    - **PBKDF2-SHA256**: 迭代 600,000 次。
    - 生成 `key_hash` 用于数据库存储验证。
- **数据加密密钥 (DEK) 派生**:
    - **HKDF-SHA256**: 使用 `key_hash` 和 `dek_salt` 派生。
    - 生成 32 字节的 **DEK** (Data Encryption Key)。
    - DEK 仅存储在内存中，随窗口关闭或手动锁定而销毁。

### 2. 数据加密 (Data Encryption)

- **算法**: AES-256-GCM。
- **流程**:
    - 为每个凭据生成随机的 12 字节 `Nonce`。
    - 使用 `DEK` 加密敏感信息（Sensitive JSON）。
    - 将 `Ciphertext` 和 `Nonce` 存入数据库。

## 🗄️ 数据库设计 (SQLite)

数据库文件位于 Tauri 的 `app_data_dir` 目录下的 `credentials.db`。

### 核心表结构

- **`master_key`**: 存储主密钥元数据。
    - `key_hash`: 派生后的验证 Hash。
    - `salt`: PBKDF2 盐值。
    - `dek_salt`: HKDF 盐值。
- **`credentials`**: 凭据核心表。
    - `title`, `username`, `url`: 明文存储以便搜索。
    - `encrypted_data`: 经过 AES 加密后的密文。
    - `nonce`: 加密所用的 Nonce。
- **`categories`**: 凭据分类表。

## 🛡️ 安全保证

1. **零明文存储**: 数据库中不包含任何主密钥或解密后的敏感数据。
2. **内存隔离**: 解密后的数据仅在前端组件中短暂存在，后端不缓存 DEK。
3. **暴力破解防护**: 极高的 PBKDF2 迭代次数（600,000）显著增加了暴力破解的成本。
4. **数据库加密**: 虽未直接使用 SQLCipher，但核心内容均经过强对称加密。
