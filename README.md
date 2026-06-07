# Chuchen-Terminal

> Windows-first local terminal workbench for multi-workspace development, split panes, and long-running AI CLI tasks.

[简体中文](README.zh-CN.md)

Chuchen-Terminal is a local desktop terminal workbench built with Tauri 2, Vue 3, TypeScript, Rust, and xterm.js.

It was created for a very specific but increasingly common workflow: you have several repositories open, multiple frontend/backend services running, one or more AI CLI sessions working in the background, and you still need a clean way to remember which terminal belongs to which project, which command was run where, and which AI session needs your attention.

Most terminal apps are powerful, but they usually treat every terminal as a tab or a pane. Chuchen-Terminal treats terminals as part of a workspace structure:

```text
Workspace -> Project Tab -> Pane -> Terminal Session
```

That structure is the core idea of this project.

## Why This Exists

AI coding tools changed how many developers use terminals.

It is common to leave a CLI agent running for minutes, switch to another project, come back later, and then wonder:

- Which terminal was running the frontend?
- Which one was the backend?
- Which AI CLI session finished?
- Which session is waiting for approval or input?
- Which command did I run in this project yesterday?
- How do I restore the same split layout after closing the app?

Chuchen-Terminal tries to make those workflows explicit instead of relying on memory, terminal titles, or a pile of unnamed tabs.

## Current Status

Chuchen-Terminal is currently an MVP / preview project.

It is already useful for local experimentation and day-to-day workflow testing, but it is not yet a polished release product. APIs, data structures, UI details, and packaging may still change.

If this problem matches your workflow, issues, suggestions, screenshots, and real-world usage feedback are very valuable.

## Features

- Workspace, project tab, pane, and terminal session hierarchy.
- Local PowerShell terminal sessions.
- Split-pane terminal workbench.
- Workspace snapshot save and restore.
- Recent workspaces, projects, terminal sessions, commands, and saved layouts.
- Workflow templates for AI CLI, frontend, backend, and full-stack setups.
- Quick search across local Chuchen-Terminal data.
- Session attention states for completed, waiting, failed, and stalled terminal tasks.
- System notifications and taskbar attention indicators for unattended AI CLI sessions.
- Provider and runtime configuration screens for local AI CLI workflows.
- Light-first UI with dark theme support.

## Screenshots

The screenshots below use local demo data. Some local paths or notification details may be masked to avoid exposing private workspace information.

| Home / Workspaces | Terminal Workbench |
| --- | --- |
| ![Home](docs/assets/screenshots/home.png) | ![Workbench](docs/assets/screenshots/workbench.png) |

| System Templates | Personal Templates |
| --- | --- |
| ![System Templates](docs/assets/screenshots/template1.png) | ![Personal Templates](docs/assets/screenshots/template2.png) |

| Search | Attention States |
| --- | --- |
| ![Search](docs/assets/screenshots/search.png) | ![Attention](docs/assets/screenshots/attention.png) |

| Settings |
| --- |
| ![Settings](docs/assets/screenshots/settings.png) |

When real screenshots are added or replaced, they should use sanitized demo data only. Do not publish screenshots that contain private project names, terminal logs, provider keys, tokens, company data, or personal account information.
## Who Might Want This

Chuchen-Terminal may be useful if you:

- work with multiple local repositories every day;
- often run frontend, backend, worker, database, and AI CLI terminals together;
- use Codex CLI, Claude Code, Gemini CLI, or other terminal-based AI coding tools;
- want workspace-level organization instead of only terminal-level tabs;
- want reminders when a long-running AI CLI task completes, waits for input, fails, or appears stalled;
- want to save and restore a project terminal layout.

It is probably not the right tool if you only need a lightweight single-terminal emulator, SSH management, or cloud-hosted terminal collaboration.

## Requirements

- Windows 10 / Windows 11
- Node.js 20+
- npm 10+
- Rust 1.77+
- WebView2 Runtime

## Quick Start

Clone the repository:

```bash
git clone https://github.com/TheYoungChen/Chuchen-Terminal.git
cd Chuchen-Terminal/app
```

Install dependencies:

```bash
npm install
```

Run the desktop app:

```bash
npm run tauri:dev
```

Frontend-only development:

```bash
npm run dev
```

Then open:

```text
http://127.0.0.1:6173/
```

## Build

Frontend build:

```bash
cd Chuchen-Terminal/app
npm run build
```

Desktop build:

```bash
cd Chuchen-Terminal/app
npm run tauri:build
```

Tauri and Rust build outputs are generated under `app/src-tauri/target/`. This directory can become very large and must not be committed.

## Demo Data

The repository uses sanitized demo workspace data only. Demo paths use examples such as:

```text
D:\Projects\demo-workspace
D:\Projects\demo-frontend
D:\Projects\demo-backend
D:\Projects\demo-agent
```

Demo data is meant to make the interface understandable on first launch. It should not contain real workspace paths, real project names, provider credentials, API keys, terminal logs, or private commands.

## Repository Layout

```text
app/
  src/                 Vue frontend, UI state, terminal workbench logic
  src-tauri/           Tauri 2 Rust desktop runtime
  scripts/             Local development helper scripts
  tests/               Desktop smoke tests
docs/
  assets/              Public screenshots and demo assets
README.md
README.zh-CN.md
LICENSE
CONTRIBUTING.md
CODE_OF_CONDUCT.md
SECURITY.md
```

Internal planning notes, AI tool state, local caches, generated builds, and personal scratch files are not part of the public source distribution.

## Development Checks

```bash
cd Chuchen-Terminal/app
npm run build
```

Desktop behavior should be verified on Windows because terminal sessions, WebView2 behavior, notifications, and taskbar attention indicators depend on the host OS.

## Roadmap

- Improve public screenshots and release documentation.
- Stabilize workspace, template, search, and settings flows.
- Improve provider and runtime configuration management.
- Add import/export flows for common AI CLI configuration tools.
- Improve long-running task supervision for AI CLI workflows.
- Prepare signed desktop releases.

## Feedback, Issues, and Stars

If you run into a bug, have a workflow that does not fit the current model, or think a feature would make AI CLI work easier, please open an issue.

Useful feedback includes:

- your operating system;
- what AI CLI or terminal workflow you use;
- how many projects and terminals you usually keep open;
- what layout or notification behavior feels wrong;
- screenshots with private data removed.

If this project matches a problem you have also run into, a Star helps other developers discover it.

## Contributing

See `CONTRIBUTING.md`.

## Security

See `SECURITY.md`.

## License

MIT

