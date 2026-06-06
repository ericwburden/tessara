# Sprint 2F Verification Matrix

| Requirement | Manual evidence | Automated/scripted evidence |
| --- | --- | --- |
| Operators inspect runtime status through UI | Open `/operations` as admin/operator and inspect Workflow Assignments table | `npm --prefix end2end test -- permissions.spec.ts`; `cargo test -p tessara-api operations_status_requires_view_capability_and_exposes_assignment_readiness` |
| Operators inspect materialization readiness through UI | Open `/operations` and inspect Dataset Readiness table and Reporting Data Status card | Operations API DTO assertions in permissions/API tests |
| `operations:view` gates nav and API access | Compare admin/scoped manager/no-access sessions | Playwright operations visibility test and no-capability API denial test |
| Scoped operators do not see out-of-scope rows | Sign in as scoped manager and inspect visible rows | Playwright scoped operations assertions |
| Stable labels and empty/error states | Review cards and tables in `/operations` | Web compile and Playwright route load |
| Existing response/workflow flows remain intact | Open Workflows, Assignments, Responses, and run UAT | `npm --prefix end2end test`; `scripts/uat-sprint.ps1` |
| Native SSR route ownership remains intact | Refresh `/operations` and touched routes | `scripts/smoke.ps1`, `scripts/validate-e2e.ps1` when run |
| Playwright generated entities are cleaned up | Inspect database after permissions suite | Post-run DB count check for `pw-permissions-*` rows |

Final pass/fail results are recorded in `docs/progress-report.md`.

## Purpose
Map Sprint 2F requirements to verification evidence.
## Completion Guidance
Use with evidence review.
## Related Checks
`verification-matrix`.
## Matrix Scope
Operations route, API, permissions, table UX, and cleanup.
## Source Inputs
Sprint 2F revised plan and browser review comments.
## Coverage Map
API tests cover permissions/DTO; Playwright covers UI/access; smoke/UAT cover deployment sanity.
## Hotspot Summary
Scoped filtering, operations visibility, and generated Playwright cleanup.
## Contradictions And Weak Signals
Clippy and audit remain outside green core validation.
## Deferred Coverage
Workflow assignment detail and mutation workflows.
## Upstream Routing Notes
Carry assignment admin planning into future sprint work.
