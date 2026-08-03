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

## 2026-08-03 — Dashboard provider-failure containment regression

- Added one exact Dashboard Playwright identity that forces a placement render
  to return the stable provider-unavailable response while a sibling placement
  succeeds.
- Increased the tracked acceptance total from 68 to 69.
- The test requires the Dashboard shell and healthy placement to remain usable,
  the failed placement to retain contained unavailable copy while bounded
  retries run, and the loading placeholder not to replace that failure state.
- Added focused Rust proof that Dashboard's shared Components-provider client
  terminates a request at its configured deadline and that an authorized
  provider outage remains distinct from a forbidden resolution.

This strengthens AC-14 and directly prevents recurrence of UAT-7A-05. It does
not change an existing expectation, increase a test timeout, or add a retry to
the Playwright runner. The source and tracked acceptance inventory changes
require a new candidate and complete SIT and UAT.

## 2026-08-03 — Component editor expected-error synchronization

- Bound the existing visual-component workflow to both expected HTTP 400
  preview responses produced while the test deliberately selects invalid
  text-field calculations.
- The synchronization prevents the browser's corresponding resource errors
  from racing past the test's expected-error reset. No assertion, timeout,
  retry, or acceptance identity changed.

This is a harness-integrity correction discovered during Candidate 10 SIT.
Because tracked test evidence changed, formal validation must restart from a
new source-exact candidate.
