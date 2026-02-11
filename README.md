# Forge

[English](#english) | [简体中文](#zh-cn)

---

<a name="zh-cn"></a>
## 简体中文

Forge 是一个面向 AI 编程助手（如 Claude Code, Codex 等）的终端启动器。它允许你在指定的“根目录”下快速切换项目，并自动执行预设的启动命令。

### 🌟 解决了哪些问题

- **项目管理繁琐**：无需频繁 `cd` 或手动查找路径。
- **启动指令多样**：集成不同 AI 工具的命令与参数，告别记忆压力。
- **环境切换缓慢**：记住默认 Root 目录，同时也支持临时指定。
- **扫描噪声大**：自动忽略 `.git`、`node_modules` 等大目录，交互流畅。

### 🚀 安装

#### 使用 Homebrew (推荐 macOS)
```bash
brew install LittleBunVerse/tap/forge
```

#### 从源码构建
```bash
go install github.com/LittleBunVerse/forge/cmd/forge@latest
```

### ⌨️ 快速开始

直接运行：
```bash
forge
```
*首次运行会引导你设置默认扫描根目录（Root）。*

临时指定扫描目录：
```bash
forge --root "~/Projects"
# 或简写
forge "~/Projects"
```

### 🛠 常用命令

- `forge config`：查看当前配置。
- `forge config set-root <path>`：设置默认扫描目录。
- `forge root`：`config set-root` 的简写。

### ⚙️ 配置文件

配置文件默认位于 `~/.config/forge/config.json`。

```json
{
  "root": "/Users/you/Projects",
  "commands": [
    { "name": "Claude Code", "command": "claude", "args": [] },
    { "name": "Codex", "command": "codex", "args": [] }
  ],
  "projects": [
    { "name": "my-app", "path": "~/Projects/my-app" }
  ]
}
```

### 🌍 环境变量

- `FORGE_ROOT`：覆盖默认扫描根目录。
- `FORGE_CONFIG_DIR`：指定自定义配置目录。
- `XDG_CONFIG_HOME`：遵循 XDG 规范的配置路径。

### 🔍 扫描规则

- **深度**：仅扫描根目录的一级子目录。
- **排除**：自动忽略以 `.` 开头的目录（如 `.git`）以及 `node_modules`、`target`、`dist`、`vendor` 等常见干扰目录。

### 🛠 开发

```bash
# 运行测试
go test ./...

# 本地构建
go build -o forge ./cmd/forge
```

---

<a name="english"></a>

## English

Forge is a terminal-based launcher designed for AI programming assistants (e.g., Claude Code, Codex). It enables rapid project switching within a "Root" directory and automates the execution of startup commands.

### 🌟 Key Features

- **Effortless Navigation**: No more constant `cd` or path hunting.
- **Unified Startup**: Consolidate commands and arguments for various AI tools.
- **Persistent Context**: Remember your default Root directory while allowing overrides.
- **Smart Scanning**: Automatically ignores `.git`, `node_modules`, and other noise for a snappy UI.

### 🚀 Installation

#### via Homebrew (Recommended for macOS)
```bash
brew install LittleBunVerse/tap/forge
```

#### From Source
```bash
go install github.com/LittleBunVerse/forge/cmd/forge@latest
```

### ⌨️ Quick Start

Run it directly:
```bash
forge
```
*The first run will guide you through setting up your default scanning directory (Root).*

Specify a temporary Root:
```bash
forge --root "~/Projects"
# or simply
forge "~/Projects"
```

### 🛠 Common Commands

- `forge config`: Display current configuration.
- `forge config set-root <path>`: Update the default scanning directory.
- `forge root`: Shorthand for updating the root.

### ⚙️ Configuration

The configuration is stored in `~/.config/forge/config.json`.

```json
{
  "root": "/Users/you/Projects",
  "commands": [
    { "name": "Claude Code", "command": "claude", "args": [] },
    { "name": "Codex", "command": "codex", "args": [] }
  ],
  "projects": [
    { "name": "my-app", "path": "~/Projects/my-app" }
  ]
}
```

### 🌍 Environment Variables

- `FORGE_ROOT`: Override the default root directory.
- `FORGE_CONFIG_DIR`: Specify a custom configuration directory.
- `XDG_CONFIG_HOME`: Follows XDG base directory specification.

### 🔍 Scanning Rules

- **Depth**: Scans only the first level of the root directory.
- **Exclusions**: Automatically ignores directories starting with `.` (e.g., `.git`) and common noise such as `node_modules`, `target`, `dist`, and `vendor`.

### 🛠 Development

```bash
# Run tests
go test ./...

# Local build
go build -o forge ./cmd/forge
```
