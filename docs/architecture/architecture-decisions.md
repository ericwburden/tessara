# Architecture Decisions

## Purpose

Record Sprint 2F architecture decisions and non-decisions.

## Completion Guidance

Keep decisions narrow to runtime/materialization visibility.

## Related Checks

`architecture-decisions`, `architecture-traceability`, `solution-architect-boundary`.

## Decision Summary

Sprint 2F will expose status/readiness through native app surfaces and explicit API contracts, not scripts or direct DB reads.

## Major Decisions

- Build the smallest coherent monitoring/readiness surface.
- Reuse shared UI primitives.
- Keep authorization in API/service boundaries.
- Keep materialization readiness separate from full dataset authoring.
- Treat Orpheum artifacts as planning backfill and traceability aids.

## Locked Decisions And Downstream Non-Negotiables

- Capability + scope + ownership remains the access model.
- No bridge or HTML-string UI returns.
- Stable client-visible status/error language.
- Permission coverage is required for new protected paths.

## Deferred Decisions

- Exact route placement and naming.
- Whether materialization refresh needs a new status table or can derive from existing state.
- Whether maintenance/import command separation is touched now or deferred.

## Architecture Assumptions

Existing runtime and analytical tables can provide enough signal for a first readiness surface.

## Risks And Tradeoffs

A too-broad surface could duplicate Sprint 2G or Phase 3/4 work. A too-thin surface could fail operator usefulness.
