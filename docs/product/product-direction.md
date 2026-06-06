# Product Direction

## Purpose

Define the Sprint 2F product direction for the active Orpheum `delivery-slice-planning` session.

## Completion Guidance

This is a lightweight backfill derived from `docs/roadmap.md`, `docs/requirements.md`, `docs/architecture.md`, and `docs/sprints/sprint-2f-plan.md`.

## Related Checks

`product-direction`, `product-traceability`, `product-owner-boundary`.

## Decision Scope

Sprint 2F covers runtime status and materialization readiness visibility. It does not reopen workflow authoring, dataset authoring, component authoring, or Sprint 2G scoped reporting compatibility work.

## Validated Inputs

- Roadmap Sprint 2F: Runtime Status And Materialization Slice.
- Requirements for runtime/response behavior, analytical materialization, permission coverage, SSR, and smoke/UAT validation.
- Architecture target flow: `Forms/Workflows -> Responses -> Materialized Sources -> DatasetRevision -> ComponentVersion -> Dashboard`.
- Current Sprint 2F plan.

## Product Goal Or Outcome Focus

Operators can inspect runtime execution and materialization readiness inside the app while respondents and scoped operators continue using existing workflow and response flows safely.

## Target Users, Stakeholders, Or Beneficiaries

- Internal operators monitoring workflow and materialization health.
- Scoped operators who must not see out-of-scope runtime or analytical data.
- Respondents whose response work must remain uninterrupted.
- Engineers and reviewers who need explicit evidence that runtime/materialization visibility did not weaken access boundaries.

## Value Hypotheses And Success Signals

- Monitoring inside the native app reduces direct database/script inspection.
- Stable readiness states make operational issues easier to triage.
- Existing workflow-mediated response paths remain green in Playwright and UAT.
- Permission scenario coverage catches out-of-scope visibility regressions.

## Acceptance Intent And Behavioral Guardrails

- Expose status and readiness, not broad new authoring.
- Use stable operator-facing status/error language.
- Preserve capability + scope + ownership access.
- Keep touched routes native SSR-owned and hydration-clean.

## Scope Boundaries And Non-Goals

In scope: runtime status, materialization readiness/refresh visibility, operator monitoring UI, related tracing/errors, and validation gates.

Out of scope: full dataset authoring, component authoring, report execution scope hardening beyond touched status surfaces, visual dashboard builder work, and pilot release hardening.

## Constraints And Decision Drivers

- Sprint must remain a vertical slice.
- Any new permission-controlled behavior requires positive and negative coverage where executable.
- `cargo audit` failures block closeout unless documented.
- The active stylesheet path remains `style/main.css`.

## Priority Themes

1. Smallest coherent monitoring surface.
2. Access-safe status data.
3. Stable errors and tracing.
4. Verification that existing response/runtime flows still work.

## Open Questions And Decision Needs

- Exact route placement for monitoring should be decided after inventory.
- Whether materialization refresh state is computed from existing records or needs a minimal new status contract must be confirmed in code.
- Whether maintenance/import command splitting is touched in this sprint depends on inventory findings.

## Recommended Next Step

Proceed to architecture and implementation planning using this bounded product direction.
