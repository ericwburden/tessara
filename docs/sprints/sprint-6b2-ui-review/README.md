# Sprint 6B2 UI Review

This directory is the quality reference for Tessara sprint UI prototypes. It follows [`docs/ui-prototype-review-standard.md`](../../ui-prototype-review-standard.md).

## Review Evidence

- [`current-ui-review.md`](./current-ui-review.md) records the inspected production baseline.
- [`screen-delta-records.md`](./screen-delta-records.md) bounds the proposed Sprint 6B2 changes.
- [`reference/`](./reference/) contains captures of the running application.
- [`prototype/`](./prototype/) contains the runnable review suite.
- [`prototype/design-qa.md`](./prototype/design-qa.md) records the visual, responsive, interaction, console, build, and test checks.
- [`prototype/screenshots/design-qa-comparison.png`](./prototype/screenshots/design-qa-comparison.png) is the combined visual comparison.
- [`prototype/screenshots/feedback-refinement-comparison.png`](./prototype/screenshots/feedback-refinement-comparison.png) records the product-owner annotation pass as before/after evidence.

## Run And Verify

From `docs/sprints/sprint-6b2-ui-review/prototype`:

```powershell
npm ci
npm run dev
```

Before handoff:

```powershell
npm run build
npm run test:sites
```

The suite's status as a quality benchmark records approval of its evidence-first review process. Product approval of the individual screen deltas remains a separate, explicit gate before production UI implementation.
