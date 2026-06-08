# macOS E2E Testing Acceptance

| Requirement | Verification | Status |
| --- | --- | --- |
| R1 Layered Verification | `docs/development/spec-driven-development.md` defines the requirement-to-evidence rule. | verified |
| R2 Safe Hook Testing | `docs/development/e2e-testing.md` requires temporary HOME/app support paths for hook tests. | verified |
| R3 Browser Preview Boundary | `docs/development/e2e-testing.md` labels browser preview as UI/layout evidence only. | verified |
| R4 Native macOS Smoke | `docs/development/e2e-testing.md` includes the native smoke checklist. | verified |
| R5 Privacy-Preserving Evidence | `docs/development/e2e-testing.md` and `requirements.md` prohibit sensitive evidence capture. | verified |
| R6 CI Boundary | `docs/development/e2e-testing.md` keeps default gates free of macOS Accessibility and user agent state. | verified |

## Current Evidence

Documentation and command wiring are the first implemented slice. The current
automated gate is:

```bash
npm run check:all
```

Future hook-specific requirements must add Rust test evidence here before being
marked `verified`.
