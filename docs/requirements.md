# Tessara Requirements

This document consolidates the product and system requirements that remain active after re-alignment.

## Product Purpose

Tessara is a configurable data platform for structuring, collecting, and analyzing complex hierarchical data.

The system must preserve the product intent of the legacy application while moving to a new, explicit, configuration-driven architecture.

## User Surfaces

The product must support distinct but connected surfaces:

- internal/administrative users who configure hierarchy, access, forms, workflows, datasets, components, and dashboards
- scoped operators who work within assigned organization scope
- respondents, parents, clients, or equivalent end users who complete and review response work

The application should use one coherent shell with role-aware navigation rather than separate unrelated tools.

## Identity And Access Requirements

- The system must support explicit user accounts and authentication.
- Administrators must be able to manage users through application UI.
- Roles must be reusable permission bundles rather than one-off hardcoded account types.
- Role assignments must support scope and descendant-aware evaluation through the organization tree.
- Navigation, route access, and mutation capabilities must reflect the user's effective scope and permissions.
- The roadmap must treat user management and RBAC as first-class delivery scope, not hidden setup details.

## Organization Requirements

- The organization model must be configurable rather than hardcoded to one legacy hierarchy.
- Nodes must support metadata-backed configuration and validated parent/child relationships.
- Users must be able to browse, inspect, create, and edit organization nodes through application UI.
- UI language should support configured terminology instead of hardcoded legacy labels.

## Forms, Fields, And Workflow Requirements

- Forms must be versioned, publishable assets.
- Published workflow steps must reference stable published form versions.
- Administrators must be able to create, edit, remove, and reorder form fields through UI.
- Form authoring must support typed fields, option sets, and constrained lookup behavior.
- Form detail must expose version, publish state, and workflow attachment information.
- Draft and published behavior must be clearly distinguished in the product UI.

## Runtime And Response Requirements

- Response workflows must support assignable work, pending work, drafts, submission, and read-only completed review.
- Canonical responses must be stored as structured payloads keyed to published field definitions.
- Submission validation must be strict at workflow boundaries.
- End-user response flows must remain understandable and usable without exposing builder or migration concerns.

## Analytical Asset Requirements

### Target model

The target analytical model is:

```text
Dataset -> Component -> Dashboard
```

### Dataset requirements

- Datasets must be reusable row-level analytical assets.
- Datasets must support source composition, row filters, calculated fields, reducers, and stable exposed contracts.
- Datasets must have mutable logical identity with internal immutable `DatasetRevision`.
- Revision history and compatibility behavior must be visible in the application.

### Component requirements

- Components must be versioned presentation assets over `DatasetRevision`.
- v1 components must support:
  - `DetailTable`
  - `AggregateTable`
  - `Bar`
  - `Line`
  - `Pie/Donut`
  - `StatCard`
- Components must own grouping, measures, bucketing, and presentation configuration.
- Components must replace target-state planning that previously assumed separate report, aggregation, and chart assets.

### Dashboard requirements

- Dashboards must compose specific `ComponentVersion` references.
- Dashboards are mutable in v1.
- Dashboard authoring and viewing must be available through application-grade UI.

## Compatibility And Versioning Requirements

- Stable dependency edges must bind to immutable revisions or versions.
- Archived or inactive records must remain resolvable for historical integrity.
- When a dependent draft is rebound to a newer dependency version, the system must classify findings as compatible, warning, or blocking.
- Publication must be blocked when blocking issues remain.
- Users must be able to skip some carry-forward work rather than being forced to resolve every dependent artifact immediately.

## Migration And Verification Requirements

- Legacy import, rehearsal, and verification must remain supported during the transition.
- Migration/operator flows must move toward the new dataset/component/dashboard model.
- Verification should land in real application detail and viewer surfaces wherever possible, not only in isolated workbenches.

## Application UI Requirements

- Every future sprint must leave the application in a user-testable condition.
- Every future sprint must deliver usable application UI along with the underlying functionality.
- Product-facing and internal/operator surfaces must be clearly distinguished, but part of one coherent application experience.
- Builder-only completion is not acceptable for roadmap closure.

## Non-Functional Requirements

- The system must remain runnable locally with a deterministic seeded demo path.
- Development and smoke workflows must continue to support local user testing.
- Core routes must SSR correctly through the shared application shell.
- Read-heavy flows should remain useful even when WASM hydration is unavailable or fails.
- Heavy operator and analytics surfaces may use route/widget splitting, but the shared shell and ordinary browse/detail flows should stay eagerly available.
- Browser hydration mismatches and uncaught console errors are release-blocking defects.
- The architecture should move domain rules out of monolithic service layers as contracts stabilize.
- Performance-sensitive analytical work may use materialization and on-demand rebuild behavior.
- Unsupported v1 areas must be documented explicitly rather than implied away.

## Out Of Scope Or Deferred

- printable report artifacts composed from prose and components
- full visual dashboard designer beyond required v1 composition flows
- fuzzy joins and complex analytical behaviors beyond the defined v1 dataset engine
- route trees or UI flows that imply unsupported permissions or scope-sharing behavior
