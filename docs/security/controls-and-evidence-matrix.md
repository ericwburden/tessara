# Controls And Evidence Matrix

## Purpose

Map Sprint 2F controls to required evidence.

## Completion Guidance

Update during implementation and closeout.

## Related Checks

`controls-and-evidence-matrix`, `security-compliance-traceability`, `security-compliance-specialist-boundary`.

## Matrix Scope

Runtime/materialization monitoring and readiness surfaces.

## Required Controls

| Control | Evidence |
| --- | --- |
| Authentication required | UAT/smoke protected shell checks and API tests |
| Capability enforcement | Playwright permissions scenarios |
| Scope filtering | Positive/negative scoped operator tests |
| Ownership/delegation preserved | Workflow-mediated assignment and permissions tests |
| Stable errors | API/UI assertions or manual evidence |
| No raw internals | Review of error payloads and UI states |
| Native SSR ownership | UAT/browser route ownership review |
| Audit posture | `cargo audit` result or documented exception |

## Evidence Expectations

Each control should have a command, manual check, or review note at closeout.

## Control Owners

Implementation owner for code controls; verification owner for evidence; reviewer for closeout judgment.

## Compensating Controls Or Exceptions

Any skipped executable coverage must be documented with reason and replacement evidence.

## Unresolved Gaps

Exact runtime/materialization permission scenarios depend on final API/route shape.

## Re-Review Or Re-Approval Triggers

New protected routes, changed auth/session contract, row-level analytical exposure, or audit exceptions.

## Recommended Next Step

Use this matrix while adding tests and closeout evidence.
