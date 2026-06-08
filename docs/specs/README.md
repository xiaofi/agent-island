# Feature Specs

This directory contains spec-driven development work items. Use it for changes
where the expected behavior needs to be stable before implementation.

Each feature directory should contain:

```text
requirements.md
design.md
tasks.md
acceptance.md
```

Keep specs focused. Product-wide behavior belongs in
[../product/spec.md](../product/spec.md), architecture-wide design belongs in
[../architecture/technical-plan.md](../architecture/technical-plan.md), and
hook architecture belongs in
[../architecture/hook-integration-plan.md](../architecture/hook-integration-plan.md).

## Status Labels

Use these labels in `tasks.md` and `acceptance.md`:

- `planned`: agreed but not started.
- `in-progress`: actively being implemented.
- `verified`: implemented and backed by the mapped evidence.
- `manual-only`: cannot be automated reliably in the current stack.
- `blocked`: cannot proceed without an external dependency or user decision.

## Current Specs

- [macos-e2e-testing](macos-e2e-testing/requirements.md): layered SDD and E2E
  testing strategy for Agent Island's Tauri macOS app.
