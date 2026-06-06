# Architecture Handoff

## Purpose

Hand Sprint 2F architecture constraints to technical planning and implementation.

## Completion Guidance

Use this as a summary companion to `docs/architecture.md`.

## Related Checks

`architecture-handoff`, `architecture-traceability`, `solution-architect-boundary`.

## Handoff Summary

Implement runtime/materialization status visibility through native SSR UI and scoped API contracts.

## Architecture Summary

The slice sits between Runtime and Materialization in the target architecture and surfaces status without changing the target analytical model.

## Review Status And Key Findings

Conditionally ready. Inventory must verify existing state sources.

## Locked Decisions To Preserve

Native SSR, root routes, stable REST contracts, bounded modules, and capability + scope + ownership.

## Semantic Review Status

Backfill review complete; human semantic review remains before Orpheum finalization.

## Readiness Ownership And Conditions

Implementation owner owns inventory and contract decisions. Verification owner owns evidence and permissions coverage.

## Interface, Dependency, And Integration Hotspots

Workflow assignments/instances, submissions, datasets, components, dashboards, seed data, local launch, smoke, UAT, and Playwright.

## Verification Focus Areas

Scope filtering, existing response flows, route ownership, hydration, console cleanliness, selector delivery, audit/clippy/test results.

## Architecture Fitness Criteria

The surface is useful to operators, access-safe, native SSR-owned, and verifiable.

## Specification Relationship

Maps to Sprint 2F roadmap, requirements, and Sprint 2F plan acceptance criteria.

## Unresolved Decisions And Risks

Exact status schema and route placement.

## Recommended Downstream Consumers

Technical planner and implementation owner.

## Next Decision Points

After inventory, decide DTOs, route placement, and test scope.
