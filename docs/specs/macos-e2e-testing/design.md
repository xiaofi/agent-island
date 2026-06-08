# macOS E2E Testing Design

## Model

Agent Island verifies behavior through a layered model:

```text
requirements
  -> deterministic tests
     -> browser-preview evidence
        -> native macOS smoke evidence
```

Deterministic tests should carry as much logic as possible. Native macOS checks
should stay small because they depend on desktop state, permissions, and runtime
window behavior.

## Test Layer Ownership

TypeScript owns UI and store semantics:

- `src/domain/*.test.ts`
- `src/stores/*.test.ts`
- `src/components/**/*.test.ts`
- `src/app/*.test.ts`

Rust owns local system and ingestion semantics:

- `src-tauri/src/services/config_store.rs`
- `src-tauri/src/services/hook_installer.rs`
- `src-tauri/src/services/hook_ingest.rs`
- `src-tauri/src/aggregator/`
- `src-tauri/src/adapters/`

Browser preview owns layout and screenshot checks for mock bridge state:

- `/`
- `/?window=settings`
- `/?window=diagnostics`

Native smoke owns window behavior:

- real Tauri app launch
- transparent always-on-top island
- full-screen Space behavior
- drag and persisted position
- settings and diagnostics windows
- mouse passthrough

## Current Project Fit

The current project already has Vitest coverage for priority, privacy, settings,
task store behavior, and island components. Rust currently uses `cargo check` as
the baseline, with hook integration tests planned around temporary HOME and app
support paths.

The first useful implementation step is not a broad Appium suite. It is to make
the acceptance mapping explicit and then add missing Rust integration tests for
hook safety and ingest behavior.

## Future Native Automation

If manual native smoke becomes too slow or inconsistent, add one small XCTest or
Appium Mac2 suite. It should launch the built app and verify only native-shell
observables:

- app process starts
- island window exists
- settings window opens
- diagnostics window opens

Do not move task priority, sanitizer, hook config merge, or state-machine
assertions into native UI tests.
