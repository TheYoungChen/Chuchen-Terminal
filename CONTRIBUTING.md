# Contributing

Thanks for considering a contribution to Chuchen-Terminal.

## Local Development

```bash
cd app
npm install
npm run dev
```

Desktop development:

```bash
cd app
npm run tauri:dev
```

Build check:

```bash
cd app
npm run build
```

## Pull Request Checklist

- Keep changes focused on one feature or fix.
- Run the relevant build or verification command before opening a PR.
- Do not commit `node_modules`, `dist`, `app/src-tauri/target`, logs, local caches, screenshots with private data, or personal workspace data.
- Remove private paths, API keys, provider tokens, terminal logs, company names, and local machine details from examples and screenshots.
- Document user-facing behavior changes in `README.md` or `CHANGELOG.md` when appropriate.

## Windows Validation

Chuchen-Terminal is a desktop terminal app. Web-only checks are not enough for changes involving:

- terminal process startup,
- WebView2 behavior,
- system notifications,
- taskbar attention indicators,
- file dialogs,
- Windows paths.

Verify those changes on a Windows host before claiming they are fully validated.
