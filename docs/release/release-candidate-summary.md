# Sprint 2F Release Candidate Summary

## Candidate

Sprint 2F Operations Status.

## Included

- Operations status API.
- Operations native UI route.
- `operations:view` capability and nav gating.
- Scoped runtime and dataset readiness visibility.
- Reporting data status visibility.
- Standard Operations table behavior.
- Playwright permissions coverage and cleanup.

## Excluded

- Assignment mutation detail page.
- Refresh jobs/ledger.
- Dataset/component/dashboard authoring.
- Sprint 2G reporting execution hardening.

## Release State

Ready for local review after mandatory closeout validation is recorded.

## Purpose
Summarize the Sprint 2F release candidate.
## Completion Guidance
Use this with rollout notes and release decision.
## Related Checks
`release-candidate-summary`.
## Release Or Adoption Objective
Provide local review access to Operations status.
## Reviewed Inputs
Implementation, validation, review comments, and closeout docs.
## Candidate Scope Included
`/operations`, `operations:view`, scoped status DTOs, actionable tables, validation script cleanup.
## Explicitly Excluded Or Deferred Scope
Assignment mutation, job ledger, refresh history, reporting hardening.
## Release Target And Consumers
Local reviewer and Sprint 2G planning.
## Upstream Decision Anchors
Sprint 2F revised plan and reviewer acceptance.
## Release-Sensitive Hotspots
Authorization and scoped data filtering.
## Candidate Limits And Assumptions
Local deployment only.
## Recommended Next Step
Use the running stack for review.
