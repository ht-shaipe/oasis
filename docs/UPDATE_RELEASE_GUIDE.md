# Oasis 应用更新发布指南

## 更新检查机制概述

应用启动后 5 秒自动检查更新（24 小时内不重复），用户也可手动触发：

- **菜单栏 Logo 菜单** → "软件更新..."
- **设置 → 关于** → 点击检查按钮

检查逻辑：调用 GitHub Releases API 获取最新版本号，与当前版本做语义化版本比较（如 `0.1.0` < `0.5.37`）。

---

## 1. 版本号配置

版本号定义在 `src-tauri/tauri.conf.json` 的 `version` 字段：

```json
{
  "version": "0.1.0"
}
```

**发版前必须更新此字段**，否则用户收到的永远是"已是最新版本"。

---

## 2. 构建发布包

```bash
# 完整构建（前端 + Rust + 打包 .app/.dmg）
bun run tauri:build
# 或
make bundle
```

构建产物位于：

```
src-tauri/target/release/bundle/
├── macos/Oasis.app          # macOS 应用
└── dmg/Oasis_0.1.0_aarch64.dmg  # macOS 安装镜像
```

> `.dmg` 文件名中的版本号和架构来自 `tauri.conf.json` 中的 `version` 和当前机器架构。

---

## 3. 创建 GitHub Release

### 3.1 创建 Git Tag

```bash
# 版本号需与 tauri.conf.json 一致，建议加 v 前缀
git tag v0.5.37
git push origin v0.5.37
```

### 3.2 在 GitHub 上创建 Release

1. 打开 https://github.com/ht-shaipe/oasis/releases/new
2. **Tag**：选择刚推送的 `v0.5.37`
3. **Title**：如 `Oasis 0.5.37`
4. **Description**（Body）：按以下格式编写更新日志（应用会自动解析展示）：

```markdown
### 新功能

- 支持深色模式自动切换
- 新增凭证管理模块

### Bug 修复

- 修复窗口拖拽在多显示器下异常 #12
- 修复 CSV 导出编码问题 #15

### 性能优化

- 启动速度提升 30%
```

格式要求：
- 用 `###` 或 `##` 作为分类标题（如"新功能"、"Bug 修复"）
- 每条更新用 `- ` 或 `* ` 开头的列表项
- 引用 Issue 用 `#编号` 格式（会自动生成可点击链接）
- 引用外部链接用 `[文字](URL)` 格式

5. **上传构建产物**：将 `.dmg` 文件拖拽到 Release 的 Assets 区域上传

> **关键**：必须上传 `.dmg` 文件作为 Asset，应用下载更新时会优先查找 `.dmg` 后缀的 Asset。如果没找到 `.dmg`，则回退到"打开下载页"（跳转 Release 页面）。

### 3.3 发布 Release

点击 **Publish release** 完成发布。

---

## 4. 完整发布流程（Checklist）

```
□ 1. 更新 src-tauri/tauri.conf.json 中的 version
□ 2. 更新 src/apps/Settings/panels/AboutPanel.vue 中的默认版本号（如有）
□ 3. 提交代码：git add . && git commit -m "release: v0.5.37"
□ 4. 打 Tag：git tag v0.5.37 && git push origin main --tags
□ 5. 构建安装包：bun run tauri:build
□ 6. 在 GitHub 创建 Release，上传 .dmg 文件
□ 7. 发布 Release
□ 8. 验证：在旧版本应用中触发"检查更新"，确认弹窗显示正确
```

---

## 5. API 调用说明

更新检查调用的 GitHub API：

```
GET https://api.github.com/repos/ht-shaipe/oasis/releases/latest
```

返回关键字段：

| 字段 | 说明 |
|------|------|
| `tag_name` | 版本标签，如 `v0.5.37`（自动去除 `v` 前缀比较） |
| `body` | Release 描述（Markdown，应用解析为分类列表） |
| `assets[].name` | 附件文件名（查找 `.dmg` 后缀） |
| `assets[].browser_download_url` | 附件下载直链 |
| `html_url` | Release 页面 URL（无 .dmg 时的回退） |
| `published_at` | 发布时间 |

> GitHub API 未认证限制 60 次/小时/IP。如果用户量大，可考虑配置 GitHub Token 增加限额。

---

## 6. 仓库地址配置

当前硬编码在 `src-tauri/src/commands.rs`：

```rust
.get("https://api.github.com/repos/ht-shaipe/oasis/releases/latest")
```

如果仓库迁移，需同步修改此 URL。

---

## 7. 当前版本为何提示"已是最新版本"

可能原因：
1. **GitHub 上没有 Release**：从未创建过 Release，API 返回 404，应用视为无更新
2. **`tauri.conf.json` 版本号 ≥ 最新 Release tag**：版本比较认为无需更新
3. **24 小时内已检查过**：自动检查被跳过（可通过手动检查按钮触发）
