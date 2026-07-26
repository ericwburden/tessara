# Sprint 6C UI Review

Status: ready for product-owner review. The mockups are not approved for
production implementation until the bounded screen deltas are explicitly
accepted.

This directory follows the evidence-first workflow established for Sprint 6B2
and the repository's `docs/ui-prototype-review-standard.md`.

## Review Evidence

- `current-ui-review.md` records the inspected Dashboard and Module Management
  baseline.
- `screen-delta-records.md` bounds the proposed Sprint 6C changes.
- `prototype/` contains the runnable, responsive review suite.
- `prototype/screenshots/` contains the desktop/mobile review set and combined
  reference comparison.
- `prototype/design-qa.md` records visual, responsive, interaction, console,
  build, and packaging checks.

## Prototype Review

The bottom-right **6C** control changes review screens. Dashboard editor and
viewer screens also include a dashed **Prototype control** for switching the
affected placement among the planned resolution states. Neither review control
is proposed production UI.

The review suite covers:

1. Dashboard Module configuration and application state.
2. Dashboard health, diagnostics, and transition Components dependency.
3. Placement-level degradation in the existing Dashboard editor.
4. Contained placement-level degradation in the existing Dashboard viewer.
5. The Core-rendered fallback when the Dashboard module is unavailable.

## Local Validation

From `docs/sprints/sprint-6c-ui-review/prototype`:

```powershell
npm run dev
npm run build
npm run test:sites
```

Product approval applies only to the deltas in
`screen-delta-records.md`. Unlisted Tessara shell, route, control, component,
responsive, and interaction behavior remains unchanged.
