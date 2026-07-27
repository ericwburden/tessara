# Sprint 6C Verification

Status: closeout-ready.

This record maps the Sprint 6C acceptance criteria to the final clean,
source-labeled deployment. Machine-readable evidence and SHA-256 sidecars are
retained under `artifacts/sprint-6c-closeout/`.

## Source And Deployment

- Deployment profile: `deploy/sprint-6c/compose.yaml`.
- The retained stack uses one clean commit/tree identity across Core,
  Dashboard, installation control, and Scoped Records release images.
- `scripts/bootstrap-sprint-6c-deployment.ps1` produced deployment receipt
  revision 1. Two subsequent invocations returned the existing revision
  without mutation.
- The receipt binds the Dashboard Module Release and Instance to the immutable
  locally built Dashboard image digest.
- Core and Dashboard each apply one squashed baseline to a fresh database.
  Applying the Core baseline alone creates no `dashboard%` product table.

The exact commit, tree, image IDs, container IDs, installation ID, timestamps,
and database snapshot are recorded in `deployment-fresh.json`; they are not
copied into this document so the retained machine record remains authoritative.

## Automated Verification

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features`: passed with only
  existing advisory warnings.
- `cargo test --workspace --no-fail-fast`: passed against disposable Core,
  enrollment, deployment-control, Scoped Records, and fresh Sprint 6C
  databases. Two obsolete Core-owned Dashboard demo fixtures remain explicitly
  ignored because the canonical split-stack smoke/UAT coverage supersedes
  them.
- Dashboard module/web focused tests cover idempotent composition replay,
  signed shell rendering, resolution vocabulary, title nondisclosure, and the
  approved warning-icon-only editor tile across every degraded resolution
  state.
- `scripts/test-sprint-6c-degraded-states.ps1`: passed the complete
  `available`, provider-unavailable, incompatible, inactive, superseded,
  resource-tombstoned, owner-tombstoned, owner-data-destroyed, missing, and
  not-evaluated matrix. Every state returned nine consistently classified
  placements with all saved titles retained, and cleanup restored the original
  provider and Module Instance enablement states.
- `scripts/verify-sprint-6c-isolation.ps1`: passed. The Dashboard runtime role
  can read its own database and cannot connect to Core, deployment-control,
  Scoped Records, or another module database; the inverse runtime checks also
  fail closed.
- Fresh smoke: passed and retained as `smoke-fresh.json`.
- Fresh UAT: passed and retained as `uat-fresh.json`.
- Canonical Playwright acceptance: all 60 tests passed with one worker, zero
  retries, zero skips, and zero failures; proof is retained as
  `playwright-acceptance-fresh.json`.

## Browser And Outage Verification

The production stack at `http://127.0.0.1:8080` was exercised with the seeded
nine-placement Dashboard.

- Normal directory, detail, editor, and viewer pages render through the
  Dashboard service inside the Tessara shell.
- With the Components provider set to `unavailable`, each placement retained
  its saved title, used full-tile warning coloring, and replaced normal content
  with one prominent warning icon. No diagnostic paragraph or retry control
  appeared inline.
- Activating the icon opened the placement-issue side sheet with the complete
  provider message, containment copy, a semantic `Retry resolution` button,
  and a centered 96-by-96-pixel warning-icon treatment.
- At 390 by 844 pixels the sheet occupied the viewport, document and body
  widths stayed at 390 pixels, the icon remained centered, the retry button
  remained visible, and no horizontal overflow occurred.
- Retry reloaded the same editor route, closed the sheet, and re-resolved all
  nine placements without changing their saved footprints.
- Stopping the Dashboard process produced the authenticated Core-shell
  fallback rather than a raw gateway error. Module Management and Scoped
  Records remained usable. Restarting Dashboard restored the same Dashboard
  identity, nine placements, and administrator manage affordances.

## Acceptance Mapping

1. Separate service, database, migration/runtime identities, manifest,
   configuration, health, readiness, API, and native UI: deployment evidence,
   module inventory tests, and browser normal-route checks.
2. Database isolation: `verify-sprint-6c-isolation.ps1`.
3. Typed references without relational Core dependency: Dashboard baseline,
   repository tests, and zero-Dashboard-table Core baseline check.
4. Transition-only Components contract and Sprint 8A migration marker:
   manifest/contract tests and Module Management diagnostics.
5. Actor, service, installation, audience, action, scope, revision, expiry,
   and replay binding: module-contract and Core gateway tests.
6. Nondisclosure plus the complete authorized resolution matrix: permission,
   adapter, Dashboard module, and UI-state tests.
7. Preserved Sprint 5A product experience: smoke, UAT, and Dashboard
   Playwright tests.
8. Core Module Management configuration/diagnostics without product-data
   access: module inventory/browser checks and database isolation.
9. Contained Dashboard and Components outages: retained browser observations
   and outage tests.
10. Deterministic fresh seed/bootstrap and one-row migration ledgers:
    deployment, smoke, UAT, and repeated bootstrap proof.
11. Exact-source quality gates: deployment evidence plus the retained smoke,
    UAT, and Playwright records.
