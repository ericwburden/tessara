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

The current application-layer crate is `tessara-web`. It is the transitional implementation of the target application/UI layer even though the long-term crate naming direction may still converge on `tessara_app`.

Frontend delivery should follow these rules:

- use `cargo-leptos` as the canonical workspace build pipeline
- keep a single `axum` binary (`tessara-api`) that serves API routes, SSR HTML, bridge assets, and the built wasm/js package
- organize `tessara-web` by application shell, feature modules, shared UI primitives, and transport/infra boundaries rather than by one large shell file
- keep REST endpoints as the stable transport contract during the migration; UI components should read and mutate data through feature-local adapters rather than embedding raw fetch logic throughout the component tree
- preserve existing application URLs during the migration even while the route implementation moves into Leptos

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

Current first-class lazy-route candidate:

- `/app/migration`

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

Versioned presentation asset over a `DatasetRevision`.

v1 component types:

- `DetailTable`
- `AggregateTable`
- `Bar`
- `Line`
- `Pie/Donut`
- `StatCard`

Owns:

- grouping
- measures
- bucketing
- presentation configuration

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
- datasets compile those sources into stable dataset revisions
- components bind to specific dataset revisions
- dashboards compose component versions for end-user consumption

## Compatibility And Upgrade Behavior

When a dependent draft is rebound to a newer dependency version:

- findings must classify as `compatible`, `warning`, or `blocking`
- publication is blocked while blocking issues remain
- users may skip some carry-forward work instead of resolving every dependent artifact immediately

This behavior applies most directly to:

- dataset revision consumers
- component drafts bound to newer dataset revisions
- dashboard composition when component versions change

## Relational Model Summary

Core table families:

- access and organization:
  - `users`
  - `roles`
  - `permissions`
  - `role_permissions`
  - `role_assignments`
  - `organization_nodes`
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
  - `dataset_revision_dependencies`
  - `components`
  - `component_versions`
  - `dashboards`

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
- `GET /datasets/{dataset_id}/revisions/{dataset_revision_id}/sql`
- `POST /components`
- `POST /components/{component_id}/versions`
- `POST /components/{component_id}/versions/{component_version_id}/validate`
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
