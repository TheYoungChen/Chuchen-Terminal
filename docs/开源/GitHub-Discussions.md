# GitHub Discussions 启动建议

> 用途：配置 GitHub Discussions 分类，并发布第一条置顶欢迎帖。  
> Purpose: Configure GitHub Discussions categories and publish the first pinned welcome post.

## 推荐分类 / Recommended Categories

建议先保留少量分类，避免新项目看起来很空。

Keep the category list small at the beginning so the community area does not feel empty.

| Category | Format | 用途 |
| --- | --- | --- |
| Announcements | Announcement | 版本发布、路线更新、重要变更 |
| Q&A | Question and Answer | 安装、运行、打包、使用问题 |
| Ideas | Open-ended discussion | 功能建议、工作流建议、UI 改进建议 |
| Show and tell | Open-ended discussion | 用户展示自己的工作区、模板、使用方式 |

## 首帖标题 / First Post Title

```text
Welcome to Chuchen-Terminal Discussions
```

## 首帖正文 / First Post Body

```md
# Welcome to Chuchen-Terminal Discussions

Chuchen-Terminal is a Windows-first terminal workbench for multi-workspace development, split panes, and long-running AI CLI tasks.

This discussion area is used for feedback that is broader than a single bug report.

## You can post here for

- setup or packaging questions
- workflow suggestions
- template ideas
- AI CLI usage scenarios
- UI / UX feedback
- examples of how you organize local terminal workspaces

## 中文说明

Chuchen-Terminal 是一个面向 Windows 本地开发和 AI CLI 工作流的终端工作台，核心目标是把工作区、项目、分屏终端、模板、搜索和长任务提醒组织起来。

这里更适合讨论那些不一定马上变成 Issue 的内容，例如：

- 安装、运行、打包问题
- 工作流建议
- 模板设计想法
- Codex CLI / Claude Code / Gemini CLI 等 AI CLI 使用场景
- UI / UX 反馈
- 你希望终端工作台如何组织工作区和项目

If you find a clear bug, opening an Issue is still better.
If you have a broader workflow idea, start a Discussion here.

如果是明确 bug，建议直接提 Issue。
如果是更开放的工作流想法，欢迎先发 Discussion。
```

## 建议置顶 / Pinning

发布后建议把这条 Discussions 置顶。

After publishing, pin this discussion.

用途：

- 让新访问者知道 Discussions 应该怎么用
- 避免所有反馈都挤到 Issue
- 给未来用户一个低压力的提建议入口

Why:

- It tells visitors how to use Discussions.
- It separates open-ended feedback from bug reports.
- It gives future users a lower-pressure place to share ideas.

## Discussions 与 Issues 的边界 / Discussions vs Issues

| 类型 | 放哪里 |
| --- | --- |
| 明确 bug | Issues |
| 安装失败并带报错日志 | Issues |
| 想法、建议、方向讨论 | Discussions |
| 模板、工作流、使用场景 | Discussions |
| 不确定是不是 bug | Discussions 或 Issues 均可 |

| Type | Use |
| --- | --- |
| Clear bug | Issues |
| Install failure with logs | Issues |
| Ideas and product direction | Discussions |
| Templates, workflows, use cases | Discussions |
| Not sure whether it is a bug | Either is fine |

