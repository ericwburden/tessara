# Tessara Canonical Docs

This `/docs` folder is the authoritative source for Tessara's active roadmap, requirements, architecture, and UI guidance.

## Canonical Files

| File | Role |
| --- | --- |
| [roadmap.md](./roadmap.md) | Current implementation baseline and forward-looking delivery plan |
| [requirements.md](./requirements.md) | Product and system requirements |
| [modular-application-platform.md](./modular-application-platform.md) | Canonical product contract for Core, modules, composition, ownership, and application releases |
| [architecture.md](./architecture.md) | Target architecture, transition model, and technical design direction |
| [api-wire-types.md](./api-wire-types.md) | Direction for module-owned public contracts, wire types, and generated clients |
| [development-workflow.md](./development-workflow.md) | Local development loops for fast host-run iteration, API-only refresh, and full-stack relaunch |
| [ui-guidance.md](./ui-guidance.md) | Canonical UI guidance for naming, brand expression, shell behavior, rendering, layout, components, states, shared primitive contracts, and transitional UI rules |
| [ui-guidance-spec.md](./ui-guidance-spec.md) | Allium behavioral specification companion to the canonical UI guidance |

## Authority Rules

- Treat this folder as the only active planning and design authority.
- If a historical document outside `/docs` disagrees with a file in `/docs`, the file in `/docs` wins.
- Historical implementation notes such as `progress-report.md` remain useful inputs, but they are not canonical project direction.
- The target product architecture is Core plus selected, independently deployed full-stack modules, as defined in [modular-application-platform.md](./modular-application-platform.md).
- `Forms/Workflows -> Responses -> Datasets -> Components -> Dashboards` is the first-party reference application flow, not the platform deployment topology.
- Current code paths that still use `Report`, `Aggregation`, or `Chart` are transitional implementation details, not the target model.

## Current Direction

The active direction for Tessara is:

- preserve completed work through Sprint 5A as the transition baseline
- keep Organization, identity/users, RBAC, the shared shell, and the module control plane in Core
- make every retained non-Core feature area a separately deployed full-stack module with its own configuration or administration UI and database; retire transitional features that do not remain supported
- require one PostgreSQL cluster per v1 installation, with one Core database and one database per module instance and no cross-module database access
- integrate modules through manifests, Feature Declarations, versioned functional contracts, namespaced security capabilities, semantic destinations, APIs/events, and durable typed resource references
- keep mutation, versioning, publication, lifecycle, audit, and historical-review decisions inside the owning module
- compose independently supported applications from declarative Blueprints and exact lockfiles through deterministic validate, plan/diff, apply, and read-back operations
- execute locked Core Release components (including the gateway) and Module Release changes through an installation-local Supervisor outside Core so upgrades and rollback do not depend on a process replacing itself
- make those same machine-readable contracts safe for LLM-driven composition without database access or improvised deployment glue
- require every sprint to remain a full, user-testable vertical slice

Start here if you are orienting:

1. Read [roadmap.md](./roadmap.md) for current status and next sprints.
2. Read [requirements.md](./requirements.md) for scope and system expectations.
3. Read [modular-application-platform.md](./modular-application-platform.md) for the Core/module product contract and vocabulary.
4. Read [architecture.md](./architecture.md) for topology, integration, data ownership, and transition rules.
5. Read [api-wire-types.md](./api-wire-types.md) before adding a public module contract or JSON/event schema.
6. Read [ui-guidance.md](./ui-guidance.md) for the canonical UI specification.
7. Read [ui-guidance-spec.md](./ui-guidance-spec.md) when you need the formal Allium behavior contract for the UI guidance.

## Reference Inputs

These remain useful but are not active direction documents:

- [progress-report.md](./progress-report.md)
- [legacy-mapping.md](./legacy-mapping.md)
- [playwright-permissions-scenarios.md](./playwright-permissions-scenarios.md)
- [../README.md](../README.md)

## Historical Sources

Some older planning and design sources were consolidated into the canonical files above and are not present in this checkout. Treat `roadmap.md`, `requirements.md`, `modular-application-platform.md`, `architecture.md`, `ui-guidance.md`, and `ui-guidance-spec.md` as the active replacements for archived planning material.
