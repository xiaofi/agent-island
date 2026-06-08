# macOS E2E Testing Tasks

## Phase 1: Documentation and Gates

- [x] Add SDD workflow documentation.
- [x] Add layered E2E and Mac app testing documentation.
- [x] Add this feature spec with requirements, design, tasks, and acceptance
  mapping.
- [x] Add combined local verification script.
- [x] Link SDD and E2E docs from `llms.txt`, `docs/README.md`, and
  `docs/ai/context-map.md`.

## Phase 2: Deterministic Hook Coverage

- [ ] Add Rust tests for hook install with temporary HOME.
- [ ] Add Rust tests for reinstall idempotency.
- [ ] Add Rust tests for uninstall preserving user commands.
- [ ] Add Rust tests for persisted failure state.
- [ ] Add Rust sanitizer fixtures for Claude Code and Codex hook payloads.
- [ ] Add Rust ingest fixtures for queued spool consumption.

## Phase 3: Browser Preview Evidence

- [ ] Decide whether to add Playwright as a dev dependency.
- [ ] Add a browser-preview smoke for `/`.
- [ ] Add a browser-preview smoke for `/?window=settings`.
- [ ] Add a browser-preview smoke for `/?window=diagnostics`.
- [ ] Store screenshots only when needed for debugging or release notes.

## Phase 4: Native macOS Smoke

- [ ] Add a release smoke checklist entry to the release workflow.
- [ ] Define a stable screenshot/evidence location outside source control.
- [ ] Revisit XCTest or Appium Mac2 after hook runtime behavior stabilizes.
