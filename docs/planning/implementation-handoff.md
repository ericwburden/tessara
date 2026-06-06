# Implementation Handoff

## Purpose

Package Sprint 2F planning for implementation.

## Completion Guidance

Use this handoff before coding and refresh after implementation if needed.

## Related Checks

`implementation-handoff`, `planning-traceability`, `technical-planner-boundary`.

## Handoff Summary

Implement runtime status and materialization readiness visibility as one bounded vertical slice.

## Planning Summary

Start with inventory, then contracts, UI, tests, and validation.

## Project Definition Of Done Summary

See `docs/planning/definition-of-done.md`.

## Review Status And Key Findings

Plan is ready with inventory-dependent decisions.

## Locked Decisions To Preserve

Native SSR, access model, bounded scope, stable errors, and validation requirements.

## Semantic Review Status

Artifacts are lightweight backfill and need human semantic review before Orpheum finalization.

## Readiness Ownership And Conditions

Implementation owner must keep scope narrow; verification owner must preserve evidence.

## Ordered Slices And Dependency Hotspots

Only one slice is planned. Hotspots are runtime state, materialization state, authorization, and route placement.

## Slice Exit Criteria Summary

Sprint plan acceptance criteria plus Orpheum checks.

## Verification And Test Strategy Touchpoints

Rust tests, Playwright permissions, workflow-mediated assignments, smoke, UAT, audit, clippy, selector delivery, Orpheum checks.

## Rollout, Migration, And Control-Point Watchouts

Do not conflate maintenance/import/demo commands with HTTP startup if touched.

## Specification Relationship

Traceable to roadmap Sprint 2F, requirements, architecture, and Sprint 2F plan.

## Unresolved Decisions And Risks

Materialization source and route placement.

## Deferred Or Not Included

Sprint 2G and later analytical authoring work.

## Recommended Downstream Consumers

Implementation owner, reviewer, verification owner.

## Next Decision Points

Complete inventory and decide DTO/route shape.
