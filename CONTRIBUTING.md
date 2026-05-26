# 贡献指南

感谢你有兴趣为 Oasis 做出贡献！本指南将帮助你了解如何参与项目开发。

---

## 🤝 如何贡献

### 报告问题

如果你发现了 bug 或有功能建议：

1. 检查 [Issues](../../issues) 是否已存在相同问题
2. 如果没有，创建新的 Issue，包含：
   - 清晰的标题
   - 详细的问题描述
   - 复现步骤（针对 bug）
   - 预期行为 vs 实际行为
   - 环境信息（操作系统、Rust 版本等）
   - 相关日志或截图

### 提交代码

1. **Fork 仓库**
   ```bash
   # 在 GitHub 上点击 Fork 按钮
   git clone https://github.com/your-username/oasis.git
   cd oasis
   ```

2. **创建功能分支**
   ```bash
   git checkout -b feature/your-feature-name
   # 或修复分支
   git checkout -b fix/your-bug-fix
   ```

3. **进行开发**
   ```bash
   # 添加你的更改
   git add .
   git commit -m "feat: add amazing feature"
   ```

4. **推送到你的 Fork**
   ```bash
   git push origin feature/your-feature-name
   ```

5. **创建 Pull Request**
   - 在 GitHub 上创建 PR
   - 填写 PR 模板
   - 等待代码审查

---

## 📋 代码规范

### Rust 代码规范

- 遵循 [Rust API 指导原则](https://rust-lang.github.io/api-guidelines/)
- 使用 `cargo fmt` 格式化代码
   ```bash
   cargo fmt
   ```
- 使用 `cargo clippy` 检查代码质量
   ```bash
   cargo clippy -- -D warnings
   ```

### 命名规范

- **函数/变量**: `snake_case`
- **类型/结构体**: `PascalCase`
- **常量**: `SCREAMING_SNAKE_CASE`
- **文件名**: `snake_case` 或 `kebab-case`

### 注释规范

- 公开 API 必须有文档注释
- 复杂逻辑需要行内注释说明
- 注释应解释"为什么"而非"做什么"

```rust
/// 计算两个数字的和
///
/// # Examples
/// ```
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

---

## 📝 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

### 格式

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### 类型 (type)

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响代码运行的变动）
- `refactor`: 重构（既不是新增功能，也不是修复 bug）
- `perf`: 性能优化
- `test`: 增加测试
- `chore`: 构建过程或辅助工具的变动
- `ci`: CI 配置文件和脚本的变动

### 示例

```bash
feat(plugin): add WASM plugin support
fix(window): resolve window resizing issue
docs(readme): update installation instructions
refactor(dock): simplify dock rendering logic
perf(wasm): optimize plugin loading time
test(plugin): add unit tests for plugin registry
chore(deps): upgrade gpui to version 0.5.0
```

---

## 🔍 Pull Request 模板

创建 PR 时请包含以下信息：

### 标题
使用与提交消息相同的格式：`feat(scope): description`

### 描述
- **动机**: 为什么需要这个更改？
- **变更**: 具体做了哪些修改？
- **测试**: 如何测试这些更改？
- **截图**: UI 相关变更需要提供截图

### 检查清单
- [ ] 代码遵循项目规范
- [ ] 已通过 `cargo fmt` 格式化
- [ ] 已通过 `cargo clippy` 检查
- [ ] 已添加必要的测试
- [ ] 文档已更新（如需要）
- [ ] 所有测试通过 `cargo test`
- [ ] 在 macOS/Linux 上测试通过

---

## 🚀 开发工作流

### 本地开发

1. **克隆仓库**
   ```bash
   git clone https://github.com/your-username/oasis.git
   cd oasis
   ```

2. **安装依赖**
   ```bash
   # Rust 工具链
   rustup update stable
   
   # 前置工具
   cargo install wasm-bindgen-cli
   ```

3. **运行开发版本**
   ```bash
   cargo run
   ```

4. **运行测试**
   ```bash
   cargo test
   cargo clippy
   ```

### 插件开发

如果你在开发插件：

1. 选择合适的插件模式（WASM/cdylib/rlib）
2. 参考 [插件开发指南](docs/development/plugin-guide.md)
3. 在 `plugins/` 目录中测试你的插件
4. 提交时包含插件使用说明

---

## 📚 文档贡献

文档同样重要！你可以：

1. 修正错误和不准确的地方
2. 改善现有文档的清晰度
3. 添加缺失的文档
4. 翻译文档

### 文档位置

- 用户文档: `docs/user/`
- 开发者文档: `docs/development/`
- 架构文档: `docs/development/architecture.md`
- API 参考: `docs/development/api-reference.md`

---

## 🎯 优先事项

当前我们特别需要贡献的领域：

- [ ] 用户界面改进
- [ ] 插件生态系统扩展
- [ ] 性能优化
- [ ] 文档完善
- [ ] 测试覆盖率提升

---

## 💬 交流渠道

- **GitHub Issues**: 问题报告和功能讨论
- **Pull Requests**: 代码审查和技术讨论
- **Discussions**: 一般性讨论和问题

---

## ⚖️ 行为准则

- 尊重所有贡献者
- 接受建设性批评
- 关注对社区最有利的事情
- 对不同观点保持开放

---

## 📄 许可证

贡献的代码将采用与项目相同的 [MIT License](LICENSE)。

---

*再次感谢你的贡献！🎉*