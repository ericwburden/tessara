# Tessara Architecture

This document defines the target architecture for Tessara and explains the transition from the current implementation baseline.

## Current Implementation Gap

The current codebase already delivers important backend and UI groundwork, but its reporting layer is still transitional. Current code and routes still expose `Report`, `Aggregation`, and `Chart` concepts. Those concepts describe shipped implementation state, not the target architecture.

Current transitional model:

```text
Forms -> Dataset -> Report -> Aggregation -> Chart -> Dashboard
```

Target model:

```text
Forms/Workflows -> Responses -> Materialized Sources -> DatasetRevision -> ComponentVersion -> Dashboard
```

## Superseded Model

### Old

```text
Dataset -> Report -> Aggregation -> Chart -> Dashboard
```

### Final

```text
Dataset -> Component -> Dashboard
```

In the target model:

- dataset absorbs former report-definition responsibilities
- component absorbs former aggregation/chart presentation responsibilities
- dashboard remains the composition surface

## Layers

1. Capture
   - fields
   - forms
   - workflows
2. Runtime
   - assignments
   - workflow instances
   - form responses
3. Materialization
   - parent materialized response relations
   - generic multi-select child rows
4. Modeling
   - dataset
   - dataset revision
   - dataset contract
5. Presentation
   - component
   - component version
6. Composition
   - dashboard

## Key Principles

- Stable dependency edges bind to immutable revisions or versions.
- User-facing authoring should prefer automatic derivation over manual metadata entry where practical.
- Archived or inactive records must remain resolvable for historical integrity.
- Materialized physical relations may be evicted and rebuilt while semantic revision metadata remains stable.
- Future planning should prefer `phase -> sprint -> user-testable UI` delivery rather than backend-first sequencing.

## Frontend Delivery Architecture

The current frontend is a native Leptos SSR application with root route ownership in `tessara-web` and feature-area implementation in focused `tessara-web-*` crates. The extracted baseline includes shared UI plus Datasets, Forms, Workflows, Responses, and Organization. This split is now the default frontend architecture for major feature areas.

`tessara-web` remains the cargo-leptos application crate. It owns app composition and route policy:

- root route registration and route parameter parsing
- `AppShell`, route titles, auth guards, session/navigation/logout policy, document integration, hydration entrypoint, CSS, and public assets
- thin route adapters that render feature-crate content components

Feature crates own feature UI implementation and feature-local browser transport:

- `tessara-web-ui` owns generic, policy-neutral UI primitives
- `tessara-web-data-ops` owns neutral data-operation authoring controls and draft contracts reused by feature crates, such as projection and filter editors shared by Dataset authoring and thin Table Component authoring
- `tessara-web-datasets`, `tessara-web-components`, `tessara-web-forms`, `tessara-web-workflows`, `tessara-web-responses`, and `tessara-web-organization` own their respective content components, loaders/actions, web DTOs, display helpers, and local support helpers
- feature crates expose narrow content facades and do not depend on root `tessara-web`, `tessara-api`, root route modules, `AppShell`, `leptos_router`, `leptos_meta`, auth/session/navigation policy, or sibling feature crates other than shared UI

Frontend delivery should follow these rules:

- use `cargo-leptos` as the canonical workspace build pipeline
- keep a single `axum` binary (`tessara-api`) that serves API routes, SSR HTML, SVG assets, and the built wasm/js package
- keep `tessara-web` focused on root application composition and route adapters rather than growing new major feature implementation inside the root crate
- implement major feature areas as focused feature crates when the boundary is clear; Component authoring/viewing is implemented in `tessara-web-components`
- replace the current broad Administration grouping with individual administrative feature areas when that work is revisited: User Management, Roles and Access, and Organization Schema are the intended slices, while Datasets is already separate and Components should be built as its own planned feature area
- keep REST endpoints as the stable transport contract during the migration; UI components should read and mutate data through feature-local adapters rather than embedding raw fetch logic throughout the component tree
- keep the root-level native application URLs as the active UI contract
- keep API DTOs and web DTOs separate by default; promote shared contracts only after ownership, representation stability, maintenance cost, and WASM dependency cost are measured
- avoid a shared `tessara-web-platform` crate by default; create one only if repeated transport/helper copies become a real maintenance cost and the crate can stay policy-neutral
- use release frontend artifacts for production bundle decisions, and treat dev-profile bundle size as a trend signal only

### Rendering policy

The frontend should be SSR-first and progressively enhanced:

- server-render shell chrome, navigation, route framing, detail/read views, and initial list/detail data
- hydrate only where interactivity materially improves the workflow
- prefer URL-driven state over large global client stores
- prefer native links/forms and graceful degradation where practical
- treat hydration mismatches as correctness bugs, not cosmetic issues

### Lazy-loading policy

Route and widget splitting should be selective:

- core shell and common browse/detail routes should not be lazy-loaded by default
- low-frequency operator surfaces and heavier analytics viewers may use route-level or widget-level splitting
- islands are allowed for focused, high-value interactive widgets on read-heavy pages, but islands are not the default whole-app architecture in the current migration phase
- feature-crate boundaries do not automatically create lazy-loading boundaries; any `cargo leptos build --split`, `#[lazy]`, or `#[lazy_route]` adoption should start with a focused pilot on one extracted route area

