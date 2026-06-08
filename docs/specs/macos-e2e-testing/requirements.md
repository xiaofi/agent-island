# macOS E2E Testing Requirements

## Scope

This spec defines how Agent Island verifies behavior across Vue, Rust, Tauri,
hook files, and real macOS windows.

## Requirements

### R1: Layered Verification

WHEN a change modifies product behavior
THE SYSTEM SHALL map each requirement to a specific verification layer before
the task is considered done.

### R2: Safe Hook Testing

WHEN tests exercise hook install, uninstall, repair, self-test, ingest, or spool
behavior
THE SYSTEM SHALL use temporary HOME and app support paths instead of real user
Claude Code, Codex, or Agent Island configuration.

### R3: Browser Preview Boundary

WHEN browser preview is used for UI checks
THE SYSTEM SHALL label the evidence as browser-preview evidence and not treat it
as proof of native Tauri window behavior.

### R4: Native macOS Smoke

WHEN a release is prepared or native window behavior changes
THE SYSTEM SHALL run a native macOS smoke check for app launch, visible island
window, settings/diagnostics window opening, and any touched window behavior.

### R5: Privacy-Preserving Evidence

WHEN test evidence or logs are captured
THE SYSTEM SHALL avoid storing prompt text, assistant text, full tool I/O, full
shell commands, patches, transcripts, or full transcript paths.

### R6: CI Boundary

WHEN default verification runs in CI or a coding-agent environment
THE SYSTEM SHALL avoid depending on macOS Accessibility permission, signed app
bundles, or a user's installed Claude Code/Codex state.

## Non-Goals

- This spec does not require a full Appium or XCTest suite before the current
  MVP can proceed.
- This spec does not replace unit or integration tests with screenshot tests.
- This spec does not make browser-preview tests a substitute for native Tauri
  window validation.
