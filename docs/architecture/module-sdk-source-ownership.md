# Module SDK Source Ownership And Deployment

Status: Accepted 2026-07-30, with Sprint 6D fast-forward implementation
contract.

## Context

Sprint 6C proved that Dashboards can run as an independent process with its own
database, runtime identity, manifest, operations, APIs, SSR pages, and
same-origin routes. It also left a deliberate source/build transition:
`tessara-dashboard-module` still links the root `tessara-web` application, and
Core still constructs Dashboard-specific web bootstrap types.

The remaining module extractions need a repeatable way to share the Tessara
shell, design system, request verification, module operations, and conformance
behavior without copying their source or linking every module back into the
Core application. Repeated compiled code and assets in separate module images
are acceptable. Repeated implementations with independent maintenance are not.

## Decision

Tessara adopts this rule:

> Shared code may be compiled into many Core or module images, but every
> behavior has one canonical source owner.

Canonical shared source is distributed as independently versioned platform
SDK/runtime packages. A module pins compatible package and contract versions,
compiles the required code and assets into its own release image, and can be
built, released, deployed, upgraded, and rolled back without rebuilding or
restarting Core or unrelated modules.

The source-ownership categories are:

| Category | Canonical owner | Integration form |
| --- | --- | --- |
| Platform contracts | Tessara platform contract packages | Versioned schemas, wire types, and generated clients |
| Module runtime | Tessara module runtime packages | HTTP/middleware, configuration, health, diagnostics, context verification, and lifecycle adapters |
| Shared UI | Tessara UI SDK and design-system packages | SSR shell components, primitives, tokens, accessibility behavior, and module-owned asset builds |
| Functional behavior | Exactly one Core area or functional module | Versioned API, event, export, or typed-resource contract |
| Module product implementation | The owning functional module | Module-local UI, API, domain rules, persistence, migrations, and tests |

Static linking, copied build layers, and repeated CSS/JavaScript/WASM bytes in
release images are deployment duplication, not source duplication. They are
allowed when each artifact is produced from the canonical package source.

## Boundary Rules

1. Shared source must not be copied into module directories. Reusable platform
   behavior moves into a canonical SDK/runtime package or remains owned by one
   service.
2. Platform packages must remain policy-neutral and must not acquire
   module-specific product rules, persistence, routes, DTOs, or lifecycle
   semantics.
3. Functional modules must not link another module's domain or persistence
   implementation. They consume versioned contracts and typed resource
   references.
4. A separately deployable module must not depend on the Core application
   binary, root `tessara-web` application, root route tree, Core API state, or
   Core-private DTOs.
5. Each module owns and serves its product pages and versioned frontend assets.
   It uses the shared UI SDK and authenticated Shell Context to render a
   complete same-origin document.
6. Core remains the canonical owner of authentication, authorization
   decisions, Organization scope, shell policy, navigation composition, and
   installation/module lifecycle. SDK code may verify or render those
   decisions; it does not recreate them.
7. Shared-package adoption is release-local. Updating canonical source changes
   deployed behavior only after a consuming module publishes and deploys a new
   image. Before production, Core supports one exact current platform tuple;
   repository consumers advance together and manifests advertise the exact
   contract and SDK versions linked into each Module Release.
8. Unsupported or vulnerable SDK versions are handled through catalog and
   compatibility policy that identifies affected Module Releases. They are not
   silently replaced inside running module images.
9. A change confined to a module implementation or its current SDK adoption
   produces only a new release of that module. The installation lockfile and
   receipt record the new module digest, while Core and unrelated module image
   digests remain unchanged.
10. Before production stability, repository functionality advances
    fast-forward. A new manifest, wire shape, package boundary, or shared
    source replaces the obsolete development shape across all repository
    consumers in the same change. Tessara does not retain dual readers,
    facades, deprecated APIs, or migration paths solely for development
    backwards compatibility.

## Required Package Direction

The extraction establishes these exact package names and responsibilities:

- `tessara-module-contract`: manifests, Shell Context, grants/decisions,
  semantic destinations, typed resource references, stable errors, and public
  wire schemas;
- `tessara-module-runtime`: server startup, authenticated context
  verification, configuration/control endpoints, health/readiness,
  diagnostics, tracing, and graceful shutdown;
- `tessara-module-ui`: complete-document SSR shell integration, shared
  primitives, design tokens, accessibility behavior, and asset conventions;
- `tessara-module-testkit`: manifest, contract, route, authorization, outage,
  asset, and deployment-independence conformance fixtures.

The complete allowed graph, provider interfaces, current manifest, generic
document route, reference module, target behavior, and version policy are
defined in the
[Module SDK Implementation Contract](./module-sdk-implementation-contract.md).

Policy-neutral `tessara-web-ui` source moves directly to
`tessara-module-ui`; its Dashboard placement editor moves to the Dashboard
product/web owner, and the old crate is deleted. `tessara-web-http` remains an
independent leaf.

## Dashboard Reference Completion

Dashboards is the first adopter after the SDK/runtime boundary exists. The
completion pass must:

- remove `tessara-dashboard-module` dependencies on root `tessara-web` and
  Core-private Dashboard bootstrap types;
- make the Dashboard release own its route tree, hydration/SSR entrypoints,
  CSS and other frontend assets;
- replace Dashboard-specific Core gateway/bootstrap behavior with generic
  module routing and versioned platform contracts;
- keep the Sprint 6C process, database, manifest, control, authorization,
  degraded-state, and Component compatibility behavior intact;
- prove a Dashboard-only image upgrade and rollback while the Core and
  unrelated module image digests remain unchanged.

That completed Dashboard boundary becomes the extraction template for
Components, Datasets, Responses, Workflows, and Forms.

## Consequences

- Module images may contain repeated compiled runtime/UI code and assets.
- A shared SDK fix reaches a deployed module only through a new release of that
  module, preserving independent rollout and rollback.
- Major platform-contract changes require an explicit compatibility migration
  rather than an implicit whole-application rebuild.
- Canonical packages require ownership, semantic versioning, compatibility
  tests, support windows, and release inventory.
- Core and module source graphs become smaller and more auditable: shared
  platform behavior is reusable, while business behavior remains owned once
  behind a functional contract.
- During pre-production the supported-version window has width one: the exact
  tuple declared by the current Core Release. Breaking changes update all
  repository consumers and baselines atomically; older or newer tuples are
  rejected rather than normalized.
- `ShellContentV1` and `tessara-web-ui` are removed in Sprint 6D instead of
  becoming transitional compatibility contracts.
