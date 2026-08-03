# Sprint 7A Test Change Log

## 2026-08-02 — Scoped analytics acceptance inventory

- Added three exact Playwright identities in
  `end2end/tests/analytics-sprint-7a.spec.ts`: canonical source-exact inventory,
  real Dashboard-mediated stat/table execution, and browser-authority rejection
  without resource disclosure.
- Increased the tracked acceptance total from 65 to 68.
- Corrected the tracked Sprint 6F composition read-back identity to match the
  existing test title. This is an inventory repair only; the assertion body is
  unchanged.
- Added tracked Sprint 7A smoke, authorization-conformance,
  known-versus-random nondisclosure, scripted UAT, acceptance-actor setup, and
  eleven manual UAT scripts.
- Aligned retained Dashboard, Module Management, and permissions coverage with
  the Sprint 7A reference topology: Dashboard execution is placement-mediated,
  the undeployed SDK reference stays absent, and permission scenarios own both
  their in-scope and out-of-scope Dashboard fixtures.
- Bound retained presentation and SSR inventory assertions to the exact
  reference composition: Table and stat-card Components are present, chart
  Components are absent, and Module Management exposes eight definitions.
- Converted retained saved-Dashboard request tracking—including concurrency
  permits and Table controls—to assert placement-mediated render routes;
  unsaved editor preview remains exact-version pinned until a placement exists.
- Rebound retained Table request-state and mobile-fullscreen coverage to the
  canonical `sprint-7a-record-table`; legacy paged-fixture assertions now run
  only when a paged Table is actually present in the selected composition.

These changes close previously missing acceptance coverage. They do not weaken
an existing assertion or remove a test. Formal SIT and UAT must start from a new
candidate because the tracked acceptance inventory changed.
