# Chuchen-Terminal

> 面向 AI CLI 时代的本地终端工作台：多工作区、多项目、多 Pane、长任务提醒。

[English](README.md)

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows-2f74ff.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.x-24c8db.svg)
![Vue](https://img.shields.io/badge/Vue-3.x-42b883.svg)

Chuchen-Terminal 是一个 Windows 优先的本地桌面终端工作台，基于 Tauri 2、Vue 3、TypeScript、Rust 和 xterm.js 构建。

它不是想再做一个“终端模拟器”，而是想解决 AI 编程工具流行之后，一个越来越明显的问题：

> 终端越来越多，任务越来越长，项目上下文越来越分散，但传统终端仍然只把它们当成一个个 Tab。

Chuchen-Terminal 把终端放回工作流里管理：

```text
工作区 -> 项目 Tab -> Pane -> 终端 Session
```

你可以把它理解成一个面向本地开发和 AI CLI 长任务的 split-pane terminal manager。

## 预览

> 以下截图使用本地演示数据；涉及本机路径或系统通知细节的位置已按需要做脱敏 / 打码处理。

| 首页 / 工作区 | 终端工作台 |
| --- | --- |
| ![Home](docs/assets/screenshots/home.png) | ![Workbench](docs/assets/screenshots/workbench.png) |

| 系统模板 | 个人模板 |
| --- | --- |
| ![System Templates](docs/assets/screenshots/template1.png) | ![Personal Templates](docs/assets/screenshots/template2.png) |

| 搜索 | 任务提醒 |
| --- | --- |
| ![Search](docs/assets/screenshots/search.png) | ![Attention](docs/assets/screenshots/attention.png) |

| 设置 |
| --- |
| ![Settings](docs/assets/screenshots/settings.png) |
## 它解决什么问题？

如果你经常使用 Codex CLI、Claude Code、Gemini CLI 或其他终端形态的 AI 编程工具，可能会遇到这些情况：

- 同时打开多个项目，前端、后端、脚本、AI CLI 分散在不同终端里。
- AI CLI 跑了几分钟后完成了，但你没有及时看到。
- 某个终端正在等审批、等输入、报错或疑似卡住，但很难从一堆标签里发现。
- 昨天跑过的命令、保存过的布局、项目路径都要重新找。
- 关闭应用后，想恢复之前的分屏现场很麻烦。

Chuchen-Terminal 的目标是把这些状态和关系显式管理起来，而不是继续靠终端标题、命令历史和人工记忆。

## 核心能力

- **工作区层级管理**：工作区、项目 Tab、Pane、终端 Session 形成清晰结构。
- **分屏终端工作台**：支持多个 Pane 同时展示本地终端会话。
- **工作现场保存 / 恢复**：保存项目 Tab、Pane、终端标签与焦点状态。
- **最近记录**：集中查看最近工作区、项目、终端、命令和布局快照。
- **工作流模板**：内置 AI CLI、前端、后端、全栈等常见模板。
- **快速搜索**：搜索工作区、项目、终端、路径、配置、历史命令和布局。
- **任务提醒**：识别完成、等待输入、异常退出、疑似停滞等状态。
- **系统通知 / 任务栏提醒**：用于无人值守的 AI CLI 长任务。
- **Provider / 运行配置入口**：为 AI CLI 工作流保留本地配置管理能力。
- **浅色 / 暗色主题**：面向日常工作台使用的浅色优先 UI。

## 适合谁？

这个项目更适合：

- 每天会同时打开多个本地仓库的开发者；
- 经常同时运行前端、后端、Worker、脚本和 AI CLI 的开发者；
- 希望管理 Codex CLI、Claude Code、Gemini CLI 等长时间运行任务的人；
- 想把终端按照“工作区 / 项目 / Pane”组织起来的人；
- 想保存和恢复某个项目终端布局的人。

它不太适合：

- 只需要一个极简终端模拟器的人；
- 主要需求是 SSH 连接管理的人；
- 需要云端多人协作终端的人；
- 期待一个已经完全稳定、开箱即用的正式发行版的人。

## 当前状态

Chuchen-Terminal 目前处于 **MVP / Preview** 阶段。

已经可以用于本地体验和工作流试用，但仍在快速迭代中。UI、数据结构、打包发布、Provider 配置和任务监督能力后续都会继续调整。

如果你也遇到类似问题，欢迎提 Issue。真实使用场景、Bug 反馈、功能建议和截图都很有价值。

## 环境要求

- Windows 10 / Windows 11
- Node.js 20+
- npm 10+
- Rust 1.77+
- WebView2 Runtime

## 从源码运行

克隆仓库：

```bash
git clone https://github.com/TheYoungChen/Chuchen-Terminal.git
cd Chuchen-Terminal/app
```

安装依赖：

```bash
npm install
```

启动桌面端：

```bash
npm run tauri:dev
```

只启动前端页面：

```bash
npm run dev
```

浏览器访问：

```text
http://127.0.0.1:6173/
```

## 构建

前端构建：

```bash
cd Chuchen-Terminal/app
npm run build
```

桌面端打包：

```bash
cd Chuchen-Terminal/app
npm run tauri:build
```

Tauri / Rust 编译产物会生成在 `app/src-tauri/target/`，这个目录可能非常大，不应该提交到 Git。

## 演示数据

仓库只保留脱敏后的演示工作区数据。示例路径类似：

```text
D:\Projects\demo-workspace
D:\Projects\demo-frontend
D:\Projects\demo-backend
D:\Projects\demo-agent
```

演示数据只用于帮助第一次打开项目时理解界面结构，不应该包含真实工作区路径、真实项目名、Provider 凭据、API Key、终端日志或私有命令。

## 仓库结构

```text
app/
  src/                 Vue 前端、UI 状态、终端工作台逻辑
  src-tauri/           Tauri 2 Rust 桌面运行时
  scripts/             本地开发辅助脚本
  tests/               桌面端冒烟测试
docs/
  assets/              公开截图和演示素材
README.md
README.zh-CN.md
LICENSE
CONTRIBUTING.md
CODE_OF_CONDUCT.md
SECURITY.md
```

内部计划文档、AI 工具状态、本地缓存、生成产物和个人草稿不属于公开源码分发内容。

## 开发检查

```bash
cd Chuchen-Terminal/app
npm run build
```

涉及终端进程、WebView2、系统通知、任务栏提醒、文件选择器、Windows 路径等行为时，需要在 Windows 桌面环境下验证。

## 路线图

- 补充正式截图和演示 GIF。
- 稳定工作区、模板、搜索和设置流程。
- 完善 Provider 和运行配置管理。
- 增加常见 AI CLI 配置工具的导入 / 导出能力。
- 强化 AI CLI 长任务监督能力。
- 准备签名后的桌面端 Release。

## 反馈、Issue 和 Star

这个项目还在早期阶段，最需要的不是“完美需求”，而是真实使用场景。

如果你愿意反馈，建议包含：

- 你使用的操作系统；
- 你正在使用的 AI CLI 或终端工作流；
- 平时会同时打开多少项目和终端；
- 哪些布局、提醒或交互不符合预期；
- 已经脱敏的截图。

如果 Chuchen-Terminal 解决了你也遇到的问题，欢迎点一个 Star。它能帮助更多有类似需求的人发现这个项目。

## 参与贡献

见 `CONTRIBUTING.md`。

## 安全

见 `SECURITY.md`。

## License

MIT

