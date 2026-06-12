# ADR 0004: SDD and Layered macOS Testing

## Status

Accepted.

## Context

Agent Island combines a Vue UI, Tauri commands/events, Rust services, local
hook files, and macOS-specific floating window behavior. The product also has
strict privacy and user-config safety constraints. A change can look correct in
browser preview while still failing in the native Tauri runtime, especially for
transparent windows, full-screen Spaces, drag, focus, Dock visibility, and
native notifications.

The project already has long product and architecture specs, but feature work
needs a smaller mechanism that maps requirements to implementation tasks and
verification evidence.

## Decision

Behavior-changing work uses spec-driven development when it affects product
contracts, hook ingestion, privacy, local config writes, or macOS window
semantics. Feature specs live in `docs/specs/<feature>/` with:

- `requirements.md`
- `design.md`
- `tasks.md`
- `acceptance.md`

Testing is layered:

- Vitest and Vue Test Utils for UI, stores, and deterministic TypeScript logic.
- Rust checks and tests for adapters, config store, hook install, hook ingest,
  sanitizer, and aggregator behavior.
- Browser preview for Vue layout and screenshot evidence using mock bridge data.
- Native macOS smoke checks for real Tauri app windows and platform behavior.

Browser-preview evidence must not be treated as proof of native Tauri window
behavior.

## Consequences

- New complex behavior has a clear requirements-to-evidence path before it is
  marked done.
- Hook tests must use temporary HOME and app support paths instead of user
  config files.
- Native macOS behavior remains a release smoke gate even if browser-preview UI
  checks pass.
- Appium Mac2 or XCTest can be added later, but only for small native smoke
  paths rather than business-logic assertions.
