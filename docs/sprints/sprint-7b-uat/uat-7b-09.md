# UAT-7B-09 — Approved visual contract

Capture Component Versions and Dashboard editor states for manager, editor, and
viewer roles at 1280, 768, and 390 CSS pixels, 1× density, in dark and light
themes. Compare against the deployed-baseline captures and approved references
in `docs/sprints/sprint-7b-ui-review/reference/`. Verify identical SDK-owned Core
and Dashboard chrome/responsive anatomy; 31×31 aligned placement symbols; the
shared vertical-dot menu; dependency health sheet, dialogs, focus order, Escape,
keyboard operation, 200% zoom, and no overflow. Verify the redundant health note,
prototype filter, and parallel local shell are absent. Record pixel comparisons
and classify every mismatch; any P0/P1/P2 mismatch fails UAT.
