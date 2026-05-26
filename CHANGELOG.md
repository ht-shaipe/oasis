# 变更日志

本文件记录 Oasis 项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

---

## [未发布]

### 计划中
- 插件间通信（plugin-ipc 协议完整实现）
- 凭据管理插件（AES-GCM 加密）
- 网络工具插件
- 插件热重载
- LSP 集成（代码补全/诊断）
- 插件市场

---

## [0.1.0] - 2026-05-24

### 新增
- 🎉 首次发布
- ✨ 基础桌面应用框架（GPUI + Rust）
- 🖥️ 浮动窗口管理系统
- 🚀 应用启动器（全屏网格）
- 🎣 底部浮动 Dock 栏
- 🎨 主题系统（深色/浅色/自定义）
- 🌍 国际化支持（中英文）
- 🖼️ 桌面背景自定义
- 📦 系统托盘支持

### 插件系统
- 🔌 三种插件模式支持
  - 静态链接插件（rlib）
  - 动态库插件（cdylib）
  - WASM 插件（wasmi 沙箱）
  - 独立进程插件

### 内置插件
- 📝 Markdown 编辑器插件
  - 基于 ropey Rope 高性能文本编辑
  - 语法高亮
  - 文件浏览器
  - 撤销/重做功能
- 🗒️ 记事本插件
  - 轻量级文本编辑
  - 实时字数/行数统计
- 🧰 工具箱插件
  - CSV 统计/分割/转换
  - 批量重命名
  - Excel 处理
  - API 请求
  - JSON 合并

### 开发工具
- 📚 完整的插件开发文档
- 🔧 Plugin SDK 和 UI Schema
- 🎯 插件示例和模板
- 🌐 Web 版本支持（WASM）

### 文档
- ✅ 项目 README
- ✅ 架构文档
- ✅ 插件开发指南
- ✅ API 参考
- ✅ 贡献指南

---

## [0.0.1] - 2026-05-18

### 新增
- 项目初始化
- 基础 GPUI 应用框架
- 插件系统架构设计

---

## 版本说明

### 版本格式：MAJOR.MINOR.PATCH

- **MAJOR**: 不兼容的 API 变更
- **MINOR**: 向后兼容的功能新增
- **PATCH**: 向后兼容的问题修正

### 变更类型

- **新增** (Added): 新功能
- **变更** (Changed): 现有功能的变更
- **弃用** (Deprecated): 即将移除的功能
- **移除** (Removed): 已移除的功能
- **修复** (Fixed): 问题修复
- **安全** (Security): 安全相关的修复

---

## 链接

- [当前版本](../../releases/latest)
- [所有版本](../../releases)
- [问题跟踪](../../issues)
- [贡献指南](CONTRIBUTING.md)

---

*本变更日志遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/) 规范。*