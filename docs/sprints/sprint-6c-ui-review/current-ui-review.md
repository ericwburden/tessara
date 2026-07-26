# Sprint 6C Current UI Review

Reviewed on 2026-07-26 before production UI implementation.

## Sources Inspected

- Sprint 5A Dashboard production captures under
  `docs/audits/sprint-5a-ui-review-2026-07-13/`, including desktop and mobile
  directory, editor, viewer, table, and fullscreen states.
- Approved Sprint 5A Dashboard mockups under `docs/mockups/`.
- Sprint 6B2 Module Management production captures and approved screen-delta
  records.
- Current Dashboard route, service, DTO, web-component, stylesheet, smoke,
  UAT, and Playwright sources in the Sprint 6C worktree.
- Sprint 6C roadmap and implementation plan.

The completed Sprint 6B2 stack was intentionally left stopped after closeout.
The checked-in production captures and current source are sufficient for this
bounded delta review; no speculative shell or healthy-state redesign is
introduced.

## Baseline Findings

### Dashboard product screens

- Directory, create, detail, editor, and viewer already have settled Sprint 5A
  layout, actions, grid behavior, responsive structure, table/charts, and
  authoring semantics.
- Desktop uses the fixed Tessara sidebar and top application bar. Mobile uses
  the compact top bar, stacked placement cards, full-width controls, and no
  horizontal page overflow.
- A resolved placement currently assumes the in-process Components provider is
  available. Existing unavailable treatment is not rich enough to represent
  the process-boundary states introduced by Sprint 6C.
- The editor already has a Placement details affordance. Resolution recovery
  should point there instead of adding a second placement-management model.
- A single placement failure must not replace the entire Dashboard viewer when
  other placements remain resolvable.

### Module Management

- Sprint 6B2 establishes the independently deployed Module detail hierarchy:
  identity and deployment actions, status badges, tabs/mobile selector,
  module-owned configuration, separate application state, health, diagnostics,
  and sanitized operational context.
- Dashboard should reuse that hierarchy and component anatomy exactly.
- The transition Components adapter needs visible, non-secret diagnostic
  readback because it is the key new dependency boundary, but it must not be
  presented as a separately deployed Components Module Instance.

### Failure containment

- Core already owns unavailable-module fallback routing. Sprint 6C needs
  Dashboard-specific copy that says what remains available and protected.
- Unauthorized or not-evaluated placement resolution must be non-disclosing.
  Detailed lifecycle, compatibility, identity, and provider states appear only
  after authorization.

## Design Direction

- Preserve every healthy Dashboard screen and existing shell pattern.
- Add information only where the new process boundary creates a user decision
  or recovery action.
- Use existing Tessara badges, bordered panels, compact inline notices, Module
  cards, and primary/secondary actions.
- Keep the editor canvas treatment limited to the affected placement footprint;
  reveal full diagnostic copy only on demand in a side sheet.
- Use one clear status, one explanation, and at most one recovery action per
  placement state.
- Treat the prototype screen/state switchers as review tooling only.
