# Changelog

本文件记录 Forge 的所有重要变更，格式遵循 [Keep a Changelog](https://keepachangelog.com/)。

All notable changes to Forge are documented here. Format follows [Keep a Changelog](https://keepachangelog.com/).

## [1.0.0] - 2025-07-10

### Added

- 交互式终端 UI（基于 ratatui），支持键盘导航选择工作区和项目
- 默认根目录配置，首次运行引导式设置
- 项目书签系统，支持快速跳转到常用项目
- 可配置的启动命令列表（内置 Claude Code、Codex，支持自定义）
- 目录扫描：一级子目录自动发现，智能忽略噪声目录
- 跨平台支持：macOS (Intel + ARM)、Linux、Windows
- 一键安装脚本：`install.sh`（Unix）和 `install.ps1`（Windows）
- 环境变量覆盖：`FORGE_ROOT`、`FORGE_CONFIG_DIR`、`XDG_CONFIG_HOME`
- 旧版 AIDEV 变量兼容
- CLI 子命令：`config`、`config path`、`config set-root`、`root`、`root set`
- CI/CD：GitHub Actions 多平台测试 + 自动构建发布
- 源码安装支持：`cargo install`

[1.0.0]: https://github.com/LittleBunVerse/forge/releases/tag/v1.0.0
