# Solution Architecture

## Purpose

Define the architecture shape for Sprint 2F runtime status and materialization readiness.

## Completion Guidance

This file summarizes existing architecture for Orpheum planning. Canonical architecture remains `docs/architecture.md`.

## Related Checks

`solution-architecture`, `architecture-traceability`, `solution-architect-boundary`.

## Problem And Scope

Operators need visibility into runtime execution and materialization readiness without database access or script-only workflows.

## Input Context

Inputs include the target flow, current runtime tables, current analytical asset routes, RBAC model, and Sprint 2F plan.

## Architectural Drivers

- Native Leptos SSR UI.
- REST transport contract.
- Capability + scope + ownership.
- Stable error envelopes.
- Bounded backend modules.

## System Boundary

Sprint 2F touches API status/readiness queries, native UI presentation, tracing/error handling, validation scripts, and possibly maintenance/refresh command separation.

## Major Components And Responsibilities

- `tessara-api`: route composition, handlers, services, repos, auth scope enforcement.
- `tessara-web`: native SSR route and shared UI primitives.
- Workflow/submission modules: runtime state source.
- Dataset/component/dashboard modules: materialization/readiness state source.
- Scripts and tests: verification and UAT evidence.

## Major Flows

1. Operator requests runtime/readiness surface.
2. API authenticates and resolves capability/scope/ownership.
3. Services assemble runtime and materialization status.
4. UI renders tables, badges, empty states, and stable errors.
5. Tests verify positive and negative visibility.

## Interfaces And Contracts

New or changed DTOs should be explicit, typed, stable, and not expose raw database states directly to users.

## Integrations And External Dependencies

Postgres, Docker Compose local stack, Playwright, cargo-leptos assets, `cargo audit`, and Orpheum planning state.

## Constraints

Do not reintroduce `/app` shell assumptions, bridge assets, HTML route strings, or JS-owned app UI.

## Decisions Made

Use a small operator-facing monitoring/readiness surface before expanding authoring. Preserve root-level native routes and existing response flows.

## Locked Constraints

Authorization must be enforced before status data leaves API boundaries. Any touched protected behavior needs executable permission coverage where possible.

## Specification Relationship

This implements roadmap Sprint 2F and requirements for runtime/response, analytical materialization, permissions, and UI testability.

## Architecture Fitness Criteria

- Scoped filtering is proven.
- SSR and hydration are clean.
- Existing workflow-mediated assignments remain green.
- Stable error messages avoid raw internals.
- Audit/clippy/test results are documented.

## Trust Boundaries And Human Control Points

Trust boundaries include authenticated API requests, scoped operator visibility, response ownership/delegation, and operator interpretation of readiness states. Human review is required before Orpheum finalization and sprint closeout.

## Risks And Open Questions

The main unknown is how much materialization state already exists versus needing a minimal readiness model.
