# Security / Compliance Scope

## Purpose

Frame Sprint 2F security and compliance concerns.

## Completion Guidance

This is not a legal or compliance approval. It identifies engineering controls and evidence.

## Related Checks

`security-compliance-scope`, `security-compliance-traceability`, `security-compliance-specialist-boundary`.

## Assessment Scope

Runtime status, materialization readiness, monitoring routes, status APIs, auth/session handling, scoped visibility, and operator-facing errors.

## Reviewed Inputs

Requirements access model, architecture access summary, Sprint 2F plan, permissions scenario docs, Orpheum scenario.

## Assets, Data, And Sensitive Surfaces

Accounts, capabilities, role assignments, scope nodes, delegations, workflow assignments, workflow instances, submissions, dataset/component/dashboard metadata, and readiness/error messages.

## Trust Boundaries And Abuse Or Threat Surfaces

Authenticated session boundary, API authorization boundary, scoped subtree filtering, ownership/delegation boundary, and operator visibility into status metadata.

## Applicable Obligations And Control Drivers

Project requirements require capability + scope + ownership controls, stable user-facing errors, and Playwright coverage for new permission-controlled behavior.

## Assumptions, Exclusions, And Non-Goals

No formal regulatory classification is asserted. Sprint 2F does not define production access policy or legal compliance posture.

## Open Questions And Escalation Needs

Escalate if materialization readiness exposes row-level data or report execution details beyond metadata/status.

## Recommended Next Step

Map controls to evidence and keep security scope current during implementation.