No route is currently designated as a standing lazy-loading boundary. Pick a
candidate through a focused bundle and route-behavior pilot when the next heavy
operator or analytics surface needs it.

## Asset Model

### Dataset

Reusable row-level analytical asset.

Owns:

- source composition
- joins and unions
- latest and earliest reducers
- row grain
- row filters
- calculated fields
- exposed field contract

Internal structure:

- mutable logical `Dataset`
- immutable `DatasetRevision`
- materialized relation for performance

### Component

Versioned presentation asset over a Dataset major line. Sprint 4A supports a single thin Table Component that renders a display-ready Dataset with one last-mile projection, one saved default filter set, display labels, default sort, page size, and viewer affordances.

Future component types may add charts or stat cards, but analytical shaping, aggregation, grouping, and bucketing belong in Dataset authoring rather than in separate table component backends.

### Dashboard

Mutable composition asset that references specific `ComponentVersion` records. Dashboards are not versioned in v1.

### Future Printable Report

Printable reports are separate future artifacts composed from prose and `ComponentVersion`. They are not part of the core v1 analytical asset chain.

## Data Flow

```text
Forms/Workflows -> Responses -> Materialized Sources -> DatasetRevision -> ComponentVersion -> Dashboard
```

Detailed flow:

- forms and workflows collect structured responses
- runtime persists canonical response payloads
- materialization produces reporting-friendly source relations
- datasets compile those sources into stable dataset revisions and major-line materializations
- components bind to Dataset major lines, not individual revisions
- dashboards compose component versions for end-user consumption

## Compatibility And Upgrade Behavior

When a dependent draft is rebound to a newer dependency version:

- changelog entries must classify version impact as `major`, `minor`, or `patch`
- publication is blocked for empty revisions and for validation failures that prevent compilation or materialization
- users may skip some carry-forward work instead of resolving every dependent artifact immediately

This behavior applies most directly to:

- dataset revision consumers
- component drafts bound to newer Dataset major lines
- dashboard composition when component versions change

Dataset major-line sources use an append-all contract. A source labeled `Version N` resolves to a single prebuilt `dataset_major_materializations` table for major version `N`, populated from every published historical revision in that major line. Minor and patch publishes rebuild that table; a new major publish leaves prior-major consumers bound to their existing `Version N`. The major-line table uses the latest published contract in that major line as its schema; rows from older revisions project `NULL` for fields added later in the same major line.

## Relational Model Summary

Core table families:

- access and organization:
  - `accounts`
  - `roles`
  - `capabilities`
  - `role_capabilities`
  - `role_assignments`
  - `account_delegations`
  - `nodes`

Access is evaluated from capability + scope + ownership:

- role capabilities determine whether an action or surface exists
- global role assignments evaluate those capabilities without node restriction
- scoped role assignments restrict those capabilities to the assigned node subtree
- response ownership and account delegation grant access to assigned response work
- profile/persona metadata is display-only and must not control behavior

- fielding and lookup support:
  - `option_sets`
  - `select_options`
  - `lookup_sources`
  - `lookup_source_revisions`
  - `field_definitions`
- forms and workflows:
  - `forms`
  - `form_versions`
  - `form_field_placements`
  - `workflows`
  - `workflow_versions`
  - `workflow_steps`
  - `workflow_transitions`
- runtime:
  - `workflow_assignments`
  - `workflow_instances`
  - `workflow_step_instances`
  - `form_responses`
- analytical assets:
  - `datasets`
  - `dataset_revisions`
  - `dataset_sources`
  - `dataset_major_materializations`
  - `components`
  - `component_versions`
  - `dashboards`

Dataset dependency impact is currently derived from dataset source bindings, component versions, and dashboard components. There is no separate `dataset_revision_dependencies` table in the active model.

## API And Resource Families

Primary resource families for the target architecture:

- `/users`
- `/roles`
- `/role-assignments`
- `/organization/nodes`
- `/field-definitions`
- `/option-sets`
- `/lookup-sources`
- `/forms`
- `/workflows`
- `/workflow-assignments`
- `/workflow-instances`
- `/form-responses`
- `/datasets`
- `/components`
- `/dashboards`

Target analytical lifecycle examples:

- `POST /datasets`
- `PATCH /datasets/{dataset_id}`
- `POST /datasets/{dataset_id}/revisions`
- `GET /datasets/{dataset_id}/revisions/{dataset_revision_id}` for revision detail, including generated SQL
- `POST /components`
- `POST /components/{component_id}/versions`
- `POST /components/{component_id}/versions/{component_version_id}/publish`

## Rust Workspace Direction

Suggested domain-oriented crate direction:

- `tessara_core`
- `tessara_access`
- `tessara_org`
- `tessara_fields`
- `tessara_forms`
- `tessara_workflows`
- `tessara_runtime`
- `tessara_lookups`
- `tessara_materialization`
- `tessara_datasets`
- `tessara_components`
- `tessara_dashboards`
- `tessara_db`
- `tessara_api`
- `tessara_app`

Current workspace crates under `tessara/` remain transitional implementation units while contracts stabilize. The architectural direction is still to pull stable domain rules out of the main API crate and make the seams more explicit over time.
