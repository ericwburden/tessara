# Sprint 2F Implementation Evidence

## Code Evidence

- Backend operations API: `crates/tessara-api/src/operations.rs`
- Capability and route registration: `crates/tessara-api/src/lib.rs`, `crates/tessara-api/src/db.rs`
- Native Operations UI: `crates/tessara-web/src/features/native/operations.rs`
- Navigation and status badges: `crates/tessara-web/src/ui/components.rs`
- Operations styling and responsive table/card behavior: `style/main.css`
- Permissions and route coverage: `end2end/tests/permissions.spec.ts`
- API regression coverage: `crates/tessara-api/tests/workflow_runtime.rs`

## Functional Evidence

- `/api/operations/status` returns scoped workflow assignment rows, dataset readiness rows, and reporting status.
- `/operations` is visible only to sessions with `operations:view` or `admin:all`.
- Scoped manager rows are constrained to assigned scope.
- No-access users cannot load operations status data.
- Operations tables use the same search/filter/pagination style as existing native table surfaces.
- Playwright cleanup removes generated `pw-permissions-*` entities and dependent assignments/submissions after the permissions suite.

## Validation Evidence

Final validation commands and outcomes are recorded in `docs/progress-report.md` under the Sprint 2F closeout entry.

## Purpose
Capture the evidence supporting Sprint 2F closeout.
## Completion Guidance
Pair this with the progress report validation table.
## Related Checks
`implementation-evidence`, `verification-traceability`.
## Evidence Scope Summary
Evidence covers API, web, end-to-end, smoke, UAT, Orpheum, audit, and clippy.
## Revision And Environment Provenance
Validated in `C:\Users\eric-dev\Projects\tessara-sprint-2f` on branch `codex/sprint-2f`.
## Validation Activities
Ran formatting, checks, tests, Playwright, smoke, UAT, audit, clippy, and Orpheum check.
## Observed Results
Core validation passed; audit and clippy reported known conditions.
## Known Failures And Skipped Checks
`cargo audit` fails on upstream advisories; clippy fails on existing frontend lint debt.
## Manual Verification Notes
Reviewer accepted `/operations` after iterative UI cleanup.
## Logs, Artifacts, And Supporting References
See `docs/progress-report.md` and release docs for command outcomes.
## Evidence Gaps And Confidence Limits
No production rollout was performed.
## Revalidation Watchouts
Re-run browser and permissions coverage after assignment detail or capability changes.
