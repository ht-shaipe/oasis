# 快速上手

## 前置要求

- [Rust](https://www.rust-lang.org/tools/install) stable
- [Bun](https://bun.sh/)
- macOS: Xcode Command Line Tools

## 安装依赖

```bash
git clone <repository-url>
cd oasis
bun install
```

## 开发环境

```bash
bun run tauri
# 或
make dev
```

- Vite Dev Server: `http://localhost:1488`
- Rust 代码修改后自动重编译
- 前端支持热重载

## 生产构建

```bash
bun run tauri:build
# 或
make bundle
```

产物位于 `src-tauri/target/release/bundle/`。

- macOS: `.app` + `.dmg`

## 项目结构速览

| 目录 | 说明 |
|------|------|
| `src/apps/` | 14 个内置应用组件 |
| `src/components/` | 桌面 Shell 系统组件 + 通用组件 |
| `src/config/` | 应用注册表、菜单栏配置 |
| `src/store/` | Pinia 状态管理 (locale/theme/fontSize/agent/chat) |
| `src/locales/` | 国际化语言文件 (zh-CN/en) |
| `src/composables/` | 可复用组合函数 (credential/appUpdate/fileDialog) |
| `src-tauri/src/` | Rust 后端核心 (命令/网络代理) |
| `src-tauri/crates/` | 13 个功能 crate |
| `docs/` | 详细技术文档 |

## 其他命令

```bash
bun run dev        # 仅启动前端 Vite 开发服务器
bun run build      # 仅构建前端 (vue-tsc --noEmit && vite build)
make install       # 安装依赖
make git MSG="xxx" # 提交并推送代码
```
