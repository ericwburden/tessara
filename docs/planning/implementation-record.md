# Sprint 2F Implementation Record

## Scope

Sprint 2F delivered a native read-only Operations status surface at `/operations`, guarded by `operations:view`.

## Implemented

- Added `operations:view` as an explicit capability and seeded it into built-in admin/operator access.
- Added `GET /api/operations/status` with scoped workflow assignment, dataset readiness, and reporting data status DTOs.
- Protected analytics status reads behind authenticated operations visibility.
- Added native SSR `/operations` navigation and page rendering.
- Refined Operations metrics to exception/status cards only.
- Added standard table search, filter, pagination, and mobile-card behavior.
- Linked dataset rows to dataset detail routes.
- Linked workflow assignment rows to the existing assignment surface with `assignment_id` query filtering until a future assignment detail page exists.
- Extended Playwright permissions coverage and cleanup of generated `pw-permissions-*` entities.

## Deferred

- Administrative workflow assignment detail page with reassignment, admin completion, and deactivation actions.
- Full refresh ledger/job history.
- Dataset, component, dashboard, and report authoring.
- Broad Sprint 2G scoped reporting execution hardening.

## Purpose
Record the implemented Sprint 2F change package.
## Completion Guidance
Use this as the implementation trace for sprint closeout.
## Related Checks
`implementation-record`, `implementation-traceability`.
## Implementation Scope And Objective
Deliver `/operations` with `operations:view`, scoped status data, and read-only operator tables.
## Input Context
Sprint 2F revised plan plus review comments on wording, metrics, links, filters, pagination, mobile cards, and Playwright cleanup.
## Traceability Map
Capability, API, route, UI, tests, smoke, and UAT are represented in code and docs.
## Target Slice Or Change Boundary
Operations status only; no mutation workflow assignment detail page in this sprint.
## Planned Versus Actual Scope
Actual scope matches the revised Operations Status plan with added review-driven cleanup.
## Definition Of Done Alignment
Core validation passed; clippy and audit exceptions are recorded as residual conditions.
## Change Summary
Added Operations status API/UI and hardened existing validation scripts.
## Change Inventory
API operations module, web operations route, navigation, CSS, tests, e2e cleanup, smoke/UAT scripts, roadmap/docs.
## Changed Components And Affected Areas
Capabilities, seeded roles, workflow assignment status, dataset readiness, reporting data, shell navigation, table controls.
## Interface, Schema, And Contract Effects
New `GET /api/operations/status`; `operations:view`; operations DTO fields for exception metrics and linked table rows.
## Deviations From Plan Or Specification
Workflow assignment row links currently target the assignment-filtered assignments route pending a true assignment detail page.
## Blockers, Risks, And Open Questions
Cargo audit advisories and broad existing clippy lint debt remain outside Sprint 2F remediation.
## Deferred Or Intentionally Not Included
Assignment mutation, refresh ledger, job history, and reporting execution hardening are deferred.
