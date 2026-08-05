# Sprint 7B UI Review

Status: product-owner approved for production implementation on 2026-08-04.

The deployed Sprint 7A application at `http://127.0.0.1:8086` is the visual
source of truth for existing product content and the richer Core chrome. The
deployed Dashboard shell is recorded as transitional drift: the prototype uses
one canonical SDK shell for both affected routes and adds only the bounded
Sprint 7B deltas recorded in `screen-delta-records.md`.

## Review package

- `current-ui-review.md` records the deployed routes, states, measurements, and
  capture conditions.
- `screen-delta-records.md` is the approval boundary for each affected screen.
- `reference/` contains fresh deployed-application captures.
- `prototype/` contains the runnable interactive review suite.
- `prototype/screenshots/` contains the desktop, mobile, theme, action, and
  confirmation evidence.
- `prototype/design-qa.md` records visual and interaction QA.
- `approval.md` records the product-owner approval and frozen visual contract.

## Interactive review

The bottom-right **7B** control switches between Dashboard dependency findings
and Component lifecycle, changes the review theme, and selects sample provider
states for the Dashboard screen. These controls are not proposed production UI
and no review-state selector appears inside the product workspace.

The Dashboard screen supports finding filters, quick deferral, Upgrade,
Replace, Remove, and the resulting healthy state. The Component Versions screen
supports activate/deactivate, archive, tombstone, and their confirmations.

Approval applies to the SDK-shell convergence and the product deltas in
`screen-delta-records.md`. All other unlisted navigation, spacing, typography,
responsive, and interaction behavior remains governed by the deployed
application.

## Local verification

From `docs/sprints/sprint-7b-ui-review/prototype`:

```powershell
npm run dev -- --host 0.0.0.0 --port 4173 --strictPort
npm run build
npm run test:sites
```

Production UI implementation is authorized against this approved package. Any
visual-contract change requires an explicit plan amendment and a renewed review.
