# 安全策略 / Security Policy

> 中文在前，English below.

---

## 中文版

### 支持版本

我们仅对最新发布的稳定版提供安全更新。

| 版本 | 安全更新 |
|------|---------|
| 1.x（最新） | ✅ |
| 主分支 `main` | ✅（开发中）|

### 报告漏洞

**请不要在公开 Issue 中报告安全漏洞**，以免在补丁发布前被恶意利用。

请通过以下渠道私下报告：

- 📧 邮件：`wanglei30@wps.cn`
- 📌 邮件标题前缀：`[Forge Security]`

#### 报告内容建议

- **漏洞类型**（如命令注入、路径遍历、配置文件注入等）
- **影响版本** 和 **复现环境**（操作系统、Rust 版本）
- **复现步骤**（越详细越好，包含 PoC 或截图）
- **潜在影响范围**（任意命令执行 / 信息泄露 / 服务不可用）
- **建议的修复方案**（如有）
- **报告者信息**（用于致谢，匿名也可）

### 响应 SLA

| 阶段 | 时限 |
|------|------|
| 首次回复确认收到 | 3 个工作日内 |
| 漏洞评估与等级判定 | 7 个工作日内 |
| 修复方案与发布计划 | 30 个工作日内（视严重程度而定）|
| Critical 漏洞的修复发布 | 14 个工作日内 |

### 披露流程

我们采用**协调披露**（Coordinated Disclosure）原则：

1. 你私下报告漏洞 → 我们确认并评估
2. 我们准备修复补丁 → 通知你验证
3. 发布修复版本 → 在 [GitHub Security Advisory](https://github.com/LittleBunVerse/forge/security/advisories) 与 CHANGELOG 公开漏洞详情
4. 在公告中致谢报告者（除非你希望匿名）

通常**至少在补丁发布 7 天后**才会公开技术细节，以便用户升级。

### 致谢

感谢所有负责任披露漏洞的研究者。维护者列表与历史致谢见 [GitHub Security Advisories](https://github.com/LittleBunVerse/forge/security/advisories)。

### 安全注意事项

使用 Forge 时请注意以下安全要点：

| 项 | 说明 |
|----|------|
| 命令配置 | `commands` 中的命令会被直接执行，请确保配置的命令路径可信 |
| 配置文件权限 | `~/.config/forge/config.json` 包含项目路径信息，建议保持默认文件权限 |
| 安装脚本 | 安装脚本从 GitHub Release 下载二进制，请确认网络环境安全 |

---

## English Version

### Supported Versions

| Version | Security Updates |
|---------|------------------|
| 1.x (latest) | ✅ |
| `main` branch | ✅ (development) |

### Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Report privately via:

- 📧 Email: `wanglei30@wps.cn`
- 📌 Subject prefix: `[Forge Security]`

#### What to include

- Vulnerability type (e.g. command injection, path traversal, config injection)
- Affected version and reproduction environment (OS, Rust version)
- Step-by-step reproduction with PoC if possible
- Potential impact (arbitrary command execution / information disclosure / DoS)
- Suggested fix (if available)
- Reporter info for credit (anonymous OK)

### Response SLA

| Stage | Timeline |
|-------|----------|
| First acknowledgment | within 3 business days |
| Severity assessment | within 7 business days |
| Fix plan | within 30 business days (severity-dependent) |
| Critical fix release | within 14 business days |

### Coordinated Disclosure

We follow coordinated disclosure: technical details are usually published **at least 7 days after** a patch is available, via [GitHub Security Advisories](https://github.com/LittleBunVerse/forge/security/advisories) and CHANGELOG.

### Security Considerations

| Item | Note |
|------|------|
| Command configuration | Commands in `commands` are executed directly; ensure configured command paths are trusted |
| Config file permissions | `~/.config/forge/config.json` contains project paths; keep default file permissions |
| Install scripts | Install scripts download binaries from GitHub Releases; ensure a secure network environment |
