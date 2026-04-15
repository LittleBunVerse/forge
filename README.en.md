<div align="center">

# Forge

**A terminal launcher for AI coding assistants such as Claude Code and Codex**

Pick a workspace, jump into a project, and launch your preferred AI CLI from one entry point.

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.94%2B-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)

[中文 README](./README.md) · [Install & Run](#install--run)

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

## Install & Run

Prebuilt install on macOS / Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/LittleBunVerse/forge/main/scripts/install.sh | sh
```

Prebuilt install on Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/LittleBunVerse/forge/main/scripts/install.ps1 | iex
```

Install from source with Rust:

```bash
cargo install --git https://github.com/LittleBunVerse/forge.git forge
```

Install from the current local checkout:

```bash
cargo install --path .
```

After installation, start Forge with:

```bash
forge
```

Forge does not install Claude Code, Codex, or other AI CLIs for you.  
Make sure the command you want to launch is already available in `PATH`.

On first run, Forge guides you through:

1. choosing a default root directory
2. selecting a project folder
3. choosing a launch command

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

## License

Licensed under [Apache License 2.0](./LICENSE).
