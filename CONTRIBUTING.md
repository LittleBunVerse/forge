# 贡献指南 / Contributing Guide

> 中文在前，English below.

感谢你愿意为 **Forge** 贡献力量！本指南帮助你快速上手开发流程。

---

## 中文版

### 行为准则

参与本项目即表示你同意遵守 [行为准则](./CODE_OF_CONDUCT.md)。请保持友善、专业、包容。

### 我可以贡献什么？

- **Bug 修复**：从 [Issues](https://github.com/LittleBunVerse/forge/issues) 中挑选一个 `good first issue` / `help wanted` 标签的任务
- **新功能**：在 [Discussions](https://github.com/LittleBunVerse/forge/discussions) 先讨论再开 Issue
- **文档改进**：发现错别字、过时内容、缺失的示例都欢迎直接 PR
- **翻译**：补充英文文档或其他语种
- **测试**：补充单元测试、提升测试覆盖率

### 开发环境要求

| 依赖 | 版本 | 用途 |
|------|------|------|
| Rust | 1.85+ | 编译器与包管理 |
| cargo | 随 Rust 安装 | 构建、测试、格式化 |
| Git | 2.x+ | 版本控制 |

### 快速开始

```bash
# 1. Fork 并 clone 仓库
git clone https://github.com/<your-username>/forge.git
cd forge

# 2. 确认工具链
rustup show

# 3. 构建项目
cargo build

# 4. 运行测试
cargo test

# 5. 运行格式检查
cargo fmt --check

# 6. 运行 lint 检查
cargo clippy --all-targets --all-features -- -D warnings

# 7. 本地安装并试用
cargo install --path .
forge
```

### 代码规范

- **提交前必须本地通过 `cargo fmt --check` 和 `cargo clippy`**
- 文件 < 800 行；函数 < 50 行
- 命名要自解释，禁止无意义缩写
- 关键逻辑加中文注释（"为什么"而非"做什么"）
- 不引入 `TODO` / `FIXME` 而无对应 Issue
- 错误必须显式处理，禁止静默吞掉（使用 `anyhow::Result`）

### 提交规范（Conventional Commits）

格式：`<type>(<scope>): <中文或英文描述>`

```
feat(ui): 新增书签搜索过滤功能
fix(config): 修复 Windows 下配置路径解析错误
docs(readme): 补充环境变量使用说明
refactor(scan): 重构目录扫描逻辑为独立模块
test(runner): 增加命令执行器边界条件测试
chore(ci): 升级 GitHub Actions 到最新版本
```

支持的 type：`feat` / `fix` / `docs` / `refactor` / `test` / `chore` / `perf` / `ci` / `style` / `build`

### Pull Request 流程

1. **从 main 分支创建特性分支**：`feat/your-feature` 或 `fix/issue-123`
2. **保持小而专注**：一个 PR 解决一件事，避免混杂提交
3. **更新文档**：影响外部行为的改动必须同步文档
4. **补充测试**：新增功能必须有对应测试
5. **运行本地检查**：`make lint && make test`
6. **填写 PR 模板**：说明动机、改动点、测试方式
7. **保持分支同步**：定期 rebase 到最新 main
8. **响应 Review**：48 小时内回复，无法解决的问题留言说明

### Issue 报告

- **Bug**：使用 [Bug Report 模板](./.github/ISSUE_TEMPLATE/bug_report.yml)，附环境信息和复现步骤
- **新功能**：先在 Discussions 讨论，达成共识后再开 Issue
- **安全漏洞**：**不要公开 Issue**，按 [SECURITY.md](./SECURITY.md) 私下报告

### 沟通渠道

- [Issues](https://github.com/LittleBunVerse/forge/issues)：Bug 与功能跟踪
- [Discussions](https://github.com/LittleBunVerse/forge/discussions)：方案讨论与问答
- 安全问题：见 [SECURITY.md](./SECURITY.md)

---

## English Version

Thanks for contributing to **Forge**!

### Code of Conduct

By participating, you agree to abide by the [Code of Conduct](./CODE_OF_CONDUCT.md).

### Quick Start

```bash
git clone https://github.com/<your-username>/forge.git
cd forge
cargo build
cargo test
cargo install --path .
forge
```

### Coding Standards

- Files < 800 lines, functions < 50 lines
- Self-documenting names, no meaningless abbreviations
- Explicit error handling with `anyhow::Result`
- Must pass `cargo fmt --check` and `cargo clippy` before committing

### Commit Convention

```
<type>(<scope>): <description>
```

Types: `feat` / `fix` / `docs` / `refactor` / `test` / `chore` / `perf` / `ci`

### Pull Request Checklist

- [ ] Branch from `main` with descriptive name
- [ ] Tests added or updated
- [ ] Documentation updated if behavior changed
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy` passes with no warnings
- [ ] PR template filled out
- [ ] Linked to a relevant Issue (if applicable)

### Reporting Issues

- **Bugs**: use the [Bug Report template](./.github/ISSUE_TEMPLATE/bug_report.yml)
- **Features**: discuss in [Discussions](https://github.com/LittleBunVerse/forge/discussions) first
- **Security**: see [SECURITY.md](./SECURITY.md), do **not** open public issues
