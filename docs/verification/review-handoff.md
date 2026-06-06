# Sprint 2F Review Handoff

## Handoff Summary

Sprint 2F is ready for closeout verification and release-readiness documentation.

## Reviewer Notes

- Operations is intentionally read-only.
- `operations:view` does not grant refresh authority.
- Workflow assignment links currently route to `/workflows/assignments?assignment_id=...` because the administrative assignment detail page is deferred.
- Metrics are exception/status-oriented.
- Detailed runtime and dataset information remains available in tables.

## Evidence To Check

- `end2end/tests/permissions.spec.ts`
- `crates/tessara-api/tests/workflow_runtime.rs`
- Sprint closeout entry in `docs/progress-report.md`

## Purpose
Hand off review context for Sprint 2F.
## Completion Guidance
Use this with verification and release artifacts.
## Related Checks
`review-handoff`.
## Reviewed Change Summary
Operations status route/API and validation cleanup.
## Review Package Included
Plan, code, tests, scripts, docs, and browser review outcomes.
## Current Approval Posture
Accepted for local review.
## Key Findings To Carry Forward
Need assignment detail page and mutation capability decision.
## Required Follow-Up
Plan assignment administration separately.
## Follow-Up Owners
Engineering/product planning.
## Verification, Release, And Trust-Boundary Watchouts
Do not conflate `operations:view` with refresh or mutation permissions.
## Re-Review Triggers
Any permission, operations DTO, or table-control change.
## Upstream Routing Notes
Route future mutation work through capability planning.
## Recommended Next Consumer
Sprint 2G planning.
