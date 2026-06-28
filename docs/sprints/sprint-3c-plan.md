# Sprint 3C Implementation Guide Draft 1: Dataset Revision And Compatibility

Status: draft for review and refinement. This is not a kickoff record yet; do not treat branch, worktree, or progress-report setup as complete until the sprint is formally started.

Kickoff acceptance: accepted for Sprint 3C kickoff on 2026-06-28. The branch, worktree, and progress-report setup are now tracked by the kickoff addendum below; earlier draft/blocker wording remains as pre-kickoff history.

## Sprint Summary

Build Sprint 3C from the roadmap `(Next)` scope: make dataset revision behavior visible and manageable through explicit draft revisions, revision publishing, revision history/detail screens, compatibility findings, dependency visibility, and typed revision/compatibility/dependency contracts.

Kickoff defaults:

- Branch: `codex/sprint-3c`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-3c`
- Plan artifact: `docs/sprints/sprint-3c-plan.md`

Assumption for Draft 1: use explicit draft revisions. Initial dataset creation may still create the first published revision, but editing an existing dataset should save a draft and require a publish action.

## Kickoff Addendum

- Sprint: Sprint 3C: Dataset Revision And Compatibility Slice
- Kickoff date: 2026-06-28
- Branch: `codex/sprint-3c`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-3c`
- Plan artifact: `docs/sprints/sprint-3c-plan.md`
- Planning source: `docs/roadmap.md` sprint heading marked `(Next)` plus the accepted Sprint 3C planning artifacts committed on `main`.
- Immediate implementation focus: start with typed revision, compatibility, dependency, and carry-forward contracts, then add revision list/detail/draft/publish API behavior before wiring the Datasets UI.
- Planned verification commands: `cargo fmt --all`, `cargo test -p tessara-api`, `cargo test -p tessara-web`, `npx playwright test`, `.\scripts\smoke.ps1`, `.\scripts\local-launch.ps1`, and `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`.

## Sprint Specifications

- Add typed dataset revision DTOs for status, compatibility severity/state, dependency kind, dependency state, and carry-forward state. Serialize stable snake_case strings for web clients; avoid ad hoc raw string comparisons in API and web code.
- Add revision APIs for listing a dataset's revisions, loading one revision detail, saving a draft revision, and publishing a draft revision.
- Revision list responses should include status, version label/number, created/published/materialized metadata, dependency counts, and compatibility summary.
- Revision detail responses should include source, operation, restriction, and output-field snapshots, generated SQL, materialization metadata, dependency visibility, and compatibility findings.
- Draft saves for existing datasets must not replace published `dataset_sources`, `dataset_fields`, or materialized output.
- Publish should supersede the current published revision, materialize the new revision, replace dataset-level current catalog rows, and update visibility/metadata atomically.
- Keep revision snapshots in `dataset_revisions` JSON fields while draft. Treat `dataset_sources` and `dataset_fields` as current-published catalog tables only.
- Define compatibility findings by comparing a candidate revision to the current published revision:
  - removed output field or changed field type: breaking
  - added output field: compatible
  - label/source/display-only changes: warning or informational
  - restriction-policy changes: warning requiring review
- Define dependency visibility from current downstream references:
  - dependent datasets via `dataset_sources.dataset_revision_id`
  - component versions via `component_versions.dataset_revision_id`
  - dashboards through `dashboard_components -> component_versions`
- Carry-forward behavior for Sprint 3C is inspect-first, not automatic mutation: downstream assets remain pinned to their existing dataset revision; the API reports whether each dependency looks safe, blocked, or manual-review for carry-forward.

## Application UI Delivered

- Extend `tessara-web-datasets` with revision history, revision detail, and compatibility views while keeping root route adapters in `tessara-web`.
- Add dataset routes for revision history/detail under the existing Datasets feature:
  - `/datasets/:dataset_id/revisions`
  - `/datasets/:dataset_id/revisions/:revision_id`
- Update the dataset editor flow for existing datasets:
  - save changes as a draft revision
  - show draft compatibility/dependency impact before publish
  - publish from the draft review screen
- Dataset detail should show current published revision status and link to revision history.
- Revision detail should show snapshot metadata, output fields, compatibility findings, downstream dependencies, carry-forward state, generated SQL, and materialization status.

## Acceptance Criteria

- A tester can edit an existing dataset, save a draft revision, review compatibility findings and downstream dependencies, then publish the revision.
- Published revision history clearly shows current, superseded, and draft revisions with typed statuses.
- Downstream components, dashboards, and dependent datasets remain pinned to their referenced revision after publish, and the UI explains the downstream impact.
- Compatibility and dependency contracts are typed in API and web DTOs; implementation does not rely on scattered raw string comparisons.
- Existing dataset preview, create, edit, source catalog, restriction, and materialization behavior continue to work.

## Manual Test Plan

- Sign in as an admin, seed demo data if needed, and open the Datasets area.
- Create a dataset and confirm the first revision is published and previewable.
- Edit an existing dataset and save the edit as a draft revision.
- Confirm the dataset detail and preview still reflect the current published revision before publish.
- Open revision history, inspect the draft revision, and verify compatibility findings and downstream dependency visibility.
- Publish the draft and confirm exactly one revision is current/published while the prior revision is superseded.
- Confirm dependent datasets, component versions, and dashboards remain pinned to their prior referenced revision and surface clear impact messaging.
- Confirm scoped dataset reader/manager accounts can only see or publish revisions permitted by their dataset capabilities.

## Automated Test Plan

- `cargo fmt --all`
- `cargo test -p tessara-api datasets:: --lib`
- `cargo test -p tessara-api --test demo_flow`
- `cargo test -p tessara-web --features hydrate --lib`
- `cargo test -p tessara-web-datasets --features hydrate --lib` if supported by crate configuration
- `npx playwright test end2end/tests/datasets.spec.ts`
- `npx playwright test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

Scenario coverage:

- draft save does not replace current published catalog or materialized preview
- publish supersedes exactly one current revision and materializes the new revision
- compatibility findings cover added, removed, type-changed, and restriction-policy-changed outputs
- dependency visibility includes dependent datasets, component versions, and dashboards
- scoped dataset reader/manager permissions still gate revision visibility and publish actions

## Ordered Implementation Plan

1. Add typed API DTOs and helper enums for revision status, compatibility findings, dependency summaries, and carry-forward state.
2. Add repository/query helpers for revision listing/detail, downstream dependency discovery, and current-vs-candidate compatibility comparison.
3. Add draft-save and publish API handlers, keeping draft changes inside `dataset_revisions` until publish.
4. Refactor existing update flow so the web editor uses draft-save plus publish for existing datasets; keep initial create path simple and published.
5. Add feature-local web contracts, API calls, loaders, and display helpers in `tessara-web-datasets`.
6. Add revision history/detail/compatibility UI and root dataset route adapters.
7. Extend API, web unit, integration, and Playwright coverage.
8. Run the full planned verification set and update sprint notes with results.

## Dependencies And Blockers

- Explicit draft revisions are the desired Sprint 3C direction unless changed in refinement.
- No automatic downstream component/dashboard/dataset repointing ships in Sprint 3C; carry-forward is reported as typed review guidance.
- No schema migration is required for the first draft unless implementation discovers the existing JSON revision snapshots cannot support draft detail or compatibility checks.
- The deferred `datasets/mod.rs` split remains out of scope unless the implementation becomes unsafe without a small mechanical extraction.
- Formal kickoff remains blocked until this draft is accepted or revised and the sprint is intentionally started from a clean `main` checkout.
