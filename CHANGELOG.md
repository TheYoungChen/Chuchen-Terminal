# Changelog

## Unreleased - June 8 to July 16, 2026

This update represents roughly five weeks of development after the first public preview.

### Provider management

- Added local Provider profile discovery for Codex CLI, Claude Code, Gemini CLI, and Hermes.
- Added complete profile import and switching instead of changing only a model or base URL. A profile can carry its name, notes, request endpoint, authentication data, model selection, permissions, MCP settings, and complete native configuration payload.
- Added native live-configuration detection and CC Switch as an optional import source rather than a required runtime dependency.
- Added Provider identity reconciliation, source classification, active-profile detection, duplicate merging, and stable `identityKey` mapping.
- Added full-profile write-back while preserving each CLI's native configuration format.
- Added AI vendor and tool icons, Provider metadata, current-period request metrics, and direct navigation into filtered Usage statistics.

### Usage and cost analytics

- Added a unified Usage query for Codex, Claude, Gemini, Hermes, and imported CC Switch history.
- Added full-range summary, trend, Provider, model, channel, and Token aggregates that are independent from request-log pagination.
- Added day, week, month, all-time, and custom time-range filtering with automatic minute/hour/day/week/month buckets.
- Added cursor-based request detail pagination, Provider grouping, model rankings, channel filtering, request status, first-token latency, total duration, and four-part Token accounting.
- Added official model pricing with local overrides for input, output, cache-read, and cache-creation Tokens.
- Added session/proxy reconciliation and duplicate suppression using Provider/model/Token fingerprints and a time window, preferring proxy records when the same request exists in both sources.
- Added CC Switch `proxy_request_logs` and historical `usage_daily_rollups` aggregation without exposing rollups as fake request rows or channels.
- Added real-time `usage-log-recorded` and `model-pricing-updated` refresh events with a low-frequency polling fallback.

### Native session ingestion and performance

- Removed the previous native-session file-count caps so historical Usage is no longer silently truncated.
- Added persistent native Usage snapshots under `~/.chuchen-terminal/`.
- Added Codex and Claude byte-offset tail reads with parser state, partial-line protection, unchanged-file zero parsing, and safe reset after truncation or rotation.
- Added Gemini whole-document refresh handling for its rewritten JSON session format and kept Hermes on its independent local database path.
- Reduced repeated Provider and Usage scans, separated aggregate queries from detail limits, and avoided repeated snapshot writes on unchanged hot queries.

### Terminal workbench and interface

- Added core Chinese/English localization across navigation, built-in examples, runtime messages, Provider pages, and Usage pages.
- Improved immersive workbench behavior, Explorer expansion, split-pane sizing, terminal tab layout, workspace restoration, and runtime state handling.
- Added multi-terminal runtime throttling and a repeatable terminal stress-test script.
- Reworked Provider cards, Usage KPI tiles, charts, tooltips, filters, tables, loading states, and responsive layouts.
- Added shared model-pricing management, Hermes branding, improved AI CLI attention handling, and multiple fixes for refresh loops, layout overlap, and high-CPU scans.

### Compatibility notes

- The Provider/Usage backend and frontend now share the same response contracts, Provider identity semantics, model statistics, pricing storage, and refresh event names.
- Balance and quota querying remains intentionally separate because third-party relay services expose incompatible APIs.
- OpenCode Usage ingestion and a built-in local proxy/failover layer are not included in this update.

## 0.1.0

- Initial MVP preview of Chuchen-Terminal.
- Added workspace, project tab, pane, and terminal session hierarchy.
- Added local terminal runtime, snapshot restore, recent activity, templates, search, settings, and session attention features.
