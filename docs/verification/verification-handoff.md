# Sprint 2F Verification Handoff

## Handoff

Verification should focus on the Operations status slice, access control, scoped data containment, standard table behavior, and existing workflow/response lifecycle preservation.

## Commands

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `npm --prefix end2end test`
- `.\scripts\smoke.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- `orpheum check run --json`

## Manual Walkthrough

Use the Sprint Handoff / Demo Instructions in `docs/progress-report.md`.

## Known Follow-Up

Dedicated workflow assignment detail and mutation capabilities remain future work.

## Purpose
Hand off Sprint 2F verification context.
## Completion Guidance
Use with release handoff.
## Related Checks
`verification-handoff`.
## Handoff Summary
Core checks passed and residual conditions are recorded.
## Verification Summary
Formatting, validation, API/web tests, Playwright, smoke, UAT passed.
## Review Status And Key Findings
Reviewer accepted Operations after cleanup.
## Evidence Provenance Summary
Local Docker stack and worktree validation.
## Readiness Ownership And Conditions
Engineering owns audit/clippy follow-up.
## Coverage And Evidence Hotspots
Permissions, scope filtering, tables, cleanup.
## Residual Risks And Weak Evidence
No production rollout; assignment mutation not implemented.
## Specification Relationship
Matches revised Sprint 2F plan.
## Scope Exclusions And Deferred Coverage
Refresh ledger, job history, assignment mutation.
## Reverification Triggers
Permissions, operations DTO, or assignment link changes.
## Recommended Downstream Consumers
Release handoff and Sprint 2G planning.
## Next Decision Points
Assignment detail capability model.
