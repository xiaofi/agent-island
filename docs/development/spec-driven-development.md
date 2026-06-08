# Spec-Driven Development

Agent Island uses spec-driven development for behavior that changes product
contracts, local system behavior, hook ingestion, privacy, or macOS window
semantics. Small mechanical fixes can still go straight to implementation, but
they must not change user-visible behavior without updating the relevant spec.

## When to Use SDD

Use this workflow when a change affects any of these surfaces:

- Floating island collapsed, expanded, detail, or attention behavior.
- Claude Code or Codex hook install, uninstall, retry, ingest, or diagnostics.
- Privacy rules for titles, paths, prompts, tool data, shell commands, patches,
  transcripts, or local logs.
- macOS window behavior such as full-screen Spaces, transparency, always-on-top,
  drag, focus, or mouse passthrough.
- Test strategy, release gates, or any workflow that decides whether a build is
  shippable.

Do not create a new spec for typo fixes, narrow refactors, dependency bumps, or
test-only changes unless the behavior contract changes.

## Spec Folder

Feature specs live under:

```text
docs/specs/<feature-slug>/
  requirements.md
  design.md
  tasks.md
  acceptance.md
```

`requirements.md` defines user-visible and safety requirements. Prefer
event-shaped requirements:

```text
WHEN <trigger or state>
THE SYSTEM SHALL <observable behavior>
```

`design.md` explains the implementation shape, module boundaries, data flow, and
tradeoffs. Keep it tied to existing architecture docs instead of repeating the
whole architecture.

`tasks.md` is the implementation checklist. Each task should be small enough to
verify and should reference the requirement it satisfies.

`acceptance.md` maps each requirement to verification evidence. Every
requirement needs one of:

- `vitest` unit or component test.
- Rust unit or integration test.
- Browser-preview E2E or screenshot check.
- Native macOS smoke check.
- Manual acceptance item with explicit evidence, used only for behaviors that
  cannot be reliably automated.

## Workflow

1. Read [../ai/context-map.md](../ai/context-map.md) and the minimum task
   documents it points to.
2. Create or update the feature spec before changing code when behavior is new
   or ambiguous.
3. Confirm the requirement list includes non-goals, privacy constraints, and
   failure states.
4. Add the verification mapping in `acceptance.md` before marking a task done.
5. Implement tasks in the smallest useful order.
6. Run the verification commands required by the changed surface.
7. Update short docs, `llms.txt`, and the AI context map when entry points,
   invariants, or commands change.

## Definition of Done

A spec-driven change is done only when:

- Requirements, design, tasks, and acceptance evidence agree.
- Hook and privacy changes use temporary test homes or fixtures, not the user's
  real config as test input.
- Browser-preview verification is labeled as browser preview, not native Tauri
  runtime proof.
- Native macOS window behavior has either an automated smoke check or an
  explicit manual evidence item.
- `npm test -- --run`, `npm run build`, and `cd src-tauri && cargo check` have
  been run when the touched files require them.
