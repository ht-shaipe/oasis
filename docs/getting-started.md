---
AIGC:
    Label: "1"
    ContentProducer: 001191440300708461136T1XGW3
    ProduceID: c8bc0470d7bdfb9241ace971be1a0b2c_d6154fa95c9011f19299525400d9a7a1
    ReservedCode1: ROnU+Ta0ohtgBFA5V9jCTeuvm29XR+j/NqsHMC066m/fEI7dO2JXMkUFMPDOWwHvjcRJTeB3mLrC05NAinzQBgQ0iUf7Q7Z4JHp34/pNWGmEU2DrJYpG6WZI27aum/E8a9MXL2FR3G7xv6OfoZQxyuYwTXHW7p9QHRb50i6wCDavbDPzHzCRnk7KvI8=
    ContentPropagator: 001191440300708461136T1XGW3
    PropagateID: c8bc0470d7bdfb9241ace971be1a0b2c_d6154fa95c9011f19299525400d9a7a1
    ReservedCode2: ROnU+Ta0ohtgBFA5V9jCTeuvm29XR+j/NqsHMC066m/fEI7dO2JXMkUFMPDOWwHvjcRJTeB3mLrC05NAinzQBgQ0iUf7Q7Z4JHp34/pNWGmEU2DrJYpG6WZI27aum/E8a9MXL2FR3G7xv6OfoZQxyuYwTXHW7p9QHRb50i6wCDavbDPzHzCRnk7KvI8=
---

# 快速上手

## 前置要求

- [Rust](https://www.rust-lang.org/tools/install) stable
- [Bun](https://bun.sh/)
- macOS: Xcode Command Line Tools
- Windows: WebView2 Runtime

## 安装依赖

```bash
git clone <repository-url>
cd oasis
bun install
```

## 开发环境

```bash
bun tauri dev
```

- Vite Dev Server: `http://localhost:1420`
- HMR WebSocket: `ws://localhost:1421`
- Rust 代码修改后自动重编译
- 前端支持热重载

## 生产构建

```bash
bun tauri build
```

产物位于 `src-tauri/target/release/bundle/`。

- macOS: `.app` + `.dmg`
- Windows: `.msi` / `.exe`

## 项目结构速览

| 目录 | 说明 |
|------|------|
| `src/apps/` | 11 个内置应用组件 |
| `src/components/` | 9 个桌面 Shell 系统组件 |
| `src/config/` | 应用注册表、菜单栏配置 |
| `src/store/` | Pinia 状态管理 (locale/theme) |
| `src/locales/` | 国际化语言文件 (zh-CN/en) |
| `src-tauri/src/` | Rust 后端核心 (命令/网络代理) |
| `src-tauri/crates/` | 3 个功能 crate (credential/toolbox/browser) |
| `docs/` | 详细技术文档 |
*（内容由AI生成，仅供参考）*
