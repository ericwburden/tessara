# Sprint 2F Code Review Scope

## In Scope

- `operations:view` capability behavior.
- `/api/operations/status` authorization, scoping, and DTO shape.
- `/operations` native SSR rendering and navigation visibility.
- Operations table search/filter/pagination/mobile behavior.
- Analytics status authentication protection.
- Playwright permissions coverage and generated entity cleanup.

## Out Of Scope

- Dataset authoring.
- Component authoring.
- Dashboard composition.
- Full refresh/job ledger.
- Workflow assignment mutation UI.
- Sprint 2G reporting execution hardening.

## Review Focus

- Permission leaks.
- Scope-filtering regressions.
- Native route ownership and hydration issues.
- Non-standard table shape drift.
- Over-granting `operations:view` beyond read-only status visibility.

## Purpose
Define the review boundary for Sprint 2F.
## Completion Guidance
Use this to focus review on operations status risk.
## Related Checks
`code-review-scope`.
## Review Objective
Confirm permissions-safe, read-only operational visibility.
## Reviewed Inputs
Sprint plan, code diffs, tests, browser comments, and validation output.
## Change Boundary Summary
Operations status API/UI and supporting validation only.
## Upstream Conformance Anchors
Sprint 2F revised plan and `operations:view` decision.
## Review Hotspots And Risk Areas
Authorization, scoped filtering, table actionability, and data leakage.
## Evidence And Context Sources
API tests, Playwright permissions tests, smoke, UAT, manual review.
## Review Constraints And Limits
No production rollout or full assignment mutation review.
## Explicitly Out Of Scope
Dataset authoring, dashboard authoring, refresh history, assignment mutation.
## Expected Follow-Up Consumers
Sprint 2G and assignment-detail planning.
