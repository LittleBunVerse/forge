# Forge 架构概览 / Architecture Overview

> 本文档面向希望参与 Forge 开发的贡献者，帮助快速理解项目结构和设计思路。

---

## 项目定位

Forge 是一个纯 Rust 终端启动器（TUI），核心目标：**让开发者在多项目之间快速切换并启动 AI 编程助手**。

设计原则：
- **零依赖外部服务**：纯本地运行，无网络请求、无数据库
- **启动即用**：首次运行引导式配置，后续秒开
- **最小化认知负担**：默认行为合理，配置项精简

---

## 模块结构

```
src/
├── main.rs        # 入口点：解析参数 → 调用 app::run()
├── lib.rs         # 模块声明（公开 API 边界）
│
├── app/
│   └── mod.rs     # 应用核心：CLI 解析、流程编排、Services trait
│
├── config.rs      # 配置管理：加载/保存/默认值/迁移
├── ui/
│   └── mod.rs     # TUI 层：ratatui 交互界面
├── runner.rs      # 命令执行：子进程启动与环境管理
├── scan.rs        # 目录扫描：一级子目录发现与过滤
└── pathutil.rs    # 路径工具：~ 展开、路径标准化
```

---

## 模块职责与依赖关系

```
main.rs
  │
  ▼
app/mod.rs  ──────────────────────────────────────┐
  │  CLI 解析 + 流程编排                           │
  │  定义 Services trait（依赖注入接口）            │
  │                                                │
  ├──▶ config.rs       配置加载/保存               │
  │      └── pathutil.rs  路径标准化               │
  │                                                │
  ├──▶ scan.rs         目录扫描                    │
  │                                                │
  ├──▶ ui/mod.rs       TUI 交互                   │
  │      ├── 工作区选择器                          │
  │      ├── 根目录选择器                          │
  │      ├── 项目选择器                            │
  │      └── 命令选择器                            │
  │                                                │
  └──▶ runner.rs       命令执行                    │
         └── 子进程管理                            │
                                                   │
  RealServices ──────── 实现 Services trait ◀──────┘
```

### 各模块说明

#### `app/mod.rs` — 应用核心

- **职责**：CLI 参数解析、子命令分发、主流程编排
- **核心设计**：`Services` trait 作为依赖注入接口，将 IO 操作（配置读写、TUI 交互、命令执行）抽象为 trait 方法
- `RealServices` 是生产环境的实现，测试中可替换为 mock
- 主流程：解析参数 → 加载配置 → 选择工作区 → 选择项目 → 选择命令 → 执行

#### `config.rs` — 配置管理

- **职责**：`~/.config/forge/config.json` 的读写与校验
- **核心结构**：
  - `Config`：根配置（root + commands + projects）
  - `CommandConfig`：单个启动命令（名称 + 可执行文件 + 参数）
  - `ProjectConfig`：项目书签（名称 + 路径）
- 支持旧版 AIDEV 配置目录迁移
- 遵循 XDG 配置目录规范

#### `ui/mod.rs` — TUI 交互层

- **职责**：基于 ratatui + crossterm 的终端交互界面
- **核心组件**：
  - `print_banner()`：启动 ASCII 艺术 Banner
  - `select_workspace()`：工作区选择器（当前目录 / 书签 / 浏览根目录 / 新根目录）
  - `select_root()`：根目录选择器（候选列表 + 手动输入）
  - `select_dir()`：项目目录选择器
  - `select_command()`：命令选择器
- 所有选择器支持键盘导航（上下选择、回车确认、Esc/q 退出）

#### `runner.rs` — 命令执行

- **职责**：在指定工作目录中启动子进程
- 定义 `Deps` 结构体封装系统调用（look_path / chdir / exec）
- 跨平台兼容：Unix 使用 exec 替换进程，Windows 使用 spawn + wait

#### `scan.rs` — 目录扫描

- **职责**：扫描根目录的一级子目录
- 自动过滤：以 `.` 开头的目录、`node_modules`/`target`/`dist`/`vendor`
- 支持自定义忽略列表

#### `pathutil.rs` — 路径工具

- **职责**：`~` 展开为 home 目录、路径标准化（去除 `.`/`..`）
- 跨平台兼容

---

## 数据流

```
用户启动 forge
       │
       ▼
  解析 CLI 参数
       │
       ├── forge -v            → 输出版本号
       ├── forge config [...]  → 配置子命令
       └── forge [--root ...]  → 主流程
              │
              ▼
         加载配置文件
              │
              ▼
         工作区选择器
              │
              ├── 当前目录      → 直接进入命令选择
              ├── 项目书签      → 直接进入命令选择
              ├── 浏览根目录    → 扫描一级子目录 → 项目选择器
              └── 新的根目录    → 根目录选择器 → 扫描 → 项目选择器
                                        │
                                        ▼
                                   命令选择器
                                        │
                                        ▼
                                   执行命令
                                  （cd + exec）
```

---

## 设计决策

### 为什么用 Services trait 做依赖注入？

Forge 是一个 TUI 工具，核心逻辑（流程编排）与 IO（终端交互、文件系统、进程启动）紧密耦合。`Services` trait 将两者解耦：

- 生产代码使用 `RealServices`，直接调用系统 API
- 测试代码可以 mock 所有 IO 操作，实现纯逻辑测试
- 无需引入额外的 mock 框架

### 为什么只扫描一级子目录？

- **性能**：深度递归扫描在大型 home 目录下耗时过长
- **简单性**：大多数开发者的项目组织形式是 `~/Projects/project-a/`
- **可预测**：用户能准确预期哪些项目会出现
- 对于不在统一根目录下的项目，通过 `projects` 书签解决

### 为什么不用 clap 做 CLI 解析？

Forge 的 CLI 接口极其简单（几个子命令 + 一个可选参数），手写解析足够且零额外依赖。

---

## 扩展点

如果你想为 Forge 添加新功能，以下是推荐的扩展方式：

| 需求 | 建议的实现路径 |
|------|---------------|
| 新增启动命令 | 修改 `config.rs` 中的 `default_commands()` |
| 新增工作区选项 | 修改 `ui/mod.rs` 中的 `select_workspace()` |
| 支持新的配置字段 | 在 `Config` 结构体中添加字段（注意 serde 兼容性）|
| 新增 CLI 子命令 | 在 `app/mod.rs` 的参数解析分支中添加 |
| 支持模糊搜索 | 在 `ui/mod.rs` 的选择器中增加输入过滤逻辑 |
| 多级目录扫描 | 修改 `scan.rs`，但需注意性能影响 |

---

## 技术栈

| 组件 | 选型 | 理由 |
|------|------|------|
| 语言 | Rust 2024 Edition | 高性能、跨平台、零运行时 |
| TUI 框架 | ratatui 0.29 | Rust 生态最活跃的 TUI 库 |
| 终端控制 | crossterm 0.29 | 真正跨平台的终端操作 |
| 错误处理 | anyhow 1.0 | 应用级错误处理，简洁灵活 |
| 序列化 | serde + serde_json | JSON 配置文件读写 |
| 路径查找 | which 7.0 | 跨平台的可执行文件路径查找 |
