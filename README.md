<div align="center">

# Forge

**给 Claude Code、Codex 等 AI 编程助手准备的终端启动器**

在一个入口里完成：选择工作区、切换项目目录、统一启动命令。

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.94%2B-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-444444)](./README.en.md)

[English](./README.en.md) · [安装与启动](#安装与启动) · [配置文件](#配置文件) · [常见问题](#常见问题)

</div>

---

## ✨ 这是什么

Forge 是一个面向 AI 编程助手的终端启动器。

它解决的不是“怎么安装 Claude Code 或 Codex”，而是“已经装好了之后，怎么在多个项目之间更快地切换并启动它们”：

- 不用反复 `cd` 到不同项目目录。
- 不用每次都手敲一长串启动命令。
- 可以记住默认项目根目录，日常打开更快。
- 可以把常用项目做成书签，直接从工作区选择器进入。
- 不只支持内置的 Claude Code / Codex，也支持你自己的自定义命令。

如果你平时在 `~/Projects`、`~/IdeaProjects` 或多个代码仓库之间频繁切换，Forge 会比手动切目录更顺手。

---

## 安装与启动

### 1. 预编译安装

macOS / Linux：

```bash
curl -fsSL https://raw.githubusercontent.com/LittleBunVerse/forge/main/scripts/install.sh | sh
```

Windows PowerShell：

```powershell
irm https://raw.githubusercontent.com/LittleBunVerse/forge/main/scripts/install.ps1 | iex
```

安装脚本会从 GitHub Release 下载与你平台匹配的二进制，并安装到：

- macOS / Linux：`~/.local/bin`
- Windows：`%LOCALAPPDATA%/Programs/forge/bin`

如果该目录还没在 `PATH` 中，脚本会直接告诉你下一条该执行的命令。

### 2. 源码安装

如果你本机已经有 Rust，也可以直接用一条命令安装：

```bash
cargo install --git https://github.com/LittleBunVerse/forge.git forge
```

本地仓库开发时则用：

```bash
cargo install --path .
```

### 3. 确保你的 AI CLI 已可用

Forge 负责“进入项目并启动命令”，不会替你安装 Claude Code、Codex 或其他工具。

至少确保你要启动的命令已经在 `PATH` 中，例如：

```bash
claude --help
codex --help
```

### 4. 启动

```bash
forge
```

第一次使用时，Forge 会引导你完成这 3 件事：

1. 选择默认根目录。
2. 选择项目目录。
3. 选择启动命令。

如果你已经知道这次要扫描哪个目录，也可以直接指定：

```bash
forge --root "~/Projects"

# 位置参数也可以直接作为 root 使用
forge "~/IdeaProjects"
```

### 5. 版本与安装入口

安装完成后，别人只需要记住两条命令：

```bash
forge --version
forge
```

如果你要把项目发给别人，推荐直接给下面两种安装命令之一：

- 预编译安装：`curl -fsSL https://raw.githubusercontent.com/LittleBunVerse/forge/main/scripts/install.sh | sh`
- Rust 源码安装：`cargo install --git https://github.com/LittleBunVerse/forge.git forge`

---

## 第一次启动会发生什么

### 首次运行且还没有配置时

Forge 会先让你选择默认根目录。常见候选包括：

- `~/Projects`
- `~/IdeaProjects`
- 当前目录 `.`
- 手动输入任意路径

选定后，Forge 会把它保存到本地配置文件中，然后继续：

1. 从该根目录的**一级子目录**中选择项目。
2. 选择要执行的启动命令，例如 `Claude Code`、`Codex` 或你自定义的命令。

### 配置完成后的日常运行

后续再执行 `forge`，你会先进入“工作区选择器”。通常会看到这些入口：

- 当前目录
- 你手动配置过的项目书签
- 从默认根目录继续浏览子项目
- 重新选择新的根目录

如果你选择的是“当前目录”或“项目书签”，Forge 会直接进入启动命令选择。

---

## 常见使用方式

### 直接使用默认配置

```bash
forge
```

适合日常从默认根目录或项目书签里选项目。

### 临时切换扫描根目录

```bash
forge --root "~/Projects"
forge "~/IdeaProjects"
```

适合这一次想临时浏览另一个目录，但不想改掉默认配置。

### 查看当前配置

```bash
forge config
```

### 查看配置文件路径

```bash
forge config path
```

### 修改默认 root

```bash
forge config set-root "~/Projects"
```

或者使用简写：

```bash
forge root set "~/Projects"
```

### 查看当前默认 root

```bash
forge root
```

### 查看版本

```bash
forge -v
```

---

## 配置文件

默认情况下，配置文件位于：

```text
~/.config/forge/config.json
```

你也可以随时运行：

```bash
forge config path
```

来查看当前机器上的实际路径。

### 配置项说明

#### `root`

默认扫描根目录。

- 执行 `forge` 时，如果没有传 `--root`，Forge 会优先使用这里的值。
- 只有这个目录的**一级子目录**会被当作候选项目。

#### `commands`

启动命令列表。

- 这里控制“Step 3/3 选择启动模式”里会出现哪些选项。
- 不限于 Claude Code 和 Codex。
- 只要命令在 `PATH` 中，就可以配置进去。

#### `projects`

项目书签列表。

- 这些项目会直接出现在“工作区选择器”里。
- 适合放一些不在统一 root 下、但你经常要开的项目。

### 一个更实用的配置示例

> Forge 目前提供了 `root` 的 CLI 配置命令；`commands` 和 `projects` 需要你直接编辑配置文件。

```json
{
  "root": "/Users/you/Projects",
  "commands": [
    {
      "name": "Claude Code",
      "command": "claude",
      "args": []
    },
    {
      "name": "Codex",
      "command": "codex",
      "args": []
    },
    {
      "name": "Aider",
      "command": "aider",
      "args": ["--yes-always"]
    }
  ],
  "projects": [
    {
      "name": "forge",
      "path": "/Users/you/src/forge"
    },
    {
      "name": "todo-project",
      "path": "/Users/you/work/todo-project"
    }
  ]
}
```

### 推荐的配置思路

- `root` 只放你最常浏览的一组项目，比如 `~/Projects`。
- `projects` 放零散但高频的项目书签。
- `commands` 只保留你真的会用到的 2 到 4 个入口，避免选择列表过长。

---

## 常用命令速查

```bash
forge
forge --root "~/Projects"
forge "~/IdeaProjects"

forge config
forge config path
forge config set-root "~/Projects"

forge root
forge root set "~/Projects"

forge -v
forge --version
```

---

## 环境变量

### `FORGE_ROOT`

覆盖本次运行使用的根目录。

```bash
FORGE_ROOT="$HOME/Projects" forge
```

适合临时切换目录，又不想改写配置文件。

### `FORGE_CONFIG_DIR`

自定义配置根目录。

Forge 会在这个目录下继续使用 `forge/config.json`。

### `XDG_CONFIG_HOME`

遵循 XDG 配置目录规范。

如果没有设置 `FORGE_CONFIG_DIR`，Forge 会优先使用这个变量。

### 兼容旧变量

Forge 对旧的 AIDEV 变量保留了兼容能力：

- `AIDEV_ROOT`
- `AIDEV_CONFIG_DIR`

如果你是从旧工具迁移过来，通常不需要额外处理。

---

## 扫描规则

为了保证终端交互足够快，Forge 的扫描策略是刻意收敛过的：

- 只扫描 root 的**一级子目录**
- 自动忽略所有以 `.` 开头的目录
- 额外忽略这些常见噪声目录：`node_modules`、`target`、`dist`、`vendor`
- 对“符号链接到目录”的情况也会尽量兼容

这意味着：

- 如果你的项目嵌套得更深，Forge 默认不会继续往下找。
- 这时更合适的做法是把 `root` 设到更上层。
- 或者直接把项目写进 `projects` 书签。

---

## 常见问题

### 为什么运行 `forge` 后没看到我的项目？

通常是下面几种原因：

- 你的项目不在当前 `root` 的一级子目录里。
- 目录名命中了忽略规则，例如以 `.` 开头，或叫 `dist`、`vendor` 等。
- 你这次是在另一个 root 下启动的。

优先检查：

```bash
forge config
```

如果项目结构比较特殊，建议直接把它加入 `projects`。

### 为什么提示“未找到命令”？

Forge 只负责启动，不负责安装命令本身。

如果你在 `commands` 里配置了：

```json
{ "name": "Codex", "command": "codex", "args": [] }
```

那 `codex` 本身必须已经能在终端里直接执行。

先单独验证：

```bash
codex --help
```

### 非交互式终端里怎么用？

如果当前环境不是交互式终端，Forge 不能弹出选择界面。

这时你应该提前指定 root，例如：

```bash
forge --root "~/Projects"
forge config set-root "~/Projects"
FORGE_ROOT="$HOME/Projects" forge
```

### Forge 只能启动 Claude Code 和 Codex 吗？

不是。

只要某个命令能在你的终端里直接运行，就可以把它写进 `commands`，例如 `aider`、`opencode` 或你自己的包装脚本。

---

## 开发

运行测试：

```bash
cargo test
```

格式检查：

```bash
cargo fmt --check
```

本地构建：

```bash
cargo build
```

查看帮助：

```bash
cargo run -- --help
```

---

## 许可证

本项目使用 [Apache License 2.0](./LICENSE)。
