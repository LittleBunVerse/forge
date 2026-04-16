<div align="center">

# Forge

**A terminal launcher for AI coding assistants such as Claude Code and Codex**

Pick a workspace, jump into a project, and launch your preferred AI CLI from one entry point.

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE)
[![CI](https://github.com/LittleBunVerse/forge/actions/workflows/ci.yml/badge.svg)](https://github.com/LittleBunVerse/forge/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-444444)](#install--run)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](./CONTRIBUTING.md)

[中文 README](./README.md) · [Install & Run](#install--run) · [Contributing](./CONTRIBUTING.md)

</div>

---

## What Forge Does

Forge is built for developers who switch between many repositories and do not want to repeatedly:

- `cd` into different project folders
- remember different startup commands
- maintain separate entry flows for Claude Code, Codex, or custom tools

Forge gives you one terminal entry for:

- choosing a workspace
- browsing projects under a saved root directory
- launching a configured AI command
- jumping directly to bookmarked projects

---

## Prerequisites

Forge is a **launcher** — it helps you pick a project and start an AI coding assistant. You need at least one AI CLI installed before using Forge:

| AI Assistant | Install | Notes |
|-------------|---------|-------|
| [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview) | `npm install -g @anthropic-ai/claude-code` | Anthropic official CLI |
| [Codex](https://github.com/openai/codex) | `npm install -g @openai/codex` | OpenAI official CLI |
| Other tools | See each tool's docs | Any terminal command can be configured in Forge |

> You can install Forge first and configure commands later.

---

## Install & Run

### Option 1: One-line Install (Recommended)

macOS / Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/LittleBunVerse/forge/main/scripts/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/LittleBunVerse/forge/main/scripts/install.ps1 | iex
```

> **Windows users**: If you see "running scripts is disabled", run this first as Administrator:
> ```powershell
> Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
> ```
> Then retry the install command above.

The script auto-detects your platform, downloads a prebuilt binary from GitHub Releases, and installs to:

- macOS / Linux: `~/.local/bin`
- Windows: `%LOCALAPPDATA%\Programs\forge\bin`

After installation, the script shows how to add the path to your `PATH` (including permanent setup). Just follow the instructions.

### Option 2: Install from Source

Install the [Rust toolchain](https://rustup.rs/) first (if you don't have it):

```bash
# Install Rust (includes cargo)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart your terminal, then install Forge
cargo install --git https://github.com/LittleBunVerse/forge.git forge
```

Install from a local checkout:

```bash
cargo install --path .
```

### Verify Installation

```bash
forge --version
```

### Launch

```bash
forge
```

On first run, Forge guides you through:

1. **Choose a default root directory** — where do you keep your projects? e.g. `~/Projects`
2. **Select a project folder** — pick one from the root directory
3. **Choose a launch command** — Claude Code, Codex, or something else?

You can also point Forge to a root directory directly:

```bash
forge --root "~/Projects"
forge "~/IdeaProjects"
```

---

## Config Overview

Default config path:

```text
~/.config/forge/config.json
```

Key fields:

- `root`: default directory used for project scanning
- `commands`: launch commands shown in the command picker
- `projects`: bookmarked projects shown directly in the workspace picker

Example:

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
    }
  ],
  "projects": [
    {
      "name": "forge",
      "path": "/Users/you/src/forge"
    }
  ]
}
```

Useful commands:

```bash
forge config
forge config path
forge config set-root "~/Projects"
forge root
forge root set "~/Projects"
forge -v
```

---

## Scanning Rules

Forge keeps scanning intentionally simple and fast:

- only scans the first level under the root directory
- always ignores dot-prefixed folders
- also ignores `node_modules`, `target`, `dist`, and `vendor`

If a project is nested deeper, either move the root up one level or add the project to `projects`.

---

## Development

Make sure `cargo` and `rustc` are available if you build from source.

Run tests:

```bash
cargo test
```

Build locally:

```bash
cargo build
```

---

## Contributing

Contributions of all kinds are welcome! Whether it's bug fixes, feature suggestions, or documentation improvements.

- Read the [Contributing Guide](./CONTRIBUTING.md) for the development workflow
- Check the [Code of Conduct](./CODE_OF_CONDUCT.md) for community standards
- Discuss ideas in [Discussions](https://github.com/LittleBunVerse/forge/discussions)

### Contributors

<a href="https://github.com/tajiaoyezi" title="tajiaoyezi">
  <img src="https://github.com/tajiaoyezi.png" width="60" height="60" alt="tajiaoyezi" style="border-radius: 50%;" />
</a>
<a href="https://github.com/LittleBunVerse" title="LittleBunVerse">
  <img src="https://github.com/LittleBunVerse.png" width="60" height="60" alt="LittleBunVerse" style="border-radius: 50%;" />
</a>

> Full contributor list: [GitHub Contributors](https://github.com/LittleBunVerse/forge/graphs/contributors).

---

## Star History

<a href="https://star-history.com/#LittleBunVerse/forge&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=LittleBunVerse/forge&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=LittleBunVerse/forge&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=LittleBunVerse/forge&type=Date" />
 </picture>
</a>

---

## License

Licensed under [Apache License 2.0](./LICENSE).
