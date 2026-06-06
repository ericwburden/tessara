# Sprint 2F Verification Strategy

## Strategy

Verify Sprint 2F through API, native UI, permission, route-ownership, smoke, UAT, and cleanup evidence.

## Required Checks

- Rust formatting and package tests.
- API checks for `operations:view`, forbidden access, scoped filtering, and DTO shape.
- Playwright route and permission checks for admin, scoped manager, and no-access users.
- Smoke and UAT scripts against `http://localhost:8080`.
- Local deployment refresh through `scripts/local-launch.ps1`.
- Orpheum check run for the active implementation scenario.

## Manual Review

- Admin opens `/operations` and sees Operations navigation and status data.
- Scoped operator opens `/operations` and sees only in-scope workflow/dataset rows.
- No-access user lacks Operations navigation and receives forbidden API status.
- Mobile viewport renders Operations table rows as cards.

## Non-Scope

- Refresh button behavior.
- Full job history.
- Workflow assignment mutation.
- Broad reporting execution hardening.

## Purpose
Define Sprint 2F verification strategy.
## Completion Guidance
Use this to understand the chosen validation mix.
## Related Checks
`verification-strategy`.
## Verification Scope And Objective
Verify read-only operations status, scoped access, and release safety.
## Input Context
Sprint 2F plan, implementation, review comments, and closeout requirements.
## Verification Drivers And Risks
Authorization leakage, scoped filtering mistakes, stale scripts, and table regressions.
## Confidence Goals
High confidence for local release review.
## Verification Levels And Methods
Unit/API tests, web tests, Playwright, smoke, UAT, manual browser review.
## Evidence Expectations
Commands and outcomes recorded in progress report.
## Scope Exclusions And Deferrals
No production rollout, job ledger, or assignment mutation verification.
## Verification Constraints And Assumptions
Local Docker stack with seeded UAT data.
## Architecture, Planning, And Specification Watchouts
Keep `operations:view` read-only and separate from refresh/mutation.
## Readiness Decision Framing
Ready if core checks pass and residual conditions are documented.
## Open Questions
Future assignment admin capabilities.
