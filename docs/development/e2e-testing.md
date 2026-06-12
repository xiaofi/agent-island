# E2E and Mac App Testing

Agent Island needs layered testing. A single E2E suite cannot cover all product
risk because the app combines Vue UI, Tauri IPC, Rust services, local hook
files, and macOS-specific floating window behavior.

## Core Decision

Use automated tests for deterministic logic and a small native macOS smoke path
for behaviors that only exist in the real desktop runtime.

Tauri WebDriver E2E is not the primary path for this project on macOS desktop:
desktop WKWebView does not provide the same WebDriver surface that makes this
approach practical on Linux or Windows. Browser-preview tests are still useful,
but they prove Vue layout and mock bridge behavior rather than native Tauri
window behavior.

## Test Layers

| Layer | Tooling | Covers | Default gate |
| --- | --- | --- | --- |
| TypeScript unit/component | Vitest, Vue Test Utils, happy-dom | task priority, task presentation, stores, compact/expanded UI, settings interactions | yes |
| Rust unit/integration | `cargo check`, later `cargo test` | adapters, config store, hook installer, hook ingest, sanitizer, aggregator | yes |
| Browser preview | Vite preview, optional Playwright | layout, full settings/diagnostics windows, screenshot evidence using mock data | optional |
| Native macOS smoke | manual checklist, later XCTest or Appium Mac2 | app launch, real windows, full-screen Space behavior, transparency, drag, Dock visibility, native notifications | release gate |
| Real agent scenario | Claude Code/Codex sample hooks and local spool fixtures | source-gated runtime status behavior without storing sensitive content | release gate for hook work |

## Current Commands

Default local verification:

```bash
npm test -- --run
npm run build
cd src-tauri && cargo check
```

Combined script:

```bash
npm run check:all
```

Browser preview entry points:

```text
http://127.0.0.1:5173/
http://127.0.0.1:5173/?window=settings
http://127.0.0.1:5173/?window=diagnostics
```

Use browser-preview screenshots only as UI/layout evidence. Do not use them as
proof that Tauri window APIs, native focus, full-screen Space behavior, Dock
visibility, or native notifications work.

## Hook Test Rules

Hook and config tests must not mutate real user files. Use a temporary HOME or
temporary app support directory for:

- `~/.codex/hooks.json`
- `~/.claude/settings.json`
- Agent Island install manifests.
- Hook spool JSONL files.
- Receipt or diagnostic logs.

Required integration scenarios for hook work:

- Install adds only Agent Island's command and preserves user commands.
- Reinstall is idempotent.
- Uninstall removes only Agent Island's command.
- Failed install, uninstall, repair, or self-test persists a visible failure.
- Retry success clears the failure.
- Disabled source does not emit or display hook tasks for that source.
- Sanitizer never persists prompt text, assistant text, full tool I/O, shell
  commands, patches, transcripts, or full transcript paths.

## Native macOS Smoke Checklist

Run this before a release and after changes to window or notification services:

- Launch `npm run tauri -- dev` or a built `.app`.
- Confirm the island window appears and is not blank.
- Confirm the island remains visible above normal desktop windows.
- Confirm full-screen Space behavior follows
  [ADR 0003](../decisions/0003-macos-fullscreen-overlay.md).
- Drag the island and confirm position persistence after restart.
- Open settings and diagnostics windows from the island.
- Toggle Dock visibility if the touched code affects app activation or window
  services.
- Send a test notification if the touched code affects notifications, and note
  whether it appears as a banner or only in Notification Center.
- Capture a screenshot or short note with app version, date, and command used.

## Automation Boundary

XCTest or Appium Mac2 can be added later for smoke automation, but they should
only cover a small native path. Do not move core business logic assertions into
native UI tests when they can be tested in Rust or Vitest.

The first automation target, if needed, should be browser-preview Playwright for
settings and diagnostics screenshots. The first native target should be a launch
smoke that verifies the app process starts and the expected windows are present.
