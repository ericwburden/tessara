# Sprint 6C Design QA

## Source and implementation

- Approved configuration mockup:
  `docs/sprints/sprint-6c-ui-review/prototype/screenshots/01-module-configuration-desktop.png`
- Approved diagnostics mockup:
  `docs/sprints/sprint-6c-ui-review/prototype/screenshots/02-diagnostics-desktop.png`
- Final live implementation:
  `http://127.0.0.1:8080/administration/modules/tessara.dashboards`
- Matched desktop viewport: 1280 × 720 CSS pixels at 1× density.
- Responsive viewports: 768 × 900 and 390 × 844 CSS pixels.
- Theme and state: dark theme, authenticated Tessara administrator, healthy and
  enabled Dashboard Module Instance.

## Same-input comparisons

- Configuration:
  `.codex-run/qa/comparison-module-configuration.png`
- Findings-based health and diagnostics:
  `.codex-run/qa/comparison-diagnostics.png`

Each comparison places the approved mockup on the left and the final hydrated
implementation on the right. Live release numbers, page-size configuration,
and observed health data intentionally reflect the running Sprint 6C stack.

## Visual review

- The existing Tessara shell, type system, color tokens, borders, spacing, and
  Lucide icon treatment remain intact.
- Module detail exposes the approved nine tabs; Diagnostics is not duplicated
  as a tenth tab.
- The transition binding note is the final row of the Configuration card.
- Application state remains a separate card with the operational enablement
  control and diagnostics action.
- Findings now renders the approved Dashboard health composition: heading,
  refresh/download actions, four metric cards, and the Components compatibility
  dependency card.
- Live liveness and dependency timestamps use compact reader-facing copy while
  preserving the machine-readable observation time.
- No actionable P0, P1, or P2 visual mismatch remains.

## Interaction and degraded-state review

- Dashboard and Scoped Records switches were disabled and re-enabled through
  the rendered controls; Core and each module agreed on the resulting state.
- The health action updates the hash, selects Findings, and renders diagnostics.
- A Dashboard-process outage rendered the Core-owned unavailable page and kept
  Module Management reachable.
- Components-provider unavailability preserved the Dashboard editor, applied
  warning treatment to affected placements, opened the issue side sheet from
  the warning icon, and exposed retry. Recovery restored normal placement
  selection without changing the saved layout.
- Document width equaled viewport width at 1280, 768, and 390 CSS pixels.
- The final browser error log was empty.

## Automated evidence

- `cargo check -p tessara-web -p tessara-api`
- Focused web and API unit tests for diagnostics hash mapping and module
  enablement document states.
- Focused Playwright regression:
  `independent module controls match the approved configuration and diagnostics flow`

final result: passed
