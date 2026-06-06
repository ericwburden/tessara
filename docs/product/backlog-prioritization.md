# Backlog Prioritization

## Purpose

Prioritize Sprint 2F work into an implementation-ready order.

## Completion Guidance

This file backfills Orpheum planning structure from the Sprint 2F roadmap and plan.

## Related Checks

`backlog-prioritization`, `product-traceability`, `product-owner-boundary`.

## Decision Scope

Only the Sprint 2F delivery slice is prioritized here.

## Prioritized Work Set

1. Inventory runtime, submission, dataset, component, dashboard, refresh, seed, and maintenance paths.
2. Define status/readiness DTOs and authorization boundaries.
3. Build the smallest native operator monitoring surface.
4. Add stable status/error language and tracing.
5. Add or extend permission, workflow-mediated assignment, smoke, UAT, and selector verification.
6. Reconcile audit/clippy/CI expectations.

## Ordering Rationale

Inventory comes first because current runtime and materialization data may already expose enough state. UI and tests follow the API contract so the sprint does not create a mismatched surface.

## Acceptance-Oriented Conditions

- Operators can inspect runtime and readiness in app UI.
- Scoped operators cannot see out-of-scope runtime or analytical readiness data.
- Existing response start/save/submit/review flows remain working.
- Orpheum, Playwright, UAT, smoke, Rust, audit, and selector checks are honestly recorded.

## Deferred Or Excluded Scope

Sprint 2G reporting compatibility hardening, Phase 3 dataset authoring, Phase 4 component authoring, Phase 5 dashboard composition, and Phase 6 pilot hardening remain outside this slice.

## Sequencing And Dependency Notes

Runtime status depends on existing workflow assignment/instance/submission contracts. Materialization readiness depends on the current analytical asset and refresh paths. UI depends on shared native primitives.

## Stakeholder Tensions And Tradeoffs

The main tradeoff is useful monitoring now versus full analytical lifecycle modeling later. Sprint 2F should favor honest lightweight visibility over premature authoring surfaces.

## Reprioritization Triggers

- Inventory shows no usable materialization status source.
- Permission boundaries require deeper reporting compatibility work.
- `cargo audit` reveals a blocker requiring replacement work.
- Monitoring needs become indistinguishable from full dataset/component authoring.

## Recommended Next Step

Use this order to guide implementation planning and keep deferred work out of Sprint 2F.
